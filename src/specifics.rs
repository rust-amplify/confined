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

use core::ops::{Index, IndexMut};

use amplify_num::u24;
use collections::{Collection, KeyedCollection};

use super::{Confined, ItemLen, ONE, U8, U16, U24, U32, ZERO};
use crate::base::{Bounds, HoldsItem};

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
    C: Default + ItemLen,
{
    /// Constructs a confinement with a collection made of a single required
    /// element.
    ///
    /// The upper bound of the confinement must leave room for an element of
    /// any size. For most of the collections it means `MAX_LEN >= 1`. The
    /// length of a [`String`](alloc::string::String) is measured in bytes, and
    /// a single `char` may take up to four of them, so a string needs
    /// `MAX_LEN >= 4`.
    ///
    /// A type which doesn't meet this requirement fails to build. The failure
    /// is reported by `cargo build` and `cargo test`, but not by
    /// `cargo check`.
    ///
    /// ```
    /// # use confined::Confined;
    /// let vec = Confined::<Vec<u8>, 1, 1>::with(7);
    /// assert_eq!(vec.as_slice(), &[7]);
    /// ```
    ///
    /// ```compile_fail,E0080
    /// # use confined::Confined;
    /// let vec = Confined::<Vec<u8>, 1, 0>::with(7);
    /// assert_eq!(vec.as_slice(), &[7]);
    /// ```
    ///
    /// ```
    /// # use confined::Confined;
    /// let s = Confined::<String, 1, 4>::with('a');
    /// assert_eq!(s.as_str(), "a");
    /// ```
    ///
    /// ```compile_fail,E0080
    /// # use confined::Confined;
    /// let s = Confined::<String, 1, 2>::with('a');
    /// assert_eq!(s.as_str(), "a");
    /// ```
    pub fn with(elem: C::Item) -> Self {
        let () = Bounds::<ONE, MAX_LEN>::VALID;
        let () = HoldsItem::<C, MAX_LEN>::VALID;
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

// TODO: Do it with `const { assert!(MIN_LEN > 0) };` inline instead of using just `ONCE`
impl<C, const MAX_LEN: usize> Confined<C, ONE, MAX_LEN>
where
    C: Collection + IndexMut<usize, Output = C::Item>,
{
    /// Returns a mutable reference to the first element.
    pub fn first_mut(&mut self) -> &mut C::Item {
        self.0.index_mut(0)
    }

    /// Returns a mutable reference to the last element.
    pub fn last_mut(&mut self) -> &mut C::Item {
        let last = self.len() - 1;
        self.0.index_mut(last)
    }
}

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
    ///
    /// A type which upper bound is zero can't hold the pair and fails to
    /// build. The failure is reported by `cargo build` and `cargo test`, but
    /// not by `cargo check`.
    ///
    /// ```
    /// # use std::collections::BTreeMap;
    /// # use confined::Confined;
    /// let map = Confined::<BTreeMap<u8, char>, 1, 1>::with_key_value(7, 'a');
    /// assert_eq!(map.len(), 1);
    /// ```
    ///
    /// ```compile_fail,E0080
    /// # use std::collections::BTreeMap;
    /// # use confined::Confined;
    /// let map = Confined::<BTreeMap<u8, char>, 1, 0>::with_key_value(7, 'a');
    /// assert_eq!(map.len(), 1);
    /// ```
    pub fn with_key_value(key: C::Key, value: C::Value) -> Self {
        let () = Bounds::<ONE, MAX_LEN>::VALID;
        let mut c = C::default();
        c.insert(key, value);
        Self(c)
    }
}
