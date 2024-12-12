// Copyright 2019 TiKV Project Authors. Licensed under MIT or Apache-2.0.

//! `crc64fast-nvme`
//! ===========
//!
//! SIMD-accelerated CRC-64/NVME computation
//! (similar to [`crc32fast`](https://crates.io/crates/crc32fast)).
//!
//! ## Usage
//!
//! ```
//! use crc64fast_nvme::Digest;
//!
//! let mut c = Digest::new();
//! c.write(b"hello ");
//! c.write(b"world!");
//! let checksum = c.sum64();
//! assert_eq!(checksum, 0xd9160d1fa8e418e3);
//! ```
//!
//! Tracking links for unstable features used here (which require nightly builds):
//!
//! - avx512: https://github.com/rust-lang/rust/issues/111137

#![cfg_attr(
    feature = "vpclmulqdq",
    feature(avx512_target_feature,stdarch_x86_avx512)
)]

mod pclmulqdq;
mod table;

type UpdateFn = unsafe fn(u64, &[u8]) -> u64;

/// Represents an in-progress CRC-64 computation.
#[derive(Clone)]
pub struct Digest {
    computer: UpdateFn,
    state: u64,
}

impl Digest {
    /// Creates a new `Digest`.
    ///
    /// It will perform runtime CPU feature detection to determine which
    /// algorithm to choose.
    pub fn new() -> Self {
        Self {
            computer: pclmulqdq::get_update(),
            state: !0,
        }
    }

    /// Creates a new `Digest` using table-based algorithm.
    pub fn new_table() -> Self {
        Self {
            computer: table::update,
            state: !0,
        }
    }

    /// Writes some data into the digest.
    pub fn write(&mut self, bytes: &[u8]) {
        unsafe {
            self.state = (self.computer)(self.state, bytes);
        }
    }

    /// Computes the current CRC-64/NVME value.
    pub fn sum64(&self) -> u64 {
        !self.state
    }
}

impl Default for Digest {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::Digest;
    use proptest::collection::size_range;
    use proptest::prelude::*;

    // CRC-64/NVME
    //
    // NVM Express® NVM Command Set Specification (Revision 1.0d, December 2023)
    //
    // https://nvmexpress.org/wp-content/uploads/NVM-Express-NVM-Command-Set-Specification-1.0d-2023.12.28-Ratified.pdf
    //
    // Note: The Check value published in the spec is incorrect (Section 5.2.1.3.4, Figure 120, page 83).
    const CRC_NVME: crc::Algorithm<u64> = crc::Algorithm {
        width: 64,
        poly: 0xAD93D23594C93659,
        init: 0xFFFFFFFFFFFFFFFF,
        refin: true,
        refout: true,
        xorout: 0xFFFFFFFFFFFFFFFF,
        check: 0xae8b14860a799888,
        residue: 0x0000000000000000,
    };

    #[test]
    fn test_standard_vectors() {
        static CASES: &[(&[u8], u64)] = &[
            // from the NVM Express® NVM Command Set Specification (Revision 1.0d, December 2023),
            // Section 5.2.1.3.5, Figure 122, page 84.
            // https://nvmexpress.org/wp-content/uploads/NVM-Express-NVM-Command-Set-Specification-1.0d-2023.12.28-Ratified.pdf
            // and the Linux kernel
            // https://github.com/torvalds/linux/blob/f3813f4b287e480b1fcd62ca798d8556644b8278/crypto/testmgr.h#L3685-L3695
            (&[0; 4096], 0x6482d367eb22b64e),
            (&[255; 4096], 0xc0ddba7302eca3ac),

            // from our own internal tests, since the Check value in the  NVM Express® NVM Command
            // Set Specification (Revision 1.0d, December 2023) is incorrect (Section 5.2.1.3.4, Figure 120, page 83).
            (b"123456789", 0xae8b14860a799888),

            // updated values from the original CRC-64/XZ fork of this project
            (b"", 0),
            (b"@", 0x2808afa9582aa47),
            (b"1\x97", 0xb4af0ae0feb08e0f),
            (b"M\"\xdf", 0x85d7cd041a2a8a5d),
            (b"l\xcd\x13\xd7", 0x1860820ea79b0fa3),

            (&[0; 32], 0xcf3473434d4ecf3b),
            (&[255; 32], 0xa0a06974c34d63c4),
            (b"\x00\x01\x02\x03\x04\x05\x06\x07\x08\x09\x0A\x0B\x0C\x0D\x0E\x0F\x10\x11\x12\x13\x14\x15\x16\x17\x18\x19\x1A\x1B\x1C\x1D\x1E\x1F", 0xb9d9d4a8492cbd7f),

            (&[0; 1024], 0x691bb2b09be5498a),

            (b"hello world!", 0xd9160d1fa8e418e3),
        ];

        for (input, result) in CASES {
            let mut hasher = Digest::new();
            hasher.write(input);
            assert_eq!(hasher.sum64(), *result, "test case {:x?}", input);
        }
    }

    fn any_buffer() -> <Box<[u8]> as Arbitrary>::Strategy {
        any_with::<Box<[u8]>>(size_range(..65536).lift())
    }

    prop_compose! {
        fn bytes_and_split_index()
            (bytes in any_buffer())
            (index in 0..=bytes.len(), bytes in Just(bytes)) -> (Box<[u8]>, usize)
        {
            (bytes, index)
        }
    }

    proptest! {
        #[test]
        fn equivalent_to_crc(bytes in any_buffer()) {
            let mut hasher = Digest::new();
            hasher.write(&bytes);

            // CRC-64/NVME
            let crc = crc::Crc::<u64>::new(&CRC_NVME);
            let mut digest = crc.digest();
            digest.update(&bytes);

            prop_assert_eq!(hasher.sum64(), digest.finalize());
        }

        #[test]
        fn concatenation((bytes, split_index) in bytes_and_split_index()) {
            let mut hasher_1 = Digest::new();
            hasher_1.write(&bytes);
            let mut hasher_2 = Digest::new();
            let (left, right) = bytes.split_at(split_index);
            hasher_2.write(left);
            hasher_2.write(right);
            prop_assert_eq!(hasher_1.sum64(), hasher_2.sum64());
        }

        #[test]
        fn state_cloning(left in any_buffer(), right in any_buffer()) {
            let mut hasher_1 = Digest::new();
            hasher_1.write(&left);
            let mut hasher_2 = hasher_1.clone();
            hasher_1.write(&right);
            hasher_2.write(&right);
            prop_assert_eq!(hasher_1.sum64(), hasher_2.sum64());
        }
    }
}
