use crate::allocator::Allocator;

#[global_allocator]
static ALLOCATOR: Allocator = Allocator;
