From https://rust-osdev.github.io/uefi-rs/

# Build the application
```
cargo build --target x86_64-unknown-uefi
```

# Install QEMU and UEFI BIOS
```
sudo apt-get install qemu-system-x86 ovmf
```

# Prepare VVFAT content
```
mkdir -p esp/efi/boot
cp target/x86_64-unknown-uefi/debug/uefi-hello-world.efi esp/efi/boot/bootx64.efi
```

# Run the application
```
qemu-system-x86_64 --nographic \
    -drive if=pflash,format=raw,readonly=on,file=/usr/share/ovmf/OVMF.fd \
    -drive format=raw,file=fat:rw:esp
```
