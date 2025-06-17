#![no_std]
#![no_main]

// tohost symbol for termination via HTIF
#[no_mangle]
#[used]
static mut tohost: u64 = 0;

#[no_mangle]
#[used]
static mut fromhost: u64 = 0;

fn exit(code: i32) -> ! {
    unsafe {
        tohost = ((code as u64) << 1) | 1;
    }
    loop {}
}

// Panic handler (required for no_std)
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    exit(1)
}

extern "C" {
    static _STACK_PTR: usize;
}

#[no_mangle]
fn _start() -> ! {
    unsafe {
        core::arch::asm!("la sp, {stk}", stk = sym _STACK_PTR);
    }
    let result = main();
    exit(result)
}

#[no_mangle]
fn main() -> i32 {
    5
}
