# GPT

[GPT] is short for [GUID] Partition Table. It's a more modern
alternative to MBR (master boot record) partition tables. Although it's
defined in the UEFI specification, it often gets used on non-UEFI
systems too. There are a couple big advantages of using GPT over MBR:
- It has a relatively clear and precise standard, unlike MBR where
  implementations often just try to match what other implementations do.
- It supports very large disks and very large numbers of partitions.

A GPT disk contains a primary header near the beginning of the disk,
followed by a partition entry array. The header and partition entry
array have a secondary copy at the end of the disk for redundency. The
partition entry arrays contain structures that describe each partition,
including a GUID to identify the individual partition, a partition type
GUID to indicate the purpose of the partition, and start/end block
addresses. In between the entry arrays is the actual partition data.

## System partition

The system partition is UEFI's version of a bootable partition. The
system partition is sometimes called the ESP, or EFI System
Partition. It is identified by a partition type of
`c12a7328-f81f-11d2-ba4b-00a0c93ec93b`. The system partition always
contains a FAT file system. There are various standardized paths that
can exist within the file system, and of particular importance are the
boot files. These are the files that UEFI will try to boot from by
default (in the absence of a different boot configuration set through
special [UEFI variables]).

Boot files are under `\EFI\BOOT`, and are named `BOOT<ARCH>.efi`, where
`<ARCH>` is a short architecture name.

|Architecture  |File name       |
|--------------|----------------|
|Intel 32-bit  |BOOTIA32.EFI    |
|X86_64        |BOOTX64.EFI     |
|Itanium       |BOOTIA64.EFI    |
|AArch32       |BOOTARM.EFI     |
|AArch64       |BOOTAA64.EFI    |
|RISC-V 32-bit |BOOTRISCV32.EFI |
|RISC-V 64-bit |BOOTRISCV64.EFI |
|RISC-V 128-bit|BOOTRISCV128.EFI|

[GPT]: https://en.wikipedia.org/wiki/GUID_Partition_Table
[GUID]: guid.md
[UEFI variables]: variables.md
