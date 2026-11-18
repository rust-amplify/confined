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
use core::ops::RangeBounds;

use collections::{CollectionMap, CollectionSet};

use crate::{Confined, Entry, Error, ItemLen, ZERO};

impl<T: Ord> ItemLen for BTreeSet<T> {
    fn item_growth(&self, item: &T) -> usize {
        usize::from(!CollectionSet::contains(self, item))
    }
}

impl<K: Ord, V> ItemLen for BTreeMap<K, V> {
    fn item_growth(&self, item: &(K, V)) -> usize {
        usize::from(!CollectionMap::contains_key(self, &item.0))
    }
}

impl<'a, K: Ord, V> Entry<'a, BTreeMap<K, V>> {
    /// Ensures a value is in the entry by inserting, if empty, the result of
    /// the default function, which gets a reference to the key of the entry.
    /// Returns a mutable reference to the value in the entry.
    pub fn or_insert_with_key<F: FnOnce(&K) -> V>(self, default: F) -> &'a mut V {
        self.0.or_insert_with_key(default)
    }
}

impl<T: Ord, const MIN_LEN: usize, const MAX_LEN: usize> Confined<BTreeSet<T>, MIN_LEN, MAX_LEN> {
    /// Removes an element from the set, returning whether it was present.
    /// Removing an element which is not in the set is not an error, even at
    /// the lower bound.
    ///
    /// # Errors
    ///
    /// Errors if the element is present and the set has no more elements than
    /// the lower bound of the confinement. The set is left unchanged.
    pub fn remove(&mut self, elem: &T) -> Result<bool, Error> {
        self.remove_from_set(elem, |set, elem| set.remove(elem))
    }

    /// Removes an element from the set and returns it, or `None` if it was
    /// not present. Taking an element which is not in the set is not an
    /// error, even at the lower bound.
    ///
    /// # Errors
    ///
    /// Errors if the element is present and the set has no more elements than
    /// the lower bound of the confinement. The set is left unchanged.
    pub fn take(&mut self, elem: &T) -> Result<Option<T>, Error> {
        self.take_from_set(elem, |set, elem| set.take(elem))
    }
}
impl<K: Ord, V, const MIN_LEN: usize, const MAX_LEN: usize>
    Confined<BTreeMap<K, V>, MIN_LEN, MAX_LEN>
{
    /// Removes a key from the map and returns its value, or `None` if the key
    /// was not present. Removing a key which is not in the map is not an
    /// error, even at the lower bound.
    ///
    /// # Errors
    ///
    /// Errors if the key is present and the map has no more entries than the
    /// lower bound of the confinement. The map is left unchanged.
    pub fn remove(&mut self, key: &K) -> Result<Option<V>, Error> {
        self.remove_from_map(key, |map, key| map.remove(key))
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
    /// `other`. Such a key doesn't add to the length of `self`. The key
    /// stored in `self` is kept and the equal key from `other` is dropped,
    /// as with `insert`.
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
    ///
    /// # Panics
    ///
    /// A panic of a comparison or of a destructor is propagated. The entries
    /// moved before it stay in `self`, and the entries not moved yet stay in
    /// `other`. The entry which was being moved is dropped if a comparison
    /// panicked; it is already in `self` if the panic came from the
    /// destructor of the value it replaced or of its duplicate key. The map
    /// never falls below its lower bound: the entries are moved one by one,
    /// and a value is replaced before the previous one is dropped. The
    /// native `append` is not used: on Rust 1.100 it leaves the map without
    /// the entry of a shared key when the destructor of its previous value
    /// panics.
    pub fn append<const MAX_LEN2: usize>(
        &mut self,
        other: &mut Confined<BTreeMap<K, V>, ZERO, MAX_LEN2>,
    ) -> Result<(), Error> {
        let room = MAX_LEN.saturating_sub(self.len());
        let growth = if other.len() <= room {
            other.len()
        } else {
            other
                .0
                .keys()
                .filter(|key| !self.0.contains_key(key))
                .count()
        };
        if growth > room {
            return Err(Error::Oversize {
                len: self.len().saturating_add(growth),
                max_len: MAX_LEN,
            });
        }
        if self.0.is_empty() {
            core::mem::swap(&mut self.0, &mut other.0);
            return Ok(());
        }
        while let Some((key, value)) = other.0.pop_first() {
            self.0.insert(key, value);
        }
        Ok(())
    }

    /// Splits the collection into two at the given key. Returns everything
    /// after the given key, including the key.
    ///
    /// # Errors
    ///
    /// Errors if the minimum confinement is not met for either of the
    /// collections, or the right-hand collection exceeds its maximum.
    ///
    /// Keys and values are cloned before any originals are removed, so a
    /// comparison or cloning panic leaves the original map unchanged. This
    /// avoids the panic-unsafe native split on Rust 1.100. Dropping originals
    /// may still panic, but cannot take the map below its minimum length.
    ///
    /// The bounds are checked for the map which is really built from the
    /// clones, and not only for the number of the entries to split off. They
    /// differ when `Eq` of the keys is inconsistent with their `Ord`: the
    /// clones which compare equal with `==` are merged into a single entry.
    /// The original map is left unchanged if the built map is out of the
    /// bounds.
    pub fn split_off<const MIN_LEN2: usize, const MAX_LEN2: usize>(
        &mut self,
        key: &K,
    ) -> Result<Confined<BTreeMap<K, V>, MIN_LEN2, MAX_LEN2>, Error>
    where
        K: Clone,
        V: Clone,
    {
        const { assert!(MIN_LEN2 <= MAX_LEN2, "invalid confinement bounds") };
        let len2 = self.0.range(key..).count();
        let len1 = self.len() - len2;

        if len1 < MIN_LEN {
            return Err(Error::Undersize {
                len: len1,
                min_len: MIN_LEN,
            });
        }

        if len2 < MIN_LEN2 {
            return Err(Error::Undersize {
                len: len2,
                min_len: MIN_LEN2,
            });
        }

        if len2 > MAX_LEN2 {
            return Err(Error::Oversize {
                len: len2,
                max_len: MAX_LEN2,
            });
        }

        let other: BTreeMap<K, V> = self
            .0
            .range(key..)
            .map(|(key, value)| (key.clone(), value.clone()))
            .collect();
        let other = Confined::try_from(other)?;
        for _ in 0..len2 {
            self.0.pop_last();
        }
        Ok(other)
    }

    /// Gets a mutable iterator over the specified range. The values can be
    /// changed through it, which doesn't affect the length of the map.
    ///
    /// # Panics
    ///
    /// Panics, like [`BTreeMap::range_mut`], if the start of the range is
    /// greater than its end, or if both bounds are equal and excluded.
    pub fn range_mut<R>(&mut self, range: R) -> btree_map::RangeMut<'_, K, V>
    where
        R: RangeBounds<K>,
    {
        self.0.range_mut(range)
    }

    /// Creates a consuming iterator visiting all the keys in sorted order.
    /// The map cannot be used after calling this.
    /// The iterator element type is `K`.
    pub fn into_keys(self) -> btree_map::IntoKeys<K, V> {
        self.0.into_keys()
    }

    /// Creates a consuming iterator visiting all the values in order by key.
    /// The map cannot be used after calling this.
    /// The iterator element type is `V`.
    pub fn into_values(self) -> btree_map::IntoValues<K, V> {
        self.0.into_values()
    }
}
