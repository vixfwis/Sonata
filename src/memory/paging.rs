use core::ops::Index;
use x86::bits64::paging::PDPTFlags;
use x86::current::paging::{VAddr, PAddr, pml4_index, pdpt_index, pd_index, pt_index, PML4, PDPT, PD, PT, PDFlags};
use crate::memory::BootInfo;
use crate::println;

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

pub struct MemoryManager {
}

impl MemoryManager {
    pub unsafe fn new(info: &BootInfo) -> Self {
        let mut pml4 = &mut *(x86::controlregs::cr3() as *mut PML4);
        for (i4, e4) in pml4.iter_mut().enumerate() {
            if e4.is_present() {
                println!("pml4 {} paddr: {:?}", i4, e4.address());
                let mut pdpt = &mut *(e4.address().0 as *mut PDPT);
                for (i3, e3) in pdpt.iter_mut().enumerate() {
                    if e3.is_present() {
                        if e3.flags().contains(PDPTFlags::PS) {
                            println!("pdpt {} PS paddr: {:?}", i3, e3.address());
                        }else {
                            println!("pdpt {} paddr: {:?}", i3, e3.address());
                            let mut pd = &mut *(e3.address().0 as *mut PD);
                            for (i2, e2) in pd.iter_mut().enumerate() {
                                if e2.is_present() {
                                    if e2.flags().contains(PDFlags::PS) {
                                        let va = VAddr(sign_extend_48((
                                            i4 << PML4_VA_SHIFT |
                                            i3 << PDPT_VA_SHIFT |
                                            i2 << PD_VA_SHIFT) as u64)
                                        );
                                        println!("pd {} PS paddr: {:?} vaddr: {:?}", i2, e2.address(), va);
                                    }else {
                                        println!("pd {} paddr: {:?}", i2, e2.address());
                                        let mut pt = &mut *(e2.address().0 as *mut PT);
                                        for (i1, e1) in pt.iter_mut().enumerate() {
                                            println!("pt {} paddr: {:?}", i1, e1.address());
                                        }
                                    }
                                }
                            }
                        }

                    }
                }
            }
        }
        MemoryManager {}
    }
}