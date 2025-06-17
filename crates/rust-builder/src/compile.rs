use super::*;
use std::path::Path;
use std::process::Command;

/// Build a Rust program from multiple .rs source files
pub fn build_rust_program(
    sources: &[impl AsRef<Path>],
    output_path: impl AsRef<Path>,
    config: &BuildConfig,
) -> Result<BuildResult> {
    let output_path = output_path.as_ref();
    if let Some(parent) = output_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let mut object_files = Vec::new();
    for source in sources {
        let source = source.as_ref();
        let obj_path = output_path
            .with_file_name(source.file_stem().unwrap_or_default())
            .with_extension("o");
        compile_to_object(source, &obj_path, config)?;
        object_files.push(obj_path);
    }

    // Link all object files to final output
    link_to_output(&object_files, output_path, config)?;

    // Cleanup if requested
    if config.cleanup_objects {
        for obj in &object_files {
            let _ = std::fs::remove_file(obj);
        }
    }

    let output_size = std::fs::metadata(output_path)?.len();
    Ok(BuildResult {
        output_path: output_path.to_path_buf(),
        output_size,
        build_time_ms: None,
    })
}

/// Step 1: Compile Rust source to object file
fn compile_to_object(source_path: &Path, obj_path: &Path, config: &BuildConfig) -> Result<()> {
    let rustflags = config.rustflags.join(" ");

    let mut cmd = Command::new("rustc");
    cmd.env("RUSTFLAGS", rustflags)
        .arg("--target")
        .arg(&config.target)
        .arg("--emit")
        .arg("obj")
        .arg("-o")
        .arg(obj_path)
        .arg(source_path);

    let output = cmd.output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(BuilderError::CompilationError(format!(
            "Failed to compile {}: {}",
            source_path.display(),
            stderr
        )));
    }

    Ok(())
}

/// Link object files to the final output (ELF, bin, etc.)
fn link_to_output(
    object_files: &[PathBuf],
    output_path: &Path,
    config: &BuildConfig,
) -> Result<()> {
    // Get rust-lld path
    let sysroot = get_rust_sysroot()?;
    let host_triple = get_host_triple()?;
    let rust_lld_path = sysroot
        .join("lib/rustlib")
        .join(host_triple)
        .join("bin/rust-lld");

    // Get target libraries
    let target_lib_dir = sysroot.join("lib/rustlib").join(&config.target).join("lib");
    let (libcore, libcompiler_builtins) = find_target_libraries(&target_lib_dir)?;

    // Build link command
    let mut cmd = Command::new(rust_lld_path);
    cmd.arg("-flavor").arg("gnu").arg("-o").arg(output_path);

    // Add all object files
    for obj in object_files {
        cmd.arg(obj);
    }

    // Add libcore and libcompiler_builtins
    cmd.arg(libcore).arg(libcompiler_builtins);

    // Add linker script if specified
    if let Some(linker_script) = &config.linker_script {
        cmd.arg("-T").arg(linker_script);
    }

    let output = cmd.output()?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(BuilderError::LinkingError(format!(
            "Failed to link {}: {}",
            output_path.display(),
            stderr
        )));
    }
    Ok(())
}

/// Find required target libraries
fn find_target_libraries(target_lib_dir: &Path) -> Result<(PathBuf, PathBuf)> {
    let libcore_pattern = target_lib_dir.join("libcore-*.rlib");
    let libcompiler_builtins_pattern = target_lib_dir.join("libcompiler_builtins-*.rlib");

    let libcore = glob::glob(libcore_pattern.to_str().unwrap())?
        .next()
        .ok_or_else(|| BuilderError::LibraryNotFound("libcore not found".to_string()))??;

    let libcompiler_builtins = glob::glob(libcompiler_builtins_pattern.to_str().unwrap())?
        .next()
        .ok_or_else(|| {
            BuilderError::LibraryNotFound("libcompiler_builtins not found".to_string())
        })??;

    Ok((libcore, libcompiler_builtins))
}
