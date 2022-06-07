use core::alloc::{GlobalAlloc, Layout};
use core::mem::{size_of, zeroed};
use core::ops::Add;
use core::ptr::NonNull;
use x86::bits64::paging;
use paging::{PAddr, PML4, PML4Entry, pml4_index, PML4Flags, VAddr, PML4_SLOT_SIZE};
use x86::bits32::paging::BASE_PAGE_SIZE;
use x86::bits64::paging::{LARGE_PAGE_SIZE, PD, PDEntry, PDFlags, PDPT, pdpt_index, PDPTEntry, PDPTFlags};
use x86::dtables::DescriptorTablePointer;
use crate::println;
pub mod vmm;
pub mod pmm;
mod heap;

const HIGHER_HALF: u64 = 0xFFFF800000000000;

fn sign_extend_48(addr: u64) -> u64 {
    if addr > 0x00007FFFFFFFFFFF {
        addr | 0xFFFF000000000000
    }else {
        addr & 0x0000FFFFFFFFFFFF
    }
}

fn align_up(addr: u64, align: u64) -> u64 {
    let align_mask = align - 1;
    if addr & align_mask == 0 {
        addr
    } else {
        (addr | align_mask) + 1
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct BootInfo {
    // mb2 boot info
    pub mb2: VAddr,
    // bootstrap GDT
    pub gdt: PAddr,
    // kernel physical start
    pub kp_start: PAddr,
    // kernel physical end
    pub kp_end: PAddr,
    // kernel virtual start
    pub kv_start: VAddr,
    // kernel virtual end
    pub kv_end: VAddr,
}

unsafe fn relocate_mb2_at_addr(info: *mut BootInfo, addr: VAddr) {
    // read MB2 header wherever it is currently
    let boot_info = multiboot2::load((*info).mb2.into()).unwrap();
    let mb2_total_size = boot_info.total_size();
    // copy the whole thing and edit its pointer
    core::ptr::copy((*info).mb2.as_usize() as *const u8, addr.0 as *mut u8, mb2_total_size);
    (*info).mb2 = addr;
}

pub unsafe fn init_memory(info: *mut BootInfo) {
    // move MB2 header to fixed location
    relocate_mb2_at_addr(info, (*info).kv_end);
    let boot_info = multiboot2::load((*info).mb2.into()).unwrap();
    // println!("boot info: {:#?}", *info);
    // println!("boot info size: {:#08x}", boot_info.total_size());
    // for mem in boot_info.memory_map_tag().unwrap().memory_areas() {
    //     println!("addr: {:#08x}, len: {:#08x}, type: {:?}", mem.start_address(), mem.size(), mem.typ());
    // }
    let pml4_pa = PAddr(0x100000);
    let pdpt_pa = PAddr(0x101000);
    let pd_pa = PAddr(0x102000);

    core::ptr::write(pml4_pa.as_u64() as *mut PML4, zeroed());
    let pml4 = &mut *(pml4_pa.as_u64() as *mut PML4);

    // 512 GiB slot
    pml4[pml4_index((*info).kv_start)] = PML4Entry::new(
        pdpt_pa,
        PML4Flags::P | PML4Flags::RW
    );
    // recursive paging
    pml4[511] = PML4Entry::new(
        pml4_pa,
        PML4Flags::P
    );

    core::ptr::write(pdpt_pa.as_u64() as *mut PDPT, zeroed());
    let pdpt = &mut *(pdpt_pa.as_u64() as *mut PDPT);

    // 1 GiB slot
    pdpt[pdpt_index((*info).kv_start)] = PDPTEntry::new(
        pd_pa,
        PDPTFlags::P | PDPTFlags::RW
    );

    core::ptr::write(pd_pa.as_u64() as *mut PD, zeroed());
    let pd = &mut *(pd_pa.as_u64() as *mut PD);

    for i in 0..2 {
        pd[i] = PDEntry::new(
            PAddr((i * LARGE_PAGE_SIZE) as u64),
            PDFlags::P | PDFlags::RW | PDFlags::PS
        )
    }

    // risky call of the day
    // x86::controlregs::cr3_write(pml4_pa.into());
}
