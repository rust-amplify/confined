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

use core::alloc::Allocator;
use core::hash::{BuildHasher, Hash};

use collections::hashbrown::{HashMap, HashSet};
use collections::{CollectionCapacity, CollectionMap, CollectionSet};

use crate::{Confined, Entry, Error, ItemLen};

impl<T: Eq + Hash, S: BuildHasher, A: Allocator> ItemLen for HashSet<T, S, A> {
    fn item_growth(&self, item: &T) -> usize {
        usize::from(!self.contains(item))
    }

    fn try_push(&mut self, item: T) -> Result<(), T> {
        // The native `insert` reserves room before it looks the item up, so
        // it is not called for an item which the set already holds.
        if self.contains(&item) {
            Ok(())
        } else {
            CollectionCapacity::try_push(self, item)
        }
    }
}

impl<K: Eq + Hash, V, S: BuildHasher, A: Allocator> ItemLen for HashMap<K, V, S, A> {
    fn item_growth(&self, item: &(K, V)) -> usize {
        usize::from(!self.contains_key(&item.0))
    }

    fn try_push(&mut self, item: (K, V)) -> Result<(), (K, V)> {
        // The native `insert` reserves room before it looks the key up, so
        // the value of an existing key is replaced in place.
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

impl<'a, K: Eq + Hash, V, S: BuildHasher, A: Allocator> Entry<'a, HashMap<K, V, S, A>> {
    /// Inserts a value derived from the key if the entry is vacant.
    pub fn or_insert_with_key<F: FnOnce(&K) -> V>(self, default: F) -> &'a mut V {
        self.0.or_insert_with_key(default)
    }
}

impl<T: Eq + Hash, S: BuildHasher, A: Allocator, const MIN: usize, const MAX: usize>
    Confined<HashSet<T, S, A>, MIN, MAX>
{
    /// Removes an item from the set while preserving the lower bound.
    pub fn remove(&mut self, item: &T) -> Result<bool, Error> {
        self.remove_from_set(item, CollectionSet::remove)
    }

    /// Removes and returns an item from the set while preserving the lower
    /// bound.
    pub fn take(&mut self, item: &T) -> Result<Option<T>, Error> {
        self.take_from_set(item, CollectionSet::take)
    }
}

impl<K: Eq + Hash, V, S: BuildHasher, A: Allocator, const MIN: usize, const MAX: usize>
    Confined<HashMap<K, V, S, A>, MIN, MAX>
{
    /// Removes a key and returns its value while preserving the lower bound.
    pub fn remove(&mut self, key: &K) -> Result<Option<V>, Error> {
        self.remove_from_map(key, CollectionMap::remove)
    }
}
