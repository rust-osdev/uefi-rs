//! UEFI services available at runtime, even after the OS boots.
//!
//! All of these functions are unsafe because UEFI runtime services require
//! an elaborate CPU configuration which may not be preserved by OS loaders.
//! See the "Calling Conventions" chapter of the UEFI specification for details.

#[cfg(feature = "alloc")]
use alloc::{boxed::Box, vec::Vec};
use core::mem::MaybeUninit;
use core::ptr::{self, NonNull};

use uefi_raw::table::boot::MemoryDescriptor;
use uefi_raw::table::runtime::{ResetType, TimeCapabilities, VariableAttributes, VariableVendor};
use uefi_raw::table::Revision;
use uefi_raw::Status;

use crate::system::{set_system_table, system_table};
#[cfg(feature = "alloc")]
use crate::table::runtime::VariableKey;
use crate::table::runtime::{Time, VariableStorageInfo};
use crate::{CStr16, Error, Result, StatusExt};

pub(crate) fn runtime_table() -> NonNull<uefi_raw::table::runtime::RuntimeServices> {
    let runtime = unsafe { system_table().as_ref().runtime_services };

    NonNull::new(runtime).expect("runtime table doesn't exist")
}

/// Query the current time and date information
///
/// # Safety
///
/// This is unsafe because UEFI runtime services require an elaborate
/// CPU configuration which may not be preserved by OS loaders. See the
/// "Calling Conventions" chapter of the UEFI specification for details.
pub unsafe fn get_time() -> Result<Time> {
    let mut time = MaybeUninit::<Time>::uninit();

    unsafe { (runtime_table().as_ref().get_time)(time.as_mut_ptr().cast(), ptr::null_mut()) }
        .to_result_with_val(|| unsafe { time.assume_init() })
}

/// Query the current time and date information and the RTC capabilities
///
/// # Safety
///
/// This is unsafe because UEFI runtime services require an elaborate
/// CPU configuration which may not be preserved by OS loaders. See the
/// "Calling Conventions" chapter of the UEFI specification for details.
pub unsafe fn get_time_and_caps() -> Result<(Time, TimeCapabilities)> {
    let mut time = MaybeUninit::<Time>::uninit();
    let mut caps = MaybeUninit::<TimeCapabilities>::uninit();

    unsafe { (runtime_table().as_ref().get_time)(time.as_mut_ptr().cast(), caps.as_mut_ptr()) }
        .to_result_with_val(|| unsafe { (time.assume_init(), caps.assume_init()) })
}

/// Sets the current local time and date information
///
/// During runtime, if a PC-AT CMOS device is present in the platform, the
/// caller must synchronize access to the device before calling `set_time`.
///
/// # Safety
/// - This is unsafe because UEFI runtime services require an elaborate
/// CPU configuration which may not be preserved by OS loaders. See the
/// "Calling Conventions" chapter of the UEFI specification for details.
/// - Undefined behavior could happen if multiple tasks try to
/// use this function at the same time without synchronisation.
pub unsafe fn set_time(time: &Time) -> Result {
    let time: *const Time = time;

    (runtime_table().as_ref().set_time)(time.cast()).to_result()
}

/// Get the size (in bytes) of a variable. This can be used to find out how
/// big of a buffer should be passed in to `get_variable`.
///
/// # Safety
///
/// This is unsafe because UEFI runtime services require an elaborate
/// CPU configuration which may not be preserved by OS loaders. See the
/// "Calling Conventions" chapter of the UEFI specification for details.
pub unsafe fn get_variable_size(name: &CStr16, vendor: &VariableVendor) -> Result<usize> {
    let mut data_size = 0;
    let status = unsafe {
        (runtime_table().as_ref().get_variable)(
            name.as_ptr().cast(),
            &vendor.0,
            ptr::null_mut(),
            &mut data_size,
            ptr::null_mut(),
        )
    };

    if status == Status::BUFFER_TOO_SMALL {
        Status::SUCCESS.to_result_with_val(|| data_size)
    } else {
        Err(Error::from(status))
    }
}

/// Get the contents and attributes of a variable. The size of `buf` must
/// be at least as big as the variable's size, although it can be
/// larger. If it is too small, `BUFFER_TOO_SMALL` is returned.
///
/// On success, a tuple containing the variable's value (a slice of `buf`)
/// and the variable's attributes is returned.
///
/// # Safety
///
/// This is unsafe because UEFI runtime services require an elaborate
/// CPU configuration which may not be preserved by OS loaders. See the
/// "Calling Conventions" chapter of the UEFI specification for details.
pub fn get_variable<'a>(
    name: &CStr16,
    vendor: &VariableVendor,
    buf: &'a mut [u8],
) -> Result<(&'a [u8], VariableAttributes)> {
    let mut attributes = VariableAttributes::empty();
    let mut data_size = buf.len();
    unsafe {
        (runtime_table().as_ref().get_variable)(
            name.as_ptr().cast(),
            &vendor.0,
            &mut attributes,
            &mut data_size,
            buf.as_mut_ptr(),
        )
        .to_result_with_val(move || (&buf[..data_size], attributes))
    }
}

/// Get the contents and attributes of a variable.
///
/// # Safety
///
/// This is unsafe because UEFI runtime services require an elaborate
/// CPU configuration which may not be preserved by OS loaders. See the
/// "Calling Conventions" chapter of the UEFI specification for details.
#[cfg(feature = "alloc")]
pub unsafe fn get_variable_boxed(
    name: &CStr16,
    vendor: &VariableVendor,
) -> Result<(Box<[u8]>, VariableAttributes)> {
    let mut attributes = VariableAttributes::empty();

    let mut data_size = unsafe { get_variable_size(name, vendor)? };
    let mut data = Vec::with_capacity(data_size);

    let status = unsafe {
        (runtime_table().as_ref().get_variable)(
            name.as_ptr().cast(),
            &vendor.0,
            &mut attributes,
            &mut data_size,
            data.as_mut_ptr(),
        )
    };
    if !status.is_success() {
        return Err(Error::from(status));
    }

    unsafe {
        data.set_len(data_size);
    }

    Ok((data.into_boxed_slice(), attributes))
}

/// Get the names and vendor GUIDs of all currently-set variables.
///
/// # Safety
///
/// This is unsafe because UEFI runtime services require an elaborate
/// CPU configuration which may not be preserved by OS loaders. See the
/// "Calling Conventions" chapter of the UEFI specification for details.
#[cfg(feature = "alloc")]
pub unsafe fn variable_keys() -> Result<Vec<VariableKey>> {
    use alloc::vec;
    use uguid::Guid;

    use core::mem;

    let mut all_variables = Vec::new();

    // The initial value of name must start with a null character. Start
    // out with a reasonable size that likely won't need to be increased.
    let mut name = vec![0u16; 32];
    // The initial value of vendor is ignored.
    let mut vendor = Guid::default();

    let mut status;
    loop {
        let mut name_size_in_bytes = name.len() * mem::size_of::<u16>();
        status = unsafe {
            (runtime_table().as_ref().get_next_variable_name)(
                &mut name_size_in_bytes,
                name.as_mut_ptr(),
                &mut vendor,
            )
        };

        match status {
            Status::SUCCESS => {
                // CStr16::from_u16_with_nul does not allow interior nulls,
                // so make the copy exactly the right size.
                let name = if let Some(nul_pos) = name.iter().position(|c| *c == 0) {
                    name[..=nul_pos].to_vec()
                } else {
                    status = Status::ABORTED;
                    break;
                };

                all_variables.push(VariableKey {
                    name,
                    vendor: VariableVendor(vendor),
                });
            }
            Status::BUFFER_TOO_SMALL => {
                // The name buffer passed in was too small, resize it to be
                // big enough for the next variable name.
                name.resize(name_size_in_bytes / 2, 0);
            }
            Status::NOT_FOUND => {
                // This status indicates the end of the list. The final
                // variable has already been received at this point, so
                // no new variable should be added to the output.
                status = Status::SUCCESS;
                break;
            }
            _ => {
                // For anything else, an error has occurred so break out of
                // the loop and return it.
                break;
            }
        }
    }

    status.to_result_with_val(|| all_variables)
}

/// Set the value of a variable. This can be used to create a new variable,
/// update an existing variable, or (when the size of `data` is zero)
/// delete a variable.
///
/// # Safety
///
/// This is unsafe because UEFI runtime services require an elaborate
/// CPU configuration which may not be preserved by OS loaders. See the
/// "Calling Conventions" chapter of the UEFI specification for details.
///
/// # Warnings
///
/// The [`Status::WARN_RESET_REQUIRED`] warning will be returned when using
/// this function to transition the Secure Boot mode to setup mode or audit
/// mode if the firmware requires a reboot for that operation.
pub unsafe fn set_variable(
    name: &CStr16,
    vendor: &VariableVendor,
    attributes: VariableAttributes,
    data: &[u8],
) -> Result {
    unsafe {
        (runtime_table().as_ref().set_variable)(
            name.as_ptr().cast(),
            &vendor.0,
            attributes,
            data.len(),
            data.as_ptr(),
        )
        .to_result()
    }
}

/// Deletes a UEFI variable.
///
/// # Safety
///
/// This is unsafe because UEFI runtime services require an elaborate
/// CPU configuration which may not be preserved by OS loaders. See the
/// "Calling Conventions" chapter of the UEFI specification for details.
pub unsafe fn delete_variable(name: &CStr16, vendor: &VariableVendor) -> Result {
    set_variable(name, vendor, VariableAttributes::empty(), &[])
}

/// Get information about UEFI variable storage space for the type
/// of variable specified in `attributes`.
///
/// This operation is only supported starting with UEFI 2.0; earlier
/// versions will fail with [`Status::UNSUPPORTED`].
///
/// See [`VariableStorageInfo`] for details of the information returned.
///
/// # Safety
///
/// This is unsafe because UEFI runtime services require an elaborate
/// CPU configuration which may not be preserved by OS loaders. See the
/// "Calling Conventions" chapter of the UEFI specification for details.
pub unsafe fn query_variable_info(attributes: VariableAttributes) -> Result<VariableStorageInfo> {
    if unsafe { runtime_table().as_ref().header.revision } < Revision::EFI_2_00 {
        return Err(Status::UNSUPPORTED.into());
    }

    let mut info = VariableStorageInfo::default();
    unsafe {
        (runtime_table().as_ref().query_variable_info)(
            attributes,
            &mut info.maximum_variable_storage_size,
            &mut info.remaining_variable_storage_size,
            &mut info.maximum_variable_size,
        )
        .to_result_with_val(|| info)
    }
}

/// Resets the computer.
///
/// # Safety
///
/// This is unsafe because UEFI runtime services require an elaborate
/// CPU configuration which may not be preserved by OS loaders. See the
/// "Calling Conventions" chapter of the UEFI specification for details.
pub unsafe fn reset(rt: ResetType, status: Status, data: Option<&[u8]>) -> ! {
    let (size, data) = match data {
        // FIXME: The UEFI spec states that the data must start with a NUL-
        //        terminated string, which we should check... but it does not
        //        specify if that string should be Latin-1 or UCS-2!
        //
        //        PlatformSpecific resets should also insert a GUID after the
        //        NUL-terminated string.
        Some(data) => (data.len(), data.as_ptr()),
        None => (0, ptr::null()),
    };

    unsafe { (runtime_table().as_ref().reset_system)(rt, status, size, data) }
}

/// Changes the runtime addressing mode of EFI firmware from physical to virtual.
/// It is up to the caller to translate the old SystemTable address to a new virtual
/// address and provide it for this function.
/// See [`get_current_system_table_addr`]
///
/// # Safety
///
/// Setting new virtual memory map is unsafe and may cause undefined behaviors.
///
/// [`get_current_system_table_addr`]: SystemTable::get_current_system_table_addr
pub unsafe fn set_virtual_address_map(
    map: &mut [MemoryDescriptor],
    new_system_table_virtual_addr: u64,
) -> Result {
    // Unsafe Code Guidelines guarantees that there is no padding in an array or a slice
    // between its elements if the element type is `repr(C)`, which is our case.
    //
    // See https://rust-lang.github.io/unsafe-code-guidelines/layout/arrays-and-slices.html
    let map_size = core::mem::size_of_val(map);
    let entry_size = core::mem::size_of::<MemoryDescriptor>();
    let entry_version = MemoryDescriptor::VERSION;
    let map_ptr = map.as_mut_ptr();

    (runtime_table().as_ref().set_virtual_address_map)(map_size, entry_size, entry_version, map_ptr)
        .to_result_with_val(|| {
            let new_table_ptr =
                new_system_table_virtual_addr as usize as *mut uefi_raw::table::system::SystemTable;

            unsafe { set_system_table(new_table_ptr) }
        })
}
