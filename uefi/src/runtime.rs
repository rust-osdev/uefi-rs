// SPDX-License-Identifier: MIT OR Apache-2.0

//! UEFI runtime services.
//!
//! These services are available both before and after exiting boot
//! services. Note that various restrictions apply when calling runtime services
//! functions after exiting boot services; see the "Calling Convention" section
//! of the UEFI specification for details.

use crate::data_types::PhysicalAddress;
use crate::table::{self, Revision};
use crate::{CStr16, Error, Result, Status, StatusExt};
use core::fmt::{self, Debug, Display, Formatter};
use core::ptr::{self, NonNull};
use uefi_raw::table::boot::MemoryDescriptor;

#[cfg(feature = "alloc")]
use {
    crate::mem::make_boxed,
    crate::CString16,
    crate::Guid,
    alloc::borrow::ToOwned,
    alloc::boxed::Box,
    alloc::{vec, vec::Vec},
};

#[cfg(all(feature = "unstable", feature = "alloc"))]
use alloc::alloc::Global;

pub use uefi_raw::capsule::{CapsuleBlockDescriptor, CapsuleFlags, CapsuleHeader};
pub use uefi_raw::table::runtime::{
    ResetType, TimeCapabilities, VariableAttributes, VariableVendor,
};
pub use uefi_raw::time::Daylight;

fn runtime_services_raw_panicking() -> NonNull<uefi_raw::table::runtime::RuntimeServices> {
    let st = table::system_table_raw_panicking();
    // SAFETY: valid per requirements of `set_system_table`.
    let st = unsafe { st.as_ref() };
    NonNull::new(st.runtime_services).expect("runtime services are not active")
}

/// Query the current time and date information.
pub fn get_time() -> Result<Time> {
    let rt = runtime_services_raw_panicking();
    let rt = unsafe { rt.as_ref() };

    let mut time = Time::invalid();
    let time_ptr: *mut Time = &mut time;
    unsafe { (rt.get_time)(time_ptr.cast(), ptr::null_mut()) }.to_result_with_val(|| time)
}

/// Query the current time and date information and the RTC capabilities.
pub fn get_time_and_caps() -> Result<(Time, TimeCapabilities)> {
    let rt = runtime_services_raw_panicking();
    let rt = unsafe { rt.as_ref() };

    let mut time = Time::invalid();
    let time_ptr: *mut Time = &mut time;
    let mut caps = TimeCapabilities::default();
    unsafe { (rt.get_time)(time_ptr.cast(), &mut caps) }.to_result_with_val(|| (time, caps))
}

/// Sets the current local time and date information
///
/// During runtime, if a PC-AT CMOS device is present in the platform, the
/// caller must synchronize access to the device before calling `set_time`.
///
/// # Safety
///
/// Undefined behavior could happen if multiple tasks try to
/// use this function at the same time without synchronisation.
pub unsafe fn set_time(time: &Time) -> Result {
    let rt = runtime_services_raw_panicking();
    let rt = unsafe { rt.as_ref() };

    let time: *const Time = time;
    unsafe { (rt.set_time)(time.cast()) }.to_result()
}

/// Checks if a variable exists.
///
/// Returns `Ok(true)` if the variable exists, `Ok(false)` if the variable does
/// not exist, or `Err` if the existence of the variable could not be determined.
///
/// # Errors
///
/// * [`Status::DEVICE_ERROR`]: variable could not be read due to a hardware error.
/// * [`Status::SECURITY_VIOLATION`]: variable could not be read due to an
///   authentication error.
/// * [`Status::UNSUPPORTED`]: this platform does not support variable storage
///   after exiting boot services.
pub fn variable_exists(name: &CStr16, vendor: &VariableVendor) -> Result<bool> {
    let rt = runtime_services_raw_panicking();
    let rt = unsafe { rt.as_ref() };

    let attributes = ptr::null_mut();
    let data = ptr::null_mut();
    let mut data_size = 0;

    let status = unsafe {
        (rt.get_variable)(
            name.as_ptr().cast(),
            &vendor.0,
            attributes,
            &mut data_size,
            data,
        )
    };

    match status {
        // If the variable exists, the status will be BUFFER_TOO_SMALL because
        // data_size is 0. Empty variables do not exist, because setting a
        // variable with empty data deletes the variable. In other words, the
        // status will never be SUCCESS.
        Status::BUFFER_TOO_SMALL => Ok(true),
        Status::NOT_FOUND => Ok(false),
        _ => Err(Error::from(status)),
    }
}

/// Gets the contents and attributes of a variable. The size of `buf` must be at
/// least as big as the variable's size, although it can be larger.
///
/// On success, returns a tuple containing the variable's value (a slice of
/// `buf`) and the variable's attributes.
///
/// # Errors
///
/// * [`Status::NOT_FOUND`]: variable was not found.
/// * [`Status::BUFFER_TOO_SMALL`]: `buf` is not large enough. The required size
///   will be returned in the error data.
/// * [`Status::DEVICE_ERROR`]: variable could not be read due to a hardware error.
/// * [`Status::SECURITY_VIOLATION`]: variable could not be read due to an
///   authentication error.
/// * [`Status::UNSUPPORTED`]: this platform does not support variable storage
///   after exiting boot services.
pub fn get_variable<'buf>(
    name: &CStr16,
    vendor: &VariableVendor,
    buf: &'buf mut [u8],
) -> Result<(&'buf mut [u8], VariableAttributes), Option<usize>> {
    let rt = runtime_services_raw_panicking();
    let rt = unsafe { rt.as_ref() };

    let mut attributes = VariableAttributes::empty();
    let mut data_size = buf.len();
    let status = unsafe {
        (rt.get_variable)(
            name.as_ptr().cast(),
            &vendor.0,
            &mut attributes,
            &mut data_size,
            buf.as_mut_ptr(),
        )
    };

    match status {
        Status::SUCCESS => Ok((&mut buf[..data_size], attributes)),
        Status::BUFFER_TOO_SMALL => Err(Error::new(status, Some(data_size))),
        _ => Err(Error::new(status, None)),
    }
}

/// Gets the contents and attributes of a variable.
///
/// # Errors
///
/// * [`Status::NOT_FOUND`]: variable was not found.
/// * [`Status::DEVICE_ERROR`]: variable could not be read due to a hardware error.
/// * [`Status::SECURITY_VIOLATION`]: variable could not be read due to an
///   authentication error.
/// * [`Status::UNSUPPORTED`]: this platform does not support variable storage
///   after exiting boot services.
#[cfg(feature = "alloc")]
pub fn get_variable_boxed(
    name: &CStr16,
    vendor: &VariableVendor,
) -> Result<(Box<[u8]>, VariableAttributes)> {
    let mut out_attr = VariableAttributes::empty();
    let get_var = |buf| {
        get_variable(name, vendor, buf).map(|(val, attr)| {
            // `make_boxed` expects only a DST value to be returned (`val` in
            // this case), so smuggle the `attr` value out via a separate
            // variable.
            out_attr = attr;
            val
        })
    };
    #[cfg(not(feature = "unstable"))]
    {
        make_boxed(get_var).map(|val| (val, out_attr))
    }
    #[cfg(feature = "unstable")]
    {
        make_boxed(get_var, Global).map(|val| (val, out_attr))
    }
}

/// Gets each variable key (name and vendor) one at a time.
///
/// This is used to iterate over variable keys. See [`variable_keys`] for a more
/// convenient interface that requires the `alloc` feature.
///
/// To get the first variable key, `name` must be initialized to start with a
/// null character. The `vendor` value is arbitrary. On success, the first
/// variable's name and vendor will be written out to `name` and `vendor`. Keep
/// calling `get_next_variable_key` with the same `name` and `vendor` references
/// to get the remaining variable keys.
///
/// All variable names should be valid strings, but this may not be enforced by
/// firmware. To convert to a string, truncate at the first null and call
/// [`CStr16::from_u16_with_nul`].
///
/// # Errors
///
/// * [`Status::NOT_FOUND`]: indicates end of iteration, the last variable keys
///   was retrieved by the previous call to `get_next_variable_key`.
/// * [`Status::BUFFER_TOO_SMALL`]: `name` is not large enough. The required
///   size (in `u16` characters, not bytes) will be returned in the error data.
/// * [`Status::INVALID_PARAMETER`]: `name` does not contain a null character, or
///   the `name` and `vendor` are not an existing variable.
/// * [`Status::DEVICE_ERROR`]: variable could not be read due to a hardware error.
/// * [`Status::UNSUPPORTED`]: this platform does not support variable storage
///   after exiting boot services.
pub fn get_next_variable_key(
    name: &mut [u16],
    vendor: &mut VariableVendor,
) -> Result<(), Option<usize>> {
    let rt = runtime_services_raw_panicking();
    let rt = unsafe { rt.as_ref() };

    let mut name_size_in_bytes = size_of_val(name);

    let status = unsafe {
        (rt.get_next_variable_name)(&mut name_size_in_bytes, name.as_mut_ptr(), &mut vendor.0)
    };
    match status {
        Status::SUCCESS => Ok(()),
        Status::BUFFER_TOO_SMALL => Err(Error::new(
            status,
            Some(name_size_in_bytes / size_of::<u16>()),
        )),
        _ => Err(Error::new(status, None)),
    }
}

/// Get an iterator over all UEFI variables.
///
/// See [`VariableKeys`] for details.
#[cfg(feature = "alloc")]
#[must_use]
pub fn variable_keys() -> VariableKeys {
    VariableKeys::new()
}

/// Iterator over all UEFI variables.
///
/// Each iteration yields a `Result<`[`VariableKey`]`>`. Error values:
///
/// * [`Status::DEVICE_ERROR`]: variable could not be read due to a hardware error.
/// * [`Status::UNSUPPORTED`]: this platform does not support variable storage
///   after exiting boot services.
#[cfg(feature = "alloc")]
#[derive(Debug)]
pub struct VariableKeys {
    name: Vec<u16>,
    vendor: VariableVendor,
    is_done: bool,
}

#[cfg(feature = "alloc")]
impl VariableKeys {
    fn new() -> Self {
        // Create a name buffer with a large default size and zero
        // initialize it. A Toshiba Satellite Pro R50-B-12P was found
        // to not correctly update the VariableNameSize passed into
        // GetNextVariableName and starting with a large buffer works
        // around this issue.
        let name = vec![0; 512];

        Self {
            name,
            // The initial vendor GUID is arbitrary.
            vendor: VariableVendor(Guid::default()),
            is_done: false,
        }
    }
}

#[cfg(feature = "alloc")]
impl Iterator for VariableKeys {
    type Item = Result<VariableKey>;

    fn next(&mut self) -> Option<Result<VariableKey>> {
        if self.is_done {
            return None;
        }

        let mut result = get_next_variable_key(&mut self.name, &mut self.vendor);

        // If the name buffer was too small, resize it to be big enough and call
        // `get_next_variable_key` again.
        if let Err(err) = &result {
            if let Some(required_size) = err.data() {
                self.name.resize(*required_size, 0u16);
                result = get_next_variable_key(&mut self.name, &mut self.vendor);
            }
        }

        match result {
            Ok(()) => {
                // Convert the name to a `CStr16`, yielding an error if invalid.
                let Ok(name) = CStr16::from_u16_until_nul(&self.name) else {
                    return Some(Err(Status::UNSUPPORTED.into()));
                };

                Some(Ok(VariableKey {
                    name: name.to_owned(),
                    vendor: self.vendor,
                }))
            }
            Err(err) => {
                if err.status() == Status::NOT_FOUND {
                    // This status indicates the end of the list. The final variable
                    // has already been yielded at this point, so return `None`.
                    self.is_done = true;
                    None
                } else {
                    // Return the error and end iteration.
                    self.is_done = true;
                    Some(Err(err.to_err_without_payload()))
                }
            }
        }
    }
}

/// Sets the value of a variable. This can be used to create a new variable,
/// update an existing variable, or (when the size of `data` is zero)
/// delete a variable.
///
/// # Warnings
///
/// The [`Status::WARN_RESET_REQUIRED`] warning will be returned when using
/// this function to transition the Secure Boot mode to setup mode or audit
/// mode if the firmware requires a reboot for that operation.
///
/// # Errors
///
/// * [`Status::INVALID_PARAMETER`]: invalid attributes, name, or vendor.
/// * [`Status::OUT_OF_RESOURCES`]: not enough storage is available to hold
///   the variable.
/// * [`Status::WRITE_PROTECTED`]: variable is read-only.
/// * [`Status::SECURITY_VIOLATION`]: variable could not be written due to an
///   authentication error.
/// * [`Status::NOT_FOUND`]: attempted to update a non-existent variable.
/// * [`Status::UNSUPPORTED`]: this platform does not support variable storage
///   after exiting boot services.
pub fn set_variable(
    name: &CStr16,
    vendor: &VariableVendor,
    attributes: VariableAttributes,
    data: &[u8],
) -> Result {
    let rt = runtime_services_raw_panicking();
    let rt = unsafe { rt.as_ref() };

    unsafe {
        (rt.set_variable)(
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
/// # Errors
///
/// * [`Status::INVALID_PARAMETER`]: invalid name or vendor.
/// * [`Status::WRITE_PROTECTED`]: variable is read-only.
/// * [`Status::SECURITY_VIOLATION`]: variable could not be deleted due to an
///   authentication error.
/// * [`Status::NOT_FOUND`]: attempted to delete a non-existent variable.
/// * [`Status::UNSUPPORTED`]: this platform does not support variable storage
///   after exiting boot services.
pub fn delete_variable(name: &CStr16, vendor: &VariableVendor) -> Result {
    set_variable(name, vendor, VariableAttributes::empty(), &[])
}

/// Get information about UEFI variable storage space for the type
/// of variable specified in `attributes`.
///
/// This operation is only supported starting with UEFI 2.0.
///
/// See [`VariableStorageInfo`] for details of the information returned.
///
/// # Errors
///
/// * [`Status::INVALID_PARAMETER`]: invalid combination of variable attributes.
/// * [`Status::UNSUPPORTED`]: the combination of variable attributes is not
///   supported on this platform, or the UEFI version is less than 2.0.
pub fn query_variable_info(attributes: VariableAttributes) -> Result<VariableStorageInfo> {
    let rt = runtime_services_raw_panicking();
    let rt = unsafe { rt.as_ref() };

    if rt.header.revision < Revision::EFI_2_00 {
        return Err(Status::UNSUPPORTED.into());
    }

    let mut info = VariableStorageInfo::default();
    unsafe {
        (rt.query_variable_info)(
            attributes,
            &mut info.maximum_variable_storage_size,
            &mut info.remaining_variable_storage_size,
            &mut info.maximum_variable_size,
        )
        .to_result_with_val(|| info)
    }
}

/// Passes capsules to the firmware.
///
/// Capsules are most commonly used to update system firmware.
///
/// # Errors
///
/// * [`Status::INVALID_PARAMETER`]: zero capsules were provided, or the
///   capsules are invalid.
/// * [`Status::DEVICE_ERROR`]: the capsule update was started but failed to a
///   device error.
/// * [`Status::OUT_OF_RESOURCES`]: before exiting boot services, indicates the
///   capsule is compatible with the platform but there are insufficient
///   resources to complete the update. After exiting boot services, indicates
///   the capsule is compatible with the platform but can only be processed
///   before exiting boot services.
/// * [`Status::UNSUPPORTED`]: this platform does not support capsule updates
///   after exiting boot services.
pub fn update_capsule(
    capsule_header_array: &[&CapsuleHeader],
    capsule_block_descriptors: &[CapsuleBlockDescriptor],
) -> Result {
    let rt = runtime_services_raw_panicking();
    let rt = unsafe { rt.as_ref() };

    unsafe {
        (rt.update_capsule)(
            capsule_header_array.as_ptr().cast(),
            capsule_header_array.len(),
            capsule_block_descriptors.as_ptr() as PhysicalAddress,
        )
        .to_result()
    }
}

/// Tests whether a capsule or capsules can be updated via [`update_capsule`].
///
/// See [`CapsuleInfo`] for details of the information returned.
///
/// # Errors
///
/// * [`Status::OUT_OF_RESOURCES`]: before exiting boot services, indicates the
///   capsule is compatible with the platform but there are insufficient
///   resources to complete the update. After exiting boot services, indicates
///   the capsule is compatible with the platform but can only be processed
///   before exiting boot services.
/// * [`Status::UNSUPPORTED`]: either the capsule type is not supported by this
///   platform, or the platform does not support capsule updates after exiting
///   boot services.
pub fn query_capsule_capabilities(capsule_header_array: &[&CapsuleHeader]) -> Result<CapsuleInfo> {
    let rt = runtime_services_raw_panicking();
    let rt = unsafe { rt.as_ref() };

    let mut info = CapsuleInfo::default();
    unsafe {
        (rt.query_capsule_capabilities)(
            capsule_header_array.as_ptr().cast(),
            capsule_header_array.len(),
            &mut info.maximum_capsule_size,
            &mut info.reset_type,
        )
        .to_result_with_val(|| info)
    }
}

/// Resets the computer.
///
/// See [`ResetType`] for details of the various reset types.
///
/// For a normal reset the value of `status` should be
/// [`Status::SUCCESS`]. Otherwise, an error code can be used.
///
/// The `data` arg is usually `None`. Otherwise, it must contain a UCS-2
/// null-terminated string followed by additional binary data. For
/// [`ResetType::PLATFORM_SPECIFIC`], the binary data must be a vendor-specific
/// [`Guid`] that indicates the type of reset to perform.
///
/// This function never returns.
pub fn reset(reset_type: ResetType, status: Status, data: Option<&[u8]>) -> ! {
    let rt = runtime_services_raw_panicking();
    let rt = unsafe { rt.as_ref() };

    let (size, data) = data
        .map(|data| (data.len(), data.as_ptr()))
        .unwrap_or((0, ptr::null()));

    unsafe { (rt.reset_system)(reset_type, status, size, data) }
}

/// Changes the runtime addressing mode of EFI firmware from physical to
/// virtual. It is up to the caller to translate the old system table address
/// to a new virtual address and provide it for this function.
///
/// If successful, this function will call [`set_system_table`] with
/// `new_system_table_virtual_addr`.
///
/// [`set_system_table`]: table::set_system_table
///
/// # Safety
///
/// The caller must ensure the memory map is valid.
///
/// # Errors
///
/// * [`Status::UNSUPPORTED`]: either boot services haven't been exited, the
///   firmware's addressing mode is already virtual, or the firmware does not
///   support this operation.
/// * [`Status::NO_MAPPING`]: `map` is missing a required range.
/// * [`Status::NOT_FOUND`]: `map` contains an address that is not in the
///   current memory map.
pub unsafe fn set_virtual_address_map(
    map: &mut [MemoryDescriptor],
    new_system_table_virtual_addr: *const uefi_raw::table::system::SystemTable,
) -> Result {
    let rt = runtime_services_raw_panicking();
    let rt = unsafe { rt.as_ref() };

    // Unsafe Code Guidelines guarantees that there is no padding in an array or a slice
    // between its elements if the element type is `repr(C)`, which is our case.
    //
    // See https://rust-lang.github.io/unsafe-code-guidelines/layout/arrays-and-slices.html
    let map_size = size_of_val(map);
    let entry_size = size_of::<MemoryDescriptor>();
    let entry_version = MemoryDescriptor::VERSION;
    let map_ptr = map.as_mut_ptr();
    unsafe { (rt.set_virtual_address_map)(map_size, entry_size, entry_version, map_ptr) }
        .to_result()?;

    // Update the global system table pointer.
    unsafe { table::set_system_table(new_system_table_virtual_addr) };

    Ok(())
}

/// Date and time representation.
#[derive(Copy, Clone, Eq, PartialEq)]
#[repr(transparent)]
pub struct Time(uefi_raw::time::Time);

/// Input parameters for [`Time::new`].
#[derive(Copy, Clone, Debug)]
pub struct TimeParams {
    /// Year in the range `1900..=9999`.
    pub year: u16,

    /// Month in the range `1..=12`.
    pub month: u8,

    /// Day in the range `1..=31`.
    pub day: u8,

    /// Hour in the range `0.=23`.
    pub hour: u8,

    /// Minute in the range `0..=59`.
    pub minute: u8,

    /// Second in the range `0..=59`.
    pub second: u8,

    /// Fraction of a second represented as nanoseconds in the range
    /// `0..=999_999_999`.
    pub nanosecond: u32,

    /// Offset in minutes from UTC in the range `-1440..=1440`, or
    /// local time if `None`.
    pub time_zone: Option<i16>,

    /// Daylight savings time information.
    pub daylight: Daylight,
}

/// Error returned by [`Time`] methods. A bool value of `true` means
/// the specified field is outside its valid range.
#[allow(missing_docs)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TimeError {
    pub year: bool,
    pub month: bool,
    pub day: bool,
    pub hour: bool,
    pub minute: bool,
    pub second: bool,
    pub nanosecond: bool,
    pub timezone: bool,
    pub daylight: bool,
}

impl core::error::Error for TimeError {}

impl Display for TimeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if self.year {
            writeln!(f, "year not within `1900..=9999`")?;
        }
        if self.month {
            writeln!(f, "month not within `1..=12")?;
        }
        if self.day {
            writeln!(f, "day not within `1..=31`")?;
        }
        if self.hour {
            writeln!(f, "hour not within `0..=23`")?;
        }
        if self.minute {
            writeln!(f, "minute not within `0..=59`")?;
        }
        if self.second {
            writeln!(f, "second not within `0..=59`")?;
        }
        if self.nanosecond {
            writeln!(f, "nanosecond not within `0..=999_999_999`")?;
        }
        if self.timezone {
            writeln!(
                f,
                "time_zone not `Time::UNSPECIFIED_TIMEZONE` nor within `-1440..=1440`"
            )?;
        }
        if self.daylight {
            writeln!(f, "unknown bits set for daylight")?;
        }
        Ok(())
    }
}

impl Time {
    /// Unspecified Timezone/local time.
    const UNSPECIFIED_TIMEZONE: i16 = uefi_raw::time::Time::UNSPECIFIED_TIMEZONE;

    /// Create a `Time` value. If a field is not in the valid range,
    /// [`TimeError`] is returned.
    pub fn new(params: TimeParams) -> core::result::Result<Self, TimeError> {
        let time = Self(uefi_raw::time::Time {
            year: params.year,
            month: params.month,
            day: params.day,
            hour: params.hour,
            minute: params.minute,
            second: params.second,
            pad1: 0,
            nanosecond: params.nanosecond,
            time_zone: params.time_zone.unwrap_or(Self::UNSPECIFIED_TIMEZONE),
            daylight: params.daylight,
            pad2: 0,
        });

        time.is_valid().map(|_| time)
    }

    /// Create an invalid `Time` with all fields set to zero. This can
    /// be used with [`FileInfo`] to indicate a field should not be
    /// updated when calling [`File::set_info`].
    ///
    /// [`FileInfo`]: uefi::proto::media::file::FileInfo
    /// [`File::set_info`]: uefi::proto::media::file::File::set_info
    #[must_use]
    pub const fn invalid() -> Self {
        Self(uefi_raw::time::Time::invalid())
    }

    /// `Ok()` if all fields are within valid ranges, `Err(TimeError)` otherwise.
    pub fn is_valid(&self) -> core::result::Result<(), TimeError> {
        let mut err = TimeError::default();
        if !(1900..=9999).contains(&self.year()) {
            err.year = true;
        }
        if !(1..=12).contains(&self.month()) {
            err.month = true;
        }
        if !(1..=31).contains(&self.day()) {
            err.day = true;
        }
        if self.hour() > 23 {
            err.hour = true;
        }
        if self.minute() > 59 {
            err.minute = true;
        }
        if self.second() > 59 {
            err.second = true;
        }
        if self.nanosecond() > 999_999_999 {
            err.nanosecond = true;
        }
        if self.time_zone().is_some() && !((-1440..=1440).contains(&self.time_zone().unwrap())) {
            err.timezone = true;
        }
        // All fields are false, i.e., within their valid range.
        if err == TimeError::default() {
            Ok(())
        } else {
            Err(err)
        }
    }

    /// Query the year.
    #[must_use]
    pub const fn year(&self) -> u16 {
        self.0.year
    }

    /// Query the month.
    #[must_use]
    pub const fn month(&self) -> u8 {
        self.0.month
    }

    /// Query the day.
    #[must_use]
    pub const fn day(&self) -> u8 {
        self.0.day
    }

    /// Query the hour.
    #[must_use]
    pub const fn hour(&self) -> u8 {
        self.0.hour
    }

    /// Query the minute.
    #[must_use]
    pub const fn minute(&self) -> u8 {
        self.0.minute
    }

    /// Query the second.
    #[must_use]
    pub const fn second(&self) -> u8 {
        self.0.second
    }

    /// Query the nanosecond.
    #[must_use]
    pub const fn nanosecond(&self) -> u32 {
        self.0.nanosecond
    }

    /// Query the time offset in minutes from UTC, or None if using local time.
    #[must_use]
    pub const fn time_zone(&self) -> Option<i16> {
        if self.0.time_zone == Self::UNSPECIFIED_TIMEZONE {
            None
        } else {
            Some(self.0.time_zone)
        }
    }

    /// Query the daylight savings time information.
    #[must_use]
    pub const fn daylight(&self) -> Daylight {
        self.0.daylight
    }
}

impl Debug for Time {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "{:04}-{:02}-{:02} ",
            self.0.year, self.0.month, self.0.day
        )?;
        write!(
            f,
            "{:02}:{:02}:{:02}.{:09}",
            self.0.hour, self.0.minute, self.0.second, self.0.nanosecond
        )?;
        if self.0.time_zone == Self::UNSPECIFIED_TIMEZONE {
            write!(f, ", Timezone=local")?;
        } else {
            write!(f, ", Timezone={}", self.0.time_zone)?;
        }
        write!(f, ", Daylight={:?}", self.0.daylight)
    }
}

impl Display for Time {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Error returned from failing to convert a byte slice into a [`Time`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TimeByteConversionError {
    /// One or more fields of the converted [`Time`] is invalid.
    InvalidFields(TimeError),
    /// The byte slice is not large enough to hold a [`Time`].
    InvalidSize,
}

impl Display for TimeByteConversionError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Self::InvalidFields(error) => write!(f, "{error}"),
            Self::InvalidSize => write!(
                f,
                "the byte slice is not large enough to hold a Time struct"
            ),
        }
    }
}

impl TryFrom<&[u8]> for Time {
    type Error = TimeByteConversionError;

    fn try_from(bytes: &[u8]) -> core::result::Result<Self, Self::Error> {
        if size_of::<Self>() <= bytes.len() {
            let year = u16::from_le_bytes(bytes[0..2].try_into().unwrap());
            let month = bytes[2];
            let day = bytes[3];
            let hour = bytes[4];
            let minute = bytes[5];
            let second = bytes[6];
            let nanosecond = u32::from_le_bytes(bytes[8..12].try_into().unwrap());
            let time_zone = match i16::from_le_bytes(bytes[12..14].try_into().unwrap()) {
                Self::UNSPECIFIED_TIMEZONE => None,
                num => Some(num),
            };
            let daylight = Daylight::from_bits(bytes[14]).ok_or(
                TimeByteConversionError::InvalidFields(TimeError {
                    daylight: true,
                    ..Default::default()
                }),
            )?;

            let time_params = TimeParams {
                year,
                month,
                day,
                hour,
                minute,
                second,
                nanosecond,
                time_zone,
                daylight,
            };

            Self::new(time_params).map_err(TimeByteConversionError::InvalidFields)
        } else {
            Err(TimeByteConversionError::InvalidSize)
        }
    }
}

/// Unique key for a variable.
#[cfg(feature = "alloc")]
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct VariableKey {
    /// Unique identifier for the vendor.
    pub vendor: VariableVendor,

    /// Name of the variable, unique with the vendor namespace.
    pub name: CString16,
}

#[cfg(feature = "alloc")]
impl VariableKey {
    /// Name of the variable.
    #[deprecated = "Use the VariableKey.name field instead"]
    #[allow(clippy::missing_const_for_fn)] // false-positive since Rust 1.86
    pub fn name(&self) -> core::result::Result<&CStr16, crate::data_types::FromSliceWithNulError> {
        Ok(&self.name)
    }
}

#[cfg(feature = "alloc")]
impl Display for VariableKey {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "VariableKey {{ name: \"{}\", vendor: ", self.name)?;

        if self.vendor == VariableVendor::GLOBAL_VARIABLE {
            write!(f, "GLOBAL_VARIABLE")?;
        } else {
            write!(f, "{}", self.vendor.0)?;
        }

        write!(f, " }}")
    }
}

/// Information about UEFI variable storage space returned by
/// [`query_variable_info`]. Note that the data here is
/// limited to a specific type of variable (as specified by the
/// `attributes` argument to `query_variable_info`).
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct VariableStorageInfo {
    /// Maximum size in bytes of the storage space available for
    /// variables of the specified type.
    pub maximum_variable_storage_size: u64,

    /// Remaining size in bytes of the storage space available for
    /// variables of the specified type.
    pub remaining_variable_storage_size: u64,

    /// Maximum size of an individual variable of the specified type.
    pub maximum_variable_size: u64,
}

/// Information about UEFI variable storage space returned by
/// [`query_capsule_capabilities`].
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct CapsuleInfo {
    /// The maximum size in bytes that [`update_capsule`]
    /// can support as input. Note that the size of an update capsule is composed of
    /// all [`CapsuleHeader`]s and [CapsuleBlockDescriptor]s.
    pub maximum_capsule_size: u64,

    /// The type of reset required for the capsule update.
    pub reset_type: ResetType,
}
