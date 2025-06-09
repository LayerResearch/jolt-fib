// #![cfg_attr(feature = "guest", no_std)]

use jolt_guest_helper::{JoltProofBundle, JoltProofWrapper};
use jolt_sdk::{self as jolt, JoltHyperKZGProof, Serializable};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[jolt::provable]
fn voj(n: u32) -> u128 {
    let mut a: u128 = 0;
    let mut b: u128 = 1;
    let mut sum: u128;
    for _ in 1..n {
        sum = a + b;
        a = b;
        b = sum;
    }

    b
}

#[derive(Serialize, Deserialize)]
pub struct GuestInput {
    pub elf_contents: Vec<u8>,
    pub proofs: Vec<JoltProofBundle<u32, u128>>,
}

#[jolt::provable]
fn verify(bytes: &[u8]) -> bool {
    use jolt_guest_helper::Program;

    let input = bincode::deserialize::<GuestInput>(bytes).unwrap();

    let mut guest = Program::<u32, u128>::new();
    guest.max_input_size(4 * 1024); // 4KB
    guest.max_output_size(4 * 1024); // 4KB
    guest.memory_size(10 * 1024 * 1024); // 10MB (should match build config)
    guest.stack_size(4 * 1024); // 4KB (should match build config)
    guest.elf_contents(&input.elf_contents);

    let proofs = input.proofs;
    for bundle in proofs {
        match guest.verify(bundle.input, bundle.output, bundle.proof.into()) {
            Ok(is_valid) => {
                if !is_valid {
                    return false;
                }
            }
            Err(e) => {
                return false;
            }
        }
    }
    true
}
