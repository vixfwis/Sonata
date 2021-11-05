use x86::bits64::paging::{PAddr, VAddr};
pub mod paging;

#[derive(Debug)]
#[repr(C)]
pub struct BootInfo {
    pub mb2_boot_info: PAddr,
    pub kernel_phys_start: PAddr,
    pub kernel_phys_end: PAddr,
    pub kernel_virt_start: VAddr,
    pub kernel_virt_end: VAddr,
}
