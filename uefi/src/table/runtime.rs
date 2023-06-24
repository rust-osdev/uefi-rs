//! UEFI services available at runtime, even after the OS boots.

use super::Revision;
use crate::table::boot::MemoryDescriptor;
use crate::{CStr16, Error, Result, Status, StatusExt};
use core::fmt::{Debug, Formatter};
use core::mem::MaybeUninit;
use core::{fmt, ptr};

pub use uefi_raw::table::runtime::{
    ResetType, TimeCapabilities, VariableAttributes, VariableVendor,
};
pub use uefi_raw::time::Daylight;

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
}

impl super::Table for RuntimeServices {
    const SIGNATURE: u64 = 0x5652_4553_544e_5552;
}

impl Debug for RuntimeServices {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("RuntimeServices")
            .field("header", &self.0.header)
            .field("get_time", &(self.0.get_time as *const u64))
            .field("set_time", &(self.0.set_time as *const u64))
            .field(
                "set_virtual_address_map",
                &(self.0.set_virtual_address_map as *const u64),
            )
            .field("reset", &(self.0.reset_system as *const u64))
            .finish()
    }
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

/// Error returned by [`Time`] methods if the input is outside the valid range.
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub struct TimeError;

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
        if time.is_valid() {
            Ok(time)
        } else {
            Err(TimeError)
        }
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

    /// True if all fields are within valid ranges, false otherwise.
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.0.is_valid()
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
        if self.0.time_zone == 2047 {
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

impl fmt::Debug for Time {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
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

impl fmt::Display for Time {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Unique key for a variable.
#[cfg(feature = "alloc")]
#[derive(Debug)]
pub struct VariableKey {
    name: Vec<u16>,
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
impl fmt::Display for VariableKey {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
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
