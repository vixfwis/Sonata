use core::fmt::Write;
use x86::bits64::paging::VAddr;

const VGA_VADDR: VAddr = VAddr(0xFFFF8000000B8000u64);
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
struct VGABuffer {
    chars: [VGAChar; SCREEN_WIDTH * SCREEN_HEIGHT]
}

pub struct VGAWriter {
    offset: usize,
    color: VGAColorCode,
    buf: *mut VGABuffer,
}

impl VGAWriter {
    pub fn new(x: usize, y: usize) -> Self {
        VGAWriter {
            offset: x + y * SCREEN_WIDTH,
            color: VGAColorCode::new(VGAColor::White, VGAColor::Black),
            buf: VGA_VADDR.as_mut_ptr()
        }
    }

    fn newline(&mut self) {
        self.offset -= self.offset % SCREEN_WIDTH;
        self.offset += SCREEN_WIDTH;
        if self.offset == SCREEN_WIDTH * SCREEN_HEIGHT {
            self.offset -= SCREEN_WIDTH;
            unsafe {
                (*self.buf).chars.copy_within(SCREEN_WIDTH..(SCREEN_WIDTH * SCREEN_HEIGHT), 0);
                let space = VGAChar { char: 0x20, color: self.color };
                for e in &mut (*self.buf).chars[SCREEN_WIDTH*(SCREEN_HEIGHT-1)..SCREEN_WIDTH*SCREEN_HEIGHT] {
                    *e = space;
                }
            }
        }
    }

    fn write_byte(&mut self, b: u8) {
        unsafe {
            match b {
                b'\n' => self.newline(),
                b => {
                    (*self.buf).chars[self.offset] = VGAChar { char: b, color: self.color };
                    self.offset += 1;
                }
            }
        }
    }

    pub fn print(&mut self, s: &str) {
        for b in s.bytes() {
            match b {
                0x20..=0x7E | b'\n' => self.write_byte(b),
                _ => self.write_byte(219) // square block
            }
        }
    }

    pub fn println(&mut self, s: &str) {
        self.print(s);
        self.print("\n");
    }
}

impl Write for VGAWriter {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.print(s);
        Ok(())
    }
}
