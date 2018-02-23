/// A pointer to an opaque data structure.
pub type Handle = *mut ();

mod guid;
pub use self::guid::Guid;
