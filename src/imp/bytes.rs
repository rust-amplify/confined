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
            let len = self.len();
            let buf_len = buf.len();
            if buf_len > MAX_LEN || len >= MAX_LEN || len.saturating_add(buf_len) > MAX_LEN {
                return Err(io::Error::from(io::ErrorKind::OutOfMemory));
            }
            self.0.extend(buf);
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

    use amplify_num::hex::{self, FromHex, ToHex};

    use crate::Confined;

    impl<A: Allocator, const MIN_LEN: usize, const MAX_LEN: usize> LowerHex
        for Confined<Vec<u8, A>, MIN_LEN, MAX_LEN>
    {
        fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
            f.write_str(&self.0.to_hex())
        }
    }

    impl<A: Allocator, const MIN_LEN: usize, const MAX_LEN: usize> UpperHex
        for Confined<Vec<u8, A>, MIN_LEN, MAX_LEN>
    {
        fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
            f.write_str(&self.0.to_hex().to_uppercase())
        }
    }

    /// The number of bytes must fit the confinement. Otherwise the result is
    /// [`hex::Error::InvalidLength`] with the violated bound and the number of
    /// bytes in the data.
    impl<A: Allocator + Default, const MIN_LEN: usize, const MAX_LEN: usize> FromHex
        for Confined<Vec<u8, A>, MIN_LEN, MAX_LEN>
    {
        fn from_byte_iter<I>(iter: I) -> Result<Self, hex::Error>
        where
            I: Iterator<Item = Result<u8, hex::Error>> + ExactSizeIterator + DoubleEndedIterator,
        {
            // The length is checked before decoding, so the data which do not
            // fit are not allocated, and after it, since an iterator may
            // report a wrong length.
            check_len::<MIN_LEN, MAX_LEN>(iter.len())?;
            let mut bytes = Vec::<u8, A>::with_capacity_in(iter.len(), A::default());
            for byte in iter {
                bytes.push(byte?);
            }
            check_len::<MIN_LEN, MAX_LEN>(bytes.len())?;
            Ok(Self(bytes))
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
