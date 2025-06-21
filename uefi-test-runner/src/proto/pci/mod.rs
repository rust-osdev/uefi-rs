// SPDX-License-Identifier: MIT OR Apache-2.0

pub mod root_bridge;

pub fn test() {
    root_bridge::test_io();
    root_bridge::test_buffer();
    root_bridge::test_mapping();
    root_bridge::test_copy();
}
