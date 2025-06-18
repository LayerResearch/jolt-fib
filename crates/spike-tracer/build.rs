use miette;
use std::env;

fn main() -> miette::Result<()> {
    // Use RISCV environment variable, default to /opt/riscv
    let riscv = env::var("RISCV").unwrap_or_else(|_| "/opt/riscv".to_string());
    let riscv_lib = format!("{}/lib", riscv);
    let riscv_include = format!("{}/include/", riscv);

    // Link against Spike libraries
    println!("cargo:rustc-link-search=native={}", riscv_lib);
    println!("cargo:rustc-link-lib=dylib=riscv"); // Main Spike C++ library
    println!("cargo:rustc-link-lib=static=fesvr"); // FESVR support library

    // Link against system libraries that Spike depends on
    println!("cargo:rustc-link-lib=dylib=stdc++");
    println!("cargo:rustc-link-lib=dylib=pthread");
    println!("cargo:rustc-link-lib=dylib=dl");

    // Tell cargo to invalidate the built crate whenever these files change
    println!("cargo:rerun-if-changed={}", riscv_include);
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=src/spike.rs");
    println!("cargo:rerun-if-changed=src/spike.cc");
    println!("cargo:rerun-if-changed=src/spike.h");
    println!("cargo:rerun-if-changed=test_programs/");

    cxx_build::bridge("src/spike.rs")
        .file("src/spike.cc")
        .std("c++17")
        .includes([&riscv_include])
        .compile("spike-tracer");

    Ok(())
}
