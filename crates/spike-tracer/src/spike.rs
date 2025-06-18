#[cxx::bridge]
mod ffi {
    unsafe extern "C++" {
        include!("spike-tracer/src/spike.h");

        pub type SpikeTracer;

        pub fn new_spike_tracer(isa: &str) -> UniquePtr<SpikeTracer>;
        fn run(
            self: Pin<&mut SpikeTracer>,
            elf: &str,
            input: &[u8],
            output: &mut [u8],
            log_path: &str,
        ) -> i32;
    }
}

pub use ffi::{new_spike_tracer, SpikeTracer};
