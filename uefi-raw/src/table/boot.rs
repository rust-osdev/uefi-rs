//! UEFI services available during boot.

newtype_enum! {
/// Task priority level.
///
/// Although the UEFI specification repeatedly states that only the variants
/// specified below should be used in application-provided input, as the other
/// are reserved for internal firmware use, it might still happen that the
/// firmware accidentally discloses one of these internal TPLs to us.
///
/// Since feeding an unexpected variant to a Rust enum is UB, this means that
/// this C enum must be interfaced via the newtype pattern.
pub enum Tpl: usize => {
    /// Normal task execution level.
    APPLICATION = 4,
    /// Async interrupt-style callbacks run at this TPL.
    CALLBACK    = 8,
    /// Notifications are masked at this level.
    ///
    /// This is used in critical sections of code.
    NOTIFY      = 16,
    /// Highest priority level.
    ///
    /// Even processor interrupts are disable at this level.
    HIGH_LEVEL  = 31,
}}
