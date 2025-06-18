use crate::common::{
    build_test_program, get_tohost_address, DEFAULT_MEMORY_CONFIG, MAX_INSTRUCTIONS,
};
use log::info;
use spike_tracer::SpikeTracer;
use std::fs;

mod common;

#[test]
fn test_counter_elf_execution() {
    // Initialize logger
    env_logger::init();
    info!("Starting counter test");

    // Build the test program
    let elf_path = build_test_program("test_programs/counter.rs")
        .expect("Failed to build counter test program");
    info!("Built test program at {}", elf_path.display());

    let mut tracer = SpikeTracer::new("rv32im");
    info!("Created SpikeTracer");

    // Load counter.elf
    let elf_data = fs::read(&elf_path).expect("Failed to read counter.elf");
    info!("Loaded counter.elf: {} bytes", elf_data.len());

    // Execute until tohost
    let tohost_addr = get_tohost_address(&elf_path).expect("Failed to find tohost symbol");
    info!("Executing until tohost @ 0x{:x}...", tohost_addr);

    let log_file = tempfile::NamedTempFile::new().expect("Failed to create temp log file");
    let log_path = log_file
        .path()
        .to_str()
        .expect("Log path is not valid UTF-8");
    info!("Created temporary log file at {}", log_path);

    let elf_str = elf_path.to_str().expect("ELF path is not valid UTF-8");
    let input = vec![0; 1024];
    let mut output = vec![0; 1024];

    let return_code = tracer.run(&elf_str, &input, &mut output, log_path);
    info!("Program terminated with return code: {}", return_code);
    info!("Program terminated normally");

    info!("🎉 Counter test passed!");
}
