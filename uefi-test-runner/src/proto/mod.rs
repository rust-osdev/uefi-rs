use uefi::boot::{self, OpenProtocolParams};
use uefi::prelude::*;
use uefi::proto::loaded_image::LoadedImage;
use uefi::{proto, Identify};

pub fn test(st: &mut SystemTable<Boot>) {
    info!("Testing various protocols");

    console::test();

    let bt = st.boot_services();
    find_protocol();
    test_protocols_per_handle();
    test_test_protocol();

    debug::test();
    device_path::test();
    driver::test();
    load::test();
    loaded_image::test();
    media::test();
    network::test(bt);
    pi::test(bt);
    rng::test();
    shell_params::test(bt);
    string::test();
    misc::test(bt);

    #[cfg(any(
        target_arch = "x86",
        target_arch = "x86_64",
        target_arch = "arm",
        target_arch = "aarch64"
    ))]
    shim::test();
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
    assert!(boot::test_protocol::<LoadedImage>(OpenProtocolParams {
        handle: boot::image_handle(),
        agent: boot::image_handle(),
        controller: None,
    })
    .unwrap());
}

mod console;
mod debug;
mod device_path;
mod driver;
mod load;
mod loaded_image;
mod media;
mod misc;
mod network;
mod pi;
mod rng;
mod shell_params;
#[cfg(any(
    target_arch = "x86",
    target_arch = "x86_64",
    target_arch = "arm",
    target_arch = "aarch64"
))]
mod shim;
mod string;
mod tcg;
