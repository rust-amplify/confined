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

use crate::{Confined, Error};

impl<T, const MIN_LEN: usize, const MAX_LEN: usize> Confined<VecDeque<T>, MIN_LEN, MAX_LEN> {
    /// Removes the first element and returns it, or `None` if the deque is
    /// empty.
    pub fn pop_front(&mut self) -> Option<T> {
        self.0.pop_front()
    }

    /// Removes the last element and returns it, or `None` if the deque is
    /// empty.
    pub fn pop_back(&mut self) -> Option<T> {
        self.0.pop_back()
    }
}

impl<T, const MIN_LEN: usize, const MAX_LEN: usize> Confined<VecDeque<T>, MIN_LEN, MAX_LEN> {
    /// Prepends an element to the deque. Errors if the new collection length
    /// will not fit the confinement requirements.
    pub fn push_front(&mut self, elem: T) -> Result<(), Error> {
        self.check_oversize()?;
        self.0.push_front(elem);
        Ok(())
    }

    #[deprecated(since = "4.7.1", note = "use `push_front`")]
    pub fn push_from(&mut self, elem: T) -> Result<(), Error> {
        self.push_front(elem)
    }

    /// Appends an element to the deque. Errors if the new collection length
    /// will not fit the confinement requirements.
    pub fn push_back(&mut self, elem: T) -> Result<(), Error> {
        self.check_oversize()?;
        self.0.push_back(elem);
        Ok(())
    }

    /// Removes an element from the deque at a given index. Errors if the index
    /// exceeds the number of elements in the deque, of if the new deque
    /// length will be less than the confinement requirement. Returns the
    /// removed element otherwise.
    pub fn remove(&mut self, index: usize) -> Result<T, Error> {
        self.check_boundary(index)?;
        self.check_undersize()?;
        Ok(self.0.remove(index).expect("element within the length"))
    }

    /// Removes the specified range from the deque in bulk, returning all
    /// removed elements as an iterator. If the iterator is dropped before
    /// being fully consumed, it drops the remaining removed elements.
    ///
    /// The returned iterator keeps a mutable borrow on the queue to optimize
    /// its implementation.
    ///
    /// # Panics
    ///
    /// Panics if the starting point is greater than the end point or if
    /// the end point is greater than the length of the deque.
    ///
    /// # Leaking
    ///
    /// If the returned iterator goes out of scope without being dropped (due to
    /// [`mem::forget`], for example), the deque may have lost and leaked
    /// elements arbitrarily, including elements outside the range.
    pub fn drain<R: RangeBounds<usize>>(&mut self, range: R) -> vec_deque::Drain<'_, T> {
        self.0.drain(range)
    }

    /// Shortens the deque, keeping the first `len` elements and dropping
    /// the rest.
    ///
    /// If `len` is greater or equal to the deque's current length, this has
    /// no effect.
    pub fn truncate(&mut self, len: usize) {
        self.0.truncate(len)
    }
}
