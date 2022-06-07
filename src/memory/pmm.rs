use alloc::vec::Vec;
use core::ops::Range;

pub struct PMM {
    zone: Range<u64>,
    pages: Vec<u64>
}

impl PMM {
    pub fn new(zone: Range<u64>) -> Self {
        Self {
            zone,
            pages: Vec::new()
        }
    }
}