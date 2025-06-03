// use jolt::{
//     tracer, Jolt, JoltField, JoltProverPreprocessing, JoltVerifierPreprocessing, MemoryConfig,
//     MemoryLayout, ProofTranscript, RV32IJoltProof, RV32IJoltVM,
// };
// use jolt_sdk::{
//     self as jolt,
//     tracer::{
//         instruction::{RV32IMCycle, RV32IMInstruction, VirtualInstructionSequence},
//         JoltDevice,
//     },
// };
// use std::io::{self, Read, Write};
// use std::{fs::File, marker::PhantomData, path::PathBuf};
// use thiserror::Error;

// use rayon::prelude::*;

// /// Errors that can occur during guest program operations
// #[derive(Error, Debug)]
// pub enum ProgramError {
//     #[error("Prover preprocessing not initialized")]
//     ProverNotInitialized,
//     #[error("Verifier preprocessing not initialized")]
//     VerifierNotInitialized,
//     #[error("Serialization error: {0}")]
//     SerializationError(#[from] postcard::Error),
//     #[error("Verification failed")]
//     VerificationFailed,
//     #[error("Failed to decode program: {0}")]
//     DecodeError(String),
//     #[error("Failed to deserialize output: {0}")]
//     DeserializationError(String),
//     #[error("Invalid state: {0}")]
//     InvalidState(String),
//     #[error("ELF contents not loaded")]
//     NoElfContent,
// }

// /// Generic Jolt guest structure for handling any function
// pub struct Program<T, U> {
//     /// Prover preprocessing data
//     preprocessing: Option<JoltProverPreprocessing<jolt::F, jolt::PCS, jolt::ProofTranscript>>,
//     /// Verifier preprocessing data
//     verifier_preprocessing:
//         Option<JoltVerifierPreprocessing<jolt::F, jolt::PCS, jolt::ProofTranscript>>,
//     /// Maximum input size in bytes
//     max_input_size: u64,
//     /// Maximum output size in bytes
//     max_output_size: u64,
//     /// Memory size in bytes
//     memory_size: u64,
//     /// Stack size in bytes
//     stack_size: u64,
//     _input: PhantomData<T>,
//     _output: PhantomData<U>,
//     elf_contents: Option<Vec<u8>>,
// }

// impl<T, U> Program<T, U>
// where
//     T: serde::Serialize + serde::de::DeserializeOwned,
//     U: serde::Serialize + serde::de::DeserializeOwned,
// {
//     /// Create a new guest program
//     pub fn new() -> Self {
//         Self {
//             preprocessing: None,
//             verifier_preprocessing: None,
//             max_input_size: 4 * 1024,      // 4KB
//             max_output_size: 4 * 1024,     // 4KB
//             memory_size: 10 * 1024 * 1024, // 10MB
//             stack_size: 4 * 1024,          // 4KB
//             _input: PhantomData,
//             _output: PhantomData,
//             elf_contents: None,
//         }
//     }

//     /// Set maximum input size
//     pub fn max_input_size(&mut self, size: u64) {
//         self.max_input_size = size;
//     }

//     /// Set maximum output size
//     pub fn max_output_size(&mut self, size: u64) {
//         self.max_output_size = size;
//     }

//     /// Set memory size
//     pub fn memory_size(&mut self, size: u64) {
//         self.memory_size = size;
//     }

//     /// Set stack size
//     pub fn stack_size(&mut self, size: u64) {
//         self.stack_size = size;
//     }

//     pub fn elf_contents(&mut self, elf: &[u8]) {
//         self.elf_contents = Some(elf.to_vec());
//     }

//     /// Get the memory configuration
//     fn memory_config(&self) -> MemoryConfig {
//         MemoryConfig {
//             max_input_size: self.max_input_size,
//             max_output_size: self.max_output_size,
//             stack_size: self.stack_size,
//             memory_size: self.memory_size,
//         }
//     }

//     pub fn decode(elf: &[u8]) -> (Vec<RV32IMInstruction>, Vec<(u64, u8)>) {
//         let (mut instructions, raw_bytes) = tracer::decode(elf);
//         // Expand virtual sequences
//         instructions = instructions
//             .into_par_iter()
//             .flat_map_iter(|instr| match instr {
//                 RV32IMInstruction::DIV(div) => div.virtual_sequence(),
//                 RV32IMInstruction::DIVU(divu) => divu.virtual_sequence(),
//                 RV32IMInstruction::LB(lb) => lb.virtual_sequence(),
//                 RV32IMInstruction::LBU(lbu) => lbu.virtual_sequence(),
//                 RV32IMInstruction::LH(lh) => lh.virtual_sequence(),
//                 RV32IMInstruction::LHU(lhu) => lhu.virtual_sequence(),
//                 RV32IMInstruction::MULH(mulh) => mulh.virtual_sequence(),
//                 RV32IMInstruction::MULHSU(mulhsu) => mulhsu.virtual_sequence(),
//                 RV32IMInstruction::REM(rem) => rem.virtual_sequence(),
//                 RV32IMInstruction::REMU(remu) => remu.virtual_sequence(),
//                 RV32IMInstruction::SB(sb) => sb.virtual_sequence(),
//                 RV32IMInstruction::SH(sh) => sh.virtual_sequence(),
//                 RV32IMInstruction::SLL(sll) => sll.virtual_sequence(),
//                 RV32IMInstruction::SLLI(slli) => slli.virtual_sequence(),
//                 RV32IMInstruction::SRA(sra) => sra.virtual_sequence(),
//                 RV32IMInstruction::SRAI(srai) => srai.virtual_sequence(),
//                 RV32IMInstruction::SRL(srl) => srl.virtual_sequence(),
//                 RV32IMInstruction::SRLI(srli) => srli.virtual_sequence(),
//                 _ => vec![instr],
//             })
//             .collect();

//         (instructions, raw_bytes)
//     }

//     pub fn trace(&self, inputs: &[u8]) -> Result<(JoltDevice, Vec<RV32IMCycle>), ProgramError> {
//         let memory_config = self.memory_config();
//         let elf_contents = match &self.elf_contents {
//             Some(contents) => contents.as_slice(),
//             None => return Err(ProgramError::NoElfContent),
//         };
//         let (trace, io_device) = tracer::trace(elf_contents.to_vec(), inputs, &memory_config);
//         Ok((io_device, trace))
//     }

//     /// Preprocess the program for proving
//     pub fn preprocess_prover(&mut self) -> Result<&mut Self, ProgramError> {
//         let elf_contents = match &self.elf_contents {
//             Some(contents) => contents.as_slice(),
//             None => return Err(ProgramError::NoElfContent),
//         };
//         let (bytecode, memory_init) = Program::<T, U>::decode(elf_contents);
//         let memory_config = self.memory_config();
//         let memory_layout = MemoryLayout::new(&memory_config);

//         self.preprocessing = Some(RV32IJoltVM::prover_preprocess(
//             bytecode,
//             memory_layout,
//             memory_init,
//             1 << 20,
//             1 << 20,
//             1 << 24,
//         ));
//         Ok(self)
//     }

//     /// Preprocess the program for verification
//     pub fn preprocess_verifier(&mut self) -> Result<&mut Self, ProgramError> {
//         let elf_contents = match &self.elf_contents {
//             Some(contents) => contents.as_slice(),
//             None => return Err(ProgramError::NoElfContent),
//         };
//         let (bytecode, memory_init) = Program::<T, U>::decode(elf_contents);
//         let memory_config = self.memory_config();
//         let memory_layout = MemoryLayout::new(&memory_config);

//         self.verifier_preprocessing = Some(RV32IJoltVM::verifier_preprocess(
//             bytecode,
//             memory_layout,
//             memory_init,
//             1 << 20,
//             1 << 20,
//             1 << 24,
//         ));
//         Ok(self)
//     }

//     /// Prove the execution of the program
//     pub fn prove(&self, input: T) -> Result<(U, jolt::JoltHyperKZGProof), ProgramError> {
//         let preprocessing = self
//             .preprocessing
//             .as_ref()
//             .ok_or(ProgramError::ProverNotInitialized)?;

//         let input_bytes = jolt::postcard::to_stdvec(&input)?;

//         let (io_device, trace) = self.trace(&input_bytes)?;
//         let (jolt_proof, output_io_device, _) =
//             RV32IJoltVM::prove(io_device, trace, preprocessing.clone());

//         let output = jolt::postcard::from_bytes::<U>(&output_io_device.outputs)?;

//         Ok((output, jolt::JoltHyperKZGProof { proof: jolt_proof }))
//     }

//     pub fn verify(
//         &self,
//         input: T,
//         output: U,
//         proof: jolt::JoltHyperKZGProof,
//     ) -> Result<bool, ProgramError> {
//         let preprocessing = self
//             .verifier_preprocessing
//             .as_ref()
//             .ok_or(ProgramError::VerifierNotInitialized)?;

//         let memory_config = MemoryConfig {
//             max_input_size: self.max_input_size,
//             max_output_size: self.max_output_size,
//             stack_size: self.stack_size,
//             memory_size: self.memory_size,
//         };

//         let mut io_device = tracer::JoltDevice::new(&memory_config);
//         io_device
//             .inputs
//             .append(&mut jolt::postcard::to_stdvec(&input)?);
//         io_device
//             .outputs
//             .append(&mut jolt::postcard::to_stdvec(&output)?);

//         Ok(RV32IJoltVM::verify(preprocessing.clone(), proof.proof, io_device, None).is_ok())
//     }
// }
