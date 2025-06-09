.PHONY: bootstrap help

help:
	@echo "Available targets:"
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-15s\033[0m %s\n", $$1, $$2}'

bootstrap: ## Install required dependencies
	rustup component add clippy rustfmt
	cargo install cargo-nextest cargo-expand

	apt-get update && apt-get install -y --no-install-recommends gh device-tree-compiler

	@if ! gh auth status >/dev/null 2>&1; then \
		echo "GitHub authentication required. Please login:"; \
		gh auth login; \
	fi
	mkdir -p /opt/riscv/
	gh release download --clobber spike-1.1.1 --repo LayerResearch/jolt-fib --pattern "spike-1.1.1-$(shell uname -s)-$(shell uname -m).tar.gz" -O /tmp/spike.tar.gz && tar -xzf /tmp/spike.tar.gz -C /opt/riscv/
	gh release download --clobber sail-riscv-0.7 --repo LayerResearch/jolt-fib --pattern "sail-riscv-0.7-$(shell uname -s)-$(shell uname -m).tar.gz" -O /tmp/sail.tar.gz && tar -xzf /tmp/sail.tar.gz -C /opt/riscv/

build-fib-guest: ## Build the fib-guest binary
	RUSTUP_TOOLCHAIN=riscv32im-jolt-zkvm-elf \
	JOLT_FUNC_NAME=fib \
	CARGO_ENCODED_RUSTFLAGS=$(shell printf -- '-Clink-arg=-T/workspaces/jolt-fib/riscv32im-unknown-none-elf.ld\x1f-Cpasses=lower-atomic\x1f-Cpanic=abort\x1f-Cstrip=symbols\x1f-Copt-level=z') \
	cargo build --release --features guest -p fib-guest --target riscv32im-jolt-zkvm-elf

build-fib-host: ## Build the fib-host binary
	cargo build --release --package fib-host

run-fib-host: build-fib-host ## Run the fib-host binary
	RUST_BACKTRACE=1 ./target/release/fib-host
