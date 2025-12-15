// SPDX-License-Identifier: MIT OR Apache-2.0

//! HII Database protocol.

use alloc::boxed::Box;
use uefi_macros::unsafe_protocol;
use uefi_raw::protocol::hii::database::HiiDatabaseProtocol;

use crate::mem::make_boxed;
use crate::{Error, StatusExt};

/// The HII Configuration Access Protocol.
///
/// This protocol grants access to the HII database definition available in every UEFI firmware.
/// This database contains internationalized strings, as well as a description of all
/// supported BIOS settings, together with their logic (e.g.: option A blocks option B if value is `true`).
///
/// # UEFI Spec Description
///
/// Database manager for HII-related data structures.
#[derive(Debug)]
#[repr(transparent)]
#[unsafe_protocol(HiiDatabaseProtocol::GUID)]
pub struct HiiDatabase(HiiDatabaseProtocol);

impl HiiDatabase {
    /// Export all package lists as raw byte buffer.
    pub fn export_all_raw(&self) -> crate::Result<Box<[u8]>> {
        fn fetch_data_fn<'a>(
            proto: &HiiDatabase,
            buf: &'a mut [u8],
        ) -> Result<&'a mut [u8], Error<Option<usize>>> {
            unsafe {
                let mut size = buf.len();
                let status = {
                    (proto.0.export_package_lists)(
                        &proto.0,
                        core::ptr::null_mut(),
                        &mut size,
                        buf.as_mut_ptr().cast(),
                    )
                };
                status.to_result_with_err(|_| Some(size)).map(|_| buf)
            }
        }

        let buf = make_boxed::<[u8], _>(|buf| fetch_data_fn(self, buf))?;

        Ok(buf)
    }
}
