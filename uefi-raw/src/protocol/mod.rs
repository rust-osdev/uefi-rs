// SPDX-License-Identifier: MIT OR Apache-2.0

//! Protocol definitions.
//!
//! # TL;DR
//! Technically, a protocol is a `C` struct holding functions and/or data, with
//! an associated [`GUID`].
//!
//! # About
//! UEFI protocols are a structured collection of functions and/or data,
//! identified by a [`GUID`], which defines an interface between components in
//! the UEFI environment, such as between drivers, applications, or firmware
//! services.
//!
//! Protocols are central to UEFI’s handle-based object model, and they provide
//! a clean, extensible way for components to discover and use services from one
//! another.
//!
//! Implementation-wise, a protocol is a `C` struct holding function pointers
//! and/or data. Please note that some protocols may use [`core::ptr::null`] as
//! interface. For example, the device path protocol can be implemented but
//! return `null`.
//!
//! [`GUID`]: crate::Guid

pub mod acpi;
pub mod ata;
pub mod block;
pub mod console;
pub mod device_path;
pub mod disk;
pub mod driver;
pub mod file_system;
pub mod firmware_volume;
pub mod hii;
pub mod iommu;
pub mod loaded_image;
pub mod media;
pub mod memory_protection;
pub mod misc;
pub mod network;
pub mod nvme;
pub mod pci;
pub mod rng;
pub mod scsi;
pub mod shell;
pub mod shell_params;
pub mod string;
pub mod tcg;
pub mod usb;
