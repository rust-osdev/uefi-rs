// SPDX-License-Identifier: MIT OR Apache-2.0

pub fn test() {
    info!("Testing String protocols");

    unicode_collation::test();
}

mod unicode_collation;
