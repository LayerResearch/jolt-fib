#![no_std]
#![no_main]

const HTIF_DEVICE_HALT: u64 = 0x0000; // Halt device
const HTIF_CMD_HALT: u64 = 0; // Command for HALT

#[unsafe(link_section = ".tohost")]
#[used]
#[unsafe(no_mangle)]
pub static mut tohost: u64 = 0;

#[unsafe(link_section = ".fromhost")]
#[used]
#[unsafe(no_mangle)]
pub static mut fromhost: u64 = 0;

#[inline(always)]
fn htif_send(device: u64, cmd: u64, payload: u64) {
    unsafe {
        core::ptr::write_volatile(
            &raw mut tohost,
            ((device & 0xFFFF) << 48) | ((cmd & 0xFFFF) << 32) | (payload & 0xFFFF_FFFF),
        );
    }
}

pub fn exit(code: u32) -> ! {
    htif_send(HTIF_DEVICE_HALT, HTIF_CMD_HALT, ((code << 1) | 1) as u64);
    loop {
        core::hint::spin_loop();
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    let _ = unsafe { core::ptr::read_volatile(&raw const fromhost) };

    // Call the user's main function and exit with its return value
    unsafe extern "C" {
        fn main() -> i32;
    }
    let exit_code: i32 = unsafe { main() };
    exit(exit_code as u32)
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    exit(1)
}

////////////////////////////////////////////////////////////
fn sum_sq(n: u32) -> u32 {
    let mut sum = 0;
    for i in 1..=n {
        sum += i * i;
    }
    sum
}

#[unsafe(no_mangle)]
pub extern "C" fn main() -> i32 {
    let sum = sum_sq(3);

    sum as i32
}
