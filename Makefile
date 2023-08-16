rwildcard=$(foreach d,$(wildcard $(1:=/*)),$(call rwildcard,$d,$2) $(filter $(subst *,%,$2),$d))
KERNEL_SRC := $(call rwildcard,kernel/src,*.rs)

boot/boot.o: boot/boot.asm
	nasm -f elf64 boot/boot.asm

boot/boot_long.o: boot/boot_long.asm
	nasm -f elf64 boot/boot_long.asm

boot/multiboot_header.o: boot/multiboot_header.asm
	nasm -f elf64 boot/multiboot_header.asm

target/x86_64-serva/debug/libkernel.a: $(KERNEL_SRC)
	cargo build -p kernel

boot/iso/boot/kernel.bin: boot/boot.o boot/multiboot_header.o boot/linker.ld boot/boot_long.o target/x86_64-serva/debug/libkernel.a
	ld -n -o boot/iso/boot/kernel.bin -T boot/linker.ld boot/multiboot_header.o boot/boot.o boot/boot_long.o target/x86_64-serva/debug/libkernel.a

boot/serva.iso: boot/iso/boot/kernel.bin
	grub-mkrescue -o boot/serva.iso boot/iso

run: boot/serva.iso
	qemu-system-x86_64 -cdrom boot/serva.iso

clean:
	@rm boot/*.o boot/iso/boot/kernel.bin boot/serva.iso target/x86_64-serva/debug/libkernel.a