#![feature(asm)]
#![feature(abi_x86_interrupt)]
#![no_std]
#![no_main]

mod vga_buffer;
mod interrupts;

use core::panic::PanicInfo;
use x86::halt;
use x86::irq;
use x86::io::outb;
use x86::dtables;
use lazy_static::lazy_static;
use spin::Mutex;
use crate::interrupts::{InterruptDescriptorTable, remap_pic, set_pic1_mask, InterruptStackFrame};

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{:?}", info);
    unsafe {
        irq::disable();
        loop {
            halt();
        }
    }
}

extern "x86-interrupt" fn kb_handler(frame: InterruptStackFrame) {
    println!("kb: {:?}", frame);
}

#[no_mangle]
pub extern "C" fn kmain() -> ! {
    // disable blinking cursor
    unsafe {
        outb(0x3D4, 0x0A);
        outb(0x3D5, 0x20);
    }
    println!("test1");
    println!("test2");

    remap_pic();
    set_pic1_mask(0b_1111_1101);  // only keyboard interrupt can go through
    let mut idt = InterruptDescriptorTable::new();
    println!("IDT is at {:p}", &idt);
    idt.keyboard.set_handler(kb_handler);
    let idt_ptr = dtables::DescriptorTablePointer{ limit: 256 * 16 - 1, base: &idt };
    unsafe {
        dtables::lidt(&idt_ptr);
        irq::enable();
        loop {
            halt()
        }
    }
    panic!("kmain: end of function")
}
