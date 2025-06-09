use std::{path::PathBuf, process::Command};
use thiserror::Error;

/// Errors that can occur during program building
#[derive(Error, Debug)]
pub enum BuilderError {
    #[error("Failed to build program: {0}")]
    BuildError(String),
    #[error("Failed to read ELF file: {0}")]
    ReadError(String),
    #[error("Cargo command failed: {0}")]
    CargoError(String),
}

/// Builder for compiling Jolt programs
pub struct Builder {
    /// Name of the program
    name: String,
    /// Function to execute
    func: Option<String>,
    /// Whether to use standard library
    use_std: bool,
    /// Memory size in bytes
    memory_size: u64,
    /// Stack size in bytes
    stack_size: u64,
    /// Target directory
    target_dir: String,
}

impl Builder {
    /// Create a new builder
    pub fn new(
        name: impl Into<String>,
        func: impl Into<String>,
        target_dir: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            func: Some(func.into()),
            use_std: false,
            memory_size: 10 * 1024 * 1024, // 10MB
            stack_size: 4 * 1024,          // 4KB
            target_dir: target_dir.into(),
        }
    }

    /// Set whether to use standard library
    pub fn use_std(mut self, use_std: bool) -> Self {
        self.use_std = use_std;
        self
    }

    /// Set memory size
    pub fn memory_size(mut self, size: u64) -> Self {
        self.memory_size = size;
        self
    }

    /// Set stack size
    pub fn stack_size(mut self, size: u64) -> Self {
        self.stack_size = size;
        self
    }

    /// Build the program using cargo
    pub fn build(&self) -> Result<PathBuf, BuilderError> {
        // Save linker script
        self.save_linker();

        // Construct rust flags
        let rust_flags = [
            &format!("-Clink-arg=-T{}", self.linker_path()),
            "-Cpasses=lower-atomic",
            "-Cpanic=abort",
            "-Cstrip=symbols",
            "-Copt-level=z",
        ];

        // Determine toolchain
        let toolchain = if self.use_std {
            "riscv32im-jolt-zkvm-elf"
        } else {
            "riscv32im-unknown-none-elf"
        };

        // Set up environment variables
        let mut envs = vec![("CARGO_ENCODED_RUSTFLAGS", rust_flags.join("\x1f"))];

        if self.use_std {
            envs.push(("RUSTUP_TOOLCHAIN", toolchain.to_string()));
        }

        let func = if let Some(func) = &self.func {
            func.to_string()
        } else {
            "".to_string()
        };

        envs.push(("JOLT_FUNC_NAME", func.clone()));

        let target_dir = format!("{}/{}-{}", self.target_dir, self.name, func);
        println!("target_dir: {}", target_dir);
        println!(
            "JOLT_FUNC_NAME={} CARGO_ENCODED_RUSTFLAGS=$'{}' cargo {} ",
            func,
            rust_flags.join("\x1f").replace("\x1f", "\\x1f"),
            [
                "build",
                "--release",
                "--features",
                "guest",
                "-p",
                &self.name,
                "--target-dir",
                &target_dir,
                "--target",
                toolchain,
            ]
            .join(" ")
        );

        // Construct cargo command
        let output = Command::new("cargo")
            .envs(envs)
            .args([
                "build",
                "--release",
                "--features",
                "guest",
                "-p",
                &self.name,
                "--target-dir",
                &target_dir,
                "--target",
                toolchain,
            ])
            .output()
            .expect("failed to build guest");

        if !output.status.success() {
            return Err(BuilderError::CargoError(
                String::from_utf8_lossy(&output.stderr).to_string(),
            ));
        }

        // Read the compiled ELF file
        let elf_path = format!("{}/{}/release/{}", target_dir, toolchain, self.name);

        Ok(PathBuf::from(elf_path))
    }

    fn save_linker(&self) {
        let linker_path = self.linker_path();
        if let Some(parent) = std::path::Path::new(&linker_path).parent() {
            std::fs::create_dir_all(parent).expect("could not create linker directory");
        }

        let linker_script = LINKER_SCRIPT_TEMPLATE
            .replace("{MEMORY_SIZE}", &self.memory_size.to_string())
            .replace("{STACK_SIZE}", &self.stack_size.to_string());

        std::fs::write(linker_path, linker_script).expect("could not write linker script");
    }

    fn linker_path(&self) -> String {
        format!("/tmp/jolt-guest-linkers/{}.ld", self.name)
    }
}

const LINKER_SCRIPT_TEMPLATE: &str = r#"
MEMORY {
  program (rwx) : ORIGIN = 0x80000000, LENGTH = {MEMORY_SIZE}
}

SECTIONS {
  .text.boot : {
    *(.text.boot)
  } > program

  .text : {
    *(.text)
  } > program

  .data : {
    *(.data)
  } > program

  .bss : {
    *(.bss)
  } > program

  . = ALIGN(8);
  . = . + {STACK_SIZE};
  _STACK_PTR = .;
  . = ALIGN(8);
  _HEAP_PTR = .;
}
"#;
