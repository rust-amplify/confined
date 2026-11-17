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

use alloc::borrow::ToOwned;
use alloc::string::String;
use core::str::FromStr;

use crate::{Confined, Error, ItemLen, ZERO};

/// The length of a string is measured in bytes, and a `char` takes from one to
/// four bytes in the UTF-8 encoding.
impl ItemLen for String {
    const MAX_ITEM_LEN: usize = 4;

    fn item_len(item: &char) -> usize {
        item.len_utf8()
    }
}

impl<const MIN_LEN: usize, const MAX_LEN: usize> TryFrom<&str>
    for Confined<String, MIN_LEN, MAX_LEN>
{
    type Error = Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::try_from(value.to_owned())
    }
}

impl<const MIN_LEN: usize, const MAX_LEN: usize> FromStr for Confined<String, MIN_LEN, MAX_LEN> {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        <Self as TryFrom<&str>>::try_from(s)
    }
}

impl<const MAX_LEN: usize> Confined<String, ZERO, MAX_LEN> {
    /// Removes the last character from a string and returns it, or [`None`] if
    /// it is empty.
    pub fn pop(&mut self) -> Option<char> {
        self.0.pop()
    }
}

impl<const MIN_LEN: usize, const MAX_LEN: usize> Confined<String, MIN_LEN, MAX_LEN> {
    /// Removes a single character from the confined string, unless the string
    /// doesn't shorten more than the confinement requirement. Errors
    /// otherwise.
    ///
    /// The `index` is a position in bytes, and it must point to the first
    /// byte of a character. The string is shortened by the number of bytes in
    /// the UTF-8 encoding of the removed character, which may be up to four.
    pub fn remove(&mut self, index: usize) -> Result<char, Error> {
        self.check_boundary(index)?;
        let len = self.len();
        let Some(ch) = self.0.get(index..).and_then(|tail| tail.chars().next()) else {
            return Err(Error::OutOfBoundary { index, len });
        };
        if len - ch.len_utf8() < MIN_LEN {
            return Err(Error::Undersize {
                len,
                min_len: MIN_LEN,
            });
        }
        Ok(self.0.remove(index))
    }
}
