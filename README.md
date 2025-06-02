# Jolt Fibonacci Example

This repository contains a Fibonacci number computation example using the Jolt ZKVM system.

## Known Issues

### Optimization Level Issue

The following configuration works correctly:

```toml
[profile.release]
debug = 1
opt-level = 0
codegen-units = 1
lto = false
```

However, if removing `opt-level = 0` for profile.release:
```toml
[profile.release]
debug = 1
codegen-units = 1
lto = false
```
The proof verification will fail in this case, even though the computation itself may be correct.
```
✓ Compiling guest code
✓ Preprocessing prover
✓ Preprocessing verifier
✓ Building prover
✓ Building verifier
⢹ ProvingTrace length: 1627
✓ Proving
⢹ Verifying
thread 'main' panicked at /usr/local/cargo/git/checkouts/jolt-bc4943ecdf5f6930/1a6227a/jolt-core/src/jolt/vm/read_write_memory.rs:1045:9:
assertion `left == right` failed: Output sumcheck check failed.
  left: 5734281880563573906467902786796582032363380313985064900072649521782370967195
 right: 2329572876866910553569107701383071334806900521271493818555442170566964523457
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

