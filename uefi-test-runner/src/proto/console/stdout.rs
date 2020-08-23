use uefi::prelude::*;
use uefi::proto::console::text::{Color, Output};

pub fn test(stdout: &mut Output) {
    info!("Running text output protocol test");

    get_current_mode(stdout);
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
    stdout.reset(false).unwrap_success();
}

// Retrieves and prints the current output mode.
fn get_current_mode(stdout: &mut Output) {
    let current_mode = stdout.current_mode().unwrap_success();
    info!("UEFI standard output current mode: {:?}", current_mode);
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
        .expect_success("Failed to change text mode");
}

// Set a new color, and paint the background with it.
fn change_color(stdout: &mut Output) {
    stdout
        .set_color(Color::White, Color::Blue)
        .expect_success("Failed to change console color");
    stdout.clear().expect_success("Failed to clear screen");
}

// Print a text centered on screen.
fn center_text(stdout: &mut Output) {
    // Move the cursor.
    // This will make this `info!` line below be (somewhat) centered.
    stdout
        .enable_cursor(true)
        .warning_as_error()
        .unwrap_or_else(|e| match e.status() {
            Status::UNSUPPORTED => info!("Cursor visibility control unavailable"),
            _ => panic!("Failed to show cursor"),
        });
    stdout
        .set_cursor_position(24, 0)
        .expect_success("Failed to move cursor");
    info!("# uefi-rs test runner");
    stdout
        .enable_cursor(false)
        .warning_as_error()
        .unwrap_or_else(|e| match e.status() {
            Status::UNSUPPORTED => info!("Cursor visibility control unavailable"),
            _ => panic!("Failed to hide cursor"),
        });
}
