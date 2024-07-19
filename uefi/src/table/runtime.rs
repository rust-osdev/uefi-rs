//! UEFI services available at runtime, even after the OS boots.

use super::Revision;
use crate::table::boot::MemoryDescriptor;
use crate::{CStr16, Error, Result, Status, StatusExt};
use core::fmt::{self, Debug, Display, Formatter};
use core::mem::{size_of, MaybeUninit};
use core::ptr;

pub use uefi_raw::capsule::{CapsuleBlockDescriptor, CapsuleFlags, CapsuleHeader};
pub use uefi_raw::table::runtime::{
    ResetType, TimeCapabilities, VariableAttributes, VariableVendor,
};
pub use uefi_raw::time::Daylight;
pub use uefi_raw::PhysicalAddress;

#[cfg(feature = "alloc")]
use {
    crate::data_types::FromSliceWithNulError,
    crate::Guid,
    alloc::boxed::Box,
    alloc::{vec, vec::Vec},
    core::mem,
};

/// Contains pointers to all of the runtime services.
///
/// This table, and the function pointers it contains are valid
/// even after the UEFI OS loader and OS have taken control of the platform.
///
/// # Accessing `RuntimeServices`
///
/// A reference to `RuntimeServices` can only be accessed by calling [`SystemTable::runtime_services`].
///
/// [`SystemTable::runtime_services`]: crate::table::SystemTable::runtime_services
#[derive(Debug)]
#[repr(C)]
pub struct RuntimeServices(uefi_raw::table::runtime::RuntimeServices);

impl RuntimeServices {
    /// Query the current time and date information
    pub fn get_time(&self) -> Result<Time> {
        let mut time = MaybeUninit::<Time>::uninit();
        unsafe { (self.0.get_time)(time.as_mut_ptr().cast(), ptr::null_mut()) }
            .to_result_with_val(|| unsafe { time.assume_init() })
    }

    /// Query the current time and date information and the RTC capabilities
    pub fn get_time_and_caps(&self) -> Result<(Time, TimeCapabilities)> {
        let mut time = MaybeUninit::<Time>::uninit();
        let mut caps = MaybeUninit::<TimeCapabilities>::uninit();
        unsafe { (self.0.get_time)(time.as_mut_ptr().cast(), caps.as_mut_ptr()) }
            .to_result_with_val(|| unsafe { (time.assume_init(), caps.assume_init()) })
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
    pub unsafe fn set_time(&mut self, time: &Time) -> Result {
        let time: *const Time = time;
        (self.0.set_time)(time.cast()).to_result()
    }

    /// Get the size (in bytes) of a variable. This can be used to find out how
    /// big of a buffer should be passed in to `get_variable`.
    pub fn get_variable_size(&self, name: &CStr16, vendor: &VariableVendor) -> Result<usize> {
        let mut data_size = 0;
        let status = unsafe {
            (self.0.get_variable)(
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
    pub fn get_variable<'a>(
        &self,
        name: &CStr16,
        vendor: &VariableVendor,
        buf: &'a mut [u8],
    ) -> Result<(&'a [u8], VariableAttributes)> {
        let mut attributes = VariableAttributes::empty();
        let mut data_size = buf.len();
        unsafe {
            (self.0.get_variable)(
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
    #[cfg(feature = "alloc")]
    pub fn get_variable_boxed(
        &self,
        name: &CStr16,
        vendor: &VariableVendor,
    ) -> Result<(Box<[u8]>, VariableAttributes)> {
        let mut attributes = VariableAttributes::empty();

        let mut data_size = self.get_variable_size(name, vendor)?;
        let mut data = Vec::with_capacity(data_size);

        let status = unsafe {
            (self.0.get_variable)(
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
    #[cfg(feature = "alloc")]
    pub fn variable_keys(&self) -> Result<Vec<VariableKey>> {
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
                (self.0.get_next_variable_name)(
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
    /// # Warnings
    ///
    /// The [`Status::WARN_RESET_REQUIRED`] warning will be returned when using
    /// this function to transition the Secure Boot mode to setup mode or audit
    /// mode if the firmware requires a reboot for that operation.
    pub fn set_variable(
        &self,
        name: &CStr16,
        vendor: &VariableVendor,
        attributes: VariableAttributes,
        data: &[u8],
    ) -> Result {
        unsafe {
            (self.0.set_variable)(
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
    pub fn delete_variable(&self, name: &CStr16, vendor: &VariableVendor) -> Result {
        self.set_variable(name, vendor, VariableAttributes::empty(), &[])
    }

    /// Get information about UEFI variable storage space for the type
    /// of variable specified in `attributes`.
    ///
    /// This operation is only supported starting with UEFI 2.0; earlier
    /// versions will fail with [`Status::UNSUPPORTED`].
    ///
    /// See [`VariableStorageInfo`] for details of the information returned.
    pub fn query_variable_info(
        &self,
        attributes: VariableAttributes,
    ) -> Result<VariableStorageInfo> {
        if self.0.header.revision < Revision::EFI_2_00 {
            return Err(Status::UNSUPPORTED.into());
        }

        let mut info = VariableStorageInfo::default();
        unsafe {
            (self.0.query_variable_info)(
                attributes,
                &mut info.maximum_variable_storage_size,
                &mut info.remaining_variable_storage_size,
                &mut info.maximum_variable_size,
            )
            .to_result_with_val(|| info)
        }
    }

    /// Resets the computer.
    pub fn reset(&self, rt: ResetType, status: Status, data: Option<&[u8]>) -> ! {
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

        unsafe { (self.0.reset_system)(rt, status, size, data) }
    }

    pub(crate) unsafe fn set_virtual_address_map(
        &self,
        map_size: usize,
        desc_size: usize,
        desc_version: u32,
        virtual_map: *mut MemoryDescriptor,
    ) -> Status {
        (self.0.set_virtual_address_map)(map_size, desc_size, desc_version, virtual_map)
    }

    /// Passes capsules to the firmware. Capsules are most commonly used to update system firmware.
    pub fn update_capsule(
        &self,
        capsule_header_array: &[&CapsuleHeader],
        capsule_block_descriptors: &[CapsuleBlockDescriptor],
    ) -> Result {
        unsafe {
            (self.0.update_capsule)(
                capsule_header_array.as_ptr().cast(),
                capsule_header_array.len(),
                capsule_block_descriptors.as_ptr() as PhysicalAddress,
            )
            .to_result()
        }
    }

    /// Tests whether a capsule or capsules can be updated via [`RuntimeServices::update_capsule`].
    ///
    /// See [`CapsuleInfo`] for details of the information returned.
    pub fn query_capsule_capabilities(
        &self,
        capsule_header_array: &[&CapsuleHeader],
    ) -> Result<CapsuleInfo> {
        let mut info = CapsuleInfo::default();
        unsafe {
            (self.0.query_capsule_capabilities)(
                capsule_header_array.as_ptr().cast(),
                capsule_header_array.len(),
                &mut info.maximum_capsule_size,
                &mut info.reset_type,
            )
            .to_result_with_val(|| info)
        }
    }
}

impl super::Table for RuntimeServices {
    const SIGNATURE: u64 = 0x5652_4553_544e_5552;
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
/// the specified field is outside of its valid range.
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

#[cfg(feature = "unstable")]
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
        if size_of::<Time>() <= bytes.len() {
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

            Time::new(time_params).map_err(TimeByteConversionError::InvalidFields)
        } else {
            Err(TimeByteConversionError::InvalidSize)
        }
    }
}

/// Unique key for a variable.
#[cfg(feature = "alloc")]
#[derive(Debug)]
pub struct VariableKey {
    pub(crate) name: Vec<u16>,
    /// Unique identifier for the vendor.
    pub vendor: VariableVendor,
}

#[cfg(feature = "alloc")]
impl VariableKey {
    /// Name of the variable.
    pub fn name(&self) -> core::result::Result<&CStr16, FromSliceWithNulError> {
        CStr16::from_u16_with_nul(&self.name)
    }
}

#[cfg(feature = "alloc")]
impl Display for VariableKey {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "VariableKey {{ name: ")?;

        match self.name() {
            Ok(name) => write!(f, "\"{name}\"")?,
            Err(err) => write!(f, "Err({err:?})")?,
        }

        write!(f, ", vendor: ")?;

        if self.vendor == VariableVendor::GLOBAL_VARIABLE {
            write!(f, "GLOBAL_VARIABLE")?;
        } else {
            write!(f, "{}", self.vendor.0)?;
        }

        write!(f, " }}")
    }
}

/// Information about UEFI variable storage space returned by
/// [`RuntimeServices::query_variable_info`]. Note that the data here is
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
/// [`RuntimeServices::query_capsule_capabilities`].
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct CapsuleInfo {
    /// The maximum size in bytes that [`RuntimeServices::update_capsule`]
    /// can support as input. Note that the size of an update capsule is composed of
    /// all [`CapsuleHeader`]s and [CapsuleBlockDescriptor]s.
    pub maximum_capsule_size: u64,

    /// The type of reset required for the capsule update.
    pub reset_type: ResetType,
}

#[cfg(test)]
mod tests {
    use super::*;

    use alloc::string::ToString;
    use core::{slice, usize};

    unsafe fn time_as_u8_slice(p: &Time) -> &[u8] {
        slice::from_raw_parts(core::ptr::addr_of!(*p).cast(), size_of::<Time>())
    }

    unsafe fn time_as_u8_slice_with_size(p: &Time, len: usize) -> &[u8] {
        slice::from_raw_parts(core::ptr::addr_of!(*p).cast(), len)
    }

    #[test]
    fn test_successful_time_from_bytes() {
        let mut time;
        let mut time_from_bytes;
        let mut time_params = TimeParams {
            year: 2024,
            month: 6,
            day: 13,
            hour: 4,
            minute: 29,
            second: 30,
            nanosecond: 123_456_789,
            time_zone: None,
            daylight: Daylight::empty(),
        };

        time = Time::new(time_params).unwrap();
        unsafe {
            time_from_bytes = Time::try_from(time_as_u8_slice(&time)).unwrap();
        }
        assert_eq!(time, time_from_bytes);

        time_params.time_zone = Some(120);
        time = Time::new(time_params).unwrap();
        unsafe {
            time_from_bytes = Time::try_from(time_as_u8_slice(&time)).unwrap();
        }
        assert_eq!(time.to_string(), time_from_bytes.to_string());

        time_params.time_zone = Some(150);
        time = Time::new(time_params).unwrap();
        unsafe {
            time_from_bytes = Time::try_from(time_as_u8_slice(&time)).unwrap();
        }
        assert_eq!(time.to_string(), time_from_bytes.to_string());
    }

    #[test]
    fn test_invalid_fields_in_time_byte_conversion() {
        let time = Time::invalid();
        let time_from_bytes;
        unsafe {
            time_from_bytes = Time::try_from(time_as_u8_slice(&time)).unwrap_err();
        }
        assert_eq!(
            TimeByteConversionError::InvalidFields(TimeError {
                year: true,
                month: true,
                day: true,
                ..Default::default()
            }),
            time_from_bytes
        );
    }

    #[test]
    fn test_byte_slice_too_small_to_convert_to_time() {
        let time = Time::invalid();
        let time_from_bytes;
        unsafe {
            time_from_bytes =
                Time::try_from(time_as_u8_slice_with_size(&time, size_of::<Time>() - 1))
                    .unwrap_err();
        }
        assert_eq!(TimeByteConversionError::InvalidSize, time_from_bytes);
    }
}
