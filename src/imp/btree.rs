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

use crate::{Confined, Error};

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
    /// # Errors
    ///
    /// Errors if the maximum confinement is exceeded after the operation.
    pub fn append<const MIN_LEN2: usize, const MAX_LEN2: usize>(
        &mut self,
        other: &mut Confined<BTreeMap<K, V>, MIN_LEN2, MAX_LEN2>,
    ) -> Result<(), Error> {
        let new_len = self.len() + other.len();
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
