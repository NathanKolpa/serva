boot/boot.o: boot/boot.asm
	nasm -f elf32 boot/boot.asm

boot/multiboot_header.o: boot/multiboot_header.asm
	nasm -f elf32 boot/multiboot_header.asm

boot/pre_kernel.bin: boot/boot.o boot/multiboot_header.o boot/linker.ld
	ld -m elf_i386 -n -o boot/pre_kernel.bin -T boot/linker.ld boot/multiboot_header.o boot/boot.o

run: boot/pre_kernel.bin
	qemu-system-x86_64 -kernel boot/pre_kernel.bin -machine type=pc-i440fx-3.1

clean:
	@rm boot/*.o boot/*.bin