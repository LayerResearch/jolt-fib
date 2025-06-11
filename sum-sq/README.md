Demonstrates how to build an ELF for target riscv32im-unknown-none-elf which can be run in Spike simulator.

Key requirements for HTIF (Host-Target Interface) communication:

1. `#[no_mangle]` on tohost and fromhost symbols makes them available to Spike 
   with exact symbol names (prevents Rust name mangling)

2. `#[used]` tells the compiler not to eliminate symbols during dead code elimination,
   even if they appear unused in the Rust code

3. However, if the two variables are not used in program logic, we need to use 
   `#[link_section = ".tohost"]` to place symbols in specific sections, and need 
   `KEEP(*(.tohost))` in linker.ld to ensure the linker will not garbage-collect 
   these sections during final linking phase

4. If the variables are used in the program (e.g., writing to tohost), `#[used]`, `#[link_section = ".tohost"]` 
   and `KEEP(*(.tohost))` are technically not needed since referenced symbols 
   won't be eliminated, but they're still recommended for robustness

Additional technical details:
- Spike looks for tohost/fromhost symbols in specific sections for HTIF protocol
- Without proper section placement, Spike shows "tohost and fromhost symbols not in ELF"
- The linker script ENTRY(_start) is required for bare-metal RISC-V execution
- Custom memory layout (ORIGIN = 0x80000000) matches Spike's expected address space
- Direct assignment `tohost = value` works as well as `write_volatile()` for HTIF

Testing shows this configuration successfully runs sum-of-squares calculation 
and communicates results through HTIF to the Spike simulator.
