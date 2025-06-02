# Jolt Guest Helper Examples

This directory contains example programs demonstrating how to use the `jolt-guest-helper` library.

## Available Examples

- `fib.rs`: Demonstrates how to use the guest helper with a Fibonacci function

## Building and Running Examples

To build an example:
```bash
cargo build --example <example-name>
```

To run an example:
```bash
cargo run --example <example-name>
```

For example, to run the Fibonacci example:
```bash
cargo run --example fib
```

## Example Structure

Each example demonstrates different aspects of the guest helper library:

- `fib.rs`: Shows basic usage of the guest helper with a simple Fibonacci function
  - Creating a guest program
  - Configuring memory and stack sizes
  - Compiling the program
  - Proving and verifying execution 