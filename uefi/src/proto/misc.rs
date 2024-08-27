//! Miscellaneous protocols.

use uefi_raw::protocol::misc::{
    ResetNotificationProtocol, ResetSystemFn, TimestampProperties, TimestampProtocol,
};

use crate::proto::unsafe_protocol;
use crate::{Result, StatusExt};

/// Protocol for retrieving a high-resolution timestamp counter.
/// **Note:**
/// If your UEFI firmware not support timestamp protocol which first added at UEFI spec 2.4 2013.
/// you also could use `RDTSC` in rust, here is a demo [Slint-UI](https://github.com/slint-ui/slint/blob/2c0ba2bc0f151eba8d1fa17839fa2ac58832ca80/examples/uefi-demo/main.rs#L28-L62) who use uefi-rs.
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
    /// use uefi::{boot, Handle};
    /// use uefi::proto::misc::{ResetNotification};
    /// use uefi_raw::Status;
    /// use uefi_raw::table::runtime;
    ///
    ///
    /// // value efi_reset_fn is the type of ResetSystemFn, a function pointer
    /// unsafe extern "efiapi" fn efi_reset_fn(
    ///         rt: runtime::ResetType,
    ///         status: Status,
    ///         data_size: usize,
    ///         data: *const u8,
    /// ){
    ///     info!("Inside the event callback");
    ///     info!("do what you want");
    /// }
    ///
    /// pub fn test(image: Handle) {
    ///
    ///     let mut rn = boot::open_protocol_exclusive::<ResetNotification>(image)
    ///         .expect("Failed to open Timestamp protocol");
    ///
    ///     rn.register_reset_notify(efi_reset_fn)
    ///         .expect("Failed to register a reset notification function!");
    /// }
    /// ```
    pub fn register_reset_notify(&mut self, reset_function: ResetSystemFn) -> Result {
        unsafe { (self.0.register_reset_notify)(&mut self.0, reset_function) }.to_result()
    }

    /// Remove a reset notification function that was previously registered with [`ResetNotification::register_reset_notify`].
    pub fn unregister_reset_notify(&mut self, reset_function: ResetSystemFn) -> Result {
        unsafe { (self.0.unregister_reset_notify)(&mut self.0, reset_function) }.to_result()
    }
}
