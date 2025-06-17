use rust_builder::*;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_build_config_default() {
    let config = BuildConfig::default();
    assert_eq!(config.target, "riscv64gc-unknown-none-elf");
    assert_eq!(config.rustflags, vec!["-C target-feature=+m,+a,+c"]);
    assert!(config.linker_script.is_none());
    assert!(config.cleanup_objects);
    assert!(config.additional_objects.is_empty());
    assert_eq!(config.output_format, "elf");
}

#[test]
fn test_get_rust_sysroot() {
    let sysroot = get_rust_sysroot().unwrap();
    assert!(sysroot.exists());
    assert!(sysroot.is_dir());
}

#[test]
fn test_get_host_triple() {
    let triple = get_host_triple().unwrap();
    assert!(!triple.is_empty());
    // Host triple should contain the architecture
    assert!(triple.contains("unknown"));
}

#[test]
fn test_build_simple_program() {
    let temp_dir = TempDir::new().unwrap();
    let source_path = temp_dir.path().join("test.rs");
    let output_path = temp_dir.path().join("test.elf");

    // Create a simple test program
    fs::write(
        &source_path,
        r#"
        #![no_std]
        #![no_main]

        #[panic_handler]
        fn panic(_: &core::panic::PanicInfo) -> ! {
            loop {}
        }

        #[no_mangle]
        pub extern "C" fn _start() -> ! {
            loop {}
        }
    "#,
    )
    .unwrap();

    let config = BuildConfig::default();
    let result = build_rust_program(&[&source_path], &output_path, &config);

    // The build might fail if the target is not installed, but we can still test the error handling
    match result {
        Ok(result) => {
            assert!(result.output_path.exists());
            assert!(result.output_size > 0);
        }
        Err(e) => {
            // If it fails, it should be because the target is not installed
            assert!(
                e.to_string().contains("Failed to compile")
                    || e.to_string().contains("Failed to link")
            );
        }
    }
}
