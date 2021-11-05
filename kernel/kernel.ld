ENTRY(_start)

SECTIONS
{
  . = 4M;

  .boottext :
    {
      boot.o (.multiboot)
      boot.o (.text)
    }
  .bootrodata :
    {
      boot.o (.rodata)
    }
  .bootdata :
    {
      boot.o (.data)
    }
  .bootbss :
    {
      boot.o (.bss)
      boot.o (COMMON)
    }

  . = ALIGN(0x1000);
  boot_end = .;

  . += 0xFFFF800000000000;
  .text : AT(boot_end)
  {
    *(.multiboot)
    *(.text)
  }

  . = ALIGN(0x1000);

  .rodata : AT ( (LOADADDR (.text) + SIZEOF (.text) + 0xFFF) & 0xFFFFFFFFFFFFF000 )
  {
    *(.rodata)
  }

  . = ALIGN(0x1000);

  .data : AT ( (LOADADDR (.rodata) + SIZEOF (.rodata) + 0xFFF) & 0xFFFFFFFFFFFFF000 )
  {
    *(.data)
  }

  . = ALIGN(0x1000);

  .bss : AT ( (LOADADDR (.data) + SIZEOF (.data) + 0xFFF) & 0xFFFFFFFFFFFFF000 )
  {
    *(COMMON)
    *(.bss)
  }

  . = ALIGN(0x1000);

  kernel_virtual_end = .;

  kernel_physical_end = kernel_virtual_end & (~0xFFFF800000000000);
}
