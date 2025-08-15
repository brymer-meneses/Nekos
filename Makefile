QEMU_FLAGS := -machine virt \
							-bios default \
							-nographic \
							-serial mon:stdio \
							--no-reboot \

QEMU := qemu-system-riscv64

build: target/riscv64gc-unknown-none-elf/debug/nekos-kernel
	cargo build --target riscv64gc-unknown-none-elf

run: build
	$(QEMU) $(QEMU_FLAGS) -kernel target/riscv64gc-unknown-none-elf/debug/nekos-kernel

