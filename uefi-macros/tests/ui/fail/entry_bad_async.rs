#![no_main]

use uefi::prelude::*;

#[entry]
async fn main() -> Status {
    Status::SUCCESS
}
