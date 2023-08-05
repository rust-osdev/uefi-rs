// This module defines the `ComponentName1` type and marks it deprecated. That
// causes warnings for uses within this module (e.g. the `impl ComponentName1`
// block), so turn off deprecated warnings. It's not yet possible to make this
// allow more fine-grained, see https://github.com/rust-lang/rust/issues/62398.
#![allow(deprecated)]

use crate::proto::unsafe_protocol;
use crate::table::boot::{BootServices, ScopedProtocol};
use crate::{CStr16, Error, Handle, Result, Status, StatusExt};
use core::fmt::{Debug, Formatter};
use core::{ptr, slice};
use uefi_raw::protocol::driver::ComponentName2Protocol;

/// Protocol that provides human-readable names for a driver and for each of the
/// controllers that the driver is managing.
///
/// This protocol was deprecated in UEFI 2.1 in favor of the new
/// [`ComponentName2`] protocol. The two protocols are identical except the
/// encoding of supported languages changed from [ISO 639-2] to [RFC 4646]. The
/// [`ComponentName`] wrapper can be used to automatically select
/// [`ComponentName2`] if available, and otherwise fall back to
/// [`ComponentName1`].
///
/// The corresponding C type is `EFI_COMPONENT_NAME_PROTOCOL`.
///
/// [ISO 639-2]: https://en.wikipedia.org/wiki/List_of_ISO_639-2_codes
/// [RFC 4646]: https://www.rfc-editor.org/rfc/rfc4646
#[deprecated = "deprecated in UEFI 2.1; use ComponentName2 where possible"]
#[unsafe_protocol(ComponentName2Protocol::DEPRECATED_COMPONENT_NAME_GUID)]
#[repr(transparent)]
pub struct ComponentName1(
    // The layout of the protocol is the same as ComponentName2, only the format
    // of the language string changed.
    ComponentName2Protocol,
);

impl ComponentName1 {
    /// Get an iterator over supported languages. Each language is identified by
    /// a three-letter ASCII string specified in [ISO 639-2]. For example,
    /// English is encoded as "eng".
    ///
    /// [ISO 639-2]: https://en.wikipedia.org/wiki/List_of_ISO_639-2_codes
    pub fn supported_languages(&self) -> core::result::Result<LanguageIter, LanguageError> {
        LanguageIter::new(self.0.supported_languages, LanguageIterKind::V1)
    }

    /// Get the human-readable name of the driver in the given language.
    ///
    /// `language` must be one of the languages returned by [`supported_languages`].
    ///
    /// [`supported_languages`]: Self::supported_languages
    pub fn driver_name(&self, language: &str) -> Result<&CStr16> {
        let language = language_to_cstr(language)?;
        let mut driver_name = ptr::null();
        unsafe { (self.0.get_driver_name)(&self.0, language.as_ptr(), &mut driver_name) }
            .to_result_with_val(|| unsafe { CStr16::from_ptr(driver_name.cast()) })
    }

    /// Get the human-readable name of a controller in the given language.
    ///
    /// `language` must be one of the languages returned by [`supported_languages`].
    ///
    /// [`supported_languages`]: Self::supported_languages
    pub fn controller_name(
        &self,
        controller_handle: Handle,
        child_handle: Option<Handle>,
        language: &str,
    ) -> Result<&CStr16> {
        let language = language_to_cstr(language)?;
        let mut driver_name = ptr::null();
        unsafe {
            (self.0.get_controller_name)(
                &self.0,
                controller_handle.as_ptr(),
                Handle::opt_to_ptr(child_handle),
                language.as_ptr(),
                &mut driver_name,
            )
        }
        .to_result_with_val(|| unsafe { CStr16::from_ptr(driver_name.cast()) })
    }
}

/// Protocol that provides human-readable names for a driver and for each of the
/// controllers that the driver is managing.
///
/// This protocol was introduced in UEFI 2.1 to replace the now-deprecated
/// [`ComponentName1`] protocol. The two protocols are identical except the
/// encoding of supported languages changed from [ISO 639-2] to [RFC 4646]. The
/// [`ComponentName`] wrapper can be used to automatically select
/// [`ComponentName2`] if available, and otherwise fall back to
/// [`ComponentName1`].
///
/// The corresponding C type is `EFI_COMPONENT_NAME2_PROTOCOL`.
///
/// [ISO 639-2]: https://en.wikipedia.org/wiki/List_of_ISO_639-2_codes
/// [RFC 4646]: https://www.rfc-editor.org/rfc/rfc4646
#[unsafe_protocol(ComponentName2Protocol::GUID)]
#[repr(transparent)]
pub struct ComponentName2(ComponentName2Protocol);

impl ComponentName2 {
    /// Get an iterator over supported languages. Each language is identified by
    /// an ASCII string specified in [RFC 4646]. For example, English is encoded
    /// as "en".
    ///
    /// [RFC 4646]: https://www.rfc-editor.org/rfc/rfc4646
    pub fn supported_languages(&self) -> core::result::Result<LanguageIter, LanguageError> {
        LanguageIter::new(self.0.supported_languages, LanguageIterKind::V2)
    }

    /// Get the human-readable name of the driver in the given language.
    ///
    /// `language` must be one of the languages returned by [`supported_languages`].
    ///
    /// [`supported_languages`]: Self::supported_languages
    pub fn driver_name(&self, language: &str) -> Result<&CStr16> {
        let language = language_to_cstr(language)?;
        let mut driver_name = ptr::null();
        unsafe { (self.0.get_driver_name)(&self.0, language.as_ptr(), &mut driver_name) }
            .to_result_with_val(|| unsafe { CStr16::from_ptr(driver_name.cast()) })
    }

    /// Get the human-readable name of a controller in the given language.
    ///
    /// `language` must be one of the languages returned by [`supported_languages`].
    ///
    /// [`supported_languages`]: Self::supported_languages
    pub fn controller_name(
        &self,
        controller_handle: Handle,
        child_handle: Option<Handle>,
        language: &str,
    ) -> Result<&CStr16> {
        let language = language_to_cstr(language)?;
        let mut driver_name = ptr::null();
        unsafe {
            (self.0.get_controller_name)(
                &self.0,
                controller_handle.as_ptr(),
                Handle::opt_to_ptr(child_handle),
                language.as_ptr(),
                &mut driver_name,
            )
        }
        .to_result_with_val(|| unsafe { CStr16::from_ptr(driver_name.cast()) })
    }
}

/// Wrapper around [`ComponentName1`] and [`ComponentName2`]. This will use
/// [`ComponentName2`] if available, otherwise it will back to
/// [`ComponentName1`].
pub enum ComponentName<'a> {
    /// Opened [`ComponentName1`] protocol.
    V1(ScopedProtocol<'a, ComponentName1>),

    /// Opened [`ComponentName2`] protocol.
    V2(ScopedProtocol<'a, ComponentName2>),
}

impl<'a> ComponentName<'a> {
    /// Open the [`ComponentName2`] protocol if available, otherwise fall back to
    /// [`ComponentName1`].
    pub fn open(boot_services: &'a BootServices, handle: Handle) -> Result<Self> {
        if let Ok(cn2) = boot_services.open_protocol_exclusive::<ComponentName2>(handle) {
            Ok(Self::V2(cn2))
        } else {
            Ok(Self::V1(
                boot_services.open_protocol_exclusive::<ComponentName1>(handle)?,
            ))
        }
    }

    /// Get an iterator over supported languages. Each language is identified by
    /// an ASCII string. If the opened protocol is [`ComponentName1`] this will
    /// be an [ISO 639-2] string. If the opened protocol is [`ComponentName2`]
    /// it will be an [RFC 4646] string. For example, English is encoded as
    /// "eng" in ISO 639-2, and "en" in RFC 4646.
    ///
    /// [ISO 639-2]: https://en.wikipedia.org/wiki/List_of_ISO_639-2_codes
    /// [RFC 4646]: https://www.rfc-editor.org/rfc/rfc4646
    pub fn supported_languages(&self) -> core::result::Result<LanguageIter, LanguageError> {
        match self {
            Self::V1(cn1) => cn1.supported_languages(),
            Self::V2(cn2) => cn2.supported_languages(),
        }
    }

    /// Get the human-readable name of the driver in the given language.
    ///
    /// `language` must be one of the languages returned by [`supported_languages`].
    ///
    /// [`supported_languages`]: Self::supported_languages
    pub fn driver_name(&self, language: &str) -> Result<&CStr16> {
        match self {
            Self::V1(cn1) => cn1.driver_name(language),
            Self::V2(cn2) => cn2.driver_name(language),
        }
    }

    /// Get the human-readable name of a controller in the given language.
    ///
    /// `language` must be one of the languages returned by [`supported_languages`].
    ///
    /// [`supported_languages`]: Self::supported_languages
    pub fn controller_name(
        &self,
        controller_handle: Handle,
        child_handle: Option<Handle>,
        language: &str,
    ) -> Result<&CStr16> {
        match self {
            Self::V1(cn1) => cn1.controller_name(controller_handle, child_handle, language),
            Self::V2(cn2) => cn2.controller_name(controller_handle, child_handle, language),
        }
    }
}

impl<'a> Debug for ComponentName<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            ComponentName::V1(_) => f.debug_tuple("V1").finish(),
            ComponentName::V2(_) => f.debug_tuple("V2").finish(),
        }
    }
}

/// Error returned by [`ComponentName1::supported_languages`] and
/// [`ComponentName2::supported_languages`].
#[derive(Debug, Eq, PartialEq)]
pub enum LanguageError {
    /// The supported languages list contains a non-ASCII character at the
    /// specified index.
    Ascii {
        /// Index of the invalid character.
        index: usize,
    },
}

#[derive(Debug, PartialEq)]
enum LanguageIterKind {
    V1,
    V2,
}

/// Iterator returned by [`ComponentName1::supported_languages`] and
/// [`ComponentName2::supported_languages`].
#[derive(Debug)]
pub struct LanguageIter<'a> {
    languages: &'a [u8],
    kind: LanguageIterKind,
}

impl<'a> LanguageIter<'a> {
    fn new(
        languages: *const u8,
        kind: LanguageIterKind,
    ) -> core::result::Result<Self, LanguageError> {
        let mut index = 0;
        loop {
            let c = unsafe { languages.add(index).read() };
            if c == 0 {
                break;
            } else if !c.is_ascii() {
                return Err(LanguageError::Ascii { index });
            } else {
                index += 1;
            }
        }

        Ok(Self {
            languages: unsafe { slice::from_raw_parts(languages, index) },
            kind,
        })
    }
}

impl<'a> Iterator for LanguageIter<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        if self.languages.is_empty() {
            return None;
        }

        let lang;
        match self.kind {
            LanguageIterKind::V1 => {
                if self.languages.len() <= 3 {
                    lang = self.languages;
                    self.languages = &[];
                } else {
                    lang = &self.languages[..3];
                    self.languages = &self.languages[3..];
                }
            }
            LanguageIterKind::V2 => {
                if let Some(index) = self.languages.iter().position(|c| *c == b';') {
                    lang = &self.languages[..index];
                    self.languages = &self.languages[index + 1..];
                } else {
                    lang = self.languages;
                    self.languages = &[];
                }
            }
        }

        // OK to unwrap because we already checked the string is ASCII.
        Some(core::str::from_utf8(lang).unwrap())
    }
}

/// Statically-sized buffer used to convert a `str` to a null-terminated C
/// string. The buffer should be at least 42 characters per
/// <https://www.rfc-editor.org/rfc/rfc4646#section-4.3.1>, plus one for the
/// null terminator. Round up to 64 bytes just for aesthetics.
type LanguageCStr = [u8; 64];

fn language_to_cstr(language: &str) -> Result<LanguageCStr> {
    let mut lang_cstr: LanguageCStr = [0; 64];
    // Ensure there's room for a null-terminator.
    if language.len() >= lang_cstr.len() - 1 {
        return Err(Error::from(Status::BUFFER_TOO_SMALL));
    }
    lang_cstr[..language.len()].copy_from_slice(language.as_bytes());
    // Assert that it's null-terminated.
    assert_eq!(*lang_cstr.last().unwrap(), 0);
    Ok(lang_cstr)
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec::Vec;
    use LanguageIterKind::{V1, V2};

    #[test]
    fn test_language_iter_v1() {
        // Empty string.
        let data = "\0";
        assert!(LanguageIter::new(data.as_ptr(), V1)
            .unwrap()
            .next()
            .is_none());

        // Two languages.
        let data = "engfra\0";
        assert_eq!(
            LanguageIter::new(data.as_ptr(), V1)
                .unwrap()
                .collect::<Vec<_>>(),
            ["eng", "fra"]
        );

        // Truncated data.
        let data = "en\0";
        assert_eq!(
            LanguageIter::new(data.as_ptr(), V1)
                .unwrap()
                .collect::<Vec<_>>(),
            ["en"]
        );

        // Non-ASCII.
        let data = "engæ\0";
        assert_eq!(
            LanguageIter::new(data.as_ptr(), V1).err().unwrap(),
            LanguageError::Ascii { index: 3 },
        );
    }

    #[test]
    fn test_language_iter_v2() {
        // Empty string.
        let data = "\0";
        assert!(LanguageIter::new(data.as_ptr(), V2)
            .unwrap()
            .next()
            .is_none());

        // Two languages.
        let data = "en;fr\0";
        assert_eq!(
            LanguageIter::new(data.as_ptr(), V2)
                .unwrap()
                .collect::<Vec<_>>(),
            ["en", "fr"]
        );

        // Non-ASCII.
        let data = "engæ\0";
        assert_eq!(
            LanguageIter::new(data.as_ptr(), V2).err().unwrap(),
            LanguageError::Ascii { index: 3 },
        );
    }

    #[test]
    fn test_language_to_cstr() {
        let mut expected = [0; 64];
        expected[0] = b'e';
        expected[1] = b'n';
        assert_eq!(language_to_cstr("en"), Ok(expected));

        assert_eq!(
            language_to_cstr(
                "0123456789012345678901234567890123456789012345678901234567890123456789"
            )
            .err()
            .unwrap()
            .status(),
            Status::BUFFER_TOO_SMALL
        );
    }
}
