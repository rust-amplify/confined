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

use alloc::collections::{BTreeMap, BTreeSet, btree_map};
use core::hash::Hash;
use core::ops::RangeBounds;

use crate::{Confined, Entry, Error, ItemLen, ZERO};

impl<T: Ord> ItemLen for BTreeSet<T> {}

impl<K: Ord + Hash, V> ItemLen for BTreeMap<K, V> {}

impl<'a, K: Ord + Hash, V> Entry<'a, BTreeMap<K, V>> {
    /// Returns a reference to this entry's key.
    pub fn key(&self) -> &K {
        self.0.key()
    }

    /// Ensures a value is in the entry by inserting the default if empty, and
    /// returns a mutable reference to the value in the entry.
    pub fn or_insert(self, default: V) -> &'a mut V {
        self.0.or_insert(default)
    }

    /// Ensures a value is in the entry by inserting the result of the default
    /// function if empty, and returns a mutable reference to the value in the
    /// entry.
    pub fn or_insert_with<F: FnOnce() -> V>(self, default: F) -> &'a mut V {
        self.0.or_insert_with(default)
    }

    /// Ensures a value is in the entry by inserting, if empty, the result of
    /// the default function, which gets a reference to the key of the entry.
    /// Returns a mutable reference to the value in the entry.
    pub fn or_insert_with_key<F: FnOnce(&K) -> V>(self, default: F) -> &'a mut V {
        self.0.or_insert_with_key(default)
    }

    /// Ensures a value is in the entry by inserting the default value if
    /// empty, and returns a mutable reference to the value in the entry.
    pub fn or_default(self) -> &'a mut V
    where
        V: Default,
    {
        self.0.or_default()
    }

    /// Provides in-place mutable access to an occupied entry before any
    /// potential inserts into the map.
    pub fn and_modify<F: FnOnce(&mut V)>(self, f: F) -> Self {
        Entry(self.0.and_modify(f))
    }
}

impl<T: Ord, const MIN_LEN: usize, const MAX_LEN: usize> Confined<BTreeSet<T>, MIN_LEN, MAX_LEN> {
    /// Removes an element from the set. Errors if the index exceeds the number
    /// of elements in the set, of if the new collection length will be less
    /// than the confinement requirement. Returns if the element was present
    /// in the set.
    pub fn remove(&mut self, elem: &T) -> Result<bool, Error> {
        if !self.0.contains(elem) {
            return Ok(false);
        }
        self.check_undersize()?;
        Ok(self.0.remove(elem))
    }

    /// Removes an element from the set. Errors if the index exceeds the number
    /// of elements in the set, of if the new collection length will be less
    /// than the confinement requirement. Returns the removed element
    /// otherwise.
    pub fn take(&mut self, elem: &T) -> Result<Option<T>, Error> {
        if !self.0.contains(elem) {
            return Ok(None);
        }
        self.check_undersize()?;
        Ok(self.0.take(elem))
    }
}
impl<K: Ord + Hash, V, const MIN_LEN: usize, const MAX_LEN: usize>
    Confined<BTreeMap<K, V>, MIN_LEN, MAX_LEN>
{
    /// Removes an element from the map. Errors if the index exceeds the number
    /// of elements in the map, of if the new collection length will be less
    /// than the confinement requirement. Returns the removed value
    /// otherwise.
    pub fn remove(&mut self, key: &K) -> Result<Option<V>, Error> {
        if !self.0.contains_key(key) {
            return Ok(None);
        }
        self.check_undersize()?;
        Ok(self.0.remove(key))
    }

    /// Removes and returns the first element from the map.
    ///
    /// # Errors
    ///
    /// Errors if the minimum confinement is not met after the removal.
    pub fn pop_first(&mut self) -> Result<Option<(K, V)>, Error> {
        if self.is_empty() {
            return Ok(None);
        }
        self.check_undersize()?;
        Ok(self.0.pop_first())
    }

    /// Removes and returns the last element from the map.
    ///
    /// # Errors
    ///
    /// Errors if the minimum confinement is not met after the removal.
    pub fn pop_last(&mut self) -> Result<Option<(K, V)>, Error> {
        if self.is_empty() {
            return Ok(None);
        }
        self.check_undersize()?;
        Ok(self.0.pop_last())
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

    /// Moves all elements from `other` into `self`, leaving `other` empty.
    ///
    /// If a key from `other` is already present in `self`, the respective
    /// value from `self` is overwritten with the respective value from
    /// `other`. Such a key doesn't add to the length of `self`.
    ///
    /// The `other` map is left empty, so it must not have a lower bound:
    ///
    /// ```
    /// # use std::collections::BTreeMap;
    /// # use confined::Confined;
    /// let mut map = Confined::<BTreeMap<u8, u8>, 1, 3>::with_key_value(1, 10);
    /// let mut other = Confined::<BTreeMap<u8, u8>, 0, 3>::try_from_iter([(2, 20)]).unwrap();
    /// map.append(&mut other).unwrap();
    /// assert_eq!(map.len(), 2);
    /// ```
    ///
    /// ```compile_fail,E0308
    /// # use std::collections::BTreeMap;
    /// # use confined::Confined;
    /// let mut map = Confined::<BTreeMap<u8, u8>, 1, 3>::with_key_value(1, 10);
    /// let mut other = Confined::<BTreeMap<u8, u8>, 1, 3>::try_from_iter([(2, 20)]).unwrap();
    /// map.append(&mut other).unwrap();
    /// assert_eq!(map.len(), 2);
    /// ```
    ///
    /// # Errors
    ///
    /// Errors if the maximum confinement is exceeded after the operation.
    /// Both maps are left unchanged in this case.
    pub fn append<const MAX_LEN2: usize>(
        &mut self,
        other: &mut Confined<BTreeMap<K, V>, ZERO, MAX_LEN2>,
    ) -> Result<(), Error> {
        let mut new_len = self.len().saturating_add(other.len());
        if new_len > MAX_LEN {
            // The keys present in both maps do not add to the length
            new_len -= other
                .0
                .keys()
                .filter(|key| self.0.contains_key(key))
                .count();
        }
        if new_len > MAX_LEN {
            return Err(Error::Oversize {
                len: new_len,
                max_len: MAX_LEN,
            });
        }
        self.0.append(&mut other.0);
        Ok(())
    }

    /// Splits the collection into two at the given key. Returns everything
    /// after the given key, including the key.
    ///
    /// # Errors
    ///
    /// Errors if the minimum confinement is not met for either of the
    /// collections.
    pub fn split_off<const MIN_LEN2: usize, const MAX_LEN2: usize>(
        &mut self,
        key: &K,
    ) -> Result<Confined<BTreeMap<K, V>, MIN_LEN2, MAX_LEN2>, Error> {
        let mut other = self.0.split_off(key);
        let len1 = self.0.len();
        let len2 = other.len();

        if len1 < MIN_LEN {
            self.0.append(&mut other);
            return Err(Error::Undersize {
                len: len1,
                min_len: MIN_LEN,
            });
        }

        if len2 < MIN_LEN2 {
            self.0.append(&mut other);
            return Err(Error::Undersize {
                len: len2,
                min_len: MIN_LEN2,
            });
        }

        if len2 > MAX_LEN2 {
            self.0.append(&mut other);
            return Err(Error::Oversize {
                len: len2,
                max_len: MAX_LEN2,
            });
        }

        Ok(Confined(other))
    }

    /// Gets a mutable iterator over the specified range.
    pub fn range_mut<R>(&mut self, range: R) -> btree_map::RangeMut<'_, K, V>
    where
        R: RangeBounds<K>,
    {
        self.0.range_mut(range)
    }

    /// Creates a consuming iterator visiting all the keys in arbitrary order.
    /// The map cannot be used after calling this.
    /// The iterator element type is `K`.
    pub fn into_keys(self) -> btree_map::IntoKeys<K, V> {
        self.0.into_keys()
    }

    /// Creates a consuming iterator visiting all the values in arbitrary order.
    /// The map cannot be used after calling this.
    /// The iterator element type is `V`.
    pub fn into_values(self) -> btree_map::IntoValues<K, V> {
        self.0.into_values()
    }
}
