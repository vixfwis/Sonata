use core::fmt::Write;
use spin::Mutex;
use lazy_static::lazy_static;
use crate::interrupts::without_interrupts;

lazy_static! {
    pub static ref WRITER: Mutex<VGAWriter> = Mutex::new(VGAWriter::new(
        unsafe {&mut *(0xb8000 as *mut VGABuffer)},
        VGAColorCode::new(VGAColor::White, VGAColor::Black)
    ));
}

const SCREEN_WIDTH: usize = 80;
const SCREEN_HEIGHT: usize = 25;

#[repr(u8)]
pub enum VGAColor {
    Black = 0x0,
    Blue = 0x1,
    Green = 0x2,
    Cyan = 0x3,
    Red = 0x4,
    Magenta = 0x5,
    Brown = 0x6,
    LightGray = 0x7,
    DarkGray = 0x8,
    LightBlue = 0x9,
    LightGreen = 0xa,
    LightCyan = 0xb,
    LightRed = 0xc,
    LightMagenta = 0xd,
    Yellow = 0xe,
    White = 0xf,
}

#[derive(Copy, Clone)]
pub struct VGAColorCode(u8);

impl VGAColorCode {
    pub fn new(fg: VGAColor, bg: VGAColor) -> Self {
        VGAColorCode((bg as u8) << 4 | (fg as u8))
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
struct VGAChar {
    char: u8,
    color: VGAColorCode
}

#[repr(transparent)]
pub struct VGABuffer {
    chars: [VGAChar; SCREEN_WIDTH * SCREEN_HEIGHT]
}

pub struct VGAWriter {
    offset: usize,
    color: VGAColorCode,
    buf: &'static mut VGABuffer
}

impl VGAWriter {
    pub fn new(buf: &'static mut VGABuffer, color: VGAColorCode) -> Self {
        VGAWriter {offset: 0, color, buf}
    }

    pub fn write_byte(&mut self, b: u8) {
        match b {
            b'\n' => self.newline(),
            b => {
                self.buf.chars[self.offset] = VGAChar {char: b, color: self.color};
                self.offset += 1;
            }
        }
    }

    pub fn write_string(&mut self, s: &str) {
        for b in s.bytes() {
            match b {
                0x20..=0x7E | b'\n' => self.write_byte(b),
                _ => self.write_byte(219) // square block
            }
        }
    }

    pub fn newline(&mut self) {
        self.offset -= self.offset % SCREEN_WIDTH;
        self.offset += SCREEN_WIDTH;
        if self.offset == SCREEN_WIDTH * SCREEN_HEIGHT {
            self.offset -= SCREEN_WIDTH;
            self.buf.chars.copy_within(SCREEN_WIDTH..(SCREEN_WIDTH * SCREEN_HEIGHT), 0);
            let space = VGAChar { char: 0x20, color: self.color };
            for e in &mut self.buf.chars[SCREEN_WIDTH*(SCREEN_HEIGHT-1)..SCREEN_WIDTH*SCREEN_HEIGHT] {
                *e = space;
            }
        }
    }
}

impl Write for VGAWriter {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::vga_buffer::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _print(args: core::fmt::Arguments) {
    without_interrupts(|| {
        WRITER.lock().write_fmt(args).unwrap();
    });
}
