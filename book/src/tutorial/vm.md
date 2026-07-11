# Running in a VM

## Install dependencies

Two dependencies are needed: [QEMU], which implements the virtual
machine itself, and [OVMF], which provides UEFI firmware that QEMU can
run.

The details of how to install QEMU and OVMF will vary depending on your
operating system.

Debian/Ubuntu:
```sh
sudo apt-get install qemu ovmf
```

Fedora:
```sh
sudo dnf install qemu-kvm edk2-ovmf
```

### Firmware files

The OVMF package provides two firmware files, one for the executable
code and one for variable storage. (The package may provide multiple
variations of these files; refer to the package's documentation for
details of the files it includes.)

For ease of access we'll copy the OVMF code and vars files to the
project directory. The location where OVMF is installed depends on your
operating system; for Debian, Ubuntu and Fedora the files are under
`/usr/share/OVMF`.

Copy the files to your project directory:
```sh
cp /usr/share/OVMF/OVMF_CODE.fd .
cp /usr/share/OVMF/OVMF_VARS.fd .
```

## System partition

Now create a directory structure containing the executable to imitate a
[UEFI System Partition]:

```sh
mkdir -p esp/efi/boot
cp target/x86_64-unknown-uefi/debug/my-uefi-app.efi esp/efi/boot/bootx64.efi
```

## Launch the VM

Now we can launch QEMU, using [VVFAT] to access the `esp` directory created above.

```sh
qemu-system-x86_64 -enable-kvm \
    -drive if=pflash,format=raw,readonly=on,file=OVMF_CODE.fd \
    -drive if=pflash,format=raw,readonly=on,file=OVMF_VARS.fd \
    -drive format=raw,file=fat:rw:esp
```

A QEMU window should appear, and after a few seconds you should see the
log message:
```text
[ INFO]:  src/main.rs@011: Hello world!
```

[QEMU]: https://www.qemu.org
[OVMF]: https://github.com/tianocore/tianocore.github.io/wiki/OVMF
[VVFAT]: https://en.m.wikibooks.org/wiki/QEMU/Devices/Storage#Virtual_FAT_filesystem_(VVFAT)
[UEFI System Partition]: ../concepts/gpt.md#system-partition
