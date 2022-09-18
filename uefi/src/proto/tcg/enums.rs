// Providing docstrings for each constant would be a lot of work, so
// allow missing docs. Each type-level doc links to the relevant spec to
// provide more info.
//
// Setting this at the module level so that we don't have to write it
// above each constant. That's also why these enums are in a separate
// module instead of `super`, since we don't want to allow missing docs
// too broadly.
#![allow(missing_docs)]

newtype_enum! {
    /// Algorithm identifiers.
    ///
    /// These values are defined in the [TCG Algorithm Registry].
    ///
    /// [TCG Algorithm Registry]: https://trustedcomputinggroup.org/resource/tcg-algorithm-registry/
    pub enum AlgorithmId: u16 => {
        ERROR = 0x0000,
        RSA = 0x0001,
        TDES = 0x0003,
        SHA1 = 0x0004,
        HMAC = 0x0005,
        AES = 0x0006,
        MGF1 = 0x0007,
        KEYED_HASH = 0x0008,
        XOR = 0x000a,
        SHA256 = 0x000b,
        SHA384 = 0x000c,
        SHA512 = 0x000d,
        NULL = 0x0010,
        SM3_256 = 0x0012,
        SM4 = 0x0013,
        // TODO: there are a bunch more, but the above list is probably
        // more than sufficient for real devices.
    }
}

newtype_enum! {
    /// Event types stored in the TPM event log. The event type defines
    /// which structure type is stored in the event data.
    ///
    /// For details of each variant, see the [TCG PC Client Platform
    /// Firmware Protocol Specification][spec], in particular the Events
    /// table in the Event Logging chapter.
    ///
    /// [spec]: https://trustedcomputinggroup.org/resource/pc-client-specific-platform-firmware-profile-specification/
    pub enum EventType: u32 => {
        PREBOOT_CERT = 0x0000_0000,
        POST_CODE = 0x0000_0001,
        UNUSED = 0x0000_0002,
        NO_ACTION = 0x0000_0003,
        SEPARATOR = 0x0000_0004,
        ACTION = 0x0000_0005,
        EVENT_TAG = 0x0000_0006,
        CRTM_CONTENTS = 0x0000_0007,
        CRTM_VERSION = 0x0000_0008,
        CPU_MICROCODE = 0x0000_0009,
        PLATFORM_CONFIG_FLAGS = 0x0000_000a,
        TABLE_OF_DEVICES = 0x0000_000b,
        COMPACT_HASH = 0x0000_000c,
        IPL = 0x0000_000d,
        IPL_PARTITION_DATA = 0x0000_000e,
        NONHOST_CODE = 0x0000_000f,
        NONHOST_CONFIG = 0x0000_0010,
        NONHOST_INFO = 0x0000_0011,
        OMIT_BOOT_DEVICE_EVENTS = 0x0000_0012,
        EFI_EVENT_BASE = 0x8000_0000,
        EFI_VARIABLE_DRIVER_CONFIG = 0x8000_0001,
        EFI_VARIABLE_BOOT = 0x8000_0002,
        EFI_BOOT_SERVICES_APPLICATION = 0x8000_0003,
        EFI_BOOT_SERVICES_DRIVER = 0x8000_0004,
        EFI_RUNTIME_SERVICES_DRIVER = 0x8000_0005,
        EFI_GPT_EVENT = 0x8000_0006,
        EFI_ACTION = 0x8000_0007,
        EFI_PLATFORM_FIRMWARE_BLOB = 0x8000_0008,
        EFI_HANDOFF_TABLES = 0x8000_0009,
        EFI_PLATFORM_FIRMWARE_BLOB2 = 0x8000_000a,
        EFI_HANDOFF_TABLES2 = 0x8000_000b,
        EFI_VARIABLE_BOOT2 = 0x8000_000c,
        EFI_HCRTM_EVENT = 0x8000_0010,
        EFI_VARIABLE_AUTHORITY = 0x8000_00e0,
        EFI_SPDM_FIRMWARE_BLOB = 0x8000_00e1,
        EFI_SPDM_FIRMWARE_CONFIG = 0x8000_00e2,
    }
}
