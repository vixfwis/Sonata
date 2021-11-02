#![no_std]
#![no_main]

mod vga_buffer;

use core::panic::PanicInfo;
use x86::halt;
use x86::io::outb;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {loop {}}

#[no_mangle]
pub extern "C" fn kmain() -> ! {
    // disable blinking cursor
    unsafe {
        outb(0x3D4, 0x0A);
        outb(0x3D5, 0x20);
    }
    println!("test1");
    println!("test2");
    println!("test3");

    loop {
        unsafe { halt(); }
    }
}
