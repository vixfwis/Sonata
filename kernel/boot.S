#include "arch/x86_64/gdt.h"
#include "arch/x86_64/mmu.h"
#include "arch/x86_64/kernel.h"
#include "sizes.h"
#include "multiboot2.h"
#include "arch/x86_64/msr.h"

.SET HEADER_LENGTH, header_end - header_start
.SET CHECKSUM, -(MULTIBOOT2_HEADER_MAGIC + MULTIBOOT_ARCHITECTURE_I386 + HEADER_LENGTH)
.section .multiboot
header_start:
    .long MULTIBOOT2_HEADER_MAGIC
    .long MULTIBOOT_ARCHITECTURE_I386
    .long HEADER_LENGTH
    .long CHECKSUM
    // multiboot tags

    .short MULTIBOOT_HEADER_TAG_INFORMATION_REQUEST // request info from bootloader
    .short 0                                        // non-optional
    .long 12                                        // tag size
    .long MULTIBOOT_TAG_TYPE_MMAP                   // memory map
    .long 0                                         // padding to next 8 byte boundary

    // end of flags array
    .short MULTIBOOT_HEADER_TAG_END
    .short 0
    .long 8
header_end:

.code32

.data
.align GDT_TABLE_ALIGNMENT
gdt_table:
        .8byte GDT_FIRST_ENTRY
        .8byte GDT_KERNEL_ENTRY

gdt_table_end:
        .skip (GDT_TABLE_SIZE - (gdt_table_end - gdt_table))

gdt_ptr:
        .short GDT_TABLE_SIZE - 1
        .long gdt_table

// pass this structure to kmain so memory manager
// can take over without overwriting itself
.align 0x10
boot_info:
        .quad 0         // will write multiboot2 boot info here at runtime
        .quad gdt_table
        .quad kstack
        .quad kernel_physical_end
        .quad kstack + KERNEL_HIGHER_HALF
        .quad kernel_virtual_end

.section .bss
// level 4 paging
.comm pml4, PML4_SIZE, PML4_ALIGNMENT
// level 3 paging for low and high memory
.comm low_pdpt, PDPT_SIZE, PDPT_ALIGNMENT
.comm high_pdpt, PDPT_SIZE, PDPT_ALIGNMENT
// level 2 paging for low and high memory
// we're using 2M pages in bootstrap code, so no level 1 paging
.comm low_page_directory_table, PAGE_DIRECTORY_SIZE, PAGE_DIRECTORY_ALIGNMENT
.comm high_page_directory_table, PAGE_DIRECTORY_SIZE, PAGE_DIRECTORY_ALIGNMENT
.comm kstack, KERNEL_BOOT_STACK_SIZE, KERNEL_BOOT_STACK_ALIGNMENT

.section .text
.global _start
.type _start, @function
_start:
        movl %ebx, boot_info  // mb2 boot info address

        // write lower half entry to level 4 page table
        movl $low_pdpt, %eax
        or $(MMU_PRESENT | MMU_WRITABLE), %eax
        movl %eax, pml4 + (PML4_ADDR_TO_ENTRY_INDEX(KERNEL_PHYSICAL_START) * PML4_ENTRY_SIZE)

        // write higher half entry to level 4 page table
        movl $high_pdpt, %eax
        or $(MMU_PRESENT | MMU_WRITABLE), %eax
        movl %eax, pml4 + (PML4_ADDR_TO_ENTRY_INDEX(KERNEL_VIRTUAL_START) * PML4_ENTRY_SIZE)

        // same with level 3
        movl $low_page_directory_table, %eax
        or $(MMU_PRESENT | MMU_WRITABLE), %eax
        movl %eax, low_pdpt + (PDPT_ADDR_TO_ENTRY_INDEX(KERNEL_PHYSICAL_START) * PDPT_ENTRY_SIZE)

        movl $high_page_directory_table, %eax
        or $(MMU_PRESENT | MMU_WRITABLE), %eax
        movl %eax, high_pdpt + (PDPT_ADDR_TO_ENTRY_INDEX(KERNEL_VIRTUAL_START) * PDPT_ENTRY_SIZE)

        // map whole table: 512 records, 2MiB each, 2GiB total
        mov $0, %ecx
        mov $512, %esi

page_directory_table_loop:
        movl $TWO_MEGABYTES, %eax
        mul %ecx
        or $(MMU_PRESENT | MMU_WRITABLE | MMU_PDE_TWO_MB), %eax
        movl %eax, low_page_directory_table(, %ecx, PAGE_DIRECTORY_ENTRY_SIZE)
        movl %eax, high_page_directory_table(, %ecx, PAGE_DIRECTORY_ENTRY_SIZE)

        inc %ecx
        cmp %esi, %ecx
        jne page_directory_table_loop

        // load top level page table to CR3
        movl $pml4, %eax
        movl %eax, %cr3

        // enable PAE
        movl $KERNEL_CR4, %eax
        movl %eax, %cr4

        // write Long Mode Enable (LME) to MSR
        movl $MSR_EFER, %ecx
        rdmsr
        or $MSR_EFER_LME, %eax
        wrmsr

        // set protected mode bit, paging bit, math coprocessor bit
        movl $KERNEL_CR0, %eax
        movl %eax, %cr0

        // load GDT and long jump to x64
        lgdt gdt_ptr
        ljmp $(KERNEL_GDT_ENTRY * GDT_ENTRY_SIZE), $_start64

.code64
.global _start64
.type _start64, @function

_start64:
        movw $0, %ax
        movw %ax, %ds
        movw %ax, %es
        movw %ax, %fs
        movw %ax, %gs
        movw %ax, %ss
        // setting up stack in higher half address space via mov
        // because `add` doesn't have r/m64 opcode
        movq $KERNEL_HIGHER_HALF, %rax
        movq %rax, kstack
        movl $kstack + KERNEL_BOOT_STACK_SIZE, %eax
        movl %eax, kstack
        movabs kstack, %rax
        movq %rax, %rsp
        // clear stack
        movq %rax, %rdi
        subq $0x4000, %rdi
        movq $0, %rax
        movl $0x800, %ecx
        rep stosq
        // clear screen
        movl $0xB8000, %edi
        movq $0x0F200F200F200F20, %rax
        movl $500, %ecx
        rep stosq
        // pass boot info
        movq $boot_info, %rdi
        movabs $kmain, %rax
        jmp *%rax
