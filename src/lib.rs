#![feature(asm)]
#![feature(abi_x86_interrupt)]
#![no_std]
#![no_main]
#![allow(dead_code)]

mod vga_buffer;
mod interrupts;
mod memory;

use core::fmt::{Arguments, Debug, Formatter};
use core::panic::PanicInfo;
use x86::halt;
use x86::irq;
use x86::io::{inb, outb};
use x86::dtables;
use crate::interrupts::{InterruptDescriptorTable, remap_pic, set_pic1_mask, InterruptStackFrame, pic1_end_of_intr, PageFaultInfo};
use multiboot2;
use crate::memory::VirtAddress;

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
    let pf_addr = unsafe { x86::controlregs::cr2() } as *const u8;
    panic!("page fault at {:p}, accessing {:p}: {:?}", frame.rip, pf_addr, info);
}

#[derive(Debug)]
#[repr(C)]
pub struct BootInfo {
    mb2_boot_info: VirtAddress,
    kernel_stack_start: VirtAddress,
    kernel_stack_end: VirtAddress,
    kernel_phys_start: VirtAddress,
    kernel_phys_end: VirtAddress,
    kernel_virt_start: VirtAddress,
    kernel_virt_end: VirtAddress,
}

#[no_mangle]
pub extern "C" fn kmain(info: &BootInfo) -> ! {
    // disable blinking cursor
    unsafe {
        outb(0x3D4, 0x0A);
        outb(0x3D5, 0x20);
    }
    remap_pic();
    set_pic1_mask(0b_1111_1101);
    let mut idt = InterruptDescriptorTable::new();
    idt.keyboard.set_handler(kb_handler);
    idt.double_fault.set_handler(double_fault);
    idt.page_fault.set_handler(page_fault);
    let idt_ptr = dtables::DescriptorTablePointer{ limit: 256 * 16 - 1, base: &idt };
    unsafe {
        dtables::lidt(&idt_ptr);
        irq::enable();
    }

    println!("boot info: {:#?}", info);
    let boot_info = unsafe { multiboot2::load(info.mb2_boot_info.into()).unwrap() };
    for mem in boot_info.memory_map_tag().unwrap().memory_areas() {
        println!("addr: {:#08x}, len: {:#08x}, type: {:?}", mem.start_address(), mem.size(), mem.typ());
    }

    // page fault
    unsafe {
        *(0xdeadbeef as *mut u64) = 0;
    }

    loop {
        unsafe { halt() };
    }
    panic!("kmain: end of function")
}
