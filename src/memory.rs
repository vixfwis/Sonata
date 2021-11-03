use core::fmt::{Debug, Formatter};
use core::ops::Deref;

#[derive(Copy, Clone, PartialEq)]
#[repr(transparent)]
pub struct VirtAddress(usize);

impl Debug for VirtAddress {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:p}", self.0 as *const Self)
    }
}

impl Into<usize> for VirtAddress {
    fn into(self) -> usize {
        self.0
    }
}

impl Into<u64> for VirtAddress {
    fn into(self) -> u64 {
        self.0 as u64
    }
}