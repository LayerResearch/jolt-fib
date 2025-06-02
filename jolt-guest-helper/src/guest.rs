use std::marker::PhantomData;
use jolt_sdk as jolt;
use jolt::{
    JoltField, host::Program, JoltProverPreprocessing, JoltVerifierPreprocessing,
    Jolt, ProofTranscript, RV32IJoltVM, RV32IJoltProof, MemoryConfig, MemoryLayout,
    tracer,
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum GuestError {
    #[error("Prover preprocessing not initialized")]
    ProverNotInitialized,
    #[error("Verifier preprocessing not initialized")]
    VerifierNotInitialized,
    #[error("Serialization error: {0}")]
    SerializationError(#[from] postcard::Error),
    #[error("Verification failed")]
    VerificationFailed,
}

/// Configuration for building a Jolt guest
#[derive(Clone)]
pub struct GuestConfig {
    pub program_name: String,
    pub function_name: String,
    pub memory_size: u64,
    pub stack_size: u64,
    pub max_input_size: u64,
    pub max_output_size: u64,
    pub use_std: bool,
}

impl Default for GuestConfig {
    fn default() -> Self {
        Self {
            program_name: String::new(),
            function_name: String::new(),
            memory_size: 10485760, // 10MB
            stack_size: 4096,      // 4KB
            max_input_size: 4096,  // 4KB
            max_output_size: 4096, // 4KB
            use_std: false,
        }
    }
}

/// Builder for creating Jolt guest configurations
pub struct GuestBuilder {
    config: GuestConfig,
}

impl GuestBuilder {
    pub fn new(program_name: &str) -> Self {
        let mut config = GuestConfig::default();
        config.program_name = program_name.to_string();
        Self { config }
    }

    pub fn with_function_name(mut self, function_name: &str) -> Self {
        self.config.function_name = function_name.to_string();
        self
    }

    pub fn with_memory_size(mut self, size: u64) -> Self {
        self.config.memory_size = size;
        self
    }

    pub fn with_stack_size(mut self, size: u64) -> Self {
        self.config.stack_size = size;
        self
    }

    pub fn with_max_input_size(mut self, size: u64) -> Self {
        self.config.max_input_size = size;
        self
    }

    pub fn with_max_output_size(mut self, size: u64) -> Self {
        self.config.max_output_size = size;
        self
    }

    pub fn with_std(mut self, use_std: bool) -> Self {
        self.config.use_std = use_std;
        self
    }

    pub fn build<T, U>(self) -> Guest<T, U>
    where
        T: serde::Serialize + serde::de::DeserializeOwned,
        U: serde::Serialize + serde::de::DeserializeOwned,
    {
        let mut program = Program::new(&self.config.program_name);
        program.set_func(&self.config.function_name);
        program.set_std(self.config.use_std);
        program.set_memory_size(self.config.memory_size);
        program.set_stack_size(self.config.stack_size);
        program.set_max_input_size(self.config.max_input_size);
        program.set_max_output_size(self.config.max_output_size);

        Guest {
            program,
            preprocessing: None,
            verifier_preprocessing: None,
            config: self.config,
            _input: PhantomData,
            _output: PhantomData,
        }
    }
}

/// Generic Jolt guest structure for handling any function
pub struct Guest<T, U> {
    program: Program,
    preprocessing: Option<JoltProverPreprocessing<jolt::F, jolt::PCS, jolt::ProofTranscript>>,
    verifier_preprocessing: Option<JoltVerifierPreprocessing<jolt::F, jolt::PCS, jolt::ProofTranscript>>,
    config: GuestConfig,
    _input: PhantomData<T>,
    _output: PhantomData<U>,
}

impl<T, U> Guest<T, U>
where
    T: serde::Serialize + serde::de::DeserializeOwned,
    U: serde::Serialize + serde::de::DeserializeOwned,
{
    /// Compile the guest program
    pub fn compile(&mut self, target_dir: &str) -> &mut Self {
        self.program.clone().build(target_dir);
        self
    }

    /// Analyze the program with given input
    pub fn analyze(&self, input: &T) -> jolt::host::analyze::ProgramSummary {
        let input_bytes = jolt::postcard::to_stdvec(input).unwrap();
        self.program.clone().trace_analyze::<jolt::F>(&input_bytes)
    }

    /// Preprocess the program for proving
    pub fn preprocess_prover(&mut self) -> &mut Self {
        let (bytecode, memory_init) = self.program.decode();
        let memory_config = MemoryConfig {
            max_input_size: self.config.max_input_size,
            max_output_size: self.config.max_output_size,
            stack_size: self.config.stack_size,
            memory_size: self.config.memory_size,
        };
        let memory_layout = MemoryLayout::new(&memory_config);
        
        self.preprocessing = Some(RV32IJoltVM::prover_preprocess(
            bytecode,
            memory_layout,
            memory_init,
            1 << 20,
            1 << 20,
            1 << 24,
        ));
        self
    }

    /// Preprocess the program for verification
    pub fn preprocess_verifier(&mut self) -> &mut Self {
        let (bytecode, memory_init) = self.program.decode();
        let memory_config = MemoryConfig {
            max_input_size: self.config.max_input_size,
            max_output_size: self.config.max_output_size,
            stack_size: self.config.stack_size,
            memory_size: self.config.memory_size,
        };
        let memory_layout = MemoryLayout::new(&memory_config);
        
        self.verifier_preprocessing = Some(RV32IJoltVM::verifier_preprocess(
            bytecode,
            memory_layout,
            memory_init,
            1 << 20,
            1 << 20,
            1 << 24,
        ));
        self
    }

    /// Prove the execution of the program
    pub fn prove(&self, input: T) -> Result<(U, jolt::JoltHyperKZGProof), GuestError> {
        let preprocessing = self.preprocessing.as_ref()
            .ok_or(GuestError::ProverNotInitialized)?;

        let input_bytes = jolt::postcard::to_stdvec(&input)?;
        
        let (io_device, trace) = self.program.clone().trace(&input_bytes);
        let (jolt_proof, output_io_device, _) = RV32IJoltVM::prove(
            io_device,
            trace,
            preprocessing.clone(),
        );

        let output = jolt::postcard::from_bytes::<U>(&output_io_device.outputs)?;

        Ok((output, jolt::JoltHyperKZGProof { proof: jolt_proof }))
    }

    /// Verify a proof
    pub fn verify(&self, input: T, output: U, proof: jolt::JoltHyperKZGProof) -> Result<bool, GuestError> {
        let preprocessing = self.verifier_preprocessing.as_ref()
            .ok_or(GuestError::VerifierNotInitialized)?;

        let memory_config = MemoryConfig {
            max_input_size: self.config.max_input_size,
            max_output_size: self.config.max_output_size,
            stack_size: self.config.stack_size,
            memory_size: self.config.memory_size,
        };

        let mut io_device = tracer::JoltDevice::new(&memory_config);
        io_device.inputs.append(&mut jolt::postcard::to_stdvec(&input)?);
        io_device.outputs.append(&mut jolt::postcard::to_stdvec(&output)?);

        Ok(RV32IJoltVM::verify(preprocessing.clone(), proof.proof, io_device, None).is_ok())
    }
} 