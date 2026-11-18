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

use core::hash::{BuildHasher, Hash};

use collections::{CollectionCapacity, CollectionMap, CollectionSet};
use indexmap::{IndexMap, IndexSet, map};

use crate::{Confined, Entry, Error, ItemLen};

impl<T: Eq + Hash, S: BuildHasher> ItemLen for IndexSet<T, S> {
    fn item_growth(&self, item: &T) -> usize {
        usize::from(!CollectionSet::contains(self, item))
    }
    fn try_push(&mut self, item: T) -> Result<(), T> {
        if self.contains(&item) {
            Ok(())
        } else {
            CollectionCapacity::try_push(self, item)
        }
    }
}

impl<K: Eq + Hash, V, S: BuildHasher> ItemLen for IndexMap<K, V, S> {
    fn item_growth(&self, item: &(K, V)) -> usize {
        usize::from(!CollectionMap::contains_key(self, &item.0))
    }
    fn try_push(&mut self, item: (K, V)) -> Result<(), (K, V)> {
        if let Some(value) = self.get_mut(&item.0) {
            *value = item.1;
            Ok(())
        } else {
            CollectionCapacity::try_push(self, item)
        }
    }

    fn prepare_entry(&mut self) -> Result<(), Error> {
        CollectionCapacity::try_reserve(self, 1)
            .map_err(|_| Error::CapacityExceeded { len: self.len() })
    }
}

impl<'a, K: Eq + Hash, V, S: BuildHasher> Entry<'a, IndexMap<K, V, S>> {
    /// Ensures a value is in the entry by inserting, if empty, the result of
    /// the default function, which gets a reference to the key of the entry.
    /// Returns a mutable reference to the value in the entry.
    pub fn or_insert_with_key<F: FnOnce(&K) -> V>(self, default: F) -> &'a mut V {
        self.0.or_insert_with_key(default)
    }
}

impl<T: Eq + Hash, S: BuildHasher, const MIN_LEN: usize, const MAX_LEN: usize>
    Confined<IndexSet<T, S>, MIN_LEN, MAX_LEN>
{
    /// Removes a value from the set, returning whether the value was at the
    /// set previously.
    ///
    /// # Errors
    ///
    /// Errors if the minimum confinement is not met after the removal.
    pub fn remove(&mut self, elem: &T) -> Result<bool, Error> {
        self.remove_from_set(elem, |set, elem| set.shift_remove(elem))
    }

    /// Removes and returns the value in the set, if any, that is equal to the
    /// given one.
    ///
    /// # Errors
    ///
    /// Errors if the minimum confinement is not met after the removal.
    pub fn take(&mut self, elem: &T) -> Result<Option<T>, Error> {
        self.take_from_set(elem, |set, elem| set.shift_take(elem))
    }
}

impl<K: Eq + Hash, V, S: BuildHasher, const MIN_LEN: usize, const MAX_LEN: usize>
    Confined<IndexMap<K, V, S>, MIN_LEN, MAX_LEN>
{
    /// Removes a key from the map, returning the value at the key if the key
    /// was previously in the map.
    ///
    /// # Errors
    ///
    /// Errors if the minimum confinement is not met after the removal.
    pub fn remove(&mut self, key: &K) -> Result<Option<V>, Error> {
        self.remove_from_map(key, |map, key| map.shift_remove(key))
    }

    /// Creates a consuming iterator visiting all the keys in the order of
    /// their insertion. The map cannot be used after calling this. The
    /// iterator element type is `K`.
    pub fn into_keys(self) -> map::IntoKeys<K, V> {
        self.0.into_keys()
    }

    /// Creates a consuming iterator visiting all the values in the order of
    /// the insertion of their keys. The map cannot be used after calling
    /// this. The iterator element type is `V`.
    pub fn into_values(self) -> map::IntoValues<K, V> {
        self.0.into_values()
    }
}
