//! Media access protocols.
//!
//! These protocols can be used to enumerate and access various media devices.
//! They provide both **high-level abstractions** such as **files and partitions**,
//! and **low-level access** such as an **block I/O** or **raw ATA** access protocol.

mod file;
pub use self::file::File;

mod file_system;
pub use self::file_system::SimpleFileSystem;
