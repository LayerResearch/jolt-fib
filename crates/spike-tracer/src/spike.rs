use cxx::UniquePtr;

#[cxx::bridge]
mod ffi {
    unsafe extern "C++" {
        include!("spike-tracer/src/spike.h");

        type SpikeTracer;

        fn new_spike_tracer(isa: &str) -> UniquePtr<SpikeTracer>;
        fn run(
            self: Pin<&mut SpikeTracer>,
            elf: &str,
            input: &[u8],
            output: &mut [u8],
            log_path: &str,
        ) -> i32;
    }
}

/// A wrapper around the Spike RISC-V simulator
pub struct SpikeTracer {
    inner: UniquePtr<ffi::SpikeTracer>,
}

impl SpikeTracer {
    /// Create a new SpikeTracer instance with the given ISA string
    pub fn new(isa: &str) -> Self {
        Self {
            inner: ffi::new_spike_tracer(isa),
        }
    }

    /// Run the simulator with the given ELF file, input data, and output buffer
    ///
    /// # Arguments
    ///
    /// * `elf` - Path to the ELF file to execute
    /// * `input` - Input data to provide to the program
    /// * `output` - Buffer to store program output
    /// * `log_path` - Optional path to write simulation logs to
    ///
    /// # Returns
    ///
    /// Returns the exit code from the simulation
    pub fn run(&mut self, elf: &str, input: &[u8], output: &mut [u8], log_path: &str) -> i32 {
        let return_code = self.inner.pin_mut().run(elf, input, output, log_path);
        // TODO: Parse the log file and return the trace
        // let log_parser = log_parser::LogParser::new(log_path);
        // let log_parser = log_parser.parse_log();
        return_code
    }
}
