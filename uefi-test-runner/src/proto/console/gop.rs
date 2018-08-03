use uefi::table::boot::BootServices;
use uefi::proto::console::gop::{GraphicsOutput, BltOp, BltPixel};
use uefi_exts::BootServicesExt;

pub fn test(bt: &BootServices) {
    if let Some(mut gop_proto) = bt.find_protocol::<GraphicsOutput>() {
        let gop = unsafe { gop_proto.as_mut() };

        set_graphics_mode(gop);
        fill_color(gop);
        draw_fb(gop);

        // TODO: For now, allow the user to inspect the visual output.
        bt.stall(1_000_000);
    } else {
        // No tests can be run.
        warn!("UEFI Graphics Output Protocol is not supported");
    }
}

// Set a larger graphics mode.
fn set_graphics_mode(gop: &mut GraphicsOutput) {
    // We know for sure QEMU has a 1024x768, mode.
    let mode = gop.modes()
        .find(|ref mode| {
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
    let mi = gop.current_mode_info();
    let stride = mi.stride();
    // BUG: we should check we have enough space to draw.
    // let (width, height) = mi.resolution();

    let fb = unsafe { gop.frame_buffer() };

    let mut set_pixel = |(row, column), (r, g, b)| {
        let index = (row * stride) + column;

        // BUG: we assume the pixel format is 32-bit BGR, as it often is on x86.
        // For RGB the red / blue channels will be inverted.
        let bi = 4 * index;
        let gi = 4 * index + 1;
        let ri = 4 * index + 2;

        fb[bi] = b;
        fb[gi] = g;
        fb[ri] = r;
    };

    let mut fill_rectangle = |(x1, y1), (x2, y2), color| {
        for row in y1..y2 {
            for column in x1..x2 {
                set_pixel((row, column), color);
            }
        }
    };

    fill_rectangle((50, 30), (150, 600), (250, 128, 64));
    fill_rectangle((400, 120), (750, 450), (16, 128, 255));
}
