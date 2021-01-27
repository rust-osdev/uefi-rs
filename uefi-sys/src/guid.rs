impl GUID {
    /// Creates a new GUID from its canonical representation
    //
    // FIXME: An unwieldy array of bytes must be used for the node ID until one
    //        can assert that an u64 has its high 16-bits cleared in a const fn.
    //        Once that is done, we can take an u64 to be even closer to the
    //        canonical UUID/GUID format.
    //
    pub const fn from_values(
        time_low: u32,
        time_mid: u16,
        time_high_and_version: u16,
        clock_seq_and_variant: u16,
        node: [u8; 6],
    ) -> Self {
        GUID {
            Data1: time_low,
            Data2: time_mid,
            Data3: time_high_and_version,
            Data4: [
                (clock_seq_and_variant / 0x100) as u8,
                (clock_seq_and_variant % 0x100) as u8,
                node[0],
                node[1],
                node[2],
                node[3],
                node[4],
                node[5],
            ],
        }
    }
}

impl core::fmt::Display for GUID {
    fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::fmt::Result {
        let d = {
            let (low, high) = (u16::from(self.Data4[0]), u16::from(self.Data4[1]));

            (low << 8) | high
        };
        // Extract and reverse byte order.
        let e = self.Data4[2..8]
            .iter()
            .enumerate()
            .fold(0, |acc, (i, &elem)| {
                acc | {
                    let shift = (5 - i) * 8;
                    u64::from(elem) << shift
                }
            });

        write!(
            fmt,
            "{:08x}-{:04x}-{:04x}-{:04x}-{:012x}",
            self.Data1, self.Data2, self.Data3, d, e
        )
    }
}
