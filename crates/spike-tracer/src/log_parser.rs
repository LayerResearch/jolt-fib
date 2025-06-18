// commit_log_parser.rs
// Parse RISC‑V Spike commit‑log lines into structured JSON enriched with:
//   • `asm`            – Capstone disassembly
//   • `regs_read`      – Registers read by the instruction
//   • `bytes_written`  – Bytes the instruction stores (None if not a store)
//   • `regs` / `mem`   – Architectural destinations as printed by Spike
//
// Build dependencies (add via `cargo add …`):
//   regex serde serde_json anyhow env_logger capstone
//
// Example:
//   cat trace.log | cargo run --release > trace.jsonl
// ──────────────────────────────────────────────────────────────
use anyhow::{Context, Result};
use capstone::{arch::riscv::RiscVReg as CsRegId, InsnDetail};
use capstone::{Arch, Capstone, Endian};
use capstone::arch::BuildsCapstone;
use capstone::arch::DetailsArchInsn;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::io::{self, BufRead};
use std::collections::HashMap;
use std::str::FromStr;
use jolt_sdk::host_utils::{RVTraceRow, ELFInstruction, RegisterState, MemoryState, RV32IM};

const PAGE_SIZE: usize = 4096;

/// Architectural register classes we understand.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum Register {
    X { index: u8 },   // integer GP
    F { index: u8 },   // FP
    V { index: u8 },   // Vector
}

/// A single register destination printed by Spike.
#[derive(Debug, Serialize, Deserialize)]
pub struct RegisterWrite {
    pub reg: Register,
    pub value: u64,
}

/// A memory access logged by Spike. `data` is `None` when the simulator
/// suppresses the payload and prints only the target address.
#[derive(Debug, Serialize, Deserialize)]
pub struct MemAccess {
    pub addr: u64,
    pub data: Option<u64>,
}

/// One parsed commit‑log line.
#[derive(Debug, Serialize, Deserialize)]
pub struct Commit {
    pub hart: u32,
    pub priv_level: u8,
    pub pc: u64,
    pub encoding: u32,
    pub asm: String,
    pub imm: Option<i64>,
    pub regs_read: Vec<Register>,
    pub bytes_written: Option<u32>,
    pub regs: Vec<RegisterWrite>,
    pub mem: Vec<MemAccess>,
}

/// Stateful parser that turns a Spike commit‑log *line* into a [`Commit`] record.
pub struct CommitLogParser {
    cs: Capstone,
    header_re: Regex,
    reg_re: Regex,
    csr_re: Regex,
    mem_re: Regex,
}

impl CommitLogParser {
    /// Build a new parser (compiles regexes and capstone once).
    pub fn new() -> Result<Self> {
        let cs = Capstone::new()
            .riscv()
            .detail(true)
            .build()
            .context("init capstone")?;

        Ok(Self {
            cs,
            header_re: Regex::new(r"^core\s+(\d+):\s+(\d)\s+(0x[0-9a-fA-F]+)\s+\((0x[0-9a-fA-F]{8})\)").unwrap(),
            reg_re: Regex::new(r"\b([xfv])(\d+)\s+(0x[0-9a-fA-F]+)").unwrap(),
            csr_re: Regex::new(r"\bcsr_([a-zA-Z0-9_]+)\s+(0x[0-9a-fA-F]+)").unwrap(),
            // `mem <addr> <data>` or sometimes just `mem <addr>`
            mem_re: Regex::new(r"\bmem\s+(0x[0-9a-fA-F]+)(?:\s+(0x[0-9a-fA-F]+))?").unwrap(),
        })
    }

    /// Attempt to parse one *line*; returns `Ok(Some(commit))` on success,
    /// `Ok(None)` if the line does not look like a commit‑log entry.
    pub fn parse_line(&self, line: &str) -> Result<Option<Commit>> {
        if !line.starts_with("core") {
            return Ok(None);
        }

        let caps = match self.header_re.captures(line) {
            Some(c) => c,
            None => return Ok(None),
        };

        let hart: u32 = caps[1].parse()?;
        let priv_level: u8 = caps[2].parse()?;
        let pc = u64::from_str_radix(&caps[3][2..], 16)?;
        let encoding = u32::from_str_radix(&caps[4][2..], 16)?;

        // Disassemble this instruction.
        let bytes = encoding.to_le_bytes();
        let insns = self
            .cs
            .disasm_all(&bytes[..4], pc)
            .context("capstone disasm")?;
        let insn = match insns.iter().next() {
            Some(i) => i,
            None => return Ok(None),
        };
        let asm = format!(
            "{} {}",
            insn.mnemonic().unwrap_or(""),
            insn.op_str().unwrap_or("")
        );

        // Registers read via Capstone.
        let detail: InsnDetail = self.cs.insn_detail(&insn)?;
        let imm = detail
            .arch_detail()
            .riscv()
            .unwrap()
            .operands()
            .find_map(|op| {
                if let capstone::arch::riscv::RiscVOperand::Imm(val) = op {
                    Some(val)
                } else {
                    None
                }
            });
        let regs_read = detail
            .regs_read()
            .iter()
            .filter_map(|&id| map_reg_id(id.0 as u32))
            .collect::<Vec<_>>();

        // Register writebacks parsed from Spike tokens.
        let mut regs = Vec::new();
        for rc in self.reg_re.captures_iter(line) {
            let class = &rc[1];
            let idx: u8 = rc[2].parse()?;
            let val = u64::from_str_radix(&rc[3][2..], 16)?;
            let reg = match class {
                "x" => Register::X { index: idx },
                "f" => Register::F { index: idx },
                "v" => Register::V { index: idx },
                _ => continue,
            };
            regs.push(RegisterWrite { reg, value: val });
        }
        for cc in self.csr_re.captures_iter(line) {
            //ignore
        }

        // Memory accesses with optional data.
        let mem = self
            .mem_re
            .captures_iter(line)
            .map(|c| MemAccess {
                addr: u64::from_str_radix(&c[1][2..], 16).unwrap(),
                data: c.get(2).map(|m| u64::from_str_radix(&m.as_str()[2..], 16).unwrap()),
            })
            .collect::<Vec<_>>();

        // Store byte count.
        let bytes_written = store_bytes(insn.mnemonic().unwrap_or(""));

        Ok(Some(Commit {
            hart,
            priv_level,
            pc,
            encoding,
            asm,
            imm,
            regs_read,
            bytes_written,
            regs,
            mem,
        }))
    }
}

/// Map Capstone RegId → enum
fn map_reg_id(id: u32) -> Option<Register> {
    match id {
        CsRegId::RISCV_REG_X0..=CsRegId::RISCV_REG_X31 => Some(Register::X {
            index: (id - CsRegId::RISCV_REG_X0) as u8,
        }),
        _ => None,
    }
}

/// Heuristic mapping of store mnemonics → byte count.
fn store_bytes(mnemonic: &str) -> Option<u32> {
    match mnemonic {
        // Integer stores
        "sb" | "c.sb" => Some(1),
        "sh" | "c.sh" => Some(2),
        "sw" | "c.sw" | "c.swsp" => Some(4),
        "sd" | "c.sd" | "c.sdsp" => Some(8),
        // Floating‑point stores
        "fsw" => Some(4),
        "fsd" => Some(8),
        // Vector stores
        m if m.starts_with("vse8") || m.starts_with("vsse8") => Some(1),
        m if m.starts_with("vse16") || m.starts_with("vsse16") => Some(2),
        m if m.starts_with("vse32") || m.starts_with("vsse32") => Some(4),
        m if m.starts_with("vse64") || m.starts_with("vsse64") => Some(8),
        _ => None,
    }
}

/// `SparseDram` is a lazily‑allocated, page‑based memory model that simulates a
/// very large DRAM.  Each page is 4 KiB and is only allocated the first time it
/// is written to.
///
/// * The address space is effectively 64‑bit (indexed by `u64`).
/// * All multi‑byte accesses are **little‑endian**.
/// * Reads from an unallocated page return zero without side‑effects.
/// * Writes automatically allocate the backing page(s) as needed.
pub struct SparseDram {
    pages: HashMap<u64, Box<[u8; PAGE_SIZE]>>, // page_index → page bytes
}

impl SparseDram {
    /// Creates an empty DRAM with no pages allocated.
    pub fn new() -> Self {
        Self { pages: HashMap::new() }
    }

    /// Returns an immutable reference to the page if it exists.
    fn get_page(&self, page_idx: u64) -> Option<&[u8; PAGE_SIZE]> {
        self.pages.get(&page_idx).map(|b| &**b)
    }

    /// Returns a mutable reference to the page, allocating it if necessary.
    fn get_page_mut(&mut self, page_idx: u64) -> &mut [u8; PAGE_SIZE] {
        self.pages
            .entry(page_idx)
            .or_insert_with(|| Box::new([0u8; PAGE_SIZE]))
    }

    // ──────────────────────────── BYTE ACCESS ──────────────────────────────
    pub fn read_u8(&self, addr: u64) -> u8 {
        let page_idx = addr / PAGE_SIZE as u64;
        let offset = (addr % PAGE_SIZE as u64) as usize;
        self.get_page(page_idx).map(|p| p[offset]).unwrap_or(0)
    }

    pub fn write_u8(&mut self, addr: u64, val: u8) {
        let page_idx = addr / PAGE_SIZE as u64;
        let offset = (addr % PAGE_SIZE as u64) as usize;
        self.get_page_mut(page_idx)[offset] = val;
    }

    // ─────────────────── GENERIC N‑BYTE ACCESS HELPERS ────────────────────
    fn read_numeric<const N: usize>(&self, addr: u64) -> u64 {
        let mut value: u64 = 0;
        for i in 0..N {
            value |= (self.read_u8(addr + i as u64) as u64) << (8 * i);
        }
        value
    }

    fn write_numeric<const N: usize>(&mut self, addr: u64, val: u64) {
        for i in 0..N {
            let byte = ((val >> (8 * i)) & 0xFF) as u8;
            self.write_u8(addr + i as u64, byte);
        }
    }

    // ────────────────────────── PUBLIC API ────────────────────────────────
    pub fn read_u16(&self, addr: u64) -> u16 {
        self.read_numeric::<2>(addr) as u16
    }
    pub fn read_u32(&self, addr: u64) -> u32 {
        self.read_numeric::<4>(addr) as u32
    }
    pub fn read_u64(&self, addr: u64) -> u64 {
        self.read_numeric::<8>(addr)
    }

    pub fn write_u16(&mut self, addr: u64, val: u16) {
        self.write_numeric::<2>(addr, val as u64)
    }
    pub fn write_u32(&mut self, addr: u64, val: u32) {
        self.write_numeric::<4>(addr, val as u64)
    }
    pub fn write_u64(&mut self, addr: u64, val: u64) {
        self.write_numeric::<8>(addr, val)
    }

    /// Returns the number of pages currently allocated.
    pub fn allocated_pages(&self) -> usize {
        self.pages.len()
    }
}

/// RISC-V integer register file (x0–x31).
#[derive(Debug, Clone)]
pub struct RegFile {
    regs: [u64; 32],
}

impl RegFile {
    /// Create a new register file with all registers set to 0.
    pub fn new() -> Self {
        Self { regs: [0; 32] }
    }

    /// Read a register.  
    /// * x0 always returns 0.
    pub fn read(&self, idx: usize) -> u64 {
        match idx {
            0 => 0,
            1..=31 => self.regs[idx],
            _ => panic!("Register index out of bounds: {}", idx),
        }
    }

    /// Write a register.  
    /// * Writing to x0 has no effect.
    pub fn write(&mut self, idx: usize, val: u64) {
        match idx {
            0 => {} // Ignore writes to x0
            1..=31 => self.regs[idx] = val,
            _ => panic!("Register index out of bounds: {}", idx),
        }
    }

    /// Convenience for debugging: get a snapshot of all registers
    /// (x0 is forced to 0 in the returned array).
    pub fn dump(&self) -> [u64; 32] {
        let mut snapshot = self.regs;
        snapshot[0] = 0;
        snapshot
    }
}

/// A simulated CPU state, with registers and memory.
pub struct Cpu {
    pub regs: RegFile,
    pub dram: SparseDram,
}

impl Cpu {
    /// Create a new CPU with zeroed registers and empty memory.
    pub fn new() -> Self {
        Self {
            regs: RegFile::new(),
            dram: SparseDram::new(),
        }
    }

    /// Execute a single instruction commit, updating the register file and memory.
    pub fn step(&mut self, commit: &Commit) -> RVTraceRow {
        let mnemonic = commit.asm.split_whitespace().next().unwrap_or("");
        let opcode = RV32IM::from_str(mnemonic).unwrap_or(RV32IM::UNIMPL);

        let mut rs1: Option<u64> = None;
        let mut rs2: Option<u64> = None;
        let mut rd: Option<u64> = None;
        let mut rs1_val: Option<u64> = None;
        let mut rs2_val: Option<u64> = None;
        
        let mut regs_read = commit.regs_read.iter();
        if let Some(Register::X { index }) = regs_read.next() {
            rs1 = Some(*index as u64);
            rs1_val = Some(self.regs.read(*index as usize));
        }
        if let Some(Register::X { index }) = regs_read.next() {
            rs2 = Some(*index as u64);
            rs2_val = Some(self.regs.read(*index as usize));
        }

        let rd_reg_write = commit.regs.iter().find(|w| matches!(w.reg, Register::X { .. }));
        if let Some(Register::X { index }) = rd_reg_write.map(|w| w.reg) {
            rd = Some(index as u64);
        }

        let rd_post_val = rd_reg_write.map(|w| w.value);
        let mem_access = commit.mem.get(0);

        // Update register file state
        if let Some(val) = rd_post_val {
            if let Some(rd_idx) = rd {
                self.regs.write(rd_idx as usize, val);
            }
        }
        
        let mut memory_state: Option<MemoryState> = None;

        // Update memory state
        if let Some(bytes_written) = commit.bytes_written {
            for mem_access in &commit.mem {
                if let Some(data) = mem_access.data {
                    let pre_value = match bytes_written {
                        1 => self.dram.read_u8(mem_access.addr) as u64,
                        2 => self.dram.read_u16(mem_access.addr) as u64,
                        4 => self.dram.read_u32(mem_access.addr) as u64,
                        8 => self.dram.read_u64(mem_access.addr),
                        _ => 0,
                    };
                    memory_state = Some(MemoryState::Write {
                        address: mem_access.addr,
                        pre_value,
                        post_value: data,
                    });

                    match bytes_written {
                        1 => self.dram.write_u8(mem_access.addr, data as u8),
                        2 => self.dram.write_u16(mem_access.addr, data as u16),
                        4 => self.dram.write_u32(mem_access.addr, data as u32),
                        8 => self.dram.write_u64(mem_access.addr, data),
                        _ => {} // Ignore other sizes
                    }
                }
            }
        } else {
            // Handle memory reads for load instructions
            let addr = if rs1.is_some() && commit.imm.is_some() {
                Some(rs1_val.unwrap().wrapping_add(commit.imm.unwrap() as u64))
            } else {
                None
            };
            if let Some(addr) = addr {
                memory_state = Some(MemoryState::Read { address: addr, value: rd_post_val.unwrap() });
            }
        }

        RVTraceRow {
            instruction: ELFInstruction {
                address: commit.pc,
                opcode,
                rs1,
                rs2,
                rd,
                imm: commit.imm,
                virtual_sequence_remaining: None,
            },
            register_state: RegisterState {
                rs1_val,
                rs2_val,
                rd_post_val,
            },
            memory_state,
            advice_value: None,
            precompile_input: None,
            precompile_output_address: None,
        }
    }

}

impl Default for Cpu {
    fn default() -> Self {
        Self::new()
    }
}

// ─────────────────────────────── TESTS ────────────────────────────────────
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_rw() {
        let mut mem = SparseDram::new();

        // Writing a u32 that spans two pages (offset 0x1FFF‑0x2002)
        let addr = 0x1FFF;
        mem.write_u32(addr, 0xDEADBEEF);
        assert_eq!(mem.read_u32(addr), 0xDEADBEEF);

        // Two pages (0x0 and 0x1) should be allocated.
        assert_eq!(mem.allocated_pages(), 2);

        // Reading from an untouched region returns zero.
        assert_eq!(mem.read_u64(0x10_0000), 0);
    }

    #[test]
    fn x0_is_always_zero() {
        let mut rf = RegFile::new();
        rf.write(0, 42);
        assert_eq!(rf.read(0), 0);
    }

    #[test]
    fn read_write_other_regs() {
        let mut rf = RegFile::new();
        rf.write(5, 123);
        assert_eq!(rf.read(5), 123);
    }

    #[test]
    #[should_panic]
    fn invalid_index_panics() {
        let rf = RegFile::new();
        rf.read(32); // Out of bounds
    }
}
