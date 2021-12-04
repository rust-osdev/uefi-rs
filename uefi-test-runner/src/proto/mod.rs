use uefi::prelude::*;

use uefi::proto::loaded_image::LoadedImage;
use uefi::{proto, Identify};

pub fn test(image: Handle, st: &mut SystemTable<Boot>) {
    info!("Testing various protocols");

    console::test(image, st);

    let bt = st.boot_services();
    find_protocol(bt);
    test_protocols_per_handle(image, bt);

    debug::test(bt);
    device_path::test(image, bt);
    media::test(bt);
    pi::test(bt);

    #[cfg(any(
        target_arch = "i386",
        target_arch = "x86_64",
        target_arch = "arm",
        target_arch = "aarch64"
    ))]
    shim::test(bt);
}

fn find_protocol(bt: &BootServices) {
    type SearchedProtocol<'boot> = proto::console::text::Output<'boot>;

    let handles = bt
        .find_handles::<SearchedProtocol>()
        .expect_success("Failed to retrieve list of handles");

    assert!(
        handles.len() > 1,
        "There should be at least one implementation of Simple Text Output (stdout)"
    );
}

fn test_protocols_per_handle(image: Handle, bt: &BootServices) {
    let pph = bt
        .protocols_per_handle(image)
        .expect_success("Failed to get protocols for image handle");

    info!("Image handle has {} protocols", pph.protocols().len());

    // Check that one of the image's protocols is `LoadedImage`.
    assert!(pph
        .protocols()
        .iter()
        .any(|guid| **guid == LoadedImage::GUID));
}

mod console;
mod debug;
mod device_path;
mod media;
mod pi;
#[cfg(any(
    target_arch = "i386",
    target_arch = "x86_64",
    target_arch = "arm",
    target_arch = "aarch64"
))]
mod shim;
