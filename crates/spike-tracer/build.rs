use std::env;

fn main() {
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
    println!("cargo:rerun-if-changed=src/autocxx_ffi.rs");
    println!("cargo:rerun-if-changed=test_programs/");

    // Build autocxx integration
    let mut build = autocxx_build::Builder::new("src/autocxx_ffi.rs", &[&riscv_include])
        .extra_clang_args(&["-std=c++17"])
        .build()
        .unwrap();

    build
        .flag_if_supported("-std=c++17")
        .include(&riscv_include)
        .compile("spike-autocxx");
}
