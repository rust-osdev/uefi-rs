// SPDX-License-Identifier: MIT OR Apache-2.0

use crate::{send_request_to_host, HostRequest};
use uefi::boot::{self, OpenProtocolAttributes, OpenProtocolParams};
use uefi::proto::console::gop::{BltOp, BltPixel, FrameBuffer, GraphicsOutput, PixelFormat};

pub unsafe fn test() {
    info!("Running graphics output protocol test");
    let handle =
        boot::get_handle_for_protocol::<GraphicsOutput>().expect("missing GraphicsOutput protocol");
    let gop = &mut boot::open_protocol::<GraphicsOutput>(
        OpenProtocolParams {
            handle,
            agent: boot::image_handle(),
            controller: None,
        },
        // For this test, don't open in exclusive mode. That
        // would break the connection between stdout and the
        // video console.
        OpenProtocolAttributes::GetProtocol,
    )
    .expect("failed to open Graphics Output Protocol");

    set_graphics_mode(gop);
    fill_color(gop);
    draw_fb(gop);

    // `draw_fb` is skipped on aarch64, so the screenshot doesn't match.
    if cfg!(not(target_arch = "aarch64")) {
        send_request_to_host(HostRequest::Screenshot("gop_test"));
    }
}

// Set a larger graphics mode.
fn set_graphics_mode(gop: &mut GraphicsOutput) {
    // We know for sure QEMU has a 1024x768 mode.
    let mode = gop
        .modes()
        .find(|mode| {
            let info = mode.info();
            info.resolution() == (1024, 768)
        })
        .unwrap();

    gop.set_mode(&mode).expect("Failed to set graphics mode");
}

// Fill the screen with color.
fn fill_color(gop: &mut GraphicsOutput) {
    let op = BltOp::VideoFill {
        // Cornflower blue.
        color: BltPixel::new(100, 149, 237),
        dest: (0, 0),
        dims: (1024, 768),
    };

    gop.blt(op).expect("Failed to fill screen with color");
}

// Draw directly to the frame buffer.
fn draw_fb(gop: &mut GraphicsOutput) {
    // The `virtio-gpu-pci` graphics device we use on aarch64 doesn't
    // support `PixelFormat::BltOnly`.
    if cfg!(target_arch = "aarch64") {
        return;
    }

    let mi = gop.current_mode_info();
    let stride = mi.stride();
    let (width, height) = mi.resolution();

    let mut fb = gop.frame_buffer();

    type PixelWriter = unsafe fn(&mut FrameBuffer, usize, [u8; 3]);
    unsafe fn write_pixel_rgb(fb: &mut FrameBuffer, pixel_base: usize, rgb: [u8; 3]) {
        fb.write_value(pixel_base, rgb);
    }
    unsafe fn write_pixel_bgr(fb: &mut FrameBuffer, pixel_base: usize, rgb: [u8; 3]) {
        fb.write_value(pixel_base, [rgb[2], rgb[1], rgb[0]]);
    }
    let write_pixel: PixelWriter = match mi.pixel_format() {
        PixelFormat::Rgb => write_pixel_rgb,
        PixelFormat::Bgr => write_pixel_bgr,
        _ => {
            info!("This pixel format is not supported by the drawing demo");
            return;
        }
    };

    let mut fill_rectangle = |(x1, y1), (x2, y2), color| {
        assert!((x1 < width) && (x2 < width), "Bad X coordinate");
        assert!((y1 < height) && (y2 < height), "Bad Y coordinate");
        for row in y1..y2 {
            for column in x1..x2 {
                unsafe {
                    let pixel_index = (row * stride) + column;
                    let pixel_base = 4 * pixel_index;
                    write_pixel(&mut fb, pixel_base, color);
                }
            }
        }
    };

    fill_rectangle((50, 30), (150, 600), [250, 128, 64]);
    fill_rectangle((400, 120), (750, 450), [16, 128, 255]);
}
