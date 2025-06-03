use std::{fs::File, io::Read, path::PathBuf};

use fib_guest;
use jolt_sdk::{JoltHyperKZGProof, Serializable};
use spinners::{Spinner, Spinners};
use voj_guest;

macro_rules! step {
    ($msg:expr, $action:expr) => {{
        let mut sp = Spinner::new(Spinners::Dots9, $msg.to_string());
        let result = $action;
        sp.stop_with_message(format!("✓ {}", $msg));
        result
    }};
}

pub fn main() {
    let target_dir = "/tmp/fib-guest-targets";
    let mut program = step!("Compiling guest code", {
        fib_guest::compile_fib(target_dir)
    });

    let prover_preprocessing = step!("Preprocessing prover", {
        fib_guest::preprocess_prover_fib(&mut program)
    });
    let verifier_preprocessing = step!("Preprocessing verifier", {
        fib_guest::preprocess_verifier_fib(&mut program)
    });

    let prove_fib = step!("Building prover", {
        fib_guest::build_prover_fib(program, prover_preprocessing)
    });
    let verify_fib = step!("Building verifier", {
        fib_guest::build_verifier_fib(verifier_preprocessing)
    });

    let elf =
        PathBuf::from(target_dir).join("fib-guest-fib/riscv32im-jolt-zkvm-elf/release/fib-guest");
    let mut elf_file =
        File::open(&elf).unwrap_or_else(|_| panic!("could not open elf file: {elf:?}"));
    let mut elf_contents = Vec::new();
    elf_file.read_to_end(&mut elf_contents).unwrap();

    let mut guest_input = voj_guest::GuestInput {
        elf_contents,
        proofs: vec![],
    };

    for i in 1..3 {
        let (output, proof) = step!("Proving", { prove_fib(i) });

        let bytes = proof
            .serialize_to_bytes()
            .expect("Failed to serialize proof for cloning");
        let proof_for_bundle = JoltHyperKZGProof::deserialize_from_bytes(&bytes)
            .expect("Failed to deserialize proof for cloning");
        let is_valid = step!("Verifying", { verify_fib(i, output, proof) });
        guest_input
            .proofs
            .push(jolt_guest_helper::JoltProofBundle::<u32, u128> {
                input: i,
                output: output,
                proof: jolt_guest_helper::JoltProofWrapper::from(proof_for_bundle),
            });

        println!("output: {:?}", output);
        println!("valid: {is_valid}");
    }

    let target_dir = "/tmp/voj-guest-targets";
    let mut program = step!("Compiling guest code", {
        voj_guest::compile_verify(target_dir)
    });

    let prover_preprocessing = step!("Preprocessing prover", {
        voj_guest::preprocess_prover_verify(&mut program)
    });
    let verifier_preprocessing = step!("Preprocessing verifier", {
        voj_guest::preprocess_verifier_verify(&mut program)
    });

    let prove_voj = step!("Building prover", {
        voj_guest::build_prover_verify(program, prover_preprocessing)
    });
    let verify_voj = step!("Building verifier", {
        voj_guest::build_verifier_verify(verifier_preprocessing)
    });

    let input_bytes = bincode::serialize(&guest_input).expect("Failed to serialize guest input");
    let (output, proof) = step!("Proving", { prove_voj(&input_bytes) });
    let is_valid = step!("Verifying", { verify_voj(&input_bytes, output, proof) });

    println!("output: {output}");
    println!("valid: {is_valid}");
}
