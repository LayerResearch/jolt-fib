.PHONY: bootstrap help

help:
	@echo "Available targets:"
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-15s\033[0m %s\n", $$1, $$2}'

build-fib-host: ## Build the fib-host binary
	cargo build --release --package fib-host

run-fib-host: build-fib-host ## Run the fib-host binary
	RUST_BACKTRACE=1 ./target/release/fib-host
