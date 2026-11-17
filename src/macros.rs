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

/// Helper macro to construct confined string of a
/// [`TinyString`](crate::TinyString) type
#[macro_export]
macro_rules! tiny_s {
    () => {
        $crate::TinyString::new()
    };
    ($lit:literal) => {
        $crate::TinyString::try_from($crate::alloc::string::String::from($lit))
            .expect("static string for tiny_s literal is too long")
    };
}

/// Helper macro to construct confined string of a
/// [`SmallString`](crate::SmallString) type
#[macro_export]
macro_rules! small_s {
    () => {
        $crate::SmallString::new()
    };
    ($lit:literal) => {
        $crate::SmallString::try_from($crate::alloc::string::String::from($lit))
            .expect("static string for small_s literal is too long")
    };
}

/// Helper macro to construct confined string of a
/// [`MediumString`](crate::MediumString) type
#[macro_export]
macro_rules! medium_s {
    () => {
        $crate::MediumString::new()
    };
    ($lit:literal) => {
        $crate::MediumString::try_from($crate::alloc::string::String::from($lit))
            .expect("static string for medium_s literal is too long")
    };
}

/// Helper macro to construct confined blob of a [`TinyBlob`](crate::TinyBlob)
/// type
#[macro_export]
macro_rules! tiny_blob {
    () => {
        $crate::TinyBlob::new()
    };
    ($elem:expr; $n:expr) => (
        $crate::TinyBlob::try_from($crate::alloc::vec![$elem; $n])
            .expect("inline tiny_blob contains invalid number of items")
    );
    ($($x:expr),+ $(,)?) => (
        $crate::TinyBlob::try_from($crate::alloc::vec![$($x,)+])
            .expect("inline tiny_blob contains invalid number of items")
    )
}

/// Helper macro to construct confined blob of a [`SmallBlob`](crate::SmallBlob)
/// type
#[macro_export]
macro_rules! small_blob {
    () => {
        $crate::SmallBlob::new()
    };
    ($elem:expr; $n:expr) => (
        $crate::SmallBlob::try_from($crate::alloc::vec![$elem; $n])
            .expect("inline small_blob contains invalid number of items")
    );
    ($($x:expr),+ $(,)?) => (
        $crate::SmallBlob::try_from($crate::alloc::vec![$($x,)+])
            .expect("inline small_blob contains invalid number of items")
    )
}

/// Helper macro to construct confined blob of a
/// [`MediumBlob`](crate::MediumBlob) type
#[macro_export]
macro_rules! medium_blob {
    () => {
        $crate::MediumBlob::new()
    };
    ($elem:expr; $n:expr) => (
        $crate::MediumBlob::try_from($crate::alloc::vec![$elem; $n])
            .expect("inline medium_blob contains invalid number of items")
    );
    ($($x:expr),+ $(,)?) => (
        $crate::MediumBlob::try_from($crate::alloc::vec![$($x,)+])
            .expect("inline medium_blob contains invalid number of items")
    )
}

/// Helper macro to construct confined vector of a [`TinyVec`](crate::TinyVec)
/// type
#[macro_export]
macro_rules! tiny_vec {
    () => {
        $crate::TinyVec::new()
    };
    ($elem:expr; $n:expr) => (
        $crate::TinyVec::try_from($crate::alloc::vec![$elem; $n])
            .expect("inline tiny_vec literal contains invalid number of items")
    );
    ($($x:expr),+ $(,)?) => (
        $crate::TinyVec::try_from($crate::alloc::vec![$($x,)+])
            .expect("inline tiny_vec literal contains invalid number of items")
    )
}

/// Helper macro to construct confined vector of a [`SmallVec`](crate::SmallVec)
/// type
#[macro_export]
macro_rules! small_vec {
    () => {
        $crate::SmallVec::new()
    };
    ($elem:expr; $n:expr) => (
        $crate::SmallVec::try_from($crate::alloc::vec![$elem; $n])
            .expect("inline small_vec literal contains invalid number of items")
    );
    ($($x:expr),+ $(,)?) => (
        $crate::SmallVec::try_from($crate::alloc::vec![$($x,)+])
            .expect("inline small_vec literal contains invalid number of items")
    )
}

/// Helper macro to construct confined vector of a
/// [`MediumVec`](crate::MediumVec) type
#[macro_export]
macro_rules! medium_vec {
    () => {
        $crate::MediumVec::new()
    };
    ($elem:expr; $n:expr) => (
        $crate::MediumVec::try_from($crate::alloc::vec![$elem; $n])
            .expect("inline medium_vec literal contains invalid number of items")
    );
    ($($x:expr),+ $(,)?) => (
        $crate::MediumVec::try_from($crate::alloc::vec![$($x,)+])
            .expect("inline medium_vec literal contains invalid number of items")
    )
}

/// Helper macro to construct a confined `VecDeque` of `TinyDeque` type.
#[macro_export]
macro_rules! tiny_deque {
    () => {
        $crate::TinyDeque::new()
    };
    ($elem:expr; $n:expr) => {
        $crate::TinyDeque::try_from_iter($crate::alloc::vec![$elem; $n])
            .expect("inline tiny_deque contains invalid number of items")
    };
    ($($item:expr),+ $(,)?) => {
        $crate::TinyDeque::try_from_iter([$($item,)+])
            .expect("inline tiny_deque contains invalid number of items")
    };
}

/// Helper macro to construct a confined `VecDeque` of `SmallDeque` type.
#[macro_export]
macro_rules! small_deque {
    () => {
        $crate::SmallDeque::new()
    };
    ($elem:expr; $n:expr) => {
        $crate::SmallDeque::try_from_iter($crate::alloc::vec![$elem; $n])
            .expect("inline small_deque contains invalid number of items")
    };
    ($($item:expr),+ $(,)?) => {
        $crate::SmallDeque::try_from_iter([$($item,)+])
            .expect("inline small_deque contains invalid number of items")
    };
}

/// Helper macro to construct a confined `VecDeque` of `MediumDeque` type.
#[macro_export]
macro_rules! medium_deque {
    () => {
        $crate::MediumDeque::new()
    };
    ($elem:expr; $n:expr) => {
        $crate::MediumDeque::try_from_iter($crate::alloc::vec![$elem; $n])
            .expect("inline medium_deque contains invalid number of items")
    };
    ($($item:expr),+ $(,)?) => {
        $crate::MediumDeque::try_from_iter([$($item,)+])
            .expect("inline medium_deque contains invalid number of items")
    };
}

/// Helper macro to construct a confined `LinkedList` of `TinyList` type.
#[macro_export]
macro_rules! tiny_list {
    () => {
        $crate::TinyList::new()
    };
    ($($item:expr),+ $(,)?) => {
        $crate::TinyList::try_from_iter([$($item,)+])
            .expect("inline tiny_list contains invalid number of items")
    };
}

/// Helper macro to construct a confined `LinkedList` of `SmallList` type.
#[macro_export]
macro_rules! small_list {
    () => {
        $crate::SmallList::new()
    };
    ($($item:expr),+ $(,)?) => {
        $crate::SmallList::try_from_iter([$($item,)+])
            .expect("inline small_list contains invalid number of items")
    };
}

/// Helper macro to construct a confined `LinkedList` of `MediumList` type.
#[macro_export]
macro_rules! medium_list {
    () => {
        $crate::MediumList::new()
    };
    ($($item:expr),+ $(,)?) => {
        $crate::MediumList::try_from_iter([$($item,)+])
            .expect("inline medium_list contains invalid number of items")
    };
}

/// Helper macro to construct a confined `BinaryHeap` of `TinyHeap` type.
#[macro_export]
macro_rules! tiny_heap {
    () => {
        $crate::TinyHeap::new()
    };
    ($($item:expr),+ $(,)?) => {
        $crate::TinyHeap::try_from_iter([$($item,)+])
            .expect("inline tiny_heap contains invalid number of items")
    };
}

/// Helper macro to construct a confined `BinaryHeap` of `SmallHeap` type.
#[macro_export]
macro_rules! small_heap {
    () => {
        $crate::SmallHeap::new()
    };
    ($($item:expr),+ $(,)?) => {
        $crate::SmallHeap::try_from_iter([$($item,)+])
            .expect("inline small_heap contains invalid number of items")
    };
}

/// Helper macro to construct a confined `BinaryHeap` of `MediumHeap` type.
#[macro_export]
macro_rules! medium_heap {
    () => {
        $crate::MediumHeap::new()
    };
    ($($item:expr),+ $(,)?) => {
        $crate::MediumHeap::try_from_iter([$($item,)+])
            .expect("inline medium_heap contains invalid number of items")
    };
}

/// Helper macro to construct confined [`HashSet`](std::collections::HashSet)
/// of a [`TinyHashSet`](crate::TinyHashSet) type
#[macro_export]
#[cfg(feature = "std")]
macro_rules! tiny_set {
    () => {
        $crate::TinyHashSet::new()
    };
    ($($x:expr),+ $(,)?) => (
        $crate::TinyHashSet::try_from_iter([$($x,)+])
            .expect("inline tiny_set literal contains invalid number of items")
    )
}

/// Helper macro to construct confined [`HashSet`](std::collections::HashSet)
/// of a [`SmallHashSet`](crate::SmallHashSet) type
#[macro_export]
#[cfg(feature = "std")]
macro_rules! small_set {
    () => {
        $crate::SmallHashSet::new()
    };
    ($($x:expr),+ $(,)?) => (
        $crate::SmallHashSet::try_from_iter([$($x,)+])
            .expect("inline small_set literal contains invalid number of items")
    )
}

/// Helper macro to construct confined [`HashSet`](std::collections::HashSet)
/// of a [`MediumHashSet`](crate::MediumHashSet) type
#[macro_export]
#[cfg(feature = "std")]
macro_rules! medium_set {
    () => {
        $crate::MediumHashSet::new()
    };
    ($($x:expr),+ $(,)?) => (
        $crate::MediumHashSet::try_from_iter([$($x,)+])
            .expect("inline medium_set literal contains invalid number of items")
    )
}

/// Helper macro to construct confined
/// [`BTreeSet`](alloc::collections::BTreeSet) of a
/// [`TinyOrdSet`](crate::TinyOrdSet) type
#[macro_export]
macro_rules! tiny_bset {
    () => {
        $crate::TinyOrdSet::new()
    };
    ($($x:expr),+ $(,)?) => (
        $crate::TinyOrdSet::try_from_iter([$($x,)+])
            .expect("inline tiny_bset literal contains invalid number of items")
    )
}

/// Helper macro to construct confined
/// [`BTreeSet`](alloc::collections::BTreeSet) of a
/// [`SmallOrdSet`](crate::SmallOrdSet) type
#[macro_export]
macro_rules! small_bset {
    () => {
        $crate::SmallOrdSet::new()
    };
    ($($x:expr),+ $(,)?) => (
        $crate::SmallOrdSet::try_from_iter([$($x,)+])
            .expect("inline small_bset literal contains invalid number of items")
    )
}

/// Helper macro to construct confined
/// [`BTreeSet`](alloc::collections::BTreeSet) of a
/// [`MediumOrdSet`](crate::MediumOrdSet) type
#[macro_export]
macro_rules! medium_bset {
    () => {
        $crate::MediumOrdSet::new()
    };
    ($($x:expr),+ $(,)?) => (
        $crate::MediumOrdSet::try_from_iter([$($x,)+])
            .expect("inline medium_bset literal contains invalid number of items")
    )
}

/// Helper macro to construct confined [`HashMap`](std::collections::HashMap)
/// of a [`TinyHashMap`](crate::TinyHashMap) type
#[macro_export]
#[cfg(feature = "std")]
macro_rules! tiny_map {
    () => {
        $crate::TinyHashMap::new()
    };
    { $($key:expr => $value:expr),+ $(,)? } => {
        $crate::TinyHashMap::try_from_iter([$(($key, $value)),+])
            .expect("inline tiny_map literal contains invalid number of items")
    }
}

/// Helper macro to construct confined [`HashMap`](std::collections::HashMap)
/// of a [`SmallHashMap`](crate::SmallHashMap) type
#[macro_export]
#[cfg(feature = "std")]
macro_rules! small_map {
    () => {
        $crate::SmallHashMap::new()
    };
    { $($key:expr => $value:expr),+ $(,)? } => {
        $crate::SmallHashMap::try_from_iter([$(($key, $value)),+])
            .expect("inline small_map literal contains invalid number of items")
    }
}

/// Helper macro to construct confined [`HashMap`](std::collections::HashMap)
/// of a [`MediumHashMap`](crate::MediumHashMap) type
#[macro_export]
#[cfg(feature = "std")]
macro_rules! medium_map {
    () => {
        $crate::MediumHashMap::new()
    };
    { $($key:expr => $value:expr),+ $(,)? } => {
        $crate::MediumHashMap::try_from_iter([$(($key, $value)),+])
            .expect("inline medium_map literal contains invalid number of items")
    }
}

/// Helper macro to construct confined
/// [`BTreeMap`](alloc::collections::BTreeMap) of a
/// [`TinyOrdMap`](crate::TinyOrdMap) type
#[macro_export]
macro_rules! tiny_bmap {
    () => {
        $crate::TinyOrdMap::new()
    };
    { $($key:expr => $value:expr),+ $(,)? } => {
        $crate::TinyOrdMap::try_from_iter([$(($key, $value)),+])
            .expect("inline tiny_bmap literal contains invalid number of items")
    }
}

/// Helper macro to construct confined
/// [`BTreeMap`](alloc::collections::BTreeMap) of a
/// [`SmallOrdMap`](crate::SmallOrdMap) type
#[macro_export]
macro_rules! small_bmap {
    () => {
        $crate::SmallOrdMap::new()
    };
    { $($key:expr => $value:expr),+ $(,)? } => {
        $crate::SmallOrdMap::try_from_iter([$(($key, $value)),+])
            .expect("inline small_bmap literal contains invalid number of items")
    }
}

/// Helper macro to construct confined
/// [`BTreeMap`](alloc::collections::BTreeMap) of a
/// [`MediumOrdMap`](crate::MediumOrdMap) type
#[macro_export]
macro_rules! medium_bmap {
    () => {
        $crate::MediumOrdMap::new()
    };
    { $($key:expr => $value:expr),+ $(,)? } => {
        $crate::MediumOrdMap::try_from_iter([$(($key, $value)),+])
            .expect("inline medium_bmap literal contains invalid number of items")
    }
}

/// Helper macro to construct confined [`indexmap::IndexSet`] of a
/// [`TinyIndexSet`](crate::TinyIndexSet) type
#[macro_export]
#[cfg(all(feature = "indexmap", feature = "std"))]
macro_rules! tiny_iset {
    () => {
        $crate::TinyIndexSet::new()
    };
    ($($x:expr),+ $(,)?) => (
        $crate::TinyIndexSet::try_from_iter([$($x,)+])
            .expect("inline tiny_iset literal contains invalid number of items")
    )
}

/// Helper macro to construct confined [`indexmap::IndexSet`] of a
/// [`SmallIndexSet`](crate::SmallIndexSet) type
#[macro_export]
#[cfg(all(feature = "indexmap", feature = "std"))]
macro_rules! small_iset {
    () => {
        $crate::SmallIndexSet::new()
    };
    ($($x:expr),+ $(,)?) => (
        $crate::SmallIndexSet::try_from_iter([$($x,)+])
            .expect("inline small_iset literal contains invalid number of items")
    )
}

/// Helper macro to construct confined [`indexmap::IndexSet`] of a
/// [`MediumIndexSet`](crate::MediumIndexSet) type
#[macro_export]
#[cfg(all(feature = "indexmap", feature = "std"))]
macro_rules! medium_iset {
    () => {
        $crate::MediumIndexSet::new()
    };
    ($($x:expr),+ $(,)?) => (
        $crate::MediumIndexSet::try_from_iter([$($x,)+])
            .expect("inline medium_iset literal contains invalid number of items")
    )
}

/// Helper macro to construct confined [`indexmap::IndexMap`] of a
/// [`TinyIndexMap`](crate::TinyIndexMap) type
#[macro_export]
#[cfg(all(feature = "indexmap", feature = "std"))]
macro_rules! tiny_imap {
    () => {
        $crate::TinyIndexMap::new()
    };
    { $($key:expr => $value:expr),+ $(,)? } => {
        $crate::TinyIndexMap::try_from_iter([$(($key, $value)),+])
            .expect("inline tiny_imap literal contains invalid number of items")
    }
}

/// Helper macro to construct confined [`indexmap::IndexMap`] of a
/// [`SmallIndexMap`](crate::SmallIndexMap) type
#[macro_export]
#[cfg(all(feature = "indexmap", feature = "std"))]
macro_rules! small_imap {
    () => {
        $crate::SmallIndexMap::new()
    };
    { $($key:expr => $value:expr),+ $(,)? } => {
        $crate::SmallIndexMap::try_from_iter([$(($key, $value)),+])
            .expect("inline small_imap literal contains invalid number of items")
    }
}

/// Helper macro to construct confined [`indexmap::IndexMap`] of a
/// [`MediumIndexMap`](crate::MediumIndexMap) type
#[macro_export]
#[cfg(all(feature = "indexmap", feature = "std"))]
macro_rules! medium_imap {
    () => {
        $crate::MediumIndexMap::new()
    };
    { $($key:expr => $value:expr),+ $(,)? } => {
        $crate::MediumIndexMap::try_from_iter([$(($key, $value)),+])
            .expect("inline medium_imap literal contains invalid number of items")
    }
}
