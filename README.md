# Jolt Fibonacci Example

This repository contains a Fibonacci number computation example using the Jolt ZKVM system.

## Build fib-host for riscv64gc-unknown-linux-gnu
```
rustup target add riscv64gc-unknown-linux-gnu
cargo run --release --package fib-host 
cargo run --release --package fib-host -- gen -o `pwd`/client-input.bin
cargo run --release --package fib-host -- verify -i `pwd`/client-input.bin
```


## Cross-compilation

if you want to cross-compile in the current environment, you might need to set the proper sources.
```bash
dpkg --add-architecture amd64
dpkg --add-architecture arm64
dpkg --add-architecture riscv64
dpkg --print-architecture && dpkg --print-foreign-architectures

tee /etc/apt/sources.list.d/ubuntu.sources <<EOF
Types: deb
URIs: http://archive.ubuntu.com/ubuntu/
Suites: noble noble-updates noble-backports noble-security
Components: main universe restricted multiverse
Architectures: amd64
Signed-By: /usr/share/keyrings/ubuntu-archive-keyring.gpg

Types: deb
URIs: http://ports.ubuntu.com/ubuntu-ports/
Suites: noble noble-updates noble-backports noble-security
Components: main universe restricted multiverse
Architectures: arm64 riscv64
Signed-By: /usr/share/keyrings/ubuntu-archive-keyring.gpg
EOF
```
or, for simplify, you can build with cross-rs.
```
cargo install cross --git https://github.com/cross-rs/cross
cross build --release --package fib-host --target riscv64gc-unknown-linux-gnu
```

## RISC-V 64 container
launch a RISC-V 64 container
```bash
docker run -it --platform linux/riscv64 --rm -v `pwd`/target/riscv64gc-unknown-linux-musl:/srv ubuntu:noble bash
```

## perf
```bash
apt-get install -y --no-install-recommends linux-tools-common linux-tools-linuxkit
```


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

## Trouble shooting
### List targets
```bash
$ rustup target list | grep riscv

riscv32i-unknown-none-elf
riscv32im-unknown-none-elf (installed)
riscv32imac-unknown-none-elf
riscv32imafc-unknown-none-elf
riscv32imc-unknown-none-elf
riscv64gc-unknown-linux-gnu (installed)
riscv64gc-unknown-linux-musl
riscv64gc-unknown-none-elf
riscv64imac-unknown-none-elf

```


### Component availability
https://rust-lang.github.io/rustup-components-history/
