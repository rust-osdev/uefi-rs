// SPDX-License-Identifier: MIT OR Apache-2.0

#![no_main]
#![no_std]

use uefi::proto::console::text::{InputEx, Key, ScanCode};
use uefi::{Result, ResultExt, Status, boot, entry, println};

fn read_keyboard_events(input: &mut InputEx) -> Result {
    loop {
        println!("waiting for key press...");

        // Pause until a keyboard event occurs.
        let event = input.wait_for_key_event()?;
        let mut events = [event];
        boot::wait_for_event(&mut events).discard_errdata()?;

        let Some(key_data) = input.read_key()? else {
            continue;
        };

        match key_data.key {
            Key::Printable(key) => {
                println!(
                    "key '{key}' was pressed with {:?}",
                    key_data.key_state
                );
            }

            // Exit the loop when the escape key is pressed.
            Key::Special(ScanCode::ESCAPE) => {
                break;
            }
            _ => {}
        }
    }

    Ok(())
}

#[entry]
fn main() -> Status {
    if let Ok(handle) = boot::get_handle_for_protocol::<InputEx>() {
        let mut input_ex = boot::open_protocol_exclusive::<InputEx>(handle)
            .expect("failed to open input ex");

        return read_keyboard_events(&mut input_ex).status();
    }

    Status::SUCCESS
}
