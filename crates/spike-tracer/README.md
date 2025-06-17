# Spike Tracer

RISC-V execution tracer using Spike (the golden standard RISC-V emulator) as a library via autocxx.

## Current Status ✅

**Phase 1 Complete**: Basic autocxx integration with Spike C++ libraries

- ✅ **autocxx Integration**: Direct C++ class access (`cfg_t::new()`)
- ✅ **ELF Loading**: Validates and loads RISC-V ELF files  
- ✅ **Build System**: Links against Spike libraries (`libriscv.so`, `libfesvr.a`)
- ✅ **Test Programs**: Both assembly and Rust programs supported
- ✅ **API Structure**: Clean SpikeTracer interface ready for real execution

## Test Programs

### counter.elf (Rust)
```rust
fn main() -> i32 { 5 }  // Returns 5 via tohost protocol
```
- **Size**: 663KB (full Rust runtime)
- **Symbols**: `_start`, `main`, `panic`, `tohost` @ 0x122c8
- **Purpose**: Clean test case with proper termination

### simple_add.elf (Rust)  
```rust
fn simple_add() -> u32 { 5 + 7 }  // Computes 12, stores at 0x1000, exits
```
- **Size**: 691KB (includes Rust runtime)
- **Purpose**: Maintainable computation test

## Running Examples

### Basic API Test
```bash
export LD_LIBRARY_PATH=/opt/riscv/lib:$LD_LIBRARY_PATH
cargo run --example simple_add
```

### Full Integration Test  
```bash
export LD_LIBRARY_PATH=/opt/riscv/lib:$LD_LIBRARY_PATH
cargo run --example spike_example
```

### Validation Test
```bash
export LD_LIBRARY_PATH=/opt/riscv/lib:$LD_LIBRARY_PATH
cargo run --example validate_execution
```

## Building Test Programs

```bash
# ELF files are built automatically by build.rs
cargo build  # Triggers rustc build of test programs if needed

# Manual build (optional - for development)
cd test_programs
rustc --target riscv32im-unknown-none-elf --crate-type bin counter.rs -o build/counter.elf
rustc --target riscv32im-unknown-none-elf --crate-type bin simple_add.rs -o build/simple_add.elf
```

## What's Working ✅

1. **C++ Integration**: `autocxx` successfully creates Spike C++ objects
2. **Library Linking**: All Spike dependencies link correctly
3. **ELF Validation**: Proper ELF file format checking
4. **API Design**: Clean, type-safe Rust interface
5. **Build System**: Robust cross-compilation for RISC-V targets

## What's Stubbed ⚠️

1. **Program Execution**: Currently simulated (increments counters)
2. **Memory Access**: Returns hardcoded values  
3. **Trace Collection**: Not yet implemented

## Next Steps 🚧

1. **Real Execution**: Replace stubs with `sim_t::run()` calls
2. **Memory Reading**: Use `mmu->load<uint32_t>(addr)` for actual memory access
3. **Trace Collection**: Capture instruction-level execution data
4. **Jolt Integration**: Convert Spike traces to Jolt format

## Architecture

```
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│   Rust Code     │────│   autocxx FFI   │────│  Spike C++ Libs │
│                 │    │                  │    │                 │
│ SpikeTracer API │────│ cfg_t, sim_t     │────│ libriscv.so     │
│                 │    │ Direct C++ calls │    │ libfesvr.a      │
└─────────────────┘    └──────────────────┘    └─────────────────┘
```

## Dependencies

- **Spike**: `/opt/riscv/` installation with development headers
- **RISC-V Toolchain**: `riscv32im-unknown-none-elf` Rust target
- **autocxx**: For C++ integration (vs. manual bindgen wrappers)

## When Used as Library Dependency

When spike-tracer is used as a dependency in other projects, test programs are **not built** automatically, avoiding unnecessary overhead and toolchain requirements.

To force building test programs in a dependency context:

```toml
[dependencies]
spike-tracer = { version = "0.1", features = ["build-test-programs"] }
```

---

**Key Achievement**: Direct C++ class access without manual wrapper complexity. autocxx provides type-safe, automatic memory management for Spike's complex C++ API. 