use fib_guest as guest;
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
