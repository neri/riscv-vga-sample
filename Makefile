.PHONY: default clean install run

MNT			= ./mnt/
KERNEL_ELF	= target/riscv64gc-unknown-none-elf/release/kernel

default: kernel

clean:
	cargo clean

kernel:
	cargo build --release

install: kernel

run:
	qemu-system-riscv64 -M virt -device virtio-vga -serial mon:stdio -kernel $(KERNEL_ELF)
