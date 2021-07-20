use super::chars::{Char16, Char8, NUL_16, NUL_8};
use core::convert::TryInto;
use core::fmt;
use core::iter::Iterator;
use core::result::Result;
#[cfg(feature = "exts")]
use crate::alloc_api::string::String;
use core::slice;

/// Errors which can occur during checked [uN] -> CStrN conversions
pub enum FromSliceWithNulError {
    /// An invalid character was encountered before the end of the slice
    InvalidChar(usize),

    /// A null character was encountered before the end of the slice
    InteriorNul(usize),

    /// The slice was not null-terminated
    NotNulTerminated,
}

/// A Latin-1 null-terminated string
///
/// This type is largely inspired by `std::ffi::CStr`, see the documentation of
/// `CStr` for more details on its semantics.
#[repr(transparent)]
pub struct CStr8([Char8]);

impl CStr8 {
    /// Wraps a raw UEFI string with a safe C string wrapper
    ///
    /// # Safety
    ///
    /// The function will start accessing memory from `ptr` until the first
    /// null byte. It's the callers responsability to ensure `ptr` points to
    /// a valid string, in accessible memory.
    pub unsafe fn from_ptr<'ptr>(ptr: *const Char8) -> &'ptr Self {
        let mut len = 0;
        while *ptr.add(len) != NUL_8 {
            len += 1
        }
        let ptr = ptr as *const u8;
        Self::from_bytes_with_nul_unchecked(slice::from_raw_parts(ptr, len + 1))
    }

    /// Creates a C string wrapper from bytes
    pub fn from_bytes_with_nul(chars: &[u8]) -> Result<&Self, FromSliceWithNulError> {
        let nul_pos = chars.iter().position(|&c| c == 0);
        if let Some(nul_pos) = nul_pos {
            if nul_pos + 1 != chars.len() {
                return Err(FromSliceWithNulError::InteriorNul(nul_pos));
            }
            Ok(unsafe { Self::from_bytes_with_nul_unchecked(chars) })
        } else {
            Err(FromSliceWithNulError::NotNulTerminated)
        }
    }

    /// Unsafely creates a C string wrapper from bytes
    ///
    /// # Safety
    ///
    /// It's the callers responsability to ensure chars is a valid Latin-1
    /// null-terminated string, with no interior null bytes.
    pub unsafe fn from_bytes_with_nul_unchecked(chars: &[u8]) -> &Self {
        &*(chars as *const [u8] as *const Self)
    }

    /// Returns the inner pointer to this C string
    pub fn as_ptr(&self) -> *const Char8 {
        self.0.as_ptr()
    }

    /// Converts this C string to a slice of bytes
    pub fn to_bytes(&self) -> &[u8] {
        let chars = self.to_bytes_with_nul();
        &chars[..chars.len() - 1]
    }

    /// Converts this C string to a slice of bytes containing the trailing 0 char
    pub fn to_bytes_with_nul(&self) -> &[u8] {
        unsafe { &*(&self.0 as *const [Char8] as *const [u8]) }
    }
}

/// An UCS-2 null-terminated string
///
/// This type is largely inspired by `std::ffi::CStr`, see the documentation of
/// `CStr` for more details on its semantics.
#[repr(transparent)]
pub struct CStr16([Char16]);

impl CStr16 {
    /// Wraps a raw UEFI string with a safe C string wrapper
    ///
    /// # Safety
    ///
    /// The function will start accessing memory from `ptr` until the first
    /// null byte. It's the callers responsability to ensure `ptr` points to
    /// a valid string, in accessible memory.
    pub unsafe fn from_ptr<'ptr>(ptr: *const Char16) -> &'ptr Self {
        let mut len = 0;
        while *ptr.add(len) != NUL_16 {
            len += 1
        }
        let ptr = ptr as *const u16;
        Self::from_u16_with_nul_unchecked(slice::from_raw_parts(ptr, len + 1))
    }

    /// Creates a C string wrapper from a u16 slice
    ///
    /// Since not every u16 value is a valid UCS-2 code point, this function
    /// must do a bit more validity checking than CStr::from_bytes_with_nul
    pub fn from_u16_with_nul(codes: &[u16]) -> Result<&Self, FromSliceWithNulError> {
        for (pos, &code) in codes.iter().enumerate() {
            match code.try_into() {
                Ok(NUL_16) => {
                    if pos != codes.len() - 1 {
                        return Err(FromSliceWithNulError::InteriorNul(pos));
                    } else {
                        return Ok(unsafe { Self::from_u16_with_nul_unchecked(codes) });
                    }
                }
                Err(_) => {
                    return Err(FromSliceWithNulError::InvalidChar(pos));
                }
                _ => {}
            }
        }
        Err(FromSliceWithNulError::NotNulTerminated)
    }

    /// Unsafely creates a C string wrapper from a u16 slice.
    ///
    /// # Safety
    ///
    /// It's the callers responsability to ensure chars is a valid UCS-2
    /// null-terminated string, with no interior null bytes.
    pub unsafe fn from_u16_with_nul_unchecked(codes: &[u16]) -> &Self {
        &*(codes as *const [u16] as *const Self)
    }

    /// Returns the inner pointer to this C string
    pub fn as_ptr(&self) -> *const Char16 {
        self.0.as_ptr()
    }

    /// Converts this C string to a u16 slice
    pub fn to_u16_slice(&self) -> &[u16] {
        let chars = self.to_u16_slice_with_nul();
        &chars[..chars.len() - 1]
    }

    /// Converts this C string to a u16 slice containing the trailing 0 char
    pub fn to_u16_slice_with_nul(&self) -> &[u16] {
        unsafe { &*(&self.0 as *const [Char16] as *const [u16]) }
    }

    /// Returns an iterator over this C string
    pub fn iter(&self) -> CStr16Iter {
        CStr16Iter {
            inner: self,
            pos: 0,
        }
    }

    /// Write a string slice into the provided buffer. If it fails, then most probably, because
    /// the buffer is not big enough. In that case, the buffer will contain the correct string
    /// until the point, where the size was not enough.
    ///
    /// ## Example
    ///
    /// ```rust
    /// use uefi::data_types::ArrayString;
    /// use uefi::{CStr16, Char16};
    /// let firmware_vendor_c16_str: CStr16 = ...;
    /// // crate "arrayvec" uses stack-allocated arrays for Strings => no heap
    /// let mut buf = arrayvec::ArrayString::<128>::new();
    /// firmware_vendor_c16_str.as_str_in_buf(&mut buf);
    /// log::info!("as rust str: {}", buf.as_str());
    /// ```
    pub fn as_str_in_buf(&self, buf: &mut dyn core::fmt::Write) -> Result<(), ()> {
        for c16 in self.iter() {
            let res = buf.write_char(char::from(*c16));
            if let Err(err) = res {
                log::error!("Failed to write CStr16 as &str into buffer. Buffer too small? ({})", err);
                return Err(())
            }
        }
        Ok(())
    }

    /// Transforms the C16Str to a regular Rust String.
    /// **WARNING** This will require **heap allocation**, i.e. you need an global allocator.
    /// If the UEFI boot services are exited, your OS/Kernel needs to provide another allocation
    /// mechanism!
    #[cfg(feature = "exts")]
    pub fn as_string(&self) -> String {
        let mut buf = String::with_capacity(self.0.len() * 2);
        for c16 in self.iter() {
            buf.push(char::from(*c16));
        }
        buf
    }
}

/// An iterator over `CStr16`.
#[derive(Debug)]
pub struct CStr16Iter<'a> {
    inner: &'a CStr16,
    pos: usize,
}

impl<'a> Iterator for CStr16Iter<'a> {
    type Item = &'a Char16;

    fn next(&mut self) -> Option<Self::Item> {
        if self.pos >= self.inner.0.len() - 1 {
            None
        } else {
            self.pos += 1;
            self.inner.0.get(self.pos - 1)
        }
    }
}

impl fmt::Debug for CStr16 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "CStr16({:?})", &self.0)
    }
}

impl fmt::Display for CStr16 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for c in self.iter() {
            <Char16 as fmt::Display>::fmt(c, f)?;
        }
        Ok(())
    }
}
