# Variables

UEFI provides fairly flexible key/value variable storage.

Each variable is identified by a key consisting of a UCS-2
null-terminated name plus a vendor [GUID]. The vendor GUID serves as a
namespace for variables so that different vendors don't accidentally
overwrite or misinterpret another vendor's variable if they happen to
have the same name.

The data stored in each variable is an arbitrary byte array.

## Attributes

Each variable has attributes (represented as bit flags) associated with
it that affect how it is stored and how it can be accessed.

If the `BOOTSERVICE_ACCESS` and `RUNTIME_ACCESS` bits are set, the
variable can be accessed during both the Boot Services and Runtime
[stages]. If only `BOOTSERVICE_ACCESS` is set then the variable can
neither be read nor written to after exiting boot services.

Another important attribute is the `NON_VOLATILE` bit. If this bit is
_not_ set, the variable will be stored in normal memory and will not
persist across a power cycle. If this bit _is_ set, the variable will be
stored in special non-volatile memory. You should be careful about
writing variables of this type, because the non-volatile storage can be
very limited in size. There have been cases where a vendor's poor UEFI
implementation caused the machine not too boot once the storage became
too full. Even figuring out how much space is in use can be tricky due
to deletion being implemented via garbage collection. Matthew Garret's
article ["Dealing with UEFI non-volatile memory quirks"] has more details.

Most of the other attributes relate to authenticated variables, which
can be used to prevent changes to a variable by unauthorized programs.

[GUID]: guid.md
[stages]: boot_stages.md
["Dealing with UEFI non-volatile memory quirks"]: https://mjg59.dreamwidth.org/25091.html
