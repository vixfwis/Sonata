#![no_std]
#![no_main]

use core::panic::PanicInfo;
use x86::halt;
use x86::io::outb;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {loop {}}

static HELLO: &[u8] = b"Hello World!";

fn disable_cursor() {
    unsafe {
        outb(0x3D4, 0x0A);
        outb(0x3D5, 0x20);
    }
}

#[no_mangle]
pub extern "C" fn kmain() -> ! {
    disable_cursor();
    let vga_buffer = 0xb8000 as *mut u8;

    for (i, &byte) in HELLO.iter().enumerate() {
        unsafe {
            *vga_buffer.offset(i as isize * 2) = byte;
            *vga_buffer.offset(i as isize * 2 + 1) = 0xf;
        }
    }

    loop {
        unsafe { halt(); }
    }
}
