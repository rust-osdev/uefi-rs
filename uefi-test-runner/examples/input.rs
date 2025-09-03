// SPDX-License-Identifier: MIT OR Apache-2.0

#![no_main]
#![no_std]

use uefi::proto::console::text::{Input, Key, ScanCode};
use uefi::{Result, ResultExt, Status, boot, entry, println, system};

fn read_keyboard_events(input: &mut Input) -> Result {
    loop {
        println!("waiting for key press...");

        // Pause until a keyboard event occurs.
        let mut events = [input.wait_for_key_event().unwrap()];
        boot::wait_for_event(&mut events).discard_errdata()?;

        match input.read_key()? {
            Some(Key::Printable(key)) => {
                println!("key '{key}' was pressed");
            }

            // Exit the loop when the escape key is pressed.
            Some(Key::Special(ScanCode::ESCAPE)) => {
                break;
            }
            _ => {}
        }
    }

    Ok(())
}

#[entry]
fn main() -> Status {
    system::with_stdin(|input| read_keyboard_events(input).status())
}
