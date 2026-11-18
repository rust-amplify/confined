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

use alloc::string::String;
use core::str::FromStr;

use collections::{CollectionBytes, CollectionCapacity};

use crate::{Confined, Error, ItemLen, ZERO};

/// The length of a string is measured in bytes, and a `char` takes from one to
/// four bytes in the UTF-8 encoding.
impl ItemLen for String {
    const MAX_ITEM_LEN: usize = char::MAX_LEN_UTF8;

    fn item_len(item: &char) -> usize {
        <String as CollectionBytes>::item_byte_len(item)
    }

    fn confined_len(&self) -> usize {
        <String as CollectionBytes>::byte_len(self)
    }

    fn try_push(&mut self, item: char) -> Result<(), char> {
        CollectionCapacity::try_push(self, item)
    }
}

impl<const MIN_LEN: usize, const MAX_LEN: usize> TryFrom<&str>
    for Confined<String, MIN_LEN, MAX_LEN>
{
    type Error = Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::check_len(value.len())?;
        let mut string = String::new();
        string
            .try_reserve_exact(value.len())
            .map_err(|_| Error::CapacityExceeded { len: 0 })?;
        string.push_str(value);
        Ok(Self(string))
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
    ///
    /// ```
    /// # use confined::Confined;
    /// let mut string: Confined<String, 0, 3> = "ab".parse().unwrap();
    /// assert_eq!(string.pop(), Some('b'));
    /// ```
    ///
    /// A string with a lower bound has no such method, so the call fails to
    /// compile. Such a string is shortened with `remove`, which checks the
    /// bound:
    ///
    /// ```compile_fail,E0596
    /// # use confined::Confined;
    /// let mut string: Confined<String, 1, 3> = "ab".parse().unwrap();
    /// assert_eq!(string.pop(), Some('b'));
    /// ```
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
