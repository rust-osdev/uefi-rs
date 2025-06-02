// SPDX-License-Identifier: MIT OR Apache-2.0

use uefi::boot::{self, OpenProtocolParams};
use uefi::proto::loaded_image::LoadedImage;
use uefi::{Identify, proto};

pub fn test() {
    info!("Testing various protocols");

    console::test();

    find_protocol();
    test_protocols_per_handle();
    test_test_protocol();

    debug::test();
    device_path::test();
    driver::test();
    load::test();
    loaded_image::test();
    media::test();
    network::test();
    pci::test();
    pi::test();
    rng::test();
    shell_params::test();
    string::test();
    usb::test();
    misc::test();

    // disable the ATA test on aarch64 for now. The aarch64 UEFI Firmware does not yet seem
    // to support SATA controllers (and providing an AtaPassThru protocol instance for them).
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    ata::test();
    scsi::test();
    nvme::test();

    #[cfg(any(
        target_arch = "x86",
        target_arch = "x86_64",
        target_arch = "arm",
        target_arch = "aarch64"
    ))]
    shim::test();
    shell::test();
    tcg::test();
}

fn find_protocol() {
    let handles = boot::find_handles::<proto::console::text::Output>()
        .expect("Failed to retrieve list of handles");

    assert!(
        !handles.is_empty(),
        "There should be at least one implementation of Simple Text Output (stdout)"
    );
}

fn test_protocols_per_handle() {
    let pph = boot::protocols_per_handle(boot::image_handle()).unwrap();
    info!("Image handle has {} protocols", pph.len());
    // Check that one of the image's protocols is `LoadedImage`.
    assert!(pph.iter().any(|guid| **guid == LoadedImage::GUID));
}

fn test_test_protocol() {
    assert!(
        boot::test_protocol::<LoadedImage>(OpenProtocolParams {
            handle: boot::image_handle(),
            agent: boot::image_handle(),
            controller: None,
        })
        .unwrap()
    );
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
mod ata;
mod console;
mod debug;
mod device_path;
mod driver;
mod load;
mod loaded_image;
mod media;
mod misc;
mod network;
mod nvme;
mod pci;
mod pi;
mod rng;
mod scsi;
#[cfg(any(
    target_arch = "x86",
    target_arch = "x86_64",
    target_arch = "arm",
    target_arch = "aarch64"
))]
mod shell;
mod shell_params;
mod shim;
mod string;
mod tcg;
mod usb;
