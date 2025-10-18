// SPDX-License-Identifier: MIT OR Apache-2.0

//! UEFI Configuration String parsing according to Spec 35.2.1

use alloc::boxed::Box;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::slice;
use core::str::{self, FromStr};
use uguid::Guid;

use crate::mem::AlignedBuffer;
use crate::proto::device_path::DevicePath;
use crate::{CStr16, Char16};

/// A helper struct to split and parse a UEFI Configuration String.
///
/// Configuration strings consist of key-value pairs separated by `&`. Keys
/// and values are separated by `=`. This struct provides an iterator for
/// easy traversal of the key-value pairs.
///
/// For reasons of developer sanity, this is operating on &str instead of &CStr16.
#[derive(Debug)]
pub struct ConfigurationStringIter<'a> {
    bfr: &'a str,
}

impl<'a> ConfigurationStringIter<'a> {
    /// Creates a new splitter instance for a given configuration string.
    #[must_use]
    pub const fn new(bfr: &'a str) -> Self {
        Self { bfr }
    }
}

impl<'a> Iterator for ConfigurationStringIter<'a> {
    type Item = (&'a str, Option<&'a str>);

    fn next(&mut self) -> Option<Self::Item> {
        if self.bfr.is_empty() {
            return None;
        }
        let (keyval, remainder) = self
            .bfr
            .split_once('&')
            .unwrap_or_else(|| (self.bfr, &self.bfr[0..0]));
        self.bfr = remainder;
        let (key, value) = keyval
            .split_once('=')
            .map(|(key, value)| (key, Some(value)))
            .unwrap_or((keyval, None));
        Some((key, value))
    }
}

/// Enum representing different sections of a UEFI Configuration Header.
///
/// These sections include GUID, Name, and Path elements, which provide
/// routing and identification information for UEFI components.
#[derive(Debug, PartialEq, Eq)]
pub enum ConfigHdrSection {
    /// UEFI ConfigurationString {GuidHdr} element
    Guid,
    /// UEFI ConfigurationString {NameHdr} element
    Name,
    /// UEFI ConfigurationString {PathHdr} element
    Path,
}

/// Enum representing possible parsing errors encountered when processing
/// UEFI Configuration Strings.
#[derive(Debug)]
pub enum ParseError {
    /// Error while parsing the UEFI {ConfigHdr} configuration string section.
    ConfigHdr(ConfigHdrSection),
    /// Error while parsing the UEFI {BlockName} configuration string section.
    BlockName,
    /// Error while parsing the UEFI {BlockConfig} configuration string section.
    BlockConfig,
}

/// Represents an individual element within a UEFI Configuration String.
///
/// Each element contains an offset, width, and value, defining the data
/// stored at specific memory locations within the configuration.
#[derive(Debug, Default)]
pub struct ConfigurationStringElement {
    /// Byte offset in the configuration block
    pub offset: u64,
    /// Length of the value starting at offset
    pub width: u64,
    /// Value bytes
    pub value: Vec<u8>,
    // TODO
    // nvconfig: HashMap<String, Vec<u8>>,
}

/// A full UEFI Configuration String representation.
///
/// This structure contains routing information such as GUID and device path,
/// along with the parsed configuration elements.
#[derive(Debug)]
pub struct ConfigurationString {
    /// GUID used for identifying the configuration
    pub guid: Guid,
    /// Name field (optional identifier)
    pub name: String,
    /// Associated UEFI device path
    pub device_path: Box<DevicePath>,
    /// Parsed UEFI {ConfigElement} sections
    pub elements: Vec<ConfigurationStringElement>,
}

impl ConfigurationString {
    fn try_parse_with<T, F: FnOnce() -> Option<T>>(
        err: ParseError,
        parse_fn: F,
    ) -> Result<T, ParseError> {
        parse_fn().ok_or(err)
    }

    /// Parses a hexadecimal string into an iterator of bytes.
    ///
    /// # Arguments
    ///
    /// * `hex` - The hexadecimal string representing binary data.
    ///
    /// # Returns
    ///
    /// An iterator over bytes.
    pub fn parse_bytes_from_hex(hex: &str) -> impl DoubleEndedIterator<Item = u8> {
        hex.as_bytes().chunks(2).map(|chunk| {
            let chunk = str::from_utf8(chunk).unwrap_or_default();
            u8::from_str_radix(chunk, 16).unwrap_or_default()
        })
    }

    /// Converts a hexadecimal string representation into a numeric value.
    ///
    /// # Arguments
    ///
    /// * `data` - The hexadecimal string to convert.
    ///
    /// # Returns
    ///
    /// An `Option<u64>` representing the parsed number.
    #[must_use]
    pub fn parse_number_from_hex(data: &str) -> Option<u64> {
        let data: Vec<_> = Self::parse_bytes_from_hex(data).collect();
        match data.len() {
            8 => Some(u64::from_be_bytes(data.try_into().unwrap())),
            4 => Some(u32::from_be_bytes(data.try_into().unwrap()) as u64),
            2 => Some(u16::from_be_bytes(data.try_into().unwrap()) as u64),
            1 => Some(u8::from_be_bytes(data.try_into().unwrap()) as u64),
            _ => None,
        }
    }

    /// Converts a hexadecimal string into a UTF-16 string.
    ///
    /// # Arguments
    ///
    /// * `data` - The hexadecimal representation of a string.
    ///
    /// # Returns
    ///
    /// An `Option<String>` containing the parsed string.
    #[must_use]
    pub fn parse_string_from_hex(data: &str) -> Option<String> {
        if data.len() % 2 != 0 {
            return None;
        }
        let size_bytes = data.len() / 2 + 2; // includes \0 terminator
        let size_chars = size_bytes / 2;
        let mut bfr = AlignedBuffer::from_size_align(size_bytes, 2).ok()?;
        bfr.copy_from_iter(Self::parse_bytes_from_hex(data).chain([0, 0]));
        bfr.as_slice_mut()
            .chunks_exact_mut(2)
            .for_each(|c| c.swap(0, 1));
        let data: &[Char16] =
            unsafe { slice::from_raw_parts(bfr.as_slice().as_ptr().cast(), size_chars) };
        Some(CStr16::from_char16_with_nul(data).ok()?.to_string())
    }

    /// Parses a hexadecimal string into a UEFI GUID.
    ///
    /// # Arguments
    ///
    /// * `data` - The hexadecimal GUID representation.
    ///
    /// # Returns
    ///
    /// An `Option<Guid>` containing the parsed GUID.
    #[must_use]
    pub fn parse_guid_from_hex(data: &str) -> Option<Guid> {
        let v: Vec<_> = Self::parse_bytes_from_hex(data).collect();
        Some(Guid::from_bytes(v.try_into().ok()?))
    }
}

impl FromStr for ConfigurationString {
    type Err = ParseError;

    fn from_str(bfr: &str) -> Result<Self, Self::Err> {
        let mut splitter = ConfigurationStringIter::new(bfr).peekable();

        let guid = Self::try_parse_with(ParseError::ConfigHdr(ConfigHdrSection::Guid), || {
            let v = splitter.next()?;
            let v = (v.0 == "GUID").then_some(v.1).flatten()?;
            Self::parse_guid_from_hex(v)
        })?;
        let name = Self::try_parse_with(ParseError::ConfigHdr(ConfigHdrSection::Name), || {
            let v = splitter.next()?;
            let v = (v.0 == "NAME").then_some(v.1).flatten()?;
            Self::parse_string_from_hex(v)
        })?;
        let device_path =
            Self::try_parse_with(ParseError::ConfigHdr(ConfigHdrSection::Path), || {
                let v = splitter.next()?.1?;
                let v: Vec<_> = Self::parse_bytes_from_hex(v).collect();
                let v = <&DevicePath>::try_from(v.as_slice()).ok()?;
                Some(v.to_boxed())
            })?;

        let mut elements = Vec::new();
        loop {
            let offset = match splitter.next() {
                Some(("OFFSET", Some(data))) => {
                    Self::parse_number_from_hex(data).ok_or(ParseError::BlockName)?
                }
                None => break,
                _ => return Err(ParseError::BlockName),
            };
            let width = match splitter.next() {
                Some(("WIDTH", Some(data))) => {
                    Self::parse_number_from_hex(data).ok_or(ParseError::BlockName)?
                }
                _ => return Err(ParseError::BlockName),
            };
            // The uefi specification declares `VALUE` as being a "number" of arbitrary length.
            // And numbers have to be endianness-corrected. Thus, the bytes represented by a value's
            // hex string of arbitrary length have to be reversed to account for that weird encoding.
            let value = match splitter.next() {
                Some(("VALUE", Some(data))) => Self::parse_bytes_from_hex(data).rev().collect(),
                _ => return Err(ParseError::BlockConfig),
            };

            while let Some(next) = splitter.peek() {
                if next.0 == "OFFSET" {
                    break;
                }
                let _ = splitter.next(); // drop nvconfig entries for now
            }

            elements.push(ConfigurationStringElement {
                offset,
                width,
                value,
            });
        }

        Ok(Self {
            guid,
            name,
            device_path,
            elements,
        })
    }
}

#[cfg(test)]
mod tests {
    use core::str::FromStr;

    use crate::proto::hii::config_str::ConfigurationString;

    #[test]
    fn parse_single() {
        // exemplary (shortened / manually constructed) UEFI configuration string
        let input = "GUID=16d6474bd6a852459d44ccad2e0f4cf9&NAME=00490053004300530049005f0043004f004e004600490047005f004900460052005f004e00560044004100540041&PATH=0104140016d6474bd6a852459d44ccad2e0f4cf97fff0400&OFFSET=01d8&WIDTH=0002&VALUE=0011&OFFSET=01d9&WIDTH=0001&VALUE=00&OFFSET=01da&WIDTH=0001&VALUE=00&OFFSET=01dc&WIDTH=0002&VALUE=03e8&OFFSET=01de&WIDTH=0001&VALUE=00&OFFSET=01df&WIDTH=0001&VALUE=00&OFFSET=05fe&WIDTH=0002&VALUE=0000&OFFSET=062a&WIDTH=0001&VALUE=00&OFFSET=062b&WIDTH=0001&VALUE=01&OFFSET=0fd4&WIDTH=0001&VALUE=00&OFFSET=0fd5&WIDTH=0001&VALUE=00";
        let parsed = ConfigurationString::from_str(input).unwrap();
        assert_eq!(parsed.guid, guid!("4b47d616-a8d6-4552-9d44-ccad2e0f4cf9"));
        assert_eq!(parsed.name, "ISCSI_CONFIG_IFR_NVDATA");
        assert_eq!(parsed.alt_cfg_id, None);
        assert_eq!(parsed.elements.len(), 11);
        assert_eq!(parsed.elements[0].offset, 0x01d8);
        assert_eq!(parsed.elements[0].width, 2);
        assert_eq!(&parsed.elements[0].value, &[0x11, 0x00]);
        assert_eq!(parsed.elements[10].offset, 0x0fd5);
        assert_eq!(parsed.elements[10].width, 1);
        assert_eq!(&parsed.elements[10].value, &[0x00]);
    }
}
