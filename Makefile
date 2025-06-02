.PHONY: bootstrap help

help:
	@echo "Available targets:"
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-15s\033[0m %s\n", $$1, $$2}'

bootstrap: ## Install required dependencies
	rustup component add clippy rustfmt
	cargo install cargo-nextest

build-fib-host: ## Build the fib-host binary
	cargo build --release --package fib-host

run-fib-host: build-fib-host ## Run the fib-host binary
	RUST_BACKTRACE=1 ./target/release/fib-host

build-voj-host: ## Build the voj-host binary
	cargo build --release --package voj-host

run-voj-host: build-voj-host ## Run the voj-host binary
	RUST_BACKTRACE=1 ./target/release/voj-host

lint: ## Fix linting errors
	cargo clippy --fix --allow-dirty --allow-staged -- -D warnings
	cargo fmt --all --