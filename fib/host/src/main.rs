use clap::{Parser, Subcommand};
use fib_guest as guest;
use jolt_guest_helper::{Builder, BuilderError, JoltProofBundle, Program};
use perf_event::{self};
use serde::{Deserialize, Serialize};
use spinners::{Spinner, Spinners};
use std::fs;
use std::path::PathBuf;

macro_rules! step {
    ($msg:expr, $action:expr) => {{
        let mut sp = Spinner::new(Spinners::Dots9, $msg.to_string());
        let result = $action;
        sp.stop_with_message(format!("✓ {}", $msg));
        result
    }};
}

#[derive(Serialize, Deserialize)]
pub struct GuestInput {
    pub elf_contents: Vec<u8>,
    pub proofs: Vec<JoltProofBundle<u32, u128>>,
}

#[derive(Parser)]
#[command(name = "fib-host")]
#[command(about = "A Fibonacci prover/verifier using Jolt")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate proofs and save to output file
    Gen {
        /// Output file path for generated proofs
        #[arg(short, long)]
        output: PathBuf,
    },
    /// Verify proofs from input file
    Verify {
        /// Input file path containing proofs to verify
        #[arg(short, long)]
        input: PathBuf,
    },
    /// Dump serialized bytes of number 5 to a file
    Dump {
        /// Output file path for dumped bytes
        #[arg(short, long)]
        output: PathBuf,
    },
    /// Run the test function (default behavior)
    Test,
}

const PKG_NAME: &str = "fib-guest";

pub fn test() -> Result<(), Box<dyn std::error::Error>> {
    let target_dir = "/tmp/fib-guest-targets";
    let mut program = step!("Compiling guest code", { guest::compile_fib(target_dir) });

    let prover_preprocessing = step!("Preprocessing prover", {
        guest::preprocess_prover_fib(&mut program)
    });
    let verifier_preprocessing = step!("Preprocessing verifier", {
        guest::preprocess_verifier_fib(&mut program)
    });

    let prove_fib = step!("Building prover", {
        guest::build_prover_fib(program, prover_preprocessing)
    });
    let verify_fib = step!("Building verifier", {
        guest::build_verifier_fib(verifier_preprocessing)
    });

    let input = 50;
    let (output, proof) = step!("Proving", { prove_fib(input) });

    let is_valid = step!("Verifying", { verify_fib(50, output, proof) });

    println!("output: {:?}", output);
    println!("valid: {is_valid}");

    Ok(())
}

fn gen_proofs(output_path: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
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

    // Test with input n = 10
    let input = 50u32;

    // Prove the execution
    let (output, proof) = step!("Proving", { guest.prove(input) })?;

    let proof_bundle = JoltProofBundle::new(input, output, proof);
    let guest_input = GuestInput {
        elf_contents,
        proofs: vec![proof_bundle],
    };

    let serialized = step!("Serializing proofs", {
        postcard::to_allocvec(&guest_input).expect("Failed to serialize GuestInput")
    });

    step!("Writing output file", {
        fs::write(&output_path, serialized).expect("Failed to write output file")
    });

    println!("Proofs generated and saved to: {:?}", output_path);

    Ok(())
}

fn verify_proofs(input_path: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let serialized = step!("Reading input file", {
        fs::read(&input_path).expect("Failed to read input file")
    });

    let guest_input: GuestInput = step!("Deserializing proofs", {
        postcard::from_bytes(&serialized).expect("Failed to deserialize GuestInput")
    });

    // Create a guest program with runtime settings
    let mut guest = Program::<u32, u128>::new();
    guest.max_input_size(4 * 1024); // 4KB
    guest.max_output_size(4 * 1024); // 4KB
    guest.memory_size(10 * 1024 * 1024); // 10MB (should match build config)
    guest.stack_size(4 * 1024); // 4KB (should match build config)
    guest.elf_contents(&guest_input.elf_contents);

    let mut group = perf_event::Group::new()?;
    let cycles = group.add(&perf_event::Builder::new(
        perf_event::events::Hardware::CPU_CYCLES,
    ))?;
    let insns = group.add(&perf_event::Builder::new(
        perf_event::events::Hardware::INSTRUCTIONS,
    ))?;
    group.enable()?;

    // Preprocess for proving and verification
    step!("Preprocessing verifier", { guest.preprocess_verifier() })?;

    for (i, proof_bundle) in guest_input.proofs.iter().enumerate() {
        let is_valid = step!(format!("Verifying proof {}", i + 1), {
            // Verify the proof
            let is_valid = step!("Verifying", {
                guest.verify(
                    proof_bundle.input,
                    proof_bundle.output,
                    proof_bundle.proof.clone().into(),
                )
            })?;

            println!("Proof is valid: {}", is_valid);
            is_valid
        });
        println!(
            "Proof {}: input={}, output={}, valid={}",
            i + 1,
            proof_bundle.input,
            proof_bundle.output,
            is_valid
        );
    }
    group.disable()?;

    let counts = group.read()?;
    println!(
        "cycles / instructions: {} / {} ({:.2} cpi)",
        counts[&cycles],
        counts[&insns],
        (counts[&cycles] as f64 / counts[&insns] as f64)
    );

    Ok(())
}

fn dump_number_5(output_path: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let number = 5u32;

    let serialized = step!("Serializing number 5", {
        postcard::to_allocvec(&number).expect("Failed to serialize number 5")
    });

    step!("Writing dump file", {
        fs::write(&output_path, &serialized).expect("Failed to write dump file")
    });

    println!(
        "Number 5 serialized to {} bytes: {:02x?}",
        serialized.len(),
        serialized
    );
    println!(
        "Hex dump: {}",
        serialized
            .iter()
            .map(|b| format!("\\x{:02x}", b))
            .collect::<String>()
    );
    println!("Dumped to: {:?}", output_path);

    postcard::take_from_bytes::<u32>(&serialized).unwrap();

    Ok(())
}

pub fn main() {
    let cli = Cli::parse();

    if let Err(e) = run(&cli) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn run(cli: &Cli) -> Result<(), Box<dyn std::error::Error>> {
    match cli.command.as_ref().unwrap_or(&Commands::Test) {
        Commands::Gen { output } => gen_proofs(output.clone())?,
        Commands::Verify { input } => verify_proofs(input.clone())?,
        Commands::Dump { output } => dump_number_5(output.clone())?,
        Commands::Test => test()?,
    }

    Ok(())
}
