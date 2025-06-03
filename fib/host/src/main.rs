use clap::{Parser, Subcommand};
use fib_guest as guest;
use jolt_guest_helper::JoltProofBundle;
use perf_event::events::Hardware;
use perf_event::{Builder, Group};
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
    /// Run the test function (default behavior)
    Test,
}

pub fn test() {
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

    let (output, proof) = step!("Proving", { prove_fib(50) });

    let is_valid = step!("Verifying", { verify_fib(50, output, proof) });

    println!("output: {:?}", output);
    println!("valid: {is_valid}");
}

fn gen_proofs(output_path: PathBuf) {
    let target_dir = "/tmp/fib-guest-targets";
    let mut program = step!("Compiling guest code", { guest::compile_fib(target_dir) });

    // Read ELF contents from the compiled guest
    let elf_path = format!(
        "{}/fib-guest-fib/riscv32im-unknown-none-elf/release/fib-guest",
        target_dir
    );
    let elf_contents = step!("Reading ELF contents", {
        fs::read(&elf_path).expect("Failed to read ELF file")
    });

    let prover_preprocessing = step!("Preprocessing prover", {
        guest::preprocess_prover_fib(&mut program)
    });

    let prove_fib = step!("Building prover", {
        guest::build_prover_fib(program, prover_preprocessing)
    });

    let (output, proof) = step!("Proving", { prove_fib(50) });

    let proof_bundle = JoltProofBundle::new(50u32, output, proof);
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
}

fn verify_proofs(input_path: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let serialized = step!("Reading input file", {
        fs::read(&input_path).expect("Failed to read input file")
    });

    let guest_input: GuestInput = step!("Deserializing proofs", {
        postcard::from_bytes(&serialized).expect("Failed to deserialize GuestInput")
    });

    let target_dir = "/tmp/fib-guest-targets";
    let mut program = step!("Compiling guest code", { guest::compile_fib(target_dir) });

    let mut group = Group::new()?;
    let cycles = group.add(&Builder::new(Hardware::CPU_CYCLES))?;
    let insns = group.add(&Builder::new(Hardware::INSTRUCTIONS))?;
    group.enable()?;

    let verifier_preprocessing = step!("Preprocessing verifier", {
        guest::preprocess_verifier_fib(&mut program)
    });

    let verify_fib = step!("Building verifier", {
        guest::build_verifier_fib(verifier_preprocessing)
    });

    for (i, proof_bundle) in guest_input.proofs.iter().enumerate() {
        let is_valid = step!(format!("Verifying proof {}", i + 1), {
            verify_fib(
                proof_bundle.input,
                proof_bundle.output,
                proof_bundle.proof.clone().into(),
            )
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

pub fn main() {
    let cli = Cli::parse();

    match cli.command.unwrap_or(Commands::Test) {
        Commands::Gen { output } => gen_proofs(output),
        Commands::Verify { input } => {
            if let Err(e) = verify_proofs(input) {
                eprintln!("Error during verification: {}", e);
                std::process::exit(1);
            }
        }
        Commands::Test => test(),
    }
}
