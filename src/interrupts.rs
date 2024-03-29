use core::fmt::Debug;
use core::marker::PhantomData;
use x86::io::{inb, outb};
use x86::segmentation;

const GATE_TYPE_INTERRUPT: u8 = 0xE;
const GATE_TYPE_TRAP: u8 = 0xF;

#[repr(u8)]
pub enum InterruptGateType {
    Interrupt,
    Trap,
}

#[derive(Debug)]
#[repr(C)]
pub struct InterruptStackFrame {
    pub rip: *const (),
    pub cs: u64,
    pub flags: u64,
    pub rsp: *const (),
    pub ss: u64,
}

pub type IntHandler = extern "x86-interrupt" fn(frame: InterruptStackFrame);
pub type IntHandlerWithErrCode = extern "x86-interrupt" fn(frame: InterruptStackFrame, err_code: u64);
pub type IntHandlerDvrgWithErrCode = extern "x86-interrupt" fn(frame: InterruptStackFrame, err_code: u64) -> !;

#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(C)]
pub struct IDTEntryOptions {
    ist: u8,
    type_attr: u8
}

impl IDTEntryOptions {
    pub fn new(gt: InterruptGateType, present: bool) -> Self {
        let mut opt = IDTEntryOptions {ist: 0, type_attr: 0};
        opt.set_gate_type(gt);
        opt.set_present(present);
        opt
    }

    pub fn set_ist_offset(&mut self, offset: u8) {
        self.ist |= offset & 0b_00000111;
    }

    pub fn get_ist_offset(&self) -> u8 {
        self.ist & 0b_00000111
    }

    pub fn set_present(&mut self, present: bool) {
        if present {
            self.type_attr |= 0b_10000000;
        }else {
            self.type_attr &= !0b_10000000;
        }
    }

    pub fn get_present(&self) -> bool {
        (self.type_attr & 0b_10000000) > 0
    }

    pub fn set_dpl(&mut self, dpl: u8) {
        self.type_attr |= (dpl << 5) & 0b_01100000;
    }

    pub fn get_dpl(&self) -> u8 {
        (self.type_attr & 0b_01100000) >> 5
    }

    pub fn set_gate_type(&mut self, gt: InterruptGateType) {
        self.type_attr &= 0b_11110000;
        match gt {
            InterruptGateType::Interrupt => { self.type_attr |= GATE_TYPE_INTERRUPT }
            InterruptGateType::Trap => { self.type_attr |= GATE_TYPE_TRAP }
        }
    }

    pub fn get_gate_type(&self) -> InterruptGateType {
        let gate_type = self.type_attr & 0b_00001111;
        match gate_type {
            GATE_TYPE_INTERRUPT => InterruptGateType::Interrupt,
            GATE_TYPE_TRAP => InterruptGateType::Trap,
            _ => panic!("unexpected gate type {}", gate_type)
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(C)]
pub struct IDTEntry<F> {
    offset_1: u16,
    selector: u16,
    options: IDTEntryOptions,
    offset_2: u16,
    offset_3: u32,
    zero: u32,
    phantom: PhantomData<F>
}

impl<F> IDTEntry<F> {
    pub fn missing() -> Self {
        let options = IDTEntryOptions::new(InterruptGateType::Interrupt, false);
        IDTEntry {
            offset_1: 0,
            selector: 0,
            options,
            offset_2: 0,
            offset_3: 0,
            zero: 0,
            phantom: PhantomData
        }
    }

    fn set_handler_address(&mut self, address: u64) {
        self.offset_1 = address as u16;
        self.offset_2 = (address >> 16) as u16;
        self.offset_3 = (address >> 32) as u32;
        self.selector = segmentation::cs().bits();
        self.options.set_present(true);
    }
}

impl IDTEntry<IntHandler> {
    pub fn set_handler(&mut self, handler: IntHandler) {
        self.set_handler_address(handler as u64);
    }
}

impl IDTEntry<IntHandlerWithErrCode> {
    pub fn set_handler(&mut self, handler: IntHandlerWithErrCode) {
        self.set_handler_address(handler as u64);
    }
}

impl IDTEntry<IntHandlerDvrgWithErrCode> {
    pub fn set_handler(&mut self, handler: IntHandlerDvrgWithErrCode) {
        self.set_handler_address(handler as u64);
    }
}

#[repr(C)]
pub struct InterruptDescriptorTable {
    // exceptions
    pub divide_by_zero: IDTEntry<IntHandler>,
    pub debug: IDTEntry<IntHandler>,
    pub non_maskable_interrupt: IDTEntry<IntHandler>,
    pub breakpoint: IDTEntry<IntHandler>,
    pub overflow: IDTEntry<IntHandler>,
    pub bound_range_exceeded: IDTEntry<IntHandler>,
    pub invalid_opcode: IDTEntry<IntHandler>,
    pub device_not_available: IDTEntry<IntHandler>,
    pub double_fault: IDTEntry<IntHandlerDvrgWithErrCode>,
    reserved_exceptions_9: IDTEntry<IntHandler>,
    pub invalid_tss: IDTEntry<IntHandlerWithErrCode>,
    pub segment_not_present: IDTEntry<IntHandlerWithErrCode>,
    pub stack_segment_fault: IDTEntry<IntHandlerWithErrCode>,
    pub general_protection_fault: IDTEntry<IntHandlerWithErrCode>,
    pub page_fault: IDTEntry<IntHandlerWithErrCode>,
    reserved_exceptions_15: IDTEntry<IntHandler>,
    pub x87_floating_point: IDTEntry<IntHandler>,
    pub alignment_check: IDTEntry<IntHandlerWithErrCode>,
    pub machine_check: IDTEntry<IntHandler>,
    pub simd_floating_point: IDTEntry<IntHandler>,
    pub virtualization: IDTEntry<IntHandler>,
    reserved_exceptions_21_29: [IDTEntry<IntHandler>; 9],
    pub security_exception: IDTEntry<IntHandlerWithErrCode>,
    reserved_exceptions_31: IDTEntry<IntHandler>,

    // IRQs
    pub programmable_timer: IDTEntry<IntHandler>,
    pub keyboard: IDTEntry<IntHandler>,
    pub reserved_irq2_cascade: IDTEntry<IntHandler>,
    pub serial2: IDTEntry<IntHandler>,
    pub serial1: IDTEntry<IntHandler>,
    pub parallel2: IDTEntry<IntHandler>,
    pub floppy: IDTEntry<IntHandler>,
    pub parallel1_spurious: IDTEntry<IntHandler>,
    pub cmos_rtc: IDTEntry<IntHandler>,
    pub peripherals_1: IDTEntry<IntHandler>,
    pub peripherals_2: IDTEntry<IntHandler>,
    pub peripherals_3: IDTEntry<IntHandler>,
    pub ps2_mouse: IDTEntry<IntHandler>,
    pub fpu: IDTEntry<IntHandler>,
    pub primary_ata: IDTEntry<IntHandler>,
    pub secondary_ata: IDTEntry<IntHandler>,

    // everything else
    interrupts: [IDTEntry<IntHandler>; 256 - 32 - 16]
}

impl InterruptDescriptorTable {
    pub fn new() -> Self {
        InterruptDescriptorTable {
            divide_by_zero: IDTEntry::missing(),
            debug: IDTEntry::missing(),
            non_maskable_interrupt: IDTEntry::missing(),
            breakpoint: IDTEntry::missing(),
            overflow: IDTEntry::missing(),
            bound_range_exceeded: IDTEntry::missing(),
            invalid_opcode: IDTEntry::missing(),
            device_not_available: IDTEntry::missing(),
            double_fault: IDTEntry::missing(),
            reserved_exceptions_9: IDTEntry::missing(),
            invalid_tss: IDTEntry::missing(),
            segment_not_present: IDTEntry::missing(),
            stack_segment_fault: IDTEntry::missing(),
            general_protection_fault: IDTEntry::missing(),
            page_fault: IDTEntry::missing(),
            reserved_exceptions_15: IDTEntry::missing(),
            x87_floating_point: IDTEntry::missing(),
            alignment_check: IDTEntry::missing(),
            machine_check: IDTEntry::missing(),
            simd_floating_point: IDTEntry::missing(),
            virtualization: IDTEntry::missing(),
            reserved_exceptions_21_29: [IDTEntry::missing(); 9],
            security_exception: IDTEntry::missing(),
            reserved_exceptions_31: IDTEntry::missing(),

            programmable_timer: IDTEntry::missing(),
            keyboard: IDTEntry::missing(),
            reserved_irq2_cascade: IDTEntry::missing(),
            serial2: IDTEntry::missing(),
            serial1: IDTEntry::missing(),
            parallel2: IDTEntry::missing(),
            floppy: IDTEntry::missing(),
            parallel1_spurious: IDTEntry::missing(),
            cmos_rtc: IDTEntry::missing(),
            peripherals_1: IDTEntry::missing(),
            peripherals_2: IDTEntry::missing(),
            peripherals_3: IDTEntry::missing(),
            ps2_mouse: IDTEntry::missing(),
            fpu: IDTEntry::missing(),
            primary_ata: IDTEntry::missing(),
            secondary_ata: IDTEntry::missing(),

            interrupts: [IDTEntry::missing(); 256 - 32 - 16]
        }
    }
}

#[derive(Debug)]
pub struct PageFaultInfo {
    present: bool,
    write: bool,
    user: bool,
    reserved_write: bool,
    instr_fetch: bool,
    protection: bool,
    shadow_stack: bool,
    software_guard_ext: bool,
}

impl PageFaultInfo {
    pub fn from_err_code(code: u64) -> Self {
        PageFaultInfo {
            present: code & (1 << 0) > 0,
            write: code & (1 << 1) > 0,
            user: code & (1 << 2) > 0,
            reserved_write: code & (1 << 3) > 0,
            instr_fetch: code & (1 << 4) > 0,
            protection: code & (1 << 5) > 0,
            shadow_stack: code & (1 << 6) > 0,
            software_guard_ext: code & (1 << 15) > 0,
        }
    }
}

pub fn without_interrupts<F>(func: F)
    where F: Fn() {
    unsafe {
        x86::irq::disable();
        func();
        x86::irq::enable();
    }
}
