use x86::bits64::paging::{PAddr, PD, PDFlags, PDPT, PDPTFlags, PML4, pml4_index, PML4Entry, PT, VAddr};
use x86::dtables::DescriptorTablePointer;
use crate::println;
pub mod vmm;
pub mod pmm;
mod heap;

const BOOTSTRAP_OFFSET: usize = 0x300000;
const PML4_VA_SHIFT: usize = 39;
const PDPT_VA_SHIFT: usize = 30;
const PD_VA_SHIFT: usize = 21;
const PT_VA_SHIFT: usize = 12;

fn sign_extend_48(addr: u64) -> u64 {
    if addr > 0x00007FFFFFFFFFFF {
        addr | 0xFFFF000000000000
    }else {
        addr & 0x0000FFFFFFFFFFFF
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct BootInfo {
    pub mb2_boot_info: PAddr,
    pub kernel_phys_start: PAddr,
    pub kernel_phys_end: PAddr,
    pub kernel_virt_start: VAddr,
    pub kernel_virt_end: VAddr,
}

pub unsafe fn init_memory(info: *const BootInfo) {
    // todo: setup memory somehow for heap allocation \ o /
}