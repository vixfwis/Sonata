ISO_FILE=sonata.iso

ifneq (, $(shell which grub2-mkrescue 2> /dev/null))
  GRUB_MKRESCUE = grub2-mkrescue
else ifneq (, $(shell which grub-mkrescue 2> /dev/null))
  GRUB_MKRESCUE = grub-mkrescue
else
    $(error "Cannot find grub-mkrescue or grub2-mkrescue")
endif

all: kernel

kernel: sonata_os
	make -C kernel all

sonata_os:
	cargo -Z unstable-options build --lib --out-dir kernel

qemu: $(ISO_FILE)
	qemu-system-x86_64 -cdrom $(ISO_FILE) -m 1024M -s

qemu-gdb: $(ISO_FILE)
	qemu-system-x86_64 -cdrom $(ISO_FILE) -m 1024M -s -S

gdb:
	gdb -ex 'target remote localhost:1234' -ex 'file kernel/kernel'

clean:
	make -C kernel clean
	rm -rf iso
	rm -rf $(ISO_FILE)

iso: $(ISO_FILE)

$(ISO_FILE): kernel
	mkdir -p iso/boot/grub
	cp grub.cfg iso/boot/grub/
	cp kernel/kernel iso/boot/
	$(GRUB_MKRESCUE) -o $(ISO_FILE) iso

.PHONY: all kernel sonata_os qemu qemu-gdb iso clean gdb
