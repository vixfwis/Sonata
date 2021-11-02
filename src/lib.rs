#![feature(asm)]
#![feature(abi_x86_interrupt)]
#![no_std]
#![no_main]
#![allow(dead_code)]

mod vga_buffer;
mod interrupts;
mod memory;

use core::panic::PanicInfo;
use x86::halt;
use x86::irq;
use x86::io::{inb, outb};
use x86::dtables;
use crate::interrupts::{InterruptDescriptorTable, remap_pic, set_pic1_mask, InterruptStackFrame, pic1_end_of_intr};

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{:#?}", info);
    unsafe {
        irq::disable();
        loop {
            halt();
        }
    }
}

extern "x86-interrupt" fn kb_handler(_frame: InterruptStackFrame) {
    let scan_code = unsafe {inb(0x60)};
    println!("{:#02X}", scan_code);
    pic1_end_of_intr();
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
    set_pic1_mask(0b_1111_1101);
    let mut idt = InterruptDescriptorTable::new();
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
