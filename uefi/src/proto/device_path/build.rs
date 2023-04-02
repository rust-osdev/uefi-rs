//! Utilities for creating new [`DevicePaths`].
//!
//! This module contains [`DevicePathBuilder`], as well as submodules
//! containing types for building each type of device path node.
//!
//! [`DevicePaths`]: DevicePath

pub use crate::proto::device_path::device_path_gen::build::*;

use crate::polyfill::{maybe_uninit_slice_as_mut_ptr, maybe_uninit_slice_assume_init_ref};
use crate::proto::device_path::{DevicePath, DevicePathNode};
use core::mem::MaybeUninit;

#[cfg(feature = "alloc")]
use alloc::vec::Vec;

/// A builder for [`DevicePaths`].
///
/// The builder can be constructed with either a fixed-length buffer or
/// (if the `alloc` feature is enabled) a `Vec`.
///
/// Nodes are added via the [`push`] method. To construct a node, use one
/// of the structs in these submodules:
/// * [`acpi`]
/// * [`bios_boot_spec`]
/// * [`end`]
/// * [`hardware`]
/// * [`media`]
/// * [`messaging`]
///
/// A node can also be constructed by copying a node from an existing
/// device path.
///
/// To complete a path, call the [`finalize`] method. This adds an
/// [`END_ENTIRE`] node and returns a [`DevicePath`] reference tied to
/// the lifetime of the buffer the builder was constructed with.
///
/// [`DevicePaths`]: DevicePath
/// [`END_ENTIRE`]: uefi::proto::device_path::DeviceSubType::END_ENTIRE
/// [`finalize`]: DevicePathBuilder::finalize
/// [`push`]: DevicePathBuilder::push
///
/// # Examples
///
/// ```
/// use core::mem::MaybeUninit;
/// use uefi::guid;
/// use uefi::proto::device_path::DevicePath;
/// use uefi::proto::device_path::build;
///
/// # fn main() -> Result<(), build::BuildError> {
/// let mut buf = [MaybeUninit::uninit(); 256];
/// let path: &DevicePath = build::DevicePathBuilder::with_buf(&mut buf)
///     .push(&build::acpi::Acpi {
///         hid: 0x41d0_0a03,
///         uid: 0x0000_0000,
///     })?
///     .push(&build::hardware::Pci {
///         function: 0x00,
///         device: 0x1f,
///     })?
///     .push(&build::hardware::Vendor {
///         vendor_guid: guid!("15e39a00-1dd2-1000-8d7f-00a0c92408fc"),
///         vendor_defined_data: &[1, 2, 3, 4, 5, 6],
///     })?
///     .finalize()?;
///
/// assert_eq!(path.node_iter().count(), 3);
/// # Ok(())
/// # }
/// ```
#[derive(Debug)]
pub struct DevicePathBuilder<'a> {
    storage: BuilderStorage<'a>,
}

impl<'a> DevicePathBuilder<'a> {
    /// Create a builder backed by a statically-sized buffer.
    pub fn with_buf(buf: &'a mut [MaybeUninit<u8>]) -> Self {
        Self {
            storage: BuilderStorage::Buf { buf, offset: 0 },
        }
    }

    /// Create a builder backed by a `Vec`.
    #[cfg(feature = "alloc")]
    pub fn with_vec(v: &'a mut Vec<u8>) -> Self {
        Self {
            storage: BuilderStorage::Vec(v),
        }
    }

    /// Add a node to the device path.
    ///
    /// An error will be returned if an [`END_ENTIRE`] node is passed to
    /// this function, as that node will be added when `finalize` is
    /// called.
    ///
    /// [`END_ENTIRE`]: uefi::proto::device_path::DeviceSubType::END_ENTIRE
    pub fn push(mut self, node: &dyn BuildNode) -> Result<Self, BuildError> {
        let node_size = usize::from(node.size_in_bytes()?);

        match &mut self.storage {
            BuilderStorage::Buf { buf, offset } => {
                node.write_data(
                    buf.get_mut(*offset..*offset + node_size)
                        .ok_or(BuildError::BufferTooSmall)?,
                );
                *offset += node_size;
            }
            #[cfg(feature = "alloc")]
            BuilderStorage::Vec(vec) => {
                let old_size = vec.len();
                vec.reserve(node_size);
                let buf = &mut vec.spare_capacity_mut()[..node_size];
                node.write_data(buf);
                unsafe {
                    vec.set_len(old_size + node_size);
                }
            }
        }

        Ok(self)
    }

    /// Add an [`END_ENTIRE`] node and return the resulting [`DevicePath`].
    ///
    /// This method consumes the builder.
    ///
    /// [`END_ENTIRE`]: uefi::proto::device_path::DeviceSubType::END_ENTIRE
    pub fn finalize(self) -> Result<&'a DevicePath, BuildError> {
        let this = self.push(&end::Entire)?;

        let data: &[u8] = match &this.storage {
            BuilderStorage::Buf { buf, offset } => unsafe {
                maybe_uninit_slice_assume_init_ref(&buf[..*offset])
            },
            #[cfg(feature = "alloc")]
            BuilderStorage::Vec(vec) => vec,
        };

        let ptr: *const () = data.as_ptr().cast();
        Ok(unsafe { &*ptr_meta::from_raw_parts(ptr, data.len()) })
    }
}

#[derive(Debug)]
enum BuilderStorage<'a> {
    Buf {
        buf: &'a mut [MaybeUninit<u8>],
        offset: usize,
    },

    #[cfg(feature = "alloc")]
    Vec(&'a mut Vec<u8>),
}

/// Error type used by [`DevicePathBuilder`].
#[derive(Clone, Copy, Debug)]
pub enum BuildError {
    /// A node was too big to fit in the remaining buffer space.
    BufferTooSmall,

    /// An individual node's length is too big to fit in a [`u16`].
    NodeTooBig,

    /// An [`END_ENTIRE`] node was passed to the builder. Use
    /// [`DevicePathBuilder::finalize`] instead.
    ///
    /// [`END_ENTIRE`]: uefi::proto::device_path::DeviceSubType::END_ENTIRE
    UnexpectedEndEntire,
}

/// Trait for types that can be used to build a node via
/// [`DevicePathBuilder::push`].
///
/// This trait is implemented for all the node types in
/// [`uefi::proto::device_path::build`]. It is also implemented for
/// [`&DevicePathNode`], which allows an existing node to be copied by
/// the builder.
///
/// # Safety
///
/// The methods of this trait are safe to call, but the trait itself is
/// `unsafe` because an incorrect implementation could cause
/// unsafety. In particular, the `write_data` is required to
/// completely initialize all bytes in the output slice.
///
/// [`&DevicePathNode`]: DevicePathNode
pub unsafe trait BuildNode {
    /// Size of the node in bytes, including the standard node
    /// header. Returns [`BuildError::NodeTooBig`] if the node's size
    /// does not fit in a [`u16`].
    fn size_in_bytes(&self) -> Result<u16, BuildError>;

    /// Write out the node data.
    ///
    /// The length of `out` must be equal to the node's `size_in_bytes`.
    ///
    /// The `out` slice will be fully initialized after the call.
    fn write_data(&self, out: &mut [MaybeUninit<u8>]);
}

unsafe impl BuildNode for &DevicePathNode {
    fn size_in_bytes(&self) -> Result<u16, BuildError> {
        Ok(self.header.length)
    }

    fn write_data(&self, out: &mut [MaybeUninit<u8>]) {
        let src: *const u8 = self.as_ffi_ptr().cast();

        let dst: *mut u8 = maybe_uninit_slice_as_mut_ptr(out);
        unsafe {
            dst.copy_from_nonoverlapping(src, out.len());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::guid;
    use crate::proto::device_path::media::{PartitionFormat, PartitionSignature};
    use crate::proto::device_path::messaging::{
        Ipv4AddressOrigin, IscsiLoginOptions, IscsiProtocol, RestServiceAccessMode, RestServiceType,
    };
    use core::{mem, slice};

    fn path_to_bytes(path: &DevicePath) -> &[u8] {
        unsafe { slice::from_raw_parts(path.as_ffi_ptr().cast::<u8>(), mem::size_of_val(path)) }
    }

    /// Test building an ACPI ADR node.
    #[test]
    fn test_acpi_adr() -> Result<(), BuildError> {
        assert!(acpi::AdrSlice::new(&[]).is_none());

        let mut v = Vec::new();
        let path = DevicePathBuilder::with_vec(&mut v)
            .push(&acpi::Adr {
                adr: acpi::AdrSlice::new(&[1, 2]).unwrap(),
            })?
            .finalize()?;

        let node: &crate::proto::device_path::acpi::Adr =
            path.node_iter().next().unwrap().try_into().unwrap();
        assert_eq!(node.adr().iter().collect::<Vec<_>>(), [1, 2]);

        let bytes = path_to_bytes(path);
        #[rustfmt::skip]
        assert_eq!(bytes, [
            // ACPI ADR node
            0x02, 0x03, 0x0c, 0x00,
            // Values
            0x01, 0x00, 0x00, 0x00,
            0x02, 0x00, 0x00, 0x00,

            // End-entire node
            0x7f, 0xff, 0x04, 0x00,
        ]);

        Ok(())
    }

    /// Test building an ACPI Expanded node.
    #[test]
    fn test_acpi_expanded() -> Result<(), BuildError> {
        let mut v = Vec::new();
        let path = DevicePathBuilder::with_vec(&mut v)
            .push(&acpi::Expanded {
                hid: 1,
                uid: 2,
                cid: 3,
                hid_str: b"a\0",
                uid_str: b"bc\0",
                cid_str: b"def\0",
            })?
            .finalize()?;

        let node: &crate::proto::device_path::acpi::Expanded =
            path.node_iter().next().unwrap().try_into().unwrap();
        assert_eq!(node.hid(), 1);
        assert_eq!(node.uid(), 2);
        assert_eq!(node.cid(), 3);
        assert_eq!(node.hid_str(), b"a\0");
        assert_eq!(node.uid_str(), b"bc\0");
        assert_eq!(node.cid_str(), b"def\0");

        let bytes = path_to_bytes(path);
        #[rustfmt::skip]
        assert_eq!(bytes, [
            // ACPI Expanded node
            0x02, 0x02, 0x19, 0x00,
            // HID
            0x01, 0x00, 0x00, 0x00,
            // UID
            0x02, 0x00, 0x00, 0x00,
            // CID
            0x03, 0x00, 0x00, 0x00,

            // HID str
            0x61, 0x00,

            // UID str
            0x62, 0x63, 0x00,

            // CID str
            0x64, 0x65, 0x66, 0x00,

            // End-entire node
            0x7f, 0xff, 0x04, 0x00,
        ]);

        Ok(())
    }

    /// Test building a messaging REST Service node.
    #[test]
    fn test_messaging_rest_service() -> Result<(), BuildError> {
        let mut v = Vec::new();
        let vendor_guid = guid!("a1005a90-6591-4596-9bab-1c4249a6d4ff");
        let path = DevicePathBuilder::with_vec(&mut v)
            .push(&messaging::RestService {
                service_type: RestServiceType::REDFISH,
                access_mode: RestServiceAccessMode::IN_BAND,
                vendor_guid_and_data: None,
            })?
            .push(&messaging::RestService {
                service_type: RestServiceType::VENDOR,
                access_mode: RestServiceAccessMode::OUT_OF_BAND,
                vendor_guid_and_data: Some(messaging::RestServiceVendorData {
                    vendor_guid,
                    vendor_defined_data: &[1, 2, 3, 4, 5],
                }),
            })?
            .finalize()?;

        let mut iter = path.node_iter();
        let mut node: &crate::proto::device_path::messaging::RestService =
            iter.next().unwrap().try_into().unwrap();
        assert!(node.vendor_guid_and_data().is_none());
        node = iter.next().unwrap().try_into().unwrap();
        assert_eq!(node.vendor_guid_and_data().unwrap().0, vendor_guid);
        assert_eq!(node.vendor_guid_and_data().unwrap().1, &[1, 2, 3, 4, 5]);

        let bytes = path_to_bytes(path);
        #[rustfmt::skip]
        assert_eq!(bytes, [
            // Messaging REST Service node.
            0x03, 0x21, 0x06, 0x00,
            // Type and access mode
            0x01, 0x01,

            // Messaging REST Service node.  The spec incorrectly says
            // the length is 21+n bytes, it's actually 22+n bytes.
            0x03, 0x21, 0x1b, 0x00,
            // Type and access mode
            0xff, 0x02,
            // Vendor guid
            0x90, 0x5a, 0x00, 0xa1,
            0x91, 0x65, 0x96, 0x45,
            0x9b, 0xab, 0x1c, 0x42,
            0x49, 0xa6, 0xd4, 0xff,
            // Vendor data
            0x01, 0x02, 0x03, 0x04, 0x05,

            // End-entire node
            0x7f, 0xff, 0x04, 0x00,
        ]);

        Ok(())
    }

    /// Test that packed nodes can be passed into the builder.
    #[test]
    fn test_build_with_packed_node() -> Result<(), BuildError> {
        // Build a path with both a statically-sized and DST nodes.
        let mut v = Vec::new();
        let path1 = DevicePathBuilder::with_vec(&mut v)
            .push(&acpi::Acpi {
                hid: 0x41d0_0a03,
                uid: 0x0000_0000,
            })?
            .push(&hardware::Vendor {
                vendor_guid: guid!("15e39a00-1dd2-1000-8d7f-00a0c92408fc"),
                vendor_defined_data: &[1, 2, 3, 4, 5, 6],
            })?
            .finalize()?;

        // Create a second path by copying in the packed nodes from the
        // first path.
        let mut v = Vec::new();
        let mut builder = DevicePathBuilder::with_vec(&mut v);
        for node in path1.node_iter() {
            builder = builder.push(&node)?;
        }
        let path2 = builder.finalize()?;

        // Verify the copied path is identical.
        assert_eq!(path1, path2);

        Ok(())
    }

    /// This test is based on the "Fibre Channel Ex Device Path Example"
    /// from the UEFI Specification.
    #[test]
    fn test_fibre_channel_ex_device_path_example() -> Result<(), BuildError> {
        // Arbitrarily choose this test to use a statically-sized
        // buffer, just to make sure that code path is tested.
        let mut buf = [MaybeUninit::uninit(); 256];
        let path = DevicePathBuilder::with_buf(&mut buf)
            .push(&acpi::Acpi {
                hid: 0x41d0_0a03,
                uid: 0x0000_0000,
            })?
            .push(&hardware::Pci {
                function: 0x00,
                device: 0x1f,
            })?
            .push(&messaging::FibreChannelEx {
                world_wide_name: [0, 1, 2, 3, 4, 5, 6, 7],
                logical_unit_number: [0, 1, 2, 3, 4, 5, 6, 7],
            })?
            .finalize()?;

        let bytes = path_to_bytes(path);
        #[rustfmt::skip]
        assert_eq!(bytes, [
            // ACPI node
            0x02, 0x01, 0x0c, 0x00,
            // HID
            0x03, 0x0a, 0xd0, 0x41,
            // UID
            0x00, 0x00, 0x00, 0x00,

            // PCI node
            0x01, 0x01, 0x06, 0x00,
            // Function
            0x00,
            // Device
            0x1f,

            // Fibre Channel Ex node
            0x03, 0x15,
            // The example in the spec is wrong here; it says 0x14 for
            // the length and leaves out the four-byte reserved field.
            0x18, 0x00,
            // Reserved
            0x00, 0x00, 0x00, 0x00,
            // World wide name
            0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07,
            // Logical unit number
            0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07,

            // End-entire node
            0x7f, 0xff, 0x04, 0x00,
        ]);

        Ok(())
    }

    /// This test is based on the "IPv4 configuration" example from the
    /// UEFI Specification.
    #[test]
    fn test_ipv4_configuration_example() -> Result<(), BuildError> {
        let mut v = Vec::new();
        let path = DevicePathBuilder::with_vec(&mut v)
            .push(&acpi::Acpi {
                hid: 0x41d0_0a03,
                uid: 0x0000_0000,
            })?
            .push(&hardware::Pci {
                function: 0x00,
                device: 0x19,
            })?
            .push(&messaging::MacAddress {
                mac_address: [
                    0x00, 0x13, 0x20, 0xf5, 0xfa, 0x77, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                    0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                ],
                interface_type: 0x01,
            })?
            .push(&messaging::Ipv4 {
                local_ip_address: [192, 168, 0, 1],
                remote_ip_address: [192, 168, 0, 100],
                local_port: 0,
                remote_port: 3260,
                protocol: 6,
                ip_address_origin: Ipv4AddressOrigin::STATIC,
                gateway_ip_address: [0, 0, 0, 0],
                subnet_mask: [0, 0, 0, 0],
            })?
            .push(&messaging::Iscsi {
                protocol: IscsiProtocol::TCP,
                options: IscsiLoginOptions::AUTH_METHOD_NONE,
                logical_unit_number: 0u64.to_le_bytes(),
                target_portal_group_tag: 1,
                iscsi_target_name: b"iqn.1991-05.com.microsoft:iscsitarget-iscsidisk-target\0",
            })?
            .push(&media::HardDrive {
                partition_number: 1,
                partition_start: 0x22,
                partition_size: 0x2710000,
                partition_format: PartitionFormat::GPT,
                partition_signature: PartitionSignature::Guid(guid!(
                    "15e39a00-1dd2-1000-8d7f-00a0c92408fc"
                )),
            })?
            .finalize()?;

        let bytes = path_to_bytes(path);
        #[rustfmt::skip]
        assert_eq!(bytes, [
            // ACPI node
            0x02, 0x01, 0x0c, 0x00,
            // HID
            0x03, 0x0a, 0xd0, 0x41,
            // UID
            0x00, 0x00, 0x00, 0x00,

            // PCI node
            0x01, 0x01, 0x06, 0x00,
            // Function
            0x00,
            // Device
            0x19,

            // MAC address node
            0x03, 0x0b, 0x25, 0x00,
            // MAC address
            0x00, 0x13, 0x20, 0xf5, 0xfa, 0x77, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            // Network interface type
            0x01,

            // IPv4 node
            0x03, 0x0c, 0x1b, 0x00,
            // Local address
            0xc0, 0xa8, 0x00, 0x01,
            // Remote address
            0xc0, 0xa8, 0x00, 0x64,
            // Local port
            0x00, 0x00,
            // Remote port
            0xbc, 0x0c,
            // Protocol
            0x06, 0x00,
            // Static IP
            0x01,
            // Gateway IP
            0x00, 0x00, 0x00, 0x00,
            // Subnet mask
            0x00, 0x00, 0x00, 0x00,

            // iSCSI node
            0x03, 0x13, 0x49, 0x00,
            // Protocol
            0x00, 0x00,
            // Login options
            0x00, 0x08,
            // LUN
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            // Target portal group tag
            0x01, 0x00,
            // Node name
            0x69, 0x71, 0x6e, 0x2e, 0x31, 0x39, 0x39, 0x31,
            0x2d, 0x30, 0x35, 0x2e, 0x63, 0x6f, 0x6d, 0x2e,
            0x6d, 0x69, 0x63, 0x72, 0x6f, 0x73, 0x6f, 0x66,
            0x74, 0x3a, 0x69, 0x73, 0x63, 0x73, 0x69, 0x74,
            0x61, 0x72, 0x67, 0x65, 0x74, 0x2d, 0x69, 0x73,
            0x63, 0x73, 0x69, 0x64, 0x69, 0x73, 0x6b, 0x2d,
            0x74, 0x61, 0x72, 0x67, 0x65, 0x74, 0x00,

            // Hard drive node
            0x04, 0x01, 0x2a, 0x00,
            // Partition number
            0x01, 0x00, 0x00, 0x00,
            // Partition start
            0x22, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            // Partition size
            0x00, 0x00, 0x71, 0x02, 0x00, 0x00, 0x00, 0x00,
            // Partition signature
            0x00, 0x9a, 0xe3, 0x15, 0xd2, 0x1d, 0x00, 0x10,
            0x8d, 0x7f, 0x00, 0xa0, 0xc9, 0x24, 0x08, 0xfc,
            // Partition format
            0x02,
            // Signature type
            0x02,

            // End-entire node
            0x7f, 0xff, 0x04, 0x00,
        ]);

        Ok(())
    }
}
