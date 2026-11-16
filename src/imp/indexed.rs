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

use core::hash::Hash;

use indexmap::{IndexMap, IndexSet, map};

use crate::{Confined, Error};

impl<T: Eq + Hash, const MIN_LEN: usize, const MAX_LEN: usize>
    Confined<IndexSet<T>, MIN_LEN, MAX_LEN>
{
    /// Removes a value from the set, returning whether the value was at the
    /// set previously.
    ///
    /// # Errors
    ///
    /// Errors if the minimum confinement is not met after the removal.
    pub fn remove(&mut self, elem: &T) -> Result<bool, Error> {
        if !self.0.contains(elem) {
            return Ok(false);
        }
        self.check_undersize()?;
        Ok(self.0.shift_remove(elem))
    }

    /// Removes and returns the value in the set, if any, that is equal to the
    /// given one.
    ///
    /// # Errors
    ///
    /// Errors if the minimum confinement is not met after the removal.
    pub fn take(&mut self, elem: &T) -> Result<Option<T>, Error> {
        if !self.0.contains(elem) {
            return Ok(None);
        }
        self.check_undersize()?;
        Ok(self.0.shift_take(elem))
    }
}

impl<K: Eq + Hash, V, const MIN_LEN: usize, const MAX_LEN: usize>
    Confined<IndexMap<K, V>, MIN_LEN, MAX_LEN>
{
    /// Removes a key from the map, returning the value at the key if the key
    /// was previously in the map.
    ///
    /// # Errors
    ///
    /// Errors if the minimum confinement is not met after the removal.
    pub fn remove(&mut self, key: &K) -> Result<Option<V>, Error> {
        if !self.0.contains_key(key) {
            return Ok(None);
        }
        self.check_undersize()?;
        Ok(self.0.shift_remove(key))
    }

    /// Creates a consuming iterator visiting all the keys in arbitrary order.
    /// The map cannot be used after calling this. The iterator element type is
    /// `K`.
    pub fn into_keys(self) -> map::IntoKeys<K, V> {
        self.0.into_keys()
    }

    /// Creates a consuming iterator visiting all the values in arbitrary order.
    /// The map cannot be used after calling this. The iterator element type is
    /// `V`.
    pub fn into_values(self) -> map::IntoValues<K, V> {
        self.0.into_values()
    }
}
