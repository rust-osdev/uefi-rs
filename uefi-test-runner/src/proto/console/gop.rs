use core::ptr;
use uefi::prelude::*;
use uefi::proto::console::gop::{BltOp, BltPixel, GraphicsOutput, PixelFormat};
use uefi::table::boot::BootServices;
use uefi_exts::BootServicesExt;

pub fn test(bt: &BootServices) {
    info!("Running graphics output protocol test");
    if let Some(mut gop_proto) = bt.find_protocol::<GraphicsOutput>() {
        let gop = unsafe { gop_proto.as_mut() };

        set_graphics_mode(gop);
        fill_color(gop);
        draw_fb(gop);

        crate::check_screenshot(bt, "gop_test");
    } else {
        // No tests can be run.
        warn!("UEFI Graphics Output Protocol is not supported");
    }
}

// Set a larger graphics mode.
fn set_graphics_mode(gop: &mut GraphicsOutput) {
    // We know for sure QEMU has a 1024x768, mode.
    let mode = gop
        .modes()
        .map(|mode| mode.expect("Warnings encountered while querying mode"))
        .find(|ref mode| {
            let info = mode.info();
            info.resolution() == (1024, 768)
        })
        .unwrap();

    gop.set_mode(&mode)
        .warn_expect("Failed to set graphics mode");
}

// Fill the screen with color.
fn fill_color(gop: &mut GraphicsOutput) {
    let op = BltOp::VideoFill {
        // Cornflower blue.
        color: BltPixel::new(100, 149, 237),
        dest: (0, 0),
        dims: (1024, 768),
    };

    gop.blt(op).warn_expect("Failed to fill screen with color");
}

// Draw directly to the frame buffer.
fn draw_fb(gop: &mut GraphicsOutput) {
    let mi = gop.current_mode_info();
    let stride = mi.stride();
    let (width, height) = mi.resolution();

    let fb = unsafe { gop.frame_buffer() };

    type PixelWriter<'a> = &'a Fn(&mut [u8], (u8, u8, u8));
    let write_pixel_rgb = |pixel: &mut [u8], (r, g, b)| {
        let p = pixel.as_mut_ptr();
        unsafe {
            ptr::write_volatile(p.offset(0), r);
            ptr::write_volatile(p.offset(1), g);
            ptr::write_volatile(p.offset(2), b);
        }
    };
    let write_pixel_bgr = |pixel: &mut [u8], (r, g, b)| {
        let p = pixel.as_mut_ptr();
        unsafe {
            ptr::write_volatile(p.offset(0), b);
            ptr::write_volatile(p.offset(1), g);
            ptr::write_volatile(p.offset(2), r);
        }
    };
    let write_pixel: PixelWriter = match mi.pixel_format() {
        PixelFormat::RGB => &write_pixel_rgb,
        PixelFormat::BGR => &write_pixel_bgr,
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
                let index = (row * stride) + column;
                let pixel = &mut fb[4 * index..4 * index + 3];
                write_pixel(pixel, color);
            }
        }
    };

    fill_rectangle((50, 30), (150, 600), (250, 128, 64));
    fill_rectangle((400, 120), (750, 450), (16, 128, 255));
}
