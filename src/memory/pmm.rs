use alloc::vec::Vec;
use core::ops::Range;

// const BLOCK_SIZE: [usize; 6] = [0x1000, 0x2000, 0x4000, 0x8000, 0x10000, 0x20000];

struct PMemBlock {
    addr: usize,
    size: usize,
    in_use: bool
}

pub struct PhysicalMemoryManager {
    free_list: Range<usize>,
    blocks: Vec<PMemBlock>
}

impl PhysicalMemoryManager {
    pub fn new() -> Self {
        PhysicalMemoryManager {
            free_list: 0..0,
            blocks: Vec::new()
        }
    }

    pub fn set_range(&mut self, range: Range<usize>) {
        self.free_list = range;
    }
}