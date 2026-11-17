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

use alloc::collections::LinkedList;

use crate::{Confined, Error, ItemLen, ViaCollection, ZERO};

impl<T> ItemLen for LinkedList<T> {
    type Item = T;
    type Access = ViaCollection;
}

impl<T, const MIN_LEN: usize, const MAX_LEN: usize> Confined<LinkedList<T>, MIN_LEN, MAX_LEN> {
    /// Prepends an element to the list. Errors if the new list length exceeds
    /// the confinement.
    pub fn push_front(&mut self, item: T) -> Result<(), Error> {
        self.check_oversize()?;
        self.0.push_front(item);
        Ok(())
    }

    /// Appends an element to the list. Errors if the new list length exceeds
    /// the confinement.
    pub fn push_back(&mut self, item: T) -> Result<(), Error> {
        self.check_oversize()?;
        self.0.push_back(item);
        Ok(())
    }

    /// Removes the element at `index`. Errors if the index is outside the list
    /// or if the removal would violate the lower bound.
    pub fn remove(&mut self, index: usize) -> Result<T, Error> {
        self.check_boundary(index)?;
        self.check_undersize()?;
        let mut tail = self.0.split_off(index);
        let removed = tail
            .pop_front()
            .expect("index was checked against the length");
        self.0.append(&mut tail);
        Ok(removed)
    }
}

impl<T, const MAX_LEN: usize> Confined<LinkedList<T>, ZERO, MAX_LEN> {
    /// Removes and returns the first element, or `None` if the list is empty.
    ///
    /// Lists with a lower bound are shortened with [`Self::remove`] instead:
    ///
    /// ```compile_fail,E0596
    /// # use confined::NonEmptyList;
    /// let mut list = NonEmptyList::<u8>::with(1);
    /// list.pop_front();
    /// ```
    pub fn pop_front(&mut self) -> Option<T> {
        self.0.pop_front()
    }

    /// Removes and returns the last element, or `None` if the list is empty.
    ///
    /// ```compile_fail,E0596
    /// # use confined::NonEmptyList;
    /// let mut list = NonEmptyList::<u8>::with(1);
    /// list.pop_back();
    /// ```
    pub fn pop_back(&mut self) -> Option<T> {
        self.0.pop_back()
    }
}
