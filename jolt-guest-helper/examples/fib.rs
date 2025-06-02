use jolt_guest_helper::{step, Guest, GuestBuilder};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a new guest builder for the Fibonacci function
    let mut guest = GuestBuilder::new("fib-guest")
        .with_function_name("fib")
        .with_memory_size(10485760)  // 10MB
        .with_stack_size(4096)       // 4KB
        .with_max_input_size(4096)   // 4KB
        .with_max_output_size(4096)  // 4KB
        .with_std(false)
        .build::<u32, (u32, u128)>();

    // Compile the program
    step!("Compiling fib guest code", { guest.compile("target/fib-guest") });

    // Preprocess for proving and verification
    step!("Preprocessing prover", { guest.preprocess_prover() });
    step!("Preprocessing verifier", { guest.preprocess_verifier() });

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