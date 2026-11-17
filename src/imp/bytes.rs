// SPDX-License-Identifier: Apache-2.0
//
// Written in 2022-2026 by Dr. Maxim Orlovsky <orlovsky@ubideco.org>
//
// Copyright 2022-2026 Laboratories for Ubiquitous and Deterministic Computing,
// Institute for Distributed and Cognitive Computing (InDCS), Switzerland.
// All rights reserved.
//
// Copyright (C) 2022-2026 Dr Maxim Orlovsky.
// All rights reserved.
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not
// use this file except in compliance with the License. You may obtain a copy of
// the License at <http://www.apache.org/licenses/LICENSE-2.0>
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS, WITHOUT
// WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the
// License for the specific language governing permissions and limitations under
// the License.

#[cfg(feature = "std")]
mod io {
    use alloc::alloc::Allocator;
    use alloc::vec::Vec;
    use std::io;

    use crate::Confined;

    impl<A: Allocator, const MIN_LEN: usize, const MAX_LEN: usize> io::Write
        for Confined<Vec<u8, A>, MIN_LEN, MAX_LEN>
    {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            if buf.is_empty() {
                return Ok(0);
            }
            self.try_reserve(buf.len())
                .map_err(|_| io::Error::from(io::ErrorKind::OutOfMemory))?;
            self.0.extend_from_slice(buf);
            Ok(buf.len())
        }

        fn flush(&mut self) -> io::Result<()> {
            // Do nothing
            Ok(())
        }
    }
}

#[cfg(feature = "hex")]
mod hex {
    use alloc::alloc::Allocator;
    use alloc::vec::Vec;
    use core::fmt::{self, Formatter, LowerHex, UpperHex};

    use amplify_num::hex::{self, FromHex, HexIterator};

    use crate::Confined;

    /// Hex without temporary allocations. Alternate formatting abbreviates
    /// blobs longer than 16 bytes to their first and last eight bytes.
    impl<A: Allocator, const MIN_LEN: usize, const MAX_LEN: usize> LowerHex
        for Confined<Vec<u8, A>, MIN_LEN, MAX_LEN>
    {
        fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
            format_hex(&self.0, f, b"0123456789abcdef")
        }
    }

    impl<A: Allocator, const MIN_LEN: usize, const MAX_LEN: usize> UpperHex
        for Confined<Vec<u8, A>, MIN_LEN, MAX_LEN>
    {
        fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
            format_hex(&self.0, f, b"0123456789ABCDEF")
        }
    }

    fn format_hex(bytes: &[u8], f: &mut Formatter<'_>, digits: &[u8; 16]) -> fmt::Result {
        fn write(bytes: &[u8], f: &mut Formatter<'_>, digits: &[u8; 16]) -> fmt::Result {
            for byte in bytes {
                let pair = [
                    digits[usize::from(byte >> 4)],
                    digits[usize::from(byte & 15)],
                ];
                f.write_str(core::str::from_utf8(&pair).expect("ASCII hex digits"))?;
            }
            Ok(())
        }

        if f.alternate() && bytes.len() > 16 {
            write(&bytes[..8], f, digits)?;
            f.write_str("..")?;
            write(&bytes[bytes.len() - 8..], f, digits)
        } else {
            write(bytes, f, digits)
        }
    }

    /// The number of bytes must fit the confinement. Otherwise the result is
    /// [`hex::Error::InvalidLength`] with the violated bound and the number of
    /// bytes in the data.
    ///
    /// The upstream error type cannot represent allocation failure; allocation
    /// follows `Vec::push` semantics. Iterator size hints are never trusted.
    impl<A: Allocator + Default, const MIN_LEN: usize, const MAX_LEN: usize> FromHex
        for Confined<Vec<u8, A>, MIN_LEN, MAX_LEN>
    {
        fn from_byte_iter<I>(iter: I) -> Result<Self, hex::Error>
        where
            I: Iterator<Item = Result<u8, hex::Error>> + ExactSizeIterator + DoubleEndedIterator,
        {
            let mut bytes = Vec::<u8, A>::new_in(A::default());
            for byte in iter {
                if bytes.len() == MAX_LEN {
                    return Err(hex::Error::InvalidLength(
                        MAX_LEN,
                        MAX_LEN.saturating_add(1),
                    ));
                }
                bytes.push(byte?);
            }
            check_len::<MIN_LEN, MAX_LEN>(bytes.len())?;
            Ok(Self(bytes))
        }

        fn from_hex(s: &str) -> Result<Self, hex::Error> {
            let iter = HexIterator::new(s)?;
            check_len::<MIN_LEN, MAX_LEN>(iter.len())?;
            Self::from_byte_iter(iter)
        }
    }

    fn check_len<const MIN_LEN: usize, const MAX_LEN: usize>(len: usize) -> Result<(), hex::Error> {
        if len < MIN_LEN {
            return Err(hex::Error::InvalidLength(MIN_LEN, len));
        }
        if len > MAX_LEN {
            return Err(hex::Error::InvalidLength(MAX_LEN, len));
        }
        Ok(())
    }
}
