use spinners::{Spinner, Spinners};

macro_rules! step {
    ($msg:expr, $action:expr) => {{
        let mut sp = Spinner::new(Spinners::Dots9, $msg.to_string());
        let result = $action;
        sp.stop_with_message(format!("✓ {}", $msg));
        result
    }};
}


pub fn main() {
    let target_dir = "/tmp/voj-guest-targets";
    let program = step!("Compiling guest code", { guest::compile_voj(target_dir) });

    let prover_preprocessing = step!("Preprocessing prover", {
        guest::preprocess_prover_voj(&program)
    });
    let verifier_preprocessing = step!("Preprocessing verifier", {
        guest::preprocess_verifier_voj(&program)
    });

    let prove_voj = step!("Building prover", {
        guest::build_prover_voj(program, prover_preprocessing)
    });
    let verify_voj = step!("Building verifier", {
        guest::build_verifier_voj(verifier_preprocessing)
    });

    let (output, proof) = step!("Proving", { prove_voj(50) });
    let is_valid = step!("Verifying", { verify_voj(50, output, proof) });

    println!("output: {output}");
    println!("valid: {is_valid}");
}
