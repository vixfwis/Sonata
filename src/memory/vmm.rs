use x86::bits64::paging::PML4;

pub struct VirtualMemoryManager {
    pml4: *mut PML4
}
