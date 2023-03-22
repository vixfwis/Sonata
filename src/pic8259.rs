use x86::io::{inb, outb};

// https://wiki.osdev.org/PIC

const PIC1_CMD: u16  = 0x20;  // command port
const PIC2_CMD: u16  = 0xA0;
const PIC1_DATA: u16 = 0x21;  // data port
const PIC2_DATA: u16 = 0xA1;

pub fn remap_pic() {
    unsafe {
        // initialization mode: expects 3 writes
        outb(PIC1_CMD, 0x11);
        outb(PIC2_CMD, 0x11);
        // vector offset
        outb(PIC1_DATA, 0x20);
        outb(PIC2_DATA, 0x28);
        // tell master PIC about slave PIC at IRQ2
        outb(PIC1_DATA, 0b_00000100);
        // tell slave PIC about its cascade identity
        outb(PIC2_DATA, 0b_00000010);
        // 8086 mode
        outb(PIC1_DATA, 0x01);
        outb(PIC2_DATA, 0x01);

        // mask all
        outb(PIC1_DATA, 0b1111_1111);
        outb(PIC2_DATA, 0b1111_1111);
    }
}

pub fn set_pic_irq_line(irq_line: u8) {
    let mut value = irq_line;
    let port = match value < 8 {
        true => PIC1_DATA,
        false => {
            value -= 8;
            PIC2_DATA
        }
    };
    unsafe {
        value = inb(port) | (1 << value);
        outb(port, value);
    }
}

pub fn clear_pic_iqr_line(irq_line: u8) {
    let mut value = irq_line;
    let port = match value < 8 {
        true => PIC1_DATA,
        false => {
            value -= 8;
            PIC2_DATA
        }
    };
    unsafe {
        value = inb(port) & !(1 << value);
        outb(port, value);
    }
}

pub fn set_pic1_mask(mask: u8) {
    unsafe {
        outb(PIC1_DATA, mask);
    }
}

pub fn set_pic2_mask(mask: u8) {
    unsafe {
        outb(PIC2_DATA, mask);
    }
}

pub fn pic1_end_of_intr() {
    unsafe {
        outb(PIC1_CMD, 0x20);
    }
}

pub fn pic2_end_of_intr() {
    unsafe {
        outb(PIC2_CMD, 0x20);
    }
}
