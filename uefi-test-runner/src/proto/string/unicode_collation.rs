// SPDX-License-Identifier: MIT OR Apache-2.0

use core::cmp::Ordering;
use uefi::proto::string::unicode_collation::{StrConversionError, UnicodeCollation};
use uefi::{boot, CStr16, CStr8};

pub fn test() {
    info!("Testing the Unicode Collation protocol");

    let handles =
        boot::find_handles::<UnicodeCollation>().expect("missing UnicodeCollation protocol");
    for handle in handles {
        let uc = boot::open_protocol_exclusive::<UnicodeCollation>(handle).unwrap();

        let mut buf1 = [0; 30];
        let mut buf2 = [0; 30];

        macro_rules! strings {
            ($s1:expr, $s2:expr) => {{
                let s1 = CStr16::from_str_with_buf($s1, &mut buf1).unwrap();
                let s2 = CStr16::from_str_with_buf($s2, &mut buf2).unwrap();
                (s1, s2)
            }};
        }

        let (s1, s2) = strings!("aab", "aaa");
        // "aab" is lexically greater than "aaa"
        assert_eq!(uc.stri_coll(s1, s2), Ordering::Greater);

        let (s1, s2) = strings!("{}", "{}");
        assert_eq!(uc.stri_coll(s1, s2), Ordering::Equal);

        let (s1, s2) = strings!("\t", "-");
        // Tab comes before dash in the unicode table
        assert_eq!(uc.stri_coll(s1, s2), Ordering::Less);

        let (s, pattern) = strings!("haaaaaaaaarderr", "h*a*r*derr");
        assert!(uc.metai_match(s, pattern));

        let (s, pattern) = strings!("haaaaaaaaarder0r", "h*a*r*derr");
        assert!(!uc.metai_match(s, pattern));

        let mut buf1 = [0; 13];
        let s = CStr16::from_str_with_buf("HeLlO World!", &mut buf1).unwrap();

        let mut buf2 = [0; 12];
        assert_eq!(
            uc.str_lwr(s, &mut buf2),
            Err(StrConversionError::BufferTooSmall)
        );

        let mut buf2 = [0; 13];
        let lower_s = uc.str_lwr(s, &mut buf2).unwrap();
        assert_eq!(
            lower_s,
            CStr16::from_str_with_buf("hello world!", &mut [0; 13]).unwrap()
        );

        let mut buf = [0; 12];
        assert_eq!(
            uc.str_upr(s, &mut buf),
            Err(StrConversionError::BufferTooSmall)
        );

        let mut buf = [0; 13];
        let upper_s = uc.str_upr(s, &mut buf).unwrap();
        assert_eq!(
            upper_s,
            CStr16::from_str_with_buf("HELLO WORLD!", &mut [0; 13]).unwrap()
        );

        let s = CStr8::from_bytes_with_nul(b"Hello World!\0").unwrap();
        assert_eq!(
            uc.fat_to_str(s, &mut [0; 12]),
            Err(StrConversionError::BufferTooSmall)
        );

        assert_eq!(
            uc.fat_to_str(s, &mut [0; 13]).unwrap(),
            CStr16::from_str_with_buf("Hello World!", &mut [0; 13]).unwrap()
        );

        let mut buf = [0; 13];
        let s = CStr16::from_str_with_buf("Hello World!", &mut buf).unwrap();
        let mut buf = [0; 12];
        assert_eq!(
            uc.str_to_fat(s, &mut buf),
            Err(StrConversionError::BufferTooSmall)
        );
        let mut buf = [0; 13];
        assert_eq!(
            uc.str_to_fat(s, &mut buf).unwrap(),
            CStr8::from_bytes_with_nul(b"HELLOWORLD!\0").unwrap()
        );
    }
}
