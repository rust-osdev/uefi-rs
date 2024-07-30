//! Rusty wrapper for the [Unified Extensible Firmware Interface][UEFI].
//!
//! # TL;DR
//!
//! Develop Rust software that leverages **safe**, **convenient**, and
//! **performant** abstractions for [UEFI] functionality.
//!
//! See the [Rust UEFI Book] for a tutorial, how-tos, and overviews of some
//! important UEFI concepts. For more details of UEFI, see the latest [UEFI
//! Specification][spec].
//!
//! # About this Document
//!
//! In this document, you find general information about this crate, such
//! as examples and the technical documentation of the public API, as well
//! as general information about the project, such as license or MSRV
//! requirements.
//!
//! # About `uefi`
//!
//! With `uefi`, you have the flexibility to integrate selected types and
//! abstractions into your project or to conveniently create EFI images,
//! addressing the entire spectrum of your development needs.
//!
//! `uefi` works with stable Rust, but additional nightly-only features are
//! gated behind an `unstable` Cargo feature flag. Please find more information
//! about supported features below.
//!
//! _Note that for producing EFI images, you also need to use a corresponding
//! `uefi` compiler target of Rust, such as `x86_64-unknown-uefi`._
//!
//! ## Example Use Cases
//!
//! With this crate you can write code for the pre- and post-exit boot services
//! epochs. However, the `uefi` crate unfolds its true potential when crafting
//! EFI images interacting with UEFI boot services and eventually exiting them.
//!
//! By EFI images (`image.efi`), we are referring to EFI applications, EFI
//! boot service drivers, and EFI runtime service drivers. An EFI application
//! might be your OS-specific loader technically similar to _GRUB_ or _Limine_.
//!
//! You can also use this crate to parse the UEFI memory map when a bootloader,
//! such as _GRUB_ or _Limine_, passed the UEFI memory map as part of the
//! corresponding boot information to your kernel, or to access runtime services
//! from your kernel. Hence, when you also use utilize `uefi` also in ELF
//! binaries and are not limited to EFI images.
//!
//! ## Supported Architectures
//!
//! `uefi` is compatible with all platforms that both the Rust compiler and
//! UEFI support, such as `i686`, `x86_64`, and `aarch64`. Please note that we
//! can't test all possible hardware/firmware/platform combinations in CI.
//!
//! # Crate organisation
//!
//! The top-level module contains some of the most used types and macros,
//! including the [`Handle`] and [`Result`] types, the [`CStr16`] and
//! [`CString16`] types for working with UCS-2 strings, and the [`entry`] and
//! [`guid`] macros.
//!
//! ## Tables
//!
//! The [`SystemTable`] provides access to almost everything in UEFI. It comes
//! in two flavors:
//! - `SystemTable<Boot>`: for boot-time applications such as bootloaders,
//!   provides access to both boot and runtime services.
//! - `SystemTable<Runtime>`: for operating systems after boot services have
//!   been exited.
//!
//! ## Protocols
//!
//! When boot services are active, most functionality is provided via UEFI
//! protocols. Protocols provide operations such as reading and writing files,
//! drawing to the screen, sending and receiving network requests, and much
//! more. The list of protocols that are actually available when running an
//! application depends on the device. For example, a PC with no network card
//! may not provide network protocols.
//!
//! See the [`BootServices`] documentation for details of how to open a
//! protocol, and see the [`proto`] module for protocol implementations. New
//! protocols can be defined with the [`unsafe_protocol`] macro.
//!
//! ## Optional Cargo crate features
//!
//! - `alloc`: Enable functionality requiring the [`alloc`] crate from
//!   the Rust standard library. For example, methods that return a
//!   `Vec` rather than filling a statically-sized array. This requires
//!   a global allocator; you can use the `global_allocator` feature or
//!   provide your own. This is independent of internal direct usages of the
//!   UEFI boot service allocator which may happen anyway, where necessary.
//! - `global_allocator`: Set [`allocator::Allocator`] as the global Rust
//!   allocator. This is a simple allocator that relies on the UEFI pool
//!   allocator. You can choose to provide your own allocator instead of
//!   using this feature, or no allocator at all if you don't need to
//!   dynamically allocate any memory.
//! - `logger`: Logging implementation for the standard [`log`] crate
//!   that prints output to the UEFI console. No buffering is done; this
//!   is not a high-performance logger.
//! - `panic_handler`: Add a default panic handler that logs to `stdout`.
//! - `unstable`: Enable functionality that depends on [unstable
//!   features] in the nightly compiler.
//!   As example, in conjunction with the `alloc`-feature, this gate allows
//!   the `allocator_api` on certain functions.
//! - `qemu`: Enable some code paths to adapt their execution when executed
//!   in QEMU, such as using the special `qemu-exit` device when the panic
//!   handler is called.
//!
//! Some of these features, such as the `logger` or `panic_handler` features,
//! only unfold their potential when you invoke `uefi::helpers::init` as soon
//! as possible in your application.
//!
//! # Discuss and Contribute
//!
//! For general discussions, feel free to join us in our [Zulip] and ask
//! your questions there.
//!
//! Further, you can submit bugs and also ask questions in our [issue tracker].
//! Contributions in form of a PR are also highly welcome. Check our
//! [contributing guide][contributing] for details.
//!
//! # Comparison to other Projects in the Ecosystem
//!
//! ## Rust `std` implementation
//!
//! There is an ongoing effort for a `std` implementation of the Rust standard
//! library. ([Platform description][rustc-uefi-std],
//! [tracking issue][uefi-std-tr-issue]), that is yet far from being mature. As
//! of Mid-2024, we recommend to use our `uefi` library in a `no_std` binary to
//! create EFI images.
//!
//! We will closely monitor the situation and update the documentation
//! accordingly, once the `std` implementation is more mature.
//!
//! ## `r-efi`
//!
//! Raw UEFI bindings without high-level convenience similar to our `uefi-raw`
//! crate, which is part of this  project, but more feature-complete. It
//! targets a lower-level than our `uefi` crate does.
//!
//! # MSRV
//! <!-- Keep in Sync with README! -->
//!
//! The minimum supported Rust version is currently 1.70.
//! Our policy is to support at least the past two stable releases.
//!
//! # License
//! <!-- Keep in Sync with README! -->
//!
//! The code in this repository is licensed under the Mozilla Public License 2.
//! This license allows you to use the crate in proprietary programs, but any
//! modifications to the files must be open-sourced.
//!
//! The full text of the license is available in the [license file][LICENSE].
//!
//! # Trivia and Background
//!
//! [UEFI] started as the successor firmware to the BIOS in x86 space and
//! developed to a universal firmware specification for various platforms, such
//! as ARM. It provides an early boot environment with a variety of
//! [specified][spec] ready-to-use "high-level" functionality, such as accessing
//! disks or the network. EFI images, the files that can be loaded by an UEFI
//! environment, can leverage these abstractions to extend the functionality in
//! form of additional drivers, OS-specific bootloaders, or any different kind
//! of low-level applications (such as an [IRC client][uefirc]).
//!
//! [LICENSE]: https://github.com/rust-osdev/uefi-rs/blob/main/uefi/LICENSE
//! [Rust UEFI Book]: https://rust-osdev.github.io/uefi-rs/HEAD/
//! [UEFI]: https://uefi.org/
//! [Zulip]: https://rust-osdev.zulipchat.com
//! [`BootServices`]: table::boot::BootServices
//! [`GlobalAlloc`]: alloc::alloc::GlobalAlloc
//! [`SystemTable`]: table::SystemTable
//! [`unsafe_protocol`]: proto::unsafe_protocol
//! [contributing]: https://github.com/rust-osdev/uefi-rs/blob/main/CONTRIBUTING.md
//! [issue tracker]: https://github.com/rust-osdev/uefi-rs/issues
//! [spec]: https://uefi.org/specifications
//! [unstable features]: https://doc.rust-lang.org/unstable-book/
//! [uefirc]: https://github.com/codyd51/uefirc
//! [rustc-uefi-std]: https://doc.rust-lang.org/nightly/rustc/platform-support/unknown-uefi.html
//! [uefi-std-tr-issue]: https://github.com/rust-lang/rust/issues/100499

#![cfg_attr(all(feature = "unstable", feature = "alloc"), feature(allocator_api))]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![no_std]
// Enable some additional warnings and lints.
#![warn(clippy::ptr_as_ptr, missing_docs, unused)]
#![deny(clippy::all)]
#![deny(clippy::must_use_candidate)]
#![deny(missing_debug_implementations)]

#[cfg(feature = "alloc")]
extern crate alloc;
// allow referring to self as ::uefi for macros to work universally (from this crate and from others)
// see https://github.com/rust-lang/rust/issues/54647
extern crate self as uefi;
#[macro_use]
extern crate uefi_raw;

#[macro_use]
pub mod data_types;
pub mod allocator;
pub mod boot;
#[cfg(feature = "alloc")]
pub mod fs;
pub mod helpers;
pub mod mem;
pub mod prelude;
pub mod proto;
pub mod runtime;
pub mod system;
pub mod table;

pub(crate) mod polyfill;

mod macros;
mod result;
mod util;

#[cfg(feature = "alloc")]
pub use data_types::CString16;
pub use data_types::{CStr16, CStr8, Char16, Char8, Event, Guid, Handle, Identify};
pub use result::{Error, Result, ResultExt, Status, StatusExt};
/// Re-export ucs2_cstr so that it can be used in the implementation of the
/// cstr16 macro. It is hidden since it's not intended to be used directly.
#[doc(hidden)]
pub use ucs2::ucs2_cstr;
pub use uefi_macros::entry;
pub use uguid::guid;
