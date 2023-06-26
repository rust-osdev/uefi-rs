use crate::{guid, Char16, Event, Guid, Status};

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
