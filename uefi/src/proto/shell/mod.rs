//! EFI Shell Protocol v2.2

#![cfg(feature = "alloc")]

use alloc::vec::Vec;

use core::ffi::c_void;
use core::marker::PhantomData;
use core::mem::MaybeUninit;
use core::ptr;

use uefi_macros::unsafe_protocol;

use crate::{CStr16, Char16, Event, Handle, Result, Status, StatusExt};

use super::media::file::FileInfo;

/// TODO
#[repr(C)]
#[unsafe_protocol("6302d008-7f9b-4f30-87ac-60c9fef5da4e")]
pub struct Shell {
    execute: extern "efiapi" fn(
        parent_image_handle: *const Handle,
        commandline: *const Char16,
        environment: *const *const Char16,
        out_status: *mut Status,
    ) -> Status,
    get_env: extern "efiapi" fn(name: *const Char16) -> *const Char16,
    set_env:
        extern "efiapi" fn(name: *const Char16, value: *const Char16, volatile: bool) -> Status,
    get_alias: usize,
    set_alias: usize,
    get_help_text: usize,
    get_device_path_from_map: usize,
    get_map_from_device_path: usize,
    get_device_path_from_file_path: usize,
    get_file_path_from_device_path: usize,
    set_map: usize,

    get_cur_dir: extern "efiapi" fn(file_system_mapping: *const Char16) -> *const Char16,
    set_cur_dir: extern "efiapi" fn(file_system: *const Char16, directory: *const Char16) -> Status,
    open_file_list: usize,
    free_file_list: extern "efiapi" fn(file_list: *mut *mut ShellFileInfo),
    remove_dup_in_file_list: usize,

    batch_is_active: extern "efiapi" fn() -> bool,
    is_root_shell: usize,
    enable_page_break: extern "efiapi" fn(),
    disable_page_break: extern "efiapi" fn(),
    get_page_break: usize,
    get_device_name: usize,

    get_file_info: usize,
    set_file_info: usize,
    open_file_by_name: usize,
    close_file: extern "efiapi" fn(file_handle: ShellFileHandle) -> Status,
    create_file: extern "efiapi" fn(
        file_name: *const Char16,
        file_attribs: u64,
        out_file_handle: ShellFileHandle,
    ) -> Status,
    read_file: usize,
    write_file: usize,
    delete_file: extern "efiapi" fn(file_handle: ShellFileHandle) -> Status,
    delete_file_by_name: extern "efiapi" fn(file_name: *const Char16) -> Status,
    get_file_position: usize,
    set_file_position: usize,
    flush_file: extern "efiapi" fn(file_handle: ShellFileHandle) -> Status,
    find_files: extern "efiapi" fn(
        file_pattern: *const Char16,
        out_file_list: *mut *mut ShellFileInfo,
    ) -> Status,
    find_files_in_dir: extern "efiapi" fn(
        file_dir_handle: ShellFileHandle,
        out_file_list: *mut *mut ShellFileInfo,
    ) -> Status,
    get_file_size: extern "efiapi" fn(file_handle: ShellFileHandle, size: *mut u64) -> Status,

    open_root: usize,
    open_root_by_handle: usize,

    execution_break: Event,

    major_version: u32,
    minor_version: u32,
    register_guid_name: usize,
    get_guid_name: usize,
    get_guid_from_name: usize,
    get_env_ex: usize,
}

impl core::fmt::Debug for Shell {
    fn fmt(&self, _f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        todo!()
    }
}

/// Enum describing output options for get_env()
///
/// `EnvOutput::Val` - Value for a given variable
/// `EnvOutput::VarVec` - Vec of current environment variables
#[derive(Debug)]
pub enum EnvOutput<'a> {
    /// Value for a given variable
    Val(&'a CStr16),
    /// Vec of current environment variable names
    Vec(Vec<&'a CStr16>),
}

impl<'a> EnvOutput<'a> {
    /// Extracts the env var value from EnvOutput
    pub fn val(self) -> Option<&'a CStr16> {
        match self {
            EnvOutput::Val(v) => Some(v),
            _ => None,
        }
    }

    /// Extracts the vector of variable names from EnvOutput
    pub fn vec(self) -> Option<Vec<&'a CStr16>> {
        match self {
            EnvOutput::Vec(v) => Some(v),
            _ => None,
        }
    }
}

impl Shell {
    /// TODO
    pub fn execute(
        &self,
        parent_image: Handle,
        command_line: &CStr16,
        environment: &[&CStr16],
    ) -> Result<Status> {
        let mut out_status: MaybeUninit<Status> = MaybeUninit::uninit();
        // We have to do this in two parts, an `as` cast straight to *const *const CStr16 doesn't compile
        // let environment = environment.as_ptr();
        // let environment = environment.cast::<*const CStr16>();

        let cl_ptr = command_line.as_ptr();
        unsafe {
            let env_ptr: *const *const Char16 = (&(*environment.as_ptr()).as_ptr()).cast();

            (self.execute)(&parent_image, cl_ptr, env_ptr, out_status.as_mut_ptr())
                .to_result_with_val(|| out_status.assume_init())
        }
    }

    /// Gets the environment variable or list of environment variables
    ///
    /// # Arguments
    ///
    /// * `name` - The environment variable name of which to retrieve the
    ///            value
    ///            If None, will return all defined shell environment
    ///            variables
    ///
    /// # Returns
    ///
    /// * `Some(env_value)` - Value of the environment variable
    /// * `Some(Vec<env_names>)` - Vector of environment variable names
    /// * `None` - Environment variable doesn't exist
    pub fn get_env<'a>(&'a self, name: Option<&CStr16>) -> Option<EnvOutput<'a>> {
        match name {
            Some(n) => {
                let name_ptr: *const Char16 = (n as *const CStr16).cast();
                let var_val = (self.get_env)(name_ptr);
                if var_val.is_null() {
                    None
                } else {
                    unsafe { Some(EnvOutput::Val(CStr16::from_ptr(var_val))) }
                }
            }
            None => {
                let mut env_vec = Vec::new();
                let cur_env_ptr = (self.get_env)(ptr::null());

                let mut cur_start = cur_env_ptr;
                let mut cur_len = 0;

                let mut i = 0;
                let mut null_count = 0;
                unsafe {
                    while null_count <= 1 {
                        if (*(cur_env_ptr.add(i))) == Char16::from_u16_unchecked(0) {
                            if cur_len > 0 {
                                env_vec.push(CStr16::from_char16_with_nul_unchecked(
                                    &(*ptr::slice_from_raw_parts(cur_start, cur_len + 1)),
                                ));
                            }
                            cur_len = 0;
                            null_count += 1;
                        } else {
                            if null_count > 0 {
                                cur_start = cur_env_ptr.add(i);
                            }
                            null_count = 0;
                            cur_len += 1;
                        }
                        i += 1;
                    }
                }
                Some(EnvOutput::Vec(env_vec))
            }
        }
    }

    /// Sets the environment variable
    ///
    /// # Arguments
    ///
    /// * `name` - The environment variable for which to set the value
    /// * `value` - The new value of the environment variable
    /// * `volatile` - Indicates whether or not the variable is volatile or
    ///                not
    ///
    /// # Returns
    ///
    /// * `Status::SUCCESS` The variable was successfully set
    pub fn set_env(&self, name: &CStr16, value: &CStr16, volatile: bool) -> Status {
        let name_ptr: *const Char16 = (name as *const CStr16).cast();
        let value_ptr: *const Char16 = (value as *const CStr16).cast();
        (self.set_env)(name_ptr, value_ptr, volatile)
    }

    /// Returns the current directory on the specified device
    ///
    /// # Arguments
    ///
    /// * `file_system_mapping` - The file system mapping for which to get
    ///                           the current directory
    /// # Returns
    ///
    /// * `Some(cwd)` - CStr16 containing the current working directory
    /// * `None` - Could not retrieve current directory
    #[must_use]
    pub fn get_cur_dir<'a>(&'a self, file_system_mapping: Option<&CStr16>) -> Option<&'a CStr16> {
        let mapping_ptr: *const Char16 = file_system_mapping.map_or(ptr::null(), |x| (x.as_ptr()));
        let cur_dir = (self.get_cur_dir)(mapping_ptr);
        if cur_dir.is_null() {
            None
        } else {
            unsafe { Some(CStr16::from_ptr(cur_dir)) }
        }
    }

    /// Changes the current directory on the specified device
    ///
    /// # Arguments
    ///
    /// * `file_system` - Pointer to the file system's mapped name.
    /// * `directory` - Points to the directory on the device specified by
    ///                 `file_system`.
    /// # Returns
    ///
    /// * `Status::SUCCESS` The directory was successfully set
    ///
    /// # Errors
    ///
    /// * `Status::EFI_NOT_FOUND` The directory does not exist
    pub fn set_cur_dir(&self, file_system: Option<&CStr16>, directory: Option<&CStr16>) -> Status {
        let fs_ptr: *const Char16 = file_system.map_or(ptr::null(), |x| (x.as_ptr()));
        let dir_ptr: *const Char16 = directory.map_or(ptr::null(), |x| (x.as_ptr()));
        (self.set_cur_dir)(fs_ptr, dir_ptr)
    }

    /// Returns `true` if any script files are currently being processed.
    #[must_use]
    pub fn batch_is_active(&self) -> bool {
        (self.batch_is_active)()
    }

    /// Disables the page break output mode.
    pub fn disable_page_break(&self) {
        (self.disable_page_break)()
    }

    /// Enables the page break output mode.
    pub fn enable_page_break(&self) {
        (self.enable_page_break)()
    }

    /// Closes `file_handle`. All data is flushed to the device and the file is closed.
    ///
    /// Per the UEFI spec, the file handle will be closed in all cases and this function
    /// only returns [`Status::SUCCESS`].
    pub fn close_file(&self, file_handle: ShellFileHandle) -> Result<()> {
        (self.close_file)(file_handle).to_result()
    }

    /// Creates a file or directory by name
    ///
    /// # Arguments
    ///
    /// * `file_name` - Name of the file to be created (null terminated)
    /// * `file_attribs` - Attributes of the new file
    /// * `file_handle` - On return, points to the created file/directory's
    /// handle
    pub fn create_file(&self, file_name: &CStr16, file_attribs: u64) -> Result<ShellFileHandle> {
        // TODO: Find out how we could take a &str instead, or maybe AsRef<str>, though I think it needs `alloc`
        // the returned handle can possibly be NULL, so we need to wrap `ShellFileHandle` in an `Option`
        //let mut out_file_handle: MaybeUninit<Option<ShellFileHandle>> = MaybeUninit::zeroed();
        // let mut file_handle: ShellFileHandle;
        let file_handle = ptr::null();
        let file_name_ptr = file_name.as_ptr();

        (self.create_file)(file_name_ptr, file_attribs, file_handle)
            .to_result_with_val(|| file_handle)
        // Safety: if this call is successful, `out_file_handle`
        // will always be initialized and valid.
        // .to_result_with_val(|| unsafe { out_file_handle.assume_init() })
    }

    /// TODO
    pub fn delete_file(&self, file_handle: ShellFileHandle) -> Result<()> {
        (self.delete_file)(file_handle).to_result()
    }

    /// TODO
    pub fn delete_file_by_name(&self, file_name: &CStr16) -> Result<()> {
        (self.delete_file_by_name)(file_name.as_ptr()).to_result()
    }

    /// TODO
    pub fn find_files(&self, file_pattern: &CStr16) -> Result<Option<FileList>> {
        let mut out_list: MaybeUninit<*mut ShellFileInfo> = MaybeUninit::uninit();
        let out_ptr = out_list.as_mut_ptr();
        if out_ptr.is_null() {
            panic!("outptr null");
        }
        let fp_ptr = file_pattern.as_ptr();
        (self.find_files)(fp_ptr, out_ptr).to_result_with_val(|| {
            // safety: if we're here, out_list is valid, but maybe null
            let out_list = unsafe { out_list.assume_init() };
            if out_list.is_null() {
                None
            } else {
                let file_list = FileList::new(out_list.cast(), self);
                Some(file_list)
            }
        })
    }

    /// TODO, basically the same as `find_files`
    pub fn find_files_in_dir(&self, file_dir_handle: ShellFileHandle) -> Result<Option<FileList>> {
        let mut out_list: MaybeUninit<*mut ShellFileInfo> = MaybeUninit::uninit();
        (self.find_files_in_dir)(file_dir_handle, out_list.as_mut_ptr()).to_result_with_val(|| {
            // safety: if we're here, out_list is valid, but maybe null
            let out_list = unsafe { out_list.assume_init() };
            if out_list.is_null() {
                None
            } else {
                let file_list = FileList::new(out_list.cast(), self);
                Some(file_list)
            }
        })
    }

    /// Flushes all modified data associated with a file to a device.
    ///
    /// # Returns
    ///
    /// * `Ok(Some(file_iter))` - if one or more files were found that match the given pattern,
    ///                           where `file_iter` is an iterator over the matching files.
    /// * `Ok(None)` - if no files were found that match the given pattern.
    /// * `Err(e)` - if an error occurred while searching for files. The specific error variants
    ///              are described below.
    ///
    /// # Errors
    ///
    /// This function returns errors directly from the UEFI function
    /// `EFI_SHELL_PROTOCOL.FlushFile()`.
    ///
    /// See the function definition in the EFI Shell Specification v2.2, Chapter 2.2
    /// for more information on each error type.
    ///
    /// * [`uefi::Status::NO_MEDIA`]
    /// * [`uefi::Status::DEVICE_ERROR`]
    /// * [`uefi::Status::VOLUME_CORRUPTED`]
    /// * [`uefi::Status::WRITE_PROTECTED`]
    /// * [`uefi::Status::ACCESS_DENIED`]
    /// * [`uefi::Status::VOLUME_FULL`]
    pub fn flush_file(&self, file_handle: ShellFileHandle) -> Result<()> {
        (self.flush_file)(file_handle).to_result()
    }

    /// Gets the size of a file
    ///
    /// # Arguments
    ///
    /// * `file_handle` - Handle to the file of which the size will be retrieved
    /// * `size` - Pointer to u64 to read into
    ///
    /// # Errors
    ///
    /// * [`STATUS::EFI_DEVICE_ERROR] The file could not be accessed
    pub fn get_file_size(&self, file_handle: ShellFileHandle, size: *mut u64) -> Result<()> {
        (self.get_file_size)(file_handle, size).to_result()
    }
}

/// TODO
// #[repr(transparent)]
// #[derive(Debug)]
pub type ShellFileHandle = *const c_void;
// pub struct ShellFileHandle(c_void);
//
// impl ShellFileHandle {
//     /// Creates a new ShellFileHandle from a given c_void pointer
//     pub const unsafe fn new(ptr: c_void) -> Self {
//         Self(ptr)
//     }
// }

/// TODO
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct ShellFileInfo {
    link: ListEntry,
    status: Status,
    full_name: *const CStr16,
    file_name: *const CStr16,
    shell_file_handle: Handle,
    info: *mut FileInfo,
}

impl ShellFileInfo {
    /// TODO
    #[must_use]
    pub fn file_name(&self) -> &CStr16 {
        unsafe { &*self.file_name }
    }
}

/// TODO
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct ListEntry {
    flink: *mut ListEntry,
    blink: *mut ListEntry,
}

/// TODO
#[derive(Debug)]
pub struct FileListIter<'list> {
    current_node: *const ListEntry,
    current_node_back: *const ListEntry,
    _marker: PhantomData<&'list ListEntry>,
}

impl<'l> FileListIter<'l> {
    fn new(start: *const ListEntry, end: *const ListEntry, _shell: &'l Shell) -> Self {
        assert!(!start.is_null());
        assert!(!end.is_null());
        // Safety: all `ShellFileInfo` pointers are `ListEntry` pointers and vica-versa
        Self {
            current_node: start,
            current_node_back: end,
            _marker: PhantomData,
        }
    }
}

impl<'l> Iterator for FileListIter<'l> {
    type Item = &'l ShellFileInfo;

    fn next(&mut self) -> Option<Self::Item> {
        // Safety: This is safe as we're dereferencing a pointer that we've already null-checked
        unsafe {
            if (*self.current_node).flink.is_null() {
                None
            } else {
                self.current_node = (*self.current_node).flink;
                let ret = self.current_node.cast::<ShellFileInfo>();
                // Safety: all `ShellFileInfo` pointers are `ListEntry` pointers and vica-versa
                Some(&*ret)
            }
        }
    }
}

impl<'l> DoubleEndedIterator for FileListIter<'l> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.current_node == self.current_node_back {
            None
        } else {
            let ret = self.current_node_back.cast::<ShellFileInfo>();
            // safety: the equality check in the other branch should ensure we're always
            // pointing to a valid node
            self.current_node_back = unsafe { (*self.current_node_back).blink };
            unsafe { Some(&*ret) }
        }
    }
}

/// Safe abstraction over the linked list returned by functions such as `Shell::find_files` or
/// `Shell::find_files_in_dir`. The list and all related structures will be freed when this
/// goes out of scope.
#[derive(Debug)]
pub struct FileList<'a> {
    start: *const ListEntry,
    end: *const ListEntry,
    shell_protocol: &'a Shell,
}

impl<'a> FileList<'a> {
    #[must_use]
    #[inline]
    fn new(root: *const ListEntry, shell: &'a Shell) -> Self {
        assert!(!root.is_null());

        Self {
            start: root,
            end: core::ptr::null(),
            shell_protocol: shell,
        }
    }

    /// Returns an iterator over the file list.
    #[must_use]
    #[inline]
    pub fn iter(&'a self) -> FileListIter<'a> {
        if self.end.is_null() {
            // generate `self.end`
            let _ = self.last();
        }

        FileListIter::new(self.start, self.end, self.shell_protocol)
    }

    /// Returns the first element of the file list
    #[must_use]
    #[inline]
    pub fn first(&'a self) -> &'a ShellFileInfo {
        // safety: once `self` is created, start is valid
        unsafe { &*self.start.cast() }
    }

    /// Returns the element at the specified index or `None` if the index is invalid.
    #[must_use]
    #[inline]
    pub fn get(&'a self, index: usize) -> Option<&'a ShellFileInfo> {
        self.iter().nth(index)
    }

    /// Returns the last element of the file list.
    ///
    /// The end position is lazily generated on the first call to this function.
    #[must_use]
    #[inline]
    pub fn last(&'a self) -> &'a ShellFileInfo {
        if !self.end.is_null() {
            unsafe { &*self.end.cast() }
        } else {
            // traverse the list, keeping track of the last seen element
            // we specifically do not use `self.iter().last()` here to avoid
            // looping forever
            let mut last = self.start;

            unsafe {
                while !(*last).flink.is_null() {
                    last = (*last).flink;
                }

                &*(last.cast())
            }
        }
    }
}

impl<'a> Drop for FileList<'a> {
    fn drop(&mut self) {
        let mut root = self.start as *mut ListEntry;
        let file_list_ptr = &mut root as *mut *mut ListEntry;
        // Call the firmware's allocator to free
        (self.shell_protocol.free_file_list)(file_list_ptr.cast::<*mut ShellFileInfo>());
    }
}
