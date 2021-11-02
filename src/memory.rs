use core::fmt::{Debug, Formatter};

#[derive(Copy, Clone, PartialEq)]
#[repr(transparent)]
pub struct VirtualAddress(u64);

impl Debug for VirtualAddress {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:p}", self.0 as *const Self)
    }
}