.section .text
.global _start

_start:
    # Load stack pointer and ensure 16-byte alignment for rv64gc ABI compliance
    la sp, _STACK_PTR
    andi sp, sp, ~15        # Clear lower 4 bits for 16-byte alignment
    
    # Call main function
    call main
    
    # main() returned, now exit with return value in a0
    # pk (proxy kernel) provides exit syscall support
    li a7, 93              # SYS_exit syscall number
    ecall                  # System call to exit
