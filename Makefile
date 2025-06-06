
.PHONY: bootstrap help

help:
	@echo "Available targets:"
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-15s\033[0m %s\n", $$1, $$2}'

bootstrap: ## Install required dependencies
	rustup component add clippy rustfmt
	cargo install cargo-nextest cargo-expand

build-fib-guest: ## Build the fib-guest binary
	RUSTUP_TOOLCHAIN=riscv32im-jolt-zkvm-elf \
	JOLT_FUNC_NAME=fib \
	CARGO_ENCODED_RUSTFLAGS=$(shell printf -- '-Clink-arg=-T/workspaces/jolt-fib/riscv32im-unknown-none-elf.ld\x1f-Cpasses=lower-atomic\x1f-Cpanic=abort\x1f-Cstrip=symbols\x1f-Copt-level=z') \
	cargo build --release --features guest -p fib-guest --target riscv32im-jolt-zkvm-elf

build-fib-host: ## Build the fib-host binary
	cargo build --release --package fib-host

run-fib-host: build-fib-host ## Run the fib-host binary
	RUST_BACKTRACE=1 ./target/release/fib-host

build-voj-guest: ## Build the voj-guest binary
	RUSTUP_TOOLCHAIN=riscv32im-jolt-zkvm-elf \
	JOLT_FUNC_NAME=voj \
	CARGO_ENCODED_RUSTFLAGS=$(shell printf -- '-Clink-arg=-T/workspaces/jolt-fib/riscv32im-unknown-none-elf.ld\x1f-Cpasses=lower-atomic\x1f-Cpanic=abort\x1f-Cstrip=symbols\x1f-Copt-level=z') \
	cargo build --release --features guest -p voj-guest --target riscv32im-jolt-zkvm-elf

build-voj-host: ## Build the voj-host binary
	cargo build --release --package voj-host

run-voj-host: build-voj-host ## Run the voj-host binary
	RUST_BACKTRACE=1 ./target/release/voj-host

expand-fib-guest: ## Expand the fib-guest binary
	JOLT_FUNC_NAME=fib \
    cargo expand --release --features "jolt-sdk/host" -p fib-guest --lib

run-example: ## Run the fib example
	cargo build -p jolt-guest-helper --example fib --release
	./target/release/examples/fib

lint: ## Fix linting errors
	cargo clippy --fix --allow-dirty --allow-staged -- -D warnings
	cargo fmt --all --