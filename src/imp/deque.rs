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

use alloc::collections::vec_deque::{self, VecDeque};
use core::ops::RangeBounds;

use collections::CollectionCapacity;

use crate::{Confined, Error, ItemLen, ZERO};

impl<T> ItemLen for VecDeque<T> {
    fn try_push(&mut self, item: T) -> Result<(), T> {
        CollectionCapacity::try_push(self, item)
    }
}

/// Methods which can't report an error, so they exist only for the deques
/// without a lower bound. A deque with a lower bound is shortened with
/// `remove`, which checks the bound.
impl<T, const MAX_LEN: usize> Confined<VecDeque<T>, ZERO, MAX_LEN> {
    /// Removes the first element and returns it, or `None` if the deque is
    /// empty.
    ///
    /// ```
    /// # use std::collections::VecDeque;
    /// # use confined::Confined;
    /// let mut deque = Confined::<VecDeque<u8>, 0, 3>::try_from_iter([1, 2]).unwrap();
    /// assert_eq!(deque.pop_front(), Some(1));
    /// ```
    ///
    /// A deque with a lower bound has no such method, so the call fails to
    /// compile:
    ///
    /// ```compile_fail,E0596
    /// # use std::collections::VecDeque;
    /// # use confined::Confined;
    /// let mut deque = Confined::<VecDeque<u8>, 1, 3>::try_from_iter([1, 2]).unwrap();
    /// assert_eq!(deque.pop_front(), Some(1));
    /// ```
    pub fn pop_front(&mut self) -> Option<T> {
        self.0.pop_front()
    }

    /// Removes the last element and returns it, or `None` if the deque is
    /// empty.
    ///
    /// ```
    /// # use std::collections::VecDeque;
    /// # use confined::Confined;
    /// let mut deque = Confined::<VecDeque<u8>, 0, 3>::try_from_iter([1, 2]).unwrap();
    /// assert_eq!(deque.pop_back(), Some(2));
    /// ```
    ///
    /// A deque with a lower bound has no such method, so the call fails to
    /// compile:
    ///
    /// ```compile_fail,E0596
    /// # use std::collections::VecDeque;
    /// # use confined::Confined;
    /// let mut deque = Confined::<VecDeque<u8>, 1, 3>::try_from_iter([1, 2]).unwrap();
    /// assert_eq!(deque.pop_back(), Some(2));
    /// ```
    pub fn pop_back(&mut self) -> Option<T> {
        self.0.pop_back()
    }

    /// Removes the specified range from the deque in bulk, returning all
    /// removed elements as an iterator. If the iterator is dropped before
    /// being fully consumed, it drops the remaining removed elements.
    ///
    /// The returned iterator keeps a mutable borrow on the queue to optimize
    /// its implementation.
    ///
    /// ```
    /// # use std::collections::VecDeque;
    /// # use confined::Confined;
    /// let mut deque = Confined::<VecDeque<u8>, 0, 3>::try_from_iter([1, 2]).unwrap();
    /// assert_eq!(deque.drain(..).count(), 2);
    /// ```
    ///
    /// A deque with a lower bound has no such method, so the call fails to
    /// compile:
    ///
    /// ```compile_fail,E0596
    /// # use std::collections::VecDeque;
    /// # use confined::Confined;
    /// let mut deque = Confined::<VecDeque<u8>, 1, 3>::try_from_iter([1, 2]).unwrap();
    /// assert_eq!(deque.drain(..).count(), 2);
    /// ```
    ///
    /// # Panics
    ///
    /// Panics if the starting point is greater than the end point or if
    /// the end point is greater than the length of the deque.
    ///
    /// # Leaking
    ///
    /// If the returned iterator goes out of scope without being dropped (due to
    /// [`mem::forget`](core::mem::forget), for example), the deque may have
    /// lost and leaked elements arbitrarily, including elements outside the
    /// range.
    pub fn drain<R: RangeBounds<usize>>(&mut self, range: R) -> vec_deque::Drain<'_, T> {
        self.0.drain(range)
    }

    /// Shortens the deque, keeping the first `len` elements and dropping
    /// the rest.
    ///
    /// If `len` is greater or equal to the deque's current length, this has
    /// no effect.
    ///
    /// ```
    /// # use std::collections::VecDeque;
    /// # use confined::Confined;
    /// let mut deque = Confined::<VecDeque<u8>, 0, 3>::try_from_iter([1, 2]).unwrap();
    /// deque.truncate(1);
    /// assert_eq!(deque.len(), 1);
    /// ```
    ///
    /// A deque with a lower bound has no such method, so the call fails to
    /// compile:
    ///
    /// ```compile_fail,E0596
    /// # use std::collections::VecDeque;
    /// # use confined::Confined;
    /// let mut deque = Confined::<VecDeque<u8>, 1, 3>::try_from_iter([1, 2]).unwrap();
    /// deque.truncate(1);
    /// assert_eq!(deque.len(), 1);
    /// ```
    pub fn truncate(&mut self, len: usize) {
        self.0.truncate(len)
    }
}

impl<T, const MIN_LEN: usize, const MAX_LEN: usize> Confined<VecDeque<T>, MIN_LEN, MAX_LEN> {
    /// Prepends an element to the deque. Errors if the new collection length
    /// will not fit the confinement requirements.
    pub fn push_front(&mut self, elem: T) -> Result<(), Error> {
        self.try_reserve(1)?;
        self.0.push_front(elem);
        Ok(())
    }

    /// Appends an element to the deque. Errors if the new collection length
    /// will not fit the confinement requirements.
    pub fn push_back(&mut self, elem: T) -> Result<(), Error> {
        self.push(elem)
    }

    /// Removes an element from the deque at a given index. Errors if the index
    /// is outside of the deque, or if the new deque length will be less than
    /// the confinement requirement; the index is checked first. Returns the
    /// removed element otherwise.
    pub fn remove(&mut self, index: usize) -> Result<T, Error> {
        self.check_boundary(index)?;
        self.check_undersize()?;
        Ok(self.0.remove(index).expect("element within the length"))
    }
}
