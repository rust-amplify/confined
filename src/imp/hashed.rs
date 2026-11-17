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

use std::collections::{HashMap, HashSet, hash_map};
use std::hash::{BuildHasher, Hash};

use crate::{Confined, Entry, Error, ItemLen, ViaCollection, ZERO};

impl<T: Hash + Eq, S: BuildHasher + Default> ItemLen for HashSet<T, S> {
    type Item = T;
    type Access = ViaCollection;
}

impl<K: Hash + Eq, V, S: BuildHasher + Default> ItemLen for HashMap<K, V, S> {
    type Item = (K, V);
    type Access = ViaCollection;
}

impl<'a, K: Hash + Eq, V, S: BuildHasher + Default> Entry<'a, HashMap<K, V, S>> {
    /// Ensures a value is in the entry by inserting, if empty, the result of
    /// the default function, which gets a reference to the key of the entry.
    /// Returns a mutable reference to the value in the entry.
    pub fn or_insert_with_key<F: FnOnce(&K) -> V>(self, default: F) -> &'a mut V {
        self.0.or_insert_with_key(default)
    }
}

impl<T: Hash + Eq, S: BuildHasher + Default, const MIN_LEN: usize, const MAX_LEN: usize>
    Confined<HashSet<T, S>, MIN_LEN, MAX_LEN>
{
    /// Removes an element from the set. Errors if the index exceeds the number
    /// of elements in the set, of if the new collection length will be less
    /// than the confinement requirement. Returns if the element was present
    /// in the set.
    pub fn remove(&mut self, elem: &T) -> Result<bool, Error> {
        self.remove_from_set(elem, |set, elem| set.remove(elem))
    }

    /// Removes an element from the set. Errors if the index exceeds the number
    /// of elements in the set, of if the new collection length will be less
    /// than the confinement requirement. Returns the removed element
    /// otherwise.
    pub fn take(&mut self, elem: &T) -> Result<Option<T>, Error> {
        self.take_from_set(elem, |set, elem| set.take(elem))
    }
}
impl<K: Hash + Eq, V, S: BuildHasher + Default, const MIN_LEN: usize, const MAX_LEN: usize>
    Confined<HashMap<K, V, S>, MIN_LEN, MAX_LEN>
{
    /// Removes an element from the map. Errors if the index exceeds the number
    /// of elements in the map, of if the new collection length will be less
    /// than the confinement requirement. Returns the removed value
    /// otherwise.
    pub fn remove(&mut self, key: &K) -> Result<Option<V>, Error> {
        self.remove_from_map(key, |map, key| map.remove(key))
    }

    /// Removes a key from the map, returning the stored key and value if the
    /// key was previously in the map.
    ///
    /// # Errors
    ///
    /// Errors if the minimum confinement is not met after the removal.
    pub fn remove_entry(&mut self, key: &K) -> Result<Option<(K, V)>, Error> {
        if !self.0.contains_key(key) {
            return Ok(None);
        }
        self.check_undersize()?;
        Ok(self.0.remove_entry(key))
    }

    /// Reserves capacity for `additional` more elements to be inserted
    /// in the `HashMap`. The collection may reserve more space to speculatively
    /// avoid frequent reallocations. After calling `reserve`, capacity will be
    /// greater than or equal to `min(self.len() + additional, MAX_LEN)`. Does
    /// nothing if capacity is already sufficient.
    ///
    /// The space is never reserved for more elements than the confinement
    /// allows to insert.
    ///
    /// # Panics
    ///
    /// Panics if the new allocation size overflows `usize`.
    pub fn reserve(&mut self, additional: usize) {
        let room = MAX_LEN.saturating_sub(self.len());
        self.0.reserve(additional.min(room))
    }

    /// Shrinks the capacity of the map as much as possible. It will drop
    /// down as much as possible while maintaining the internal rules
    /// and possibly leaving some space in accordance with the resize policy.
    pub fn shrink_to_fit(&mut self) {
        self.0.shrink_to_fit();
    }

    /// Shrinks the capacity of the map with a lower bound.
    ///
    /// The capacity will remain at least as large as both the length
    /// and the supplied value.
    ///
    /// If the current capacity is less than the lower bound, this is a no-op.
    pub fn shrink_to(&mut self, min_capacity: usize) {
        self.0.shrink_to(min_capacity);
    }

    /// Creates a consuming iterator visiting all the keys in arbitrary order.
    /// The map cannot be used after calling this.
    /// The iterator element type is `K`.
    pub fn into_keys(self) -> hash_map::IntoKeys<K, V> {
        self.0.into_keys()
    }

    /// Creates a consuming iterator visiting all the values in arbitrary order.
    /// The map cannot be used after calling this.
    /// The iterator element type is `V`.
    pub fn into_values(self) -> hash_map::IntoValues<K, V> {
        self.0.into_values()
    }
}

impl<K: Hash + Eq, V, S: BuildHasher + Default, const MAX_LEN: usize>
    Confined<HashMap<K, V, S>, ZERO, MAX_LEN>
{
    /// Clears the map, returning all key-value pairs as an iterator. Keeps the
    /// allocated memory for reuse.
    pub fn drain(&mut self) -> hash_map::Drain<'_, K, V> {
        self.0.drain()
    }
}
