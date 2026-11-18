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

use collections::heapless::binary_heap::Kind;
use collections::heapless::{
    BinaryHeap, Deque, IndexMap, IndexSet, LenType, LinearMap, String, Vec,
};
use collections::{CollectionBytes, CollectionCapacity, CollectionMap};

use crate::{Confined, Error, ItemLen};

impl<T, const N: usize, L: LenType> ItemLen for Vec<T, N, L> {
    fn try_push(&mut self, item: T) -> Result<(), T> {
        CollectionCapacity::try_push(self, item)
    }
}

impl<const N: usize, L: LenType> ItemLen for String<N, L> {
    const MAX_ITEM_LEN: usize = char::MAX_LEN_UTF8;

    fn confined_len(&self) -> usize {
        <Self as CollectionBytes>::byte_len(self)
    }

    fn item_len(item: &char) -> usize {
        <Self as CollectionBytes>::item_byte_len(item)
    }

    fn try_push(&mut self, item: char) -> Result<(), char> {
        CollectionCapacity::try_push(self, item)
    }
}

impl<T, const N: usize> ItemLen for Deque<T, N> {
    fn try_push(&mut self, item: T) -> Result<(), T> {
        CollectionCapacity::try_push(self, item)
    }
}

impl<T: Ord, K: Kind, const N: usize> ItemLen for BinaryHeap<T, K, N> {
    fn try_push(&mut self, item: T) -> Result<(), T> {
        CollectionCapacity::try_push(self, item)
    }
}

impl<T: Eq + Hash, S: BuildHasher, const N: usize> ItemLen for IndexSet<T, S, N> {
    fn item_growth(&self, item: &T) -> usize {
        usize::from(!self.contains(item))
    }
    fn try_push(&mut self, item: T) -> Result<(), T> {
        CollectionCapacity::try_push(self, item)
    }
}

impl<K: Eq + Hash, V, S: BuildHasher, const N: usize> ItemLen for IndexMap<K, V, S, N> {
    fn item_growth(&self, item: &(K, V)) -> usize {
        usize::from(!CollectionMap::contains_key(self, &item.0))
    }
    fn try_push(&mut self, item: (K, V)) -> Result<(), (K, V)> {
        CollectionCapacity::try_push(self, item)
    }

    fn prepare_entry(&mut self) -> Result<(), Error> {
        CollectionCapacity::try_reserve(self, 1)
            .map_err(|_| Error::CapacityExceeded { len: self.len() })
    }
}

impl<K: Eq, V, const N: usize> ItemLen for LinearMap<K, V, N> {
    fn item_growth(&self, item: &(K, V)) -> usize {
        usize::from(!CollectionMap::contains_key(self, &item.0))
    }
    fn try_push(&mut self, item: (K, V)) -> Result<(), (K, V)> {
        CollectionCapacity::try_push(self, item)
    }
}

impl<T, const N: usize, L: LenType, const MIN: usize, const MAX: usize>
    Confined<Vec<T, N, L>, MIN, MAX>
{
    /// Removes an item while preserving the lower bound.
    pub fn remove(&mut self, index: usize) -> Result<T, Error> {
        self.check_boundary(index)?;
        self.check_undersize()?;
        Ok(self.0.remove(index))
    }
}

impl<T, const N: usize, L: LenType, const MAX: usize> Confined<Vec<T, N, L>, 0, MAX> {
    /// Removes the last item, or returns `None` if empty.
    ///
    /// ```
    /// # use confined::HeaplessVec;
    /// let mut vec = HeaplessVec::<u8, 4, 0, 3>::try_from_iter([1, 2]).unwrap();
    /// assert_eq!(vec.pop(), Some(2));
    /// ```
    ///
    /// A vector with a lower bound has no such method, so the call fails to
    /// compile. Such a vector is shortened with `remove`, which checks the
    /// bound:
    ///
    /// ```compile_fail,E0596
    /// # use confined::HeaplessVec;
    /// let mut vec = HeaplessVec::<u8, 4, 1, 3>::try_from_iter([1, 2]).unwrap();
    /// assert_eq!(vec.pop(), Some(2));
    /// ```
    pub fn pop(&mut self) -> Option<T> {
        self.0.pop()
    }
}

impl<const N: usize, L: LenType, const MAX: usize> Confined<String<N, L>, 0, MAX> {
    /// Removes the last character, or returns `None` if empty.
    ///
    /// ```
    /// # use confined::HeaplessString;
    /// let mut string = HeaplessString::<8, 0, 4>::try_from_iter(['a', 'b']).unwrap();
    /// assert_eq!(string.pop(), Some('b'));
    /// ```
    ///
    /// A string with a lower bound has no such method, so the call fails to
    /// compile:
    ///
    /// ```compile_fail,E0596
    /// # use confined::HeaplessString;
    /// let mut string = HeaplessString::<8, 1, 4>::try_from_iter(['a', 'b']).unwrap();
    /// assert_eq!(string.pop(), Some('b'));
    /// ```
    pub fn pop(&mut self) -> Option<char> {
        self.0.pop()
    }
}

impl<T, const N: usize, const MIN: usize, const MAX: usize> Confined<Deque<T, N>, MIN, MAX> {
    /// Prepends an item, checking confinement and physical capacity.
    pub fn push_front(&mut self, item: T) -> Result<(), Error> {
        self.check_oversize()?;
        self.0
            .push_front(item)
            .map_err(|_| Error::CapacityExceeded { len: self.len() })
    }

    /// Appends an item, checking confinement and physical capacity.
    pub fn push_back(&mut self, item: T) -> Result<(), Error> {
        self.push(item)
    }
}

impl<T, const N: usize, const MAX: usize> Confined<Deque<T, N>, 0, MAX> {
    /// Removes the first item, or returns `None` if empty.
    ///
    /// ```
    /// # use confined::HeaplessDeque;
    /// let mut deque = HeaplessDeque::<u8, 4, 0, 3>::try_from_iter([1, 2]).unwrap();
    /// assert_eq!(deque.pop_front(), Some(1));
    /// ```
    ///
    /// A deque with a lower bound has no such method, so the call fails to
    /// compile:
    ///
    /// ```compile_fail,E0596
    /// # use confined::HeaplessDeque;
    /// let mut deque = HeaplessDeque::<u8, 4, 1, 3>::try_from_iter([1, 2]).unwrap();
    /// assert_eq!(deque.pop_front(), Some(1));
    /// ```
    pub fn pop_front(&mut self) -> Option<T> {
        self.0.pop_front()
    }

    /// Removes the last item, or returns `None` if empty.
    ///
    /// ```
    /// # use confined::HeaplessDeque;
    /// let mut deque = HeaplessDeque::<u8, 4, 0, 3>::try_from_iter([1, 2]).unwrap();
    /// assert_eq!(deque.pop_back(), Some(2));
    /// ```
    ///
    /// A deque with a lower bound has no such method, so the call fails to
    /// compile:
    ///
    /// ```compile_fail,E0596
    /// # use confined::HeaplessDeque;
    /// let mut deque = HeaplessDeque::<u8, 4, 1, 3>::try_from_iter([1, 2]).unwrap();
    /// assert_eq!(deque.pop_back(), Some(2));
    /// ```
    pub fn pop_back(&mut self) -> Option<T> {
        self.0.pop_back()
    }
}

impl<T: Ord, K: Kind, const N: usize, const MAX: usize> Confined<BinaryHeap<T, K, N>, 0, MAX> {
    /// Removes the highest-priority item, or returns `None` if empty.
    ///
    /// ```
    /// # use confined::HeaplessHeap;
    /// # use confined::heapless::binary_heap::Max;
    /// let mut heap = HeaplessHeap::<u8, Max, 4, 0, 3>::try_from_iter([1, 2]).unwrap();
    /// assert_eq!(heap.pop(), Some(2));
    /// ```
    ///
    /// A heap with a lower bound has no such method, so the call fails to
    /// compile:
    ///
    /// ```compile_fail,E0596
    /// # use confined::HeaplessHeap;
    /// # use confined::heapless::binary_heap::Max;
    /// let mut heap = HeaplessHeap::<u8, Max, 4, 1, 3>::try_from_iter([1, 2]).unwrap();
    /// assert_eq!(heap.pop(), Some(2));
    /// ```
    pub fn pop(&mut self) -> Option<T> {
        self.0.pop()
    }
}

impl<K: Eq + Hash, V, S: BuildHasher, const N: usize, const MIN: usize, const MAX: usize>
    Confined<IndexMap<K, V, S, N>, MIN, MAX>
{
    /// Removes a key while preserving the lower bound.
    pub fn remove(&mut self, key: &K) -> Result<Option<V>, Error> {
        self.remove_from_map(key, |map, key| map.remove(key))
    }
}

impl<K: Eq, V, const N: usize, const MIN: usize, const MAX: usize>
    Confined<LinearMap<K, V, N>, MIN, MAX>
{
    /// Removes a key while preserving the lower bound.
    pub fn remove(&mut self, key: &K) -> Result<Option<V>, Error> {
        self.remove_from_map(key, |map, key| map.remove(key))
    }
}

impl<T: Eq + Hash, S: BuildHasher, const N: usize, const MIN: usize, const MAX: usize>
    Confined<IndexSet<T, S, N>, MIN, MAX>
{
    /// Removes an item while preserving the lower bound.
    pub fn remove(&mut self, item: &T) -> Result<bool, Error> {
        if !self.0.contains(item) {
            return Ok(false);
        }
        self.check_undersize()?;
        Ok(self.0.remove(item))
    }
}

/// A confined [`crate::heapless::Vec`] with a fixed storage capacity.
pub type HeaplessVec<
    T,
    const CAP: usize,
    const MIN: usize = 0,
    const MAX: usize = { usize::MAX },
    L = usize,
> = Confined<Vec<T, CAP, L>, MIN, MAX>;

/// A confined [`crate::heapless::String`] with a fixed byte capacity.
pub type HeaplessString<
    const CAP: usize,
    const MIN: usize = 0,
    const MAX: usize = { usize::MAX },
    L = usize,
> = Confined<String<CAP, L>, MIN, MAX>;

/// A confined [`crate::heapless::Deque`] with a fixed storage capacity.
pub type HeaplessDeque<
    T,
    const CAP: usize,
    const MIN: usize = 0,
    const MAX: usize = { usize::MAX },
> = Confined<Deque<T, CAP>, MIN, MAX>;

/// A confined [`crate::heapless::BinaryHeap`] with fixed storage capacity.
pub type HeaplessHeap<
    T,
    K,
    const CAP: usize,
    const MIN: usize = 0,
    const MAX: usize = { usize::MAX },
> = Confined<BinaryHeap<T, K, CAP>, MIN, MAX>;

/// A confined [`crate::heapless::IndexMap`] with a fixed storage capacity.
pub type HeaplessIndexMap<
    K,
    V,
    S,
    const CAP: usize,
    const MIN: usize = 0,
    const MAX: usize = { usize::MAX },
> = Confined<IndexMap<K, V, S, CAP>, MIN, MAX>;

/// A confined [`crate::heapless::IndexSet`] with a fixed storage capacity.
pub type HeaplessIndexSet<
    T,
    S,
    const CAP: usize,
    const MIN: usize = 0,
    const MAX: usize = { usize::MAX },
> = Confined<IndexSet<T, S, CAP>, MIN, MAX>;

/// A confined [`crate::heapless::LinearMap`] with a fixed storage capacity.
pub type HeaplessLinearMap<
    K,
    V,
    const CAP: usize,
    const MIN: usize = 0,
    const MAX: usize = { usize::MAX },
> = Confined<LinearMap<K, V, CAP>, MIN, MAX>;
