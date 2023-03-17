#![feature(asm)]
#![feature(abi_x86_interrupt)]
#![feature(alloc_error_handler)]
#![feature(const_mut_refs)]
#![no_std]
#![no_main]
#![allow(dead_code)]

extern crate alloc;

mod vga_buffer;
mod interrupts;
mod memory;

use alloc::format;
use core::fmt::Write;
use core::panic::PanicInfo;
use x86::halt;
use x86::irq;
use x86::io::{inb, outb};
use x86::dtables;
use crate::interrupts::{InterruptDescriptorTable, InterruptStackFrame, PageFaultInfo, pic1_end_of_intr, remap_pic, set_pic1_mask, set_pic2_mask};
use multiboot2;
use memory::BootInfo;
use crate::vga_buffer::{VGABuffer, VGAColor, VGAColorCode, VGAWriter};

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    // println macro uses spinlock and global writer,
    // so we can't call println here - it's going to triple failt and boot loop
    // if something goes wrong. instead just pick a place and print panic info
    unsafe {
        irq::disable();
        let mut writer = VGAWriter::new(unsafe { &mut *((0xFFFF8000000B8000u64 + 80 * 20) as *mut VGABuffer) },
                                        VGAColorCode::new(VGAColor::White, VGAColor::Black));
        writer.write_fmt(format_args!("{}", info)).unwrap();
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

extern "x86-interrupt" fn gp_fault(frame: InterruptStackFrame, _err_code: u64) {
    panic!("gp fault: {:?}", frame);
}

extern "x86-interrupt" fn invalid_opcode_fault(frame: InterruptStackFrame) {
    panic!("invalid opcode: {:?}", frame);
}

extern "x86-interrupt" fn page_fault(frame: InterruptStackFrame, err_code: u64) {
    let info = PageFaultInfo::from_err_code(err_code);
    let pf_addr = unsafe { x86::controlregs::cr2() } as *const ();
    panic!("page fault at {:p}, accessing {:p}: {:?}", frame.rip, pf_addr, info);
}

// Memory layout:
// 0x200000 - 0x201000          MB2 header and bootstrap assembly from boot.S
// 0x201000 - 0x201800          GDT
// 0x201800 - 0x201806          GDT size/addr for lgdt ins
// 0x201810 - 0x202000          BootInfo and free space up to next page
// 0x202000 - 0x203000          PML4
// 0x203000 - 0x204000          PDPT for low addresses
// 0x204000 - 0x205000          PDPT for high addresses
// 0x205000 - 0x206000          PD for low addresses
// 0x206000 - 0x207000          PD for high addresses
// 0x207000 - 0x217000          kernel stack (also info->kp_start)
// 0x217000 - info->kp_end      kernel code
// somewhere at info->mb2       multiboot2 info from bootloader

// low PD is mapping 0x0 to 2GiB
// high PD is mirroring low PD, with 0xFFFF800000000000 offset

#[no_mangle]
pub extern "C" fn kmain(info: *mut BootInfo) -> ! {
    unsafe {
        // disable blinking cursor
        outb(0x3D4, 0x0A);
        outb(0x3D5, 0x20);
        remap_pic();
        set_pic1_mask(0b_1111_1111);
        set_pic2_mask(0b_1111_1111);
        let mut idt = InterruptDescriptorTable::new();
        idt.keyboard.set_handler(kb_handler);
        idt.double_fault.set_handler(double_fault);
        idt.general_protection_fault.set_handler(gp_fault);
        idt.invalid_opcode.set_handler(invalid_opcode_fault);
        idt.page_fault.set_handler(page_fault);
        let idt_ptr = dtables::DescriptorTablePointer{ limit: 256 * 16 - 1, base: &idt };
        dtables::lidt(&idt_ptr);
        irq::enable();

        memory::init_memory(info);

        // page fault
        // *(0xdeadbeef as *mut u64) = 0;

        panic!("kmain: end of function")
    }
}
