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

use collections::{Collection, CollectionAlloc};

mod sealed {
    pub trait Sealed {}
}

/// Internal dispatch for the collection operations used by [`ItemLen`].
///
/// [`ItemLen`]: crate::ItemLen
pub trait Access<C>: sealed::Sealed {
    /// Item type stored by the collection.
    type Item;

    /// Returns the number of items in the collection.
    fn len(collection: &C) -> usize;

    /// Adds an item to the collection.
    fn push(collection: &mut C, item: Self::Item);

    /// Removes all items from the collection.
    fn clear(collection: &mut C);
}

/// Access strategy for collections implementing [`Collection`].
pub struct ViaCollection;

/// Access strategy for allocator-aware collections implementing
/// [`CollectionAlloc`].
pub struct ViaCollectionAlloc;

impl sealed::Sealed for ViaCollection {}
impl sealed::Sealed for ViaCollectionAlloc {}

impl<C: Collection> Access<C> for ViaCollection {
    type Item = C::Item;

    fn len(collection: &C) -> usize {
        Collection::len(collection)
    }

    fn push(collection: &mut C, item: Self::Item) {
        Collection::push(collection, item)
    }

    fn clear(collection: &mut C) {
        Collection::clear(collection)
    }
}

impl<C: CollectionAlloc> Access<C> for ViaCollectionAlloc {
    type Item = C::Item;

    fn len(collection: &C) -> usize {
        CollectionAlloc::len(collection)
    }

    fn push(collection: &mut C, item: Self::Item) {
        CollectionAlloc::push(collection, item)
    }

    fn clear(collection: &mut C) {
        CollectionAlloc::clear(collection)
    }
}
