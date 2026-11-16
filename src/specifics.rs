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

use core::ops::Index;

use amplify_num::u24;
use collections::{Collection, KeyedCollection};

use super::{Confined, ONE, U8, U16, U24, U32, ZERO};

impl<C: Collection, const MAX_LEN: usize> Confined<C, ZERO, MAX_LEN> {
    /// Removes all elements from the confined collection.
    pub fn clear(&mut self) {
        self.0.clear()
    }
}

impl<C: Collection, const MAX_LEN: usize> Confined<C, ZERO, MAX_LEN>
where
    C: Default,
{
    /// Constructs a new confinement containing no elements.
    pub fn new() -> Self {
        Self::default()
    }

    /// Constructs a new confinement containing no elements, but with a
    /// pre-allocated storage for the `capacity` of elements.
    pub fn with_capacity(capacity: usize) -> Self {
        Self(C::with_capacity(capacity))
    }
}

impl<C: Collection, const MAX_LEN: usize> Default for Confined<C, ZERO, MAX_LEN>
where
    C: Default,
{
    fn default() -> Self {
        Self(C::default())
    }
}

impl<C: Collection, const MAX_LEN: usize> Confined<C, ONE, MAX_LEN>
where
    C: Default,
{
    /// Constructs a confinement with a collection made of a single required
    /// element.
    pub fn with(elem: C::Item) -> Self {
        let mut c = C::default();
        c.push(elem);
        Self(c)
    }
}

impl<C, const MAX_LEN: usize> Confined<C, ONE, MAX_LEN>
where
    C: Collection + Index<usize, Output = C::Item>,
{
    /// Returns the first element.
    pub fn first(&self) -> &C::Item {
        self.0.index(0)
    }

    /// Returns the last element.
    pub fn last(&self) -> &C::Item {
        self.0.index(self.len() - 1)
    }
}

// TODO: Add first_mut and last_mut

impl<C: Collection, const MIN_LEN: usize> Confined<C, MIN_LEN, U8> {
    /// Returns the number of elements in the confined collection as `u8`. The
    /// confinement guarantees that the collection length can't exceed
    /// `u8::MAX`.
    pub fn len_u8(&self) -> u8 {
        self.len() as u8
    }
}

impl<C: Collection, const MIN_LEN: usize> Confined<C, MIN_LEN, U16> {
    /// Returns the number of elements in the confined collection as `u16`. The
    /// confinement guarantees that the collection length can't exceed
    /// `u16::MAX`.
    pub fn len_u16(&self) -> u16 {
        self.len() as u16
    }
}

impl<C: Collection, const MIN_LEN: usize> Confined<C, MIN_LEN, U24> {
    /// Returns the number of elements in the confined collection as `u24`. The
    /// confinement guarantees that the collection length can't exceed
    /// `u24::MAX`.
    pub fn len_u24(&self) -> u24 {
        u24::try_from(self.len() as u32).expect("confinement broken")
    }
}

impl<C: Collection, const MIN_LEN: usize> Confined<C, MIN_LEN, U32> {
    /// Returns the number of elements in the confined collection as `u32`. The
    /// confinement guarantees that the collection length can't exceed
    /// `u32::MAX`.
    pub fn len_u32(&self) -> u32 {
        self.len() as u32
    }
}

impl<C: KeyedCollection, const MAX_LEN: usize> Confined<C, ZERO, MAX_LEN> {
    /// Retains only the elements specified by the predicate.
    ///
    /// In other words, remove all pairs `(k, v)` for which `f(&k, &mut v)`
    /// returns `false`. The elements are visited in unsorted (and
    /// unspecified) order.
    pub fn retain(&mut self, f: impl FnMut(&C::Key, &mut C::Value) -> bool) {
        self.0.retain(f)
    }
}

impl<C: KeyedCollection, const MAX_LEN: usize> Confined<C, ONE, MAX_LEN>
where
    C: Default,
{
    /// Constructs a confinement with a collection made of a single required
    /// key-value pair.
    pub fn with_key_value(key: C::Key, value: C::Value) -> Self {
        let mut c = C::default();
        c.insert(key, value);
        Self(c)
    }
}
