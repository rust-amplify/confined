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

use crate::{Confined, Error, ZERO};

impl<const MIN_LEN: usize, const MAX_LEN: usize> TryFrom<&str>
    for Confined<String, MIN_LEN, MAX_LEN>
{
    type Error = Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::try_from(value.to_owned())
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
    pub fn remove(&mut self, index: usize) -> Result<char, Error> {
        self.check_boundary(index)?;
        if !self.0.is_char_boundary(index) {
            return Err(Error::OutOfBoundary {
                index,
                len: self.len(),
            });
        }
        self.check_undersize()?;
        Ok(self.0.remove(index))
    }
}
