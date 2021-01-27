/// Trait for querying the alignment of a struct
///
/// Needed for dynamic-sized types because `mem::align_of` has a `Sized` bound (due to `dyn Trait`)
pub trait Align {
    /// Required memory alignment for this type
    fn alignment() -> usize;

    /// Assert that some storage is correctly aligned for this type
    fn assert_aligned(storage: &mut [u8]) {
        if !storage.is_empty() {
            assert_eq!(
                (storage.as_ptr() as usize) % Self::alignment(),
                0,
                "The provided storage is not correctly aligned for this type"
            )
        }
    }
}
