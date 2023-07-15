pub mod serial;

use crate::{guid, Char16, Event, Guid, PhysicalAddress, Status};
use core::ptr;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
#[repr(C)]
pub struct InputKey {
    pub scan_code: u16,
    pub unicode_char: Char16,
}

#[repr(C)]
pub struct SimpleTextInputProtocol {
    pub reset: unsafe extern "efiapi" fn(this: *mut Self, extended_verification: bool) -> Status,
    pub read_key_stroke: unsafe extern "efiapi" fn(this: *mut Self, key: *mut InputKey) -> Status,
    pub wait_for_key: Event,
}

impl SimpleTextInputProtocol {
    pub const GUID: Guid = guid!("387477c1-69c7-11d2-8e39-00a0c969723b");
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
#[repr(C)]
pub struct SimpleTextOutputMode {
    pub max_mode: i32,
    pub mode: i32,
    pub attribute: i32,
    pub cursor_column: i32,
    pub cursor_row: i32,
    pub cursor_visible: bool,
}

#[repr(C)]
pub struct SimpleTextOutputProtocol {
    pub reset: unsafe extern "efiapi" fn(this: *mut Self, extended: bool) -> Status,
    pub output_string: unsafe extern "efiapi" fn(this: *mut Self, string: *const Char16) -> Status,
    pub test_string: unsafe extern "efiapi" fn(this: *mut Self, string: *const Char16) -> Status,
    pub query_mode: unsafe extern "efiapi" fn(
        this: *mut Self,
        mode: usize,
        columns: *mut usize,
        rows: *mut usize,
    ) -> Status,
    pub set_mode: unsafe extern "efiapi" fn(this: *mut Self, mode: usize) -> Status,
    pub set_attribute: unsafe extern "efiapi" fn(this: *mut Self, attribute: usize) -> Status,
    pub clear_screen: unsafe extern "efiapi" fn(this: *mut Self) -> Status,
    pub set_cursor_position:
        unsafe extern "efiapi" fn(this: *mut Self, column: usize, row: usize) -> Status,
    pub enable_cursor: unsafe extern "efiapi" fn(this: *mut Self, visible: bool) -> Status,
    pub mode: *mut SimpleTextOutputMode,
}

impl SimpleTextOutputProtocol {
    pub const GUID: Guid = guid!("387477c2-69c7-11d2-8e39-00a0c969723b");
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
#[repr(C)]
pub struct SimplePointerMode {
    pub resolution_x: u64,
    pub resolution_y: u64,
    pub resolution_z: u64,
    pub left_button: u8,
    pub right_button: u8,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
#[repr(C)]
pub struct SimplePointerState {
    pub relative_movement_x: i32,
    pub relative_movement_y: i32,
    pub relative_movement_z: i32,
    pub left_button: u8,
    pub right_button: u8,
}

#[repr(C)]
pub struct SimplePointerProtocol {
    pub reset: unsafe extern "efiapi" fn(
        this: *mut SimplePointerProtocol,
        extended_verification: bool,
    ) -> Status,
    pub get_state: unsafe extern "efiapi" fn(
        this: *mut SimplePointerProtocol,
        state: *mut SimplePointerState,
    ) -> Status,
    pub wait_for_input: Event,
    pub mode: *const SimplePointerMode,
}

impl SimplePointerProtocol {
    pub const GUID: Guid = guid!("31878c87-0b75-11d5-9a4f-0090273fc14d");
}

#[repr(C)]
pub struct GraphicsOutputProtocol {
    pub query_mode: unsafe extern "efiapi" fn(
        *const Self,
        mode_number: u32,
        size_of_info: *mut usize,
        info: *mut *const GraphicsOutputModeInformation,
    ) -> Status,
    pub set_mode: unsafe extern "efiapi" fn(*mut Self, mode_number: u32) -> Status,
    pub blt: unsafe extern "efiapi" fn(
        *mut Self,
        blt_buffer: *mut GraphicsOutputBltPixel,
        blt_operation: GraphicsOutputBltOperation,
        source_x: usize,
        source_y: usize,
        destination_x: usize,
        destination_y: usize,
        width: usize,
        height: usize,
        delta: usize,
    ) -> Status,
    pub mode: *mut GraphicsOutputProtocolMode,
}

impl GraphicsOutputProtocol {
    pub const GUID: Guid = guid!("9042a9de-23dc-4a38-96fb-7aded080516a");
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[repr(C)]
pub struct GraphicsOutputProtocolMode {
    pub max_mode: u32,
    pub mode: u32,
    pub info: *mut GraphicsOutputModeInformation,
    pub size_of_info: usize,
    pub frame_buffer_base: PhysicalAddress,
    pub frame_buffer_size: usize,
}

impl Default for GraphicsOutputProtocolMode {
    fn default() -> Self {
        Self {
            max_mode: 0,
            mode: 0,
            info: ptr::null_mut(),
            size_of_info: 0,
            frame_buffer_base: 0,
            frame_buffer_size: 0,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[repr(C)]
pub struct GraphicsOutputModeInformation {
    pub version: u32,
    pub horizontal_resolution: u32,
    pub vertical_resolution: u32,
    pub pixel_format: GraphicsPixelFormat,
    pub pixel_information: PixelBitmask,
    pub pixels_per_scan_line: u32,
}

/// Bitmask used to indicate which bits of a pixel represent a given color.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[repr(C)]
pub struct PixelBitmask {
    /// The bits indicating the red channel.
    pub red: u32,

    /// The bits indicating the green channel.
    pub green: u32,

    /// The bits indicating the blue channel.
    pub blue: u32,

    /// The reserved bits, which are ignored by the video hardware.
    pub reserved: u32,
}

newtype_enum! {
    #[derive(Default)]
    pub enum GraphicsPixelFormat: u32 => {
        PIXEL_RED_GREEN_BLUE_RESERVED_8_BIT_PER_COLOR = 0,
        PIXEL_BLUE_GREEN_RED_RESERVED_8_BIT_PER_COLOR = 1,
        PIXEL_BIT_MASK = 2,
        PIXEL_BLT_ONLY = 3,
        PIXEL_FORMAT_MAX = 4,
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[repr(C)]
pub struct GraphicsOutputBltPixel {
    pub blue: u8,
    pub green: u8,
    pub red: u8,
    pub reserved: u8,
}

newtype_enum! {
    #[derive(Default)]
    pub enum GraphicsOutputBltOperation: u32 => {
        BLT_VIDEO_FILL = 0,
        BLT_VIDEO_TO_BLT_BUFFER = 1,
        BLT_BUFFER_TO_VIDEO = 2,
        BLT_VIDEO_TO_VIDEO = 3,
        GRAPHICS_OUTPUT_BLT_OPERATION_MAX = 4,
    }
}
