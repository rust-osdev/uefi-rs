// SPDX-License-Identifier: MIT OR Apache-2.0

use alloc::boxed::Box;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::ffi::c_void;
use core::pin::Pin;
use core::ptr;
use core::ptr::addr_of;
use uefi::proto::device_path::build::DevicePathBuilder;
use uefi::proto::media::load_file::{LoadFile, LoadFile2};
use uefi::proto::BootPolicy;
use uefi::{boot, Guid, Handle};
use uefi_raw::protocol::device_path::DevicePathProtocol;
use uefi_raw::protocol::media::{LoadFile2Protocol, LoadFileProtocol};
use uefi_raw::{Boolean, Status};

unsafe extern "efiapi" fn raw_load_file(
    this: *mut LoadFile2Protocol,
    _file_path: *const DevicePathProtocol,
    _boot_policy: Boolean,
    buffer_size: *mut usize,
    buffer: *mut c_void,
) -> Status {
    log::debug!("Called static extern \"efiapi\" `raw_load_file` glue function");
    let this = this.cast::<CustomLoadFile2Protocol>().as_ref().unwrap();
    this.load_file(buffer_size, buffer.cast())
}

#[repr(C)]
struct CustomLoadFile2Protocol {
    inner: LoadFile2Protocol,
    file_data: Vec<u8>,
}

impl CustomLoadFile2Protocol {
    fn new(file_data: Vec<u8>) -> Pin<Box<Self>> {
        let inner = Self {
            inner: LoadFile2Protocol {
                load_file: raw_load_file,
            },
            file_data,
        };
        Box::pin(inner)
    }

    fn load_file(&self, buf_len: *mut usize, buf: *mut c_void) -> Status {
        if buf.is_null() || unsafe { *buf_len } < self.file_data.len() {
            log::debug!("Returning buffer size");
            unsafe { *buf_len = self.file_data.len() };
            Status::BUFFER_TOO_SMALL
        } else {
            log::debug!("Writing file content to buffer");
            unsafe {
                ptr::copy_nonoverlapping(self.file_data.as_ptr(), buf.cast(), self.file_data.len());
            }
            Status::SUCCESS
        }
    }
}

unsafe fn install_protocol(handle: Handle, guid: Guid, protocol: &mut CustomLoadFile2Protocol) {
    boot::install_protocol_interface(Some(handle), &guid, addr_of!(*protocol).cast()).unwrap();
}

unsafe fn uninstall_protocol(handle: Handle, guid: Guid, protocol: &mut CustomLoadFile2Protocol) {
    boot::uninstall_protocol_interface(handle, &guid, addr_of!(*protocol).cast()).unwrap();
}

/// This tests the LoadFile and LoadFile2 protocols. As this protocol is not
/// implemented in OVMF for the default handle, we implement it manually using
/// `install_protocol_interface`. Then, we load a file from our custom installed
/// protocol leveraging our protocol abstraction.
///
/// The way we are implementing the LoadFile(2) protocol is roughly what certain
/// Linux loaders do so that Linux can find its initrd [0, 1].
///
/// [0] https://github.com/u-boot/u-boot/commit/ec80b4735a593961fe701cc3a5d717d4739b0fd0#diff-1f940face4d1cf74f9d2324952759404d01ee0a81612b68afdcba6b49803bdbbR171
/// [1] https://github.com/torvalds/linux/blob/ee9a43b7cfe2d8a3520335fea7d8ce71b8cabd9d/drivers/firmware/efi/libstub/efi-stub-helper.c#L550
pub fn test() {
    let image = boot::image_handle();

    let load_data_msg = "Example file content.";
    let load_data = load_data_msg.to_string().into_bytes();
    let mut proto_load_file = CustomLoadFile2Protocol::new(load_data);
    // Get the ptr to the inner value, not the wrapping smart pointer type.
    let proto_load_file_ptr = proto_load_file.as_mut().get_mut();

    // Install our custom protocol implementation as LoadFile and LoadFile2
    // protocol.
    unsafe {
        install_protocol(image, LoadFileProtocol::GUID, proto_load_file_ptr);
        install_protocol(image, LoadFile2Protocol::GUID, proto_load_file_ptr);
    }

    let mut dvp_vec = Vec::new();
    let dummy_dvp = DevicePathBuilder::with_vec(&mut dvp_vec);
    let dummy_dvp = dummy_dvp.finalize().unwrap();

    let mut load_file_protocol = boot::open_protocol_exclusive::<LoadFile>(image).unwrap();
    let loadfile_file = load_file_protocol
        .load_file(dummy_dvp, BootPolicy::BootSelection)
        .unwrap();
    let loadfile_file_string = String::from_utf8(loadfile_file.to_vec()).unwrap();

    let mut load_file2_protocol = boot::open_protocol_exclusive::<LoadFile2>(image).unwrap();
    let loadfile2_file = load_file2_protocol.load_file(dummy_dvp).unwrap();
    let loadfile2_file_string = String::from_utf8(loadfile2_file.to_vec()).unwrap();

    assert_eq!(load_data_msg, &loadfile_file_string);
    assert_eq!(load_data_msg, &loadfile2_file_string);

    // Cleanup: Uninstall protocols again.
    drop(load_file_protocol);
    drop(load_file2_protocol);
    unsafe {
        uninstall_protocol(image, LoadFileProtocol::GUID, proto_load_file_ptr);
        uninstall_protocol(image, LoadFile2Protocol::GUID, proto_load_file_ptr);
    }
    // Ensure protocols have been uninstalled:
    assert_eq!(
        boot::open_protocol_exclusive::<LoadFile>(image)
            .map(|_| ()) // make Result Eq'able
            .map_err(|e| e.status()),
        Err(Status::UNSUPPORTED)
    );
    assert_eq!(
        boot::open_protocol_exclusive::<LoadFile2>(image)
            .map(|_| ()) // make Result Eq'able
            .map_err(|e| e.status()),
        Err(Status::UNSUPPORTED)
    );
}
