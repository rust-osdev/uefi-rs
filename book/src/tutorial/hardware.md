# Running on Hardware

Prerequisite: To allow loading your unsigned binary on your personal machine,
[secure boot] needs to be disabled in the BIOS setup.

To run on real hardware you'll need a specially-prepared USB drive.

## Preparation

The general steps to prepare the drive are:

1. Partition the drive using [GPT].
2. Create a partition.
3. Set the partition type GUID to
   `C12A7328-F81F-11D2-BA4B-00A0C93EC93B`. That marks it as an EFI
   System partition. (On many UEFI implementations this is not strictly
   necessary, see note below.)
4. Format the partition as [FAT].
5. Mount the partition.
6. Create the directory path `EFI/BOOT` on the partition. (FAT is case
   insensitive, so capitalization doesn't matter.)
7. Copy your EFI application to a file under `EFI/BOOT`. The file name
   is specific to the architecture. For example, on x86_64 the file name
   must be `BOOTX64.EFI`. See the [boot files] table for other
   architectures.
   
The details of exactly how to do these steps will vary depending on your OS.

Note that most UEFI implementations do not strictly require GPT
partitioning or the EFI System partition GUID; they will look for any
FAT partition with the appropriate directory structure. This is not
required however; the UEFI Specification says "UEFI implementations may
allow the use of conforming FAT partitions which do not use the ESP
GUID."
   
### Example on Linux

**Warning: these operations are destructive! Do not run these commands
on a disk if you care about the data it contains.**

```sh
# Create the GPT, create a 9MB partition starting at 1MB, and set the
# partition type to EFI System.
sgdisk \
    --clear \
    --new=1:1M:10M \
    --typecode=1:C12A7328-F81F-11D2-BA4B-00A0C93EC93B \
    /path/to/disk

# Format the partition as FAT.
mkfs.fat /path/to/disk_partition

# Mount the partition.
mkdir esp
mount /path/to/disk_partition esp

# Create the boot directory.
mkdir esp/EFI/BOOT

# Copy in the boot executable.
cp /path/to/your-executable.efi esp/EFI/BOOT/BOOTX64.EFI
```

## Booting the USB

Insert the USB into the target computer. Reboot the machine, then press
the one-time boot key. Which key to press depends on the vendor. For
example, Dell uses F12, HP uses F9, and on Macs you hold down the Option
key.

Once the one-time boot menu appears, select your USB drive and press enter.

[secure boot]: https://en.wikipedia.org/wiki/UEFI#Secure_Boot
[GPT]: https://en.wikipedia.org/wiki/GUID_Partition_Table
[FAT]: https://en.wikipedia.org/wiki/File_Allocation_Table
[boot files]: ../concepts/gpt.html#system-partition
