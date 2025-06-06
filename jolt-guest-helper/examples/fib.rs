use jolt_guest_helper::{step, Builder, BuilderError, Program};

const PKG_NAME: &str = "fib-guest";
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a builder for compilation with build-time settings
    let builder = Builder::new(PKG_NAME, "fib", "target/fib-guest")
        .use_std(false)
        .memory_size(10 * 1024 * 1024) // 10MB
        .stack_size(4 * 1024); // 4KB

    // Build the program
    let elf_path = step!("Building program", { builder.build() })?;
    let elf_contents = std::fs::read(&elf_path)
        .map_err(|e| BuilderError::ReadError(format!("Failed to read ELF file: {}", e)))?;

    // Create a guest program with runtime settings
    let mut guest = Program::<u32, u128>::new();
    guest.max_input_size(4 * 1024); // 4KB
    guest.max_output_size(4 * 1024); // 4KB
    guest.memory_size(10 * 1024 * 1024); // 10MB (should match build config)
    guest.stack_size(4 * 1024); // 4KB (should match build config)
    guest.elf_contents(&elf_contents);

    // Preprocess for proving and verification
    step!("Preprocessing prover", { guest.preprocess_prover() })?;
    step!("Preprocessing verifier", { guest.preprocess_verifier() })?;

    // Test with input n = 10
    let input = 10u32;

    // Prove the execution
    let (output, proof) = step!("Proving", { guest.prove(input) })?;

    println!("Input: {}", input);
    println!("Output: {:?}", output);

    // Verify the proof
    let is_valid = step!("Verifying", { guest.verify(input, output, proof) })?;

    println!("Proof is valid: {}", is_valid);

    Ok(())
}
