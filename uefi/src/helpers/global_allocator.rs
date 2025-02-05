// SPDX-License-Identifier: MIT OR Apache-2.0

use crate::allocator::Allocator;

#[global_allocator]
static ALLOCATOR: Allocator = Allocator;
