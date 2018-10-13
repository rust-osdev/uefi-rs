use uefi::prelude::*;
use uefi::proto::console::text::{Color, Output};

pub fn test(stdout: &mut Output) {
    info!("Running text output protocol test");

    change_text_mode(stdout);
    change_color(stdout);
    center_text(stdout);

    // Print all modes.
    for (index, mode) in stdout.modes().enumerate() {
        let mode = mode.expect("Warnings encountered while querying text mode");
        info!(
            "- Text mode #{}: {} rows by {} columns",
            index,
            mode.rows(),
            mode.columns()
        );
    }

    // Should clean up after us.
    stdout.reset(false).warn_unwrap();
}

// Switch to the maximum supported text mode.
fn change_text_mode(stdout: &mut Output) {
    let best_mode = stdout
        .modes()
        .last()
        .unwrap()
        .expect("Warnings encountered while querying text mode");
    stdout
        .set_mode(best_mode)
        .warn_expect("Failed to change text mode");
}

// Set a new color, and paint the background with it.
fn change_color(stdout: &mut Output) {
    stdout
        .set_color(Color::White, Color::Blue)
        .warn_expect("Failed to change console color");
    stdout.clear().warn_expect("Failed to clear screen");
}

// Print a text centered on screen.
fn center_text(stdout: &mut Output) {
    // Move the cursor.
    // This will make this `info!` line below be (somewhat) centered.
    stdout
        .enable_cursor(true)
        .warn_err()
        .unwrap_or_else(|s| match s {
            Status::UNSUPPORTED => info!("Cursor visibility control unavailable"),
            _ => panic!("Failed to show cursor"),
        });
    stdout
        .set_cursor_position(24, 0)
        .warn_expect("Failed to move cursor");
    info!("# uefi-rs test runner");
    stdout
        .enable_cursor(false)
        .warn_err()
        .unwrap_or_else(|s| match s {
            Status::UNSUPPORTED => info!("Cursor visibility control unavailable"),
            _ => panic!("Failed to hide cursor"),
        });
}
