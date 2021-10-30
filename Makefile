ISO_FILE=sonata.iso

.PHONY: all
.PHONY: kernel
.PHONY: sonata_os
.PHONY: qemu
.PHONY: iso
.PHONY: clean

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
	qemu-system-x86_64 -cdrom $(ISO_FILE) -m 1024M

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
