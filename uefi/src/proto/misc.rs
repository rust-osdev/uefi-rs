//! Miscellaneous protocols.

use uefi_raw::protocol::misc::{
    ResetNotificationProtocol, ResetSystemFn, TimestampProperties, TimestampProtocol,
};

use crate::proto::unsafe_protocol;
use crate::{Result, StatusExt};

/// Protocol for retrieving a high-resolution timestamp counter.
#[derive(Debug)]
#[repr(transparent)]
#[unsafe_protocol(TimestampProtocol::GUID)]
pub struct Timestamp(TimestampProtocol);

impl Timestamp {
    /// Get the current value of the timestamp counter.
    #[must_use]
    pub fn get_timestamp(&self) -> u64 {
        unsafe { (self.0.get_timestamp)() }
    }

    /// Get the properties of the timestamp counter.
    pub fn get_properties(&self) -> Result<TimestampProperties> {
        let mut properties = TimestampProperties::default();
        unsafe { (self.0.get_properties)(&mut properties) }.to_result_with_val(|| properties)
    }
}

/// Protocol to register for a notification when ResetSystem is called.
#[derive(Debug)]
#[repr(transparent)]
#[unsafe_protocol(ResetNotificationProtocol::GUID)]
pub struct ResetNotification(ResetNotificationProtocol);

impl ResetNotification {
    /// Register a notification function to be called when ResetSystem() is called.
    ///
    ///
    /// # Example
    ///
    /// ```rust
    /// use log::info;
    /// use uefi::Handle;
    /// use uefi::prelude::BootServices;
    /// use uefi::proto::misc::{ResetNotification};
    /// use uefi_raw::Status;
    /// use uefi_raw::table::runtime;
    ///
    ///
    /// /* value efi_reset_fn is the type of ResetSystemFn, a function pointer*/
    ///     unsafe extern "efiapi" fn efi_reset_fn(
    ///             rt: runtime::ResetType,
    ///             status: Status,
    ///             data_size: usize,
    ///             data: *const u8,
    ///     ){
    ///         info!("Inside the event callback");
    ///         info!("do what you want");
    ///     }
    ///
    ///     pub fn test(image: Handle, bt: &BootServices) {
    ///
    ///         let mut rn = bt
    ///             .open_protocol_exclusive::<ResetNotification>(image)
    ///             .expect("Failed to open Timestamp protocol");
    ///
    ///         rn.register_reset_notify(Some(efi_reset_fn))
    ///             .expect("Failed to register a reset notification function!");
    ///     }
    /// ```
    pub fn register_reset_notify(&mut self, reset_function: Option<ResetSystemFn>) -> Result {
        unsafe { (self.0.register_reset_notify)(&mut self.0, reset_function) }.to_result()
    }

    /// Removes a reset notification function that has been previously registered with RegisterResetNotify().
    /// Tips: RegisterResetNotify() has named as `register_reset_notify()` in uefi-rs.
    pub fn unregister_reset_notify(&mut self, reset_function: Option<ResetSystemFn>) -> Result {
        unsafe { (self.0.unregister_reset_notify)(&mut self.0, reset_function) }.to_result()
    }
}
