_default:
  @just --list

iso: _kernel _deps
  rm -rf target/iso_root/
  mkdir -p target/iso_root/boot/limine/ target/iso_root/EFI/BOOT

  cp -v target/limine/limine-bios.sys target/limine/limine-bios-cd.bin target/limine/limine-uefi-cd.bin target/iso_root/boot/limine/
  cp -v target/limine/BOOTRISCV64.EFI target/iso_root/EFI/BOOT/
  cp -v nekos-kernel/limine.conf target/iso_root/boot/limine/
  cp -v target/riscv64gc-unknown-none-elf/debug/nekos-kernel target/iso_root/boot/kernel

  xorriso -as mkisofs \
     -R -r -J \
    --hfsplus \
    --apm-block-size 2048 \
    --efi-boot boot/limine/limine-uefi-cd.bin \
    --efi-boot-part \
    --efi-boot-image \
    --protective-msdos-label \
    target/iso_root -o target/nekos.iso

run: iso 
  @qemu-system-riscv64 \
    -M virt \
    -cpu rv64 \
    -device ramfb \
    -device qemu-xhci \
    -device usb-kbd \
    -device usb-mouse \
    -drive if=pflash,unit=0,format=raw,file=target/edk2-ovmf/ovmf-code-riscv64.fd,readonly=on \
    -cdrom target/nekos.iso \
    -serial stdio \
    -m 1G

_kernel:
  cargo build --target riscv64gc-unknown-none-elf

_deps:
  @if test ! -d "target/limine"; then \
    git clone https://github.com/limine-bootloader/limine.git --branch=v10.x-binary --depth=1 target/limine && \
    make -C target/limine; \
  fi

  @if test ! -d "target/edk2-ovmf"; then \
    mkdir -p target/edk2-ovmf && \
    curl -L https://github.com/osdev0/edk2-ovmf-nightly/releases/latest/download/edk2-ovmf.tar.gz | tar -xzf - -C target; \
  fi
