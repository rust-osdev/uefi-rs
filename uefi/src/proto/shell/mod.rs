//! EFI Shell Protocol v2.2

use core::{ffi::c_void, marker::PhantomData, mem::MaybeUninit, ptr::NonNull};

use uefi_macros::unsafe_protocol;

use crate::{CStr16, Char16, Event, Handle, Result, Status, StatusExt};

use super::media::file::FileInfo;

/// TODO
#[repr(C)]
#[unsafe_protocol("6302d008-7f9b-4f30-87ac-60c9fef5da4e")]
pub struct Shell {
    execute: extern "efiapi" fn(
        parent_image_handle: *const Handle,
        commandline: *const CStr16,
        environment: *const *const CStr16,
        out_status: *mut Status,
    ) -> Status,
    get_env: usize,
    set_env: usize,
    get_alias: usize,
    set_alias: usize,
    get_help_text: usize,
    get_device_path_from_map: usize,
    get_map_from_device_path: usize,
    get_device_path_from_file_path: usize,
    get_file_path_from_device_path: usize,
    set_map: usize,

    get_cur_dir: extern "efiapi" fn(file_system_mapping: *const Char16) -> *const CStr16,
    set_cur_dir: usize,
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
        file_name: &CStr16,
        file_attribs: u64,
        out_file_handle: *mut ShellFileHandle,
    ) -> Status,
    read_file: usize,
    write_file: usize,
    delete_file: extern "efiapi" fn(file_handle: ShellFileHandle) -> Status,
    delete_file_by_name: extern "efiapi" fn(file_name: &CStr16) -> Status,
    get_file_position: usize,
    set_file_position: usize,
    flush_file: extern "efiapi" fn(file_handle: ShellFileHandle) -> Status,
    find_files: extern "efiapi" fn(
        file_pattern: *const CStr16,
        out_file_list: *mut *mut ShellFileInfo,
    ) -> Status,
    find_files_in_dir: extern "efiapi" fn(
        file_dir_handle: ShellFileHandle,
        out_file_list: *mut *mut ShellFileInfo,
    ) -> Status,
    get_file_size: usize,

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
        let environment = environment.as_ptr();
        let environment = environment.cast::<*const CStr16>();

        (self.execute)(
            &parent_image,
            command_line,
            environment,
            out_status.as_mut_ptr(),
        )
        .to_result_with_val(|| unsafe { out_status.assume_init() })
    }

    /// TODO
    #[must_use]
    pub fn get_cur_dir<'a>(&'a self, file_system_mapping: Option<&CStr16>) -> Option<&'a CStr16> {
        let mapping_ptr: *const Char16 =
            file_system_mapping.map_or(core::ptr::null(), |x| (x as *const CStr16).cast());
        let cur_dir = (self.get_cur_dir)(mapping_ptr);
        if cur_dir.is_null() {
            None
        } else {
            unsafe { Some(&*cur_dir) }
        }
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

    /// TODO
    pub fn create_file(
        &self,
        file_name: &CStr16,
        file_attribs: u64,
    ) -> Result<Option<ShellFileHandle>> {
        // TODO: Find out how we could take a &str instead, or maybe AsRef<str>, though I think it needs `alloc`
        // the returned handle can possibly be NULL, so we need to wrap `ShellFileHandle` in an `Option`
        let mut out_file_handle: MaybeUninit<Option<ShellFileHandle>> = MaybeUninit::zeroed();

        (self.create_file)(file_name, file_attribs, out_file_handle.as_mut_ptr().cast())
            // Safety: if this call is successful, `out_file_handle`
            // will always be initialized and valid.
            .to_result_with_val(|| unsafe { out_file_handle.assume_init() })
    }

    /// TODO
    pub fn delete_file(&self, file_handle: ShellFileHandle) -> Result<()> {
        (self.delete_file)(file_handle).to_result()
    }

    /// TODO
    pub fn delete_file_by_name(&self, file_name: &CStr16) -> Result<()> {
        (self.delete_file_by_name)(file_name).to_result()
    }

    /// TODO
    pub fn find_files(&self, file_pattern: &CStr16) -> Result<Option<FileList>> {
        let mut out_list: MaybeUninit<*mut ShellFileInfo> = MaybeUninit::uninit();
        let out_ptr = out_list.as_mut_ptr();
        if out_ptr.is_null() {
            panic!("outptr null");
        }
        (self.find_files)(file_pattern, out_ptr).to_result_with_val(|| {
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
}

/// TODO
#[repr(transparent)]
#[derive(Debug)]
pub struct ShellFileHandle(NonNull<c_void>);

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
