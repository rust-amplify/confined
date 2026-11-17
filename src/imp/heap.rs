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

use alloc::collections::BinaryHeap;

use crate::{Confined, ItemLen, ViaCollection, ZERO};

impl<T: Ord> ItemLen for BinaryHeap<T> {
    type Item = T;
    type Access = ViaCollection;
}

impl<T: Ord, const MAX_LEN: usize> Confined<BinaryHeap<T>, ZERO, MAX_LEN> {
    /// Removes and returns the greatest element, or `None` if the heap is
    /// empty.
    ///
    /// Heaps with a lower bound cannot be popped:
    ///
    /// ```compile_fail,E0596
    /// # use confined::NonEmptyHeap;
    /// let mut heap = NonEmptyHeap::<u8>::with(1);
    /// heap.pop();
    /// ```
    pub fn pop(&mut self) -> Option<T> {
        self.0.pop()
    }
}
