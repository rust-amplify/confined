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

use alloc::alloc::Allocator;
use alloc::vec::Vec;

use collections::CollectionCapacity;

use crate::{Confined, Error, ItemLen, ZERO};

impl<T, A: Allocator> ItemLen for Vec<T, A> {
    fn try_push(&mut self, item: T) -> Result<(), T> {
        CollectionCapacity::try_push(self, item)
    }
}

impl<T, const MIN_LEN: usize, const MAX_LEN: usize> Confined<Vec<T>, MIN_LEN, MAX_LEN> {
    /// Constructs confinement out of an array of items. Does allocation.
    ///
    /// The length of an array is a part of its type, so it is checked against
    /// the confinement bounds when the program is built, and the function
    /// can't fail when the program runs. An array which length is outside of
    /// the bounds fails to build. The failure is reported by `cargo build` and
    /// `cargo test`, but not by `cargo check`.
    ///
    /// ```
    /// # use confined::Confined;
    /// let vec = Confined::<Vec<u8>, 1, 3>::from_array([1, 2, 3]);
    /// assert_eq!(vec.len(), 3);
    /// ```
    ///
    /// ```compile_fail,E0080
    /// # use confined::Confined;
    /// let vec = Confined::<Vec<u8>, 1, 2>::from_array([1, 2, 3]);
    /// assert_eq!(vec.len(), 3);
    /// ```
    #[inline]
    pub fn from_array<const N: usize>(items: [T; N]) -> Self {
        const {
            assert!(
                MIN_LEN <= N && N <= MAX_LEN,
                "the number of items is outside of the confinement bounds"
            )
        };
        Self(Vec::from(items))
    }

    /// Constructs confinement out of slice of items. Does allocation.
    ///
    /// # Panics
    ///
    /// Panics if the size of the slice doesn't match the confinement type
    /// bounds.
    #[inline]
    pub fn from_slice_checked(slice: &[T]) -> Self
    where
        T: Clone,
    {
        Self::try_from_slice(slice).expect("slice size mismatch, use try_from_slice instead")
    }

    /// Constructs confinement out of slice of items. Does allocation.
    #[inline]
    pub fn try_from_slice(slice: &[T]) -> Result<Self, Error>
    where
        T: Clone,
    {
        Self::check_len(slice.len())?;
        let mut items = Vec::new();
        items
            .try_reserve_exact(slice.len())
            .map_err(|_| Error::CapacityExceeded { len: 0 })?;
        items.extend_from_slice(slice);
        Ok(Self(items))
    }
}

impl<T, A: Allocator, const MAX_LEN: usize> Confined<Vec<T, A>, ZERO, MAX_LEN> {
    /// Removes the last element from a vector and returns it, or [`None`] if it
    /// is empty.
    ///
    /// ```
    /// # use confined::Confined;
    /// let mut vec = Confined::<Vec<u8>, 0, 3>::try_from(vec![1, 2]).unwrap();
    /// assert_eq!(vec.pop(), Some(2));
    /// ```
    ///
    /// A vector with a lower bound has no such method, so the call fails to
    /// compile. Such a vector is shortened with `remove`, which checks the
    /// bound:
    ///
    /// ```compile_fail,E0596
    /// # use confined::Confined;
    /// let mut vec = Confined::<Vec<u8>, 1, 3>::try_from(vec![1, 2]).unwrap();
    /// assert_eq!(vec.pop(), Some(2));
    /// ```
    #[inline]
    pub fn pop(&mut self) -> Option<T> {
        self.0.pop()
    }
}

impl<T, A: Allocator, const MIN_LEN: usize, const MAX_LEN: usize>
    Confined<Vec<T, A>, MIN_LEN, MAX_LEN>
{
    /// Removes an element from the vector at a given index. Errors if the index
    /// is outside of the vector, or if the new vector length will be less
    /// than the confinement requirement; the index is checked first. Returns
    /// the removed element otherwise.
    pub fn remove(&mut self, index: usize) -> Result<T, Error> {
        self.check_boundary(index)?;
        self.check_undersize()?;
        Ok(self.0.remove(index))
    }

    /// Returns an iterator over the slice.
    ///
    /// The iterator yields all items from start to end.
    pub fn iter(&self) -> core::slice::Iter<'_, T> {
        self.0.iter()
    }

    /// Returns the slice representation of the vector.
    #[inline]
    pub fn as_slice(&self) -> &[T] {
        &self.0
    }

    /// Converts into the inner unconfined vector.
    #[inline]
    pub fn into_vec(self) -> Vec<T, A> {
        self.0
    }
}
