#![feature(asm)]
#![feature(abi_x86_interrupt)]
#![feature(alloc_error_handler)]
#![no_std]
#![no_main]
#![allow(dead_code)]

extern crate alloc;

mod vga_buffer;
mod interrupts;
mod memory;

use core::panic::PanicInfo;
use x86::halt;
use x86::irq;
use x86::io::{inb, outb};
use x86::dtables;
use crate::interrupts::{InterruptDescriptorTable, InterruptStackFrame, PageFaultInfo, pic1_end_of_intr, remap_pic, set_pic1_mask};
use multiboot2;
use memory::BootInfo;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
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

extern "x86-interrupt" fn double_fault(frame: InterruptStackFrame, _err_code: u64) -> ! {
    panic!("double fault: {:?}", frame);
}

extern "x86-interrupt" fn page_fault(frame: InterruptStackFrame, err_code: u64) {
    let info = PageFaultInfo::from_err_code(err_code);
    let pf_addr = unsafe { x86::controlregs::cr2() } as *const ();
    panic!("page fault at {:p}, accessing {:p}: {:?}", frame.rip, pf_addr, info);
}

#[no_mangle]
pub extern "C" fn kmain(info: *const BootInfo) -> ! {
    unsafe {
        // disable blinking cursor
        outb(0x3D4, 0x0A);
        outb(0x3D5, 0x20);
        remap_pic();
        set_pic1_mask(0b_1111_1101);
        let mut idt = InterruptDescriptorTable::new();
        idt.keyboard.set_handler(kb_handler);
        idt.double_fault.set_handler(double_fault);
        idt.page_fault.set_handler(page_fault);
        let idt_ptr = dtables::DescriptorTablePointer{ limit: 256 * 16 - 1, base: &idt };
        dtables::lidt(&idt_ptr);
        irq::enable();

        memory::init_memory(info);

        println!("boot info: {:#?}", *info);
        let boot_info = multiboot2::load((*info).mb2_boot_info.into()).unwrap();
        for mem in boot_info.memory_map_tag().unwrap().memory_areas() {
            println!("addr: {:#08x}, len: {:#08x}, type: {:?}", mem.start_address(), mem.size(), mem.typ());
        }

        // page fault
        *(0xdeadbeef as *mut u64) = 0;

        loop {
            halt();
        }
        // panic!("kmain: end of function")
    }
}
