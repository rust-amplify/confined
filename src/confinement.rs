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

//! Confinement puts a constraint on the number of elements within a collection.

use alloc::borrow::ToOwned;
use alloc::collections::vec_deque::Drain;
use alloc::collections::{BTreeMap, BTreeSet, VecDeque, btree_map};
use alloc::string::String;
use alloc::vec::Vec;
use core::borrow::{Borrow, BorrowMut};
use core::fmt::{self, Display, Formatter, LowerHex, UpperHex};
use core::hash::Hash;
use core::ops::{Deref, Index, IndexMut, RangeBounds};
use core::slice::SliceIndex;
use core::str::FromStr;
#[cfg(feature = "std")]
use std::{
    collections::{HashMap, HashSet, hash_map},
    io,
};

use amplify_num::hex;
use amplify_num::hex::{FromHex, ToHex};
use ascii::{AsAsciiStrError, AsciiChar, AsciiString};
#[cfg(feature = "indexmap")]
pub use indexmap;
#[cfg(feature = "smallvec")]
pub use smallvec;

use crate::num::u24;

// Type aliases

/// [`String`] with maximum 255 characters.
pub type TinyString = Confined<String, ZERO, U8>;
/// [`String`] with maximum 2^16-1 characters.
pub type SmallString = Confined<String, ZERO, U16>;
/// [`String`] with maximum 2^24-1 characters.
pub type MediumString = Confined<String, ZERO, U24>;
/// [`String`] with maximum 2^32-1 characters.
pub type LargeString = Confined<String, ZERO, U32>;
/// Confined [`String`].
pub type ConfinedString<const MIN: usize = 0, const MAX: usize = U64> = Confined<String, MIN, MAX>;
/// [`String`] which contains at least a single character.
pub type NonEmptyString<const MAX: usize = U64> = Confined<String, ONE, MAX>;

/// [`AsciiString`] with maximum 255 characters.
pub type TinyAscii = Confined<AsciiString, ZERO, U8>;
/// [`AsciiString`] with maximum 2^16-1 characters.
pub type SmallAscii = Confined<AsciiString, ZERO, U16>;
/// [`AsciiString`] with maximum 2^24-1 characters.
pub type MediumAscii = Confined<AsciiString, ZERO, U24>;
/// [`AsciiString`] with maximum 2^32-1 characters.
pub type LargeAscii = Confined<AsciiString, ZERO, U32>;
/// Confined [`AsciiString`].
pub type ConfinedAscii<const MIN: usize = 0, const MAX: usize = U64> =
    Confined<AsciiString, MIN, MAX>;
/// [`AsciiString`] which contains at least a single character.
pub type NonEmptyAscii<const MAX: usize = U64> = Confined<AsciiString, ONE, MAX>;

/// [`Vec<u8>`] with maximum 255 characters.
pub type TinyBlob = Confined<Vec<u8>, ZERO, U8>;
/// [`Vec<u8>`] with maximum 2^16-1 characters.
pub type SmallBlob = Confined<Vec<u8>, ZERO, U16>;
/// [`Vec<u8>`] with maximum 2^24-1 characters.
pub type MediumBlob = Confined<Vec<u8>, ZERO, U24>;
/// [`Vec<u8>`] with maximum 2^32-1 characters.
pub type LargeBlob = Confined<Vec<u8>, ZERO, U32>;
/// Confined [`Vec<u8>`].
pub type ConfinedBlob<const MIN: usize = 0, const MAX: usize = U64> = Confined<Vec<u8>, MIN, MAX>;
/// [`Vec<u8>`] which contains at least a single character.
pub type NonEmptyBlob<const MAX: usize = U64> = Confined<Vec<u8>, ONE, MAX>;

/// [`Vec`] with maximum 255 items of type `T`.
pub type TinyVec<T> = Confined<Vec<T>, ZERO, U8>;
/// [`Vec`] with maximum 2^16-1 items of type `T`.
pub type SmallVec<T> = Confined<Vec<T>, ZERO, U16>;
/// [`Vec`] with maximum 2^24-1 items of type `T`.
pub type MediumVec<T> = Confined<Vec<T>, ZERO, U24>;
/// [`Vec`] with maximum 2^32-1 items of type `T`.
pub type LargeVec<T> = Confined<Vec<T>, ZERO, U32>;
/// Confined [`Vec`].
pub type ConfinedVec<T, const MIN: usize = 0, const MAX: usize = U64> = Confined<Vec<T>, MIN, MAX>;
/// [`Vec`] which contains at least a single item.
pub type NonEmptyVec<T, const MAX: usize = U64> = Confined<Vec<T>, ONE, MAX>;

/// [`VecDeque`] with maximum 255 items of type `T`.
pub type TinyDeque<T> = Confined<VecDeque<T>, ZERO, U8>;
/// [`VecDeque`] with maximum 2^16-1 items of type `T`.
pub type SmallDeque<T> = Confined<VecDeque<T>, ZERO, U16>;
/// [`VecDeque`] with maximum 2^24-1 items of type `T`.
pub type MediumDeque<T> = Confined<VecDeque<T>, ZERO, U24>;
/// [`VecDeque`] with maximum 2^32-1 items of type `T`.
pub type LargeDeque<T> = Confined<VecDeque<T>, ZERO, U32>;
/// Confined [`VecDeque`].
pub type ConfinedDeque<T, const MIN: usize = 0, const MAX: usize = U64> =
    Confined<VecDeque<T>, MIN, MAX>;
/// [`VecDeque`] which contains at least a single item.
pub type NonEmptyDeque<T, const MAX: usize = U64> = Confined<VecDeque<T>, ONE, MAX>;

/// [`HashSet`] with maximum 255 items of type `T`.
#[cfg(feature = "std")]
pub type TinyHashSet<T> = Confined<HashSet<T>, ZERO, U8>;
/// [`HashSet`] with maximum 2^16-1 items of type `T`.
#[cfg(feature = "std")]
pub type SmallHashSet<T> = Confined<HashSet<T>, ZERO, U16>;
/// [`HashSet`] with maximum 2^24-1 items of type `T`.
#[cfg(feature = "std")]
pub type MediumHashSet<T> = Confined<HashSet<T>, ZERO, U24>;
/// [`HashSet`] with maximum 2^32-1 items of type `T`.
#[cfg(feature = "std")]
pub type LargeHashSet<T> = Confined<HashSet<T>, ZERO, U32>;
#[cfg(feature = "std")]
/// Confined [`HashSet`].
pub type ConfinedHashSet<T, const MIN: usize = 0, const MAX: usize = U64> =
    Confined<HashSet<T>, MIN, MAX>;
/// [`HashSet`] which contains at least a single item.
#[cfg(feature = "std")]
pub type NonEmptyHashSet<T, const MAX: usize = U64> = Confined<HashSet<T>, ONE, MAX>;

/// [`BTreeSet`] with maximum 255 items of type `T`.
pub type TinyOrdSet<T> = Confined<BTreeSet<T>, ZERO, U8>;
/// [`BTreeSet`] with maximum 2^16-1 items of type `T`.
pub type SmallOrdSet<T> = Confined<BTreeSet<T>, ZERO, U16>;
/// [`BTreeSet`] with maximum 2^24-1 items of type `T`.
pub type MediumOrdSet<T> = Confined<BTreeSet<T>, ZERO, U24>;
/// [`BTreeSet`] with maximum 2^32-1 items of type `T`.
pub type LargeOrdSet<T> = Confined<BTreeSet<T>, ZERO, U32>;
/// Confined [`BTreeSet`].
pub type ConfinedOrdSet<T, const MIN: usize = 0, const MAX: usize = U64> =
    Confined<BTreeSet<T>, MIN, MAX>;
/// [`BTreeSet`] which contains at least a single item.
pub type NonEmptyOrdSet<T, const MAX: usize = U64> = Confined<BTreeSet<T>, ONE, MAX>;

/// [`HashMap`] with maximum 255 items.
#[cfg(feature = "std")]
pub type TinyHashMap<K, V> = Confined<HashMap<K, V>, ZERO, U8>;
/// [`HashMap`] with maximum 2^16-1 items.
#[cfg(feature = "std")]
pub type SmallHashMap<K, V> = Confined<HashMap<K, V>, ZERO, U16>;
/// [`HashMap`] with maximum 2^24-1 items.
#[cfg(feature = "std")]
pub type MediumHashMap<K, V> = Confined<HashMap<K, V>, ZERO, U24>;
/// [`HashMap`] with maximum 2^32-1 items.
#[cfg(feature = "std")]
pub type LargeHashMap<K, V> = Confined<HashMap<K, V>, ZERO, U32>;
#[cfg(feature = "std")]
/// Confined [`HashMap`].
pub type ConfinedHashMap<K, V, const MIN: usize = 0, const MAX: usize = U64> =
    Confined<HashSet<K, V>, MIN, MAX>;
/// [`HashMap`] which contains at least a single item.
#[cfg(feature = "std")]
pub type NonEmptyHashMap<K, V, const MAX: usize = U64> = Confined<HashMap<K, V>, ONE, MAX>;

/// [`BTreeMap`] with maximum 255 items.
pub type TinyOrdMap<K, V> = Confined<BTreeMap<K, V>, ZERO, U8>;
/// [`BTreeMap`] with maximum 2^16-1 items.
pub type SmallOrdMap<K, V> = Confined<BTreeMap<K, V>, ZERO, U16>;
/// [`BTreeMap`] with maximum 2^24-1 items.
pub type MediumOrdMap<K, V> = Confined<BTreeMap<K, V>, ZERO, U24>;
/// [`BTreeMap`] with maximum 2^32-1 items.
pub type LargeOrdMap<K, V> = Confined<BTreeMap<K, V>, ZERO, U32>;
/// Confined [`BTreeMap`].
pub type ConfinedOrdMap<K, V, const MIN: usize = 0, const MAX: usize = U64> =
    Confined<BTreeMap<K, V>, MIN, MAX>;
/// [`BTreeMap`] which contains at least a single item.
pub type NonEmptyOrdMap<K, V, const MAX: usize = U64> = Confined<BTreeMap<K, V>, ONE, MAX>;

/// [`indexmap::IndexSet`] with maximum 255 items of type `T`.
#[cfg(feature = "indexmap")]
pub type TinyIndexSet<T> = Confined<indexmap::IndexSet<T>, ZERO, U8>;
/// [`indexmap::IndexSet`] with maximum 2^16-1 items of type `T`.
#[cfg(feature = "indexmap")]
pub type SmallIndexSet<T> = Confined<indexmap::IndexSet<T>, ZERO, U16>;
/// [`indexmap::IndexSet`] with maximum 2^24-1 items of type `T`.
#[cfg(feature = "indexmap")]
pub type MediumIndexSet<T> = Confined<indexmap::IndexSet<T>, ZERO, U24>;
/// [`indexmap::IndexSet`] with maximum 2^32-1 items of type `T`.
#[cfg(feature = "indexmap")]
pub type LargeIndexSet<T> = Confined<indexmap::IndexSet<T>, ZERO, U32>;
#[cfg(feature = "indexmap")]
/// Confined [`indexmap::IndexSet`].
pub type ConfinedIndexSet<T, const MIN: usize = 0, const MAX: usize = U64> =
    Confined<indexmap::IndexSet<T>, MIN, MAX>;
/// [`indexmap::IndexSet`] which contains at least a single item.
#[cfg(feature = "indexmap")]
pub type NonEmptyIndexSet<T, const MAX: usize = U64> = Confined<indexmap::IndexSet<T>, ONE, MAX>;

/// [`indexmap::IndexMap`] with maximum 255 items.
#[cfg(feature = "indexmap")]
pub type TinyIndexMap<K, V> = Confined<indexmap::IndexMap<K, V>, ZERO, U8>;
/// [`indexmap::IndexMap`] with maximum 2^16-1 items.
#[cfg(feature = "indexmap")]
pub type SmallIndexMap<K, V> = Confined<indexmap::IndexMap<K, V>, ZERO, U16>;
/// [`indexmap::IndexMap`] with maximum 2^24-1 items.
#[cfg(feature = "indexmap")]
pub type MediumIndexMap<K, V> = Confined<indexmap::IndexMap<K, V>, ZERO, U24>;
/// [`indexmap::IndexMap`] with maximum 2^32-1 items.
#[cfg(feature = "indexmap")]
pub type LargeIndexMap<K, V> = Confined<indexmap::IndexMap<K, V>, ZERO, U32>;
#[cfg(feature = "indexmap")]
/// Confined [`indexmap::IndexMap`].
pub type ConfinedIndexMap<K, V, const MIN: usize = 0, const MAX: usize = U64> =
    Confined<indexmap::IndexMap<K, V>, MIN, MAX>;
/// [`indexmap::IndexMap`] which contains at least a single item.
#[cfg(feature = "indexmap")]
pub type NonEmptyIndexMap<K, V, const MAX: usize = U64> =
    Confined<indexmap::IndexMap<K, V>, ONE, MAX>;

/// [`smallvec::SmallVec`] with maximum 255 items.
#[cfg(feature = "smallvec")]
pub type TinySmallVec<A> = Confined<smallvec::SmallVec<A>, ZERO, U8>;
/// [`smallvec::SmallVec`] with maximum 2^16-1 items.
#[cfg(feature = "smallvec")]
pub type SmallSmallVec<A> = Confined<smallvec::SmallVec<A>, ZERO, U16>;
/// [`smallvec::SmallVec`] with maximum 2^24-1 items.
#[cfg(feature = "smallvec")]
pub type MediumSmallVec<A> = Confined<smallvec::SmallVec<A>, ZERO, U24>;
/// [`smallvec::SmallVec`] with maximum 2^32-1 items.
#[cfg(feature = "smallvec")]
pub type LargeSmallVec<A> = Confined<smallvec::SmallVec<A>, ZERO, U32>;
#[cfg(feature = "smallvec")]
/// Confined [`smallvec::SmallVec`].
pub type ConfinedSmallVec<A, const MIN: usize = 0, const MAX: usize = U64> =
    Confined<smallvec::SmallVec<A>, MIN, MAX>;
/// [`smallvec::SmallVec`] which contains at least a single item.
#[cfg(feature = "smallvec")]
pub type NonEmptySmallVec<A, const MAX: usize = U64> = Confined<smallvec::SmallVec<A>, ONE, MAX>;

/// Helper macro to construct confined string
#[macro_export]
#[deprecated(since = "4.7.0", note = "use size-specific macros")]
macro_rules! confined_s {
    () => {
        $crate::confinement::Confined::<String>::new()
    };
    ($s:literal) => {
        $crate::confinement::Confined::try_from(s!($s))
            .expect("inline confined_s literal exceeds confinement length")
            .into()
    };
}

/// Helper macro to construct confined string of a [`TinyString`] type
#[macro_export]
macro_rules! tiny_s {
    () => {
        $crate::confinement::TinyString::new()
    };
    ($lit:literal) => {
        $crate::confinement::TinyString::try_from(s!($lit))
            .expect("static string for tiny_s literal cis too long")
    };
}

/// Helper macro to construct confined string of a [`SmallString`] type
#[macro_export]
macro_rules! small_s {
    () => {
        $crate::confinement::SmallString::new()
    };
    ($lit:literal) => {
        $crate::confinement::SmallString::try_from(s!($lit))
            .expect("static string for small_s literal cis too long")
    };
}

/// Helper macro to construct confined string of a [`MediumString`] type
#[macro_export]
macro_rules! medium_s {
    () => {
        $crate::confinement::MediumString::new()
    };
    ($lit:literal) => {
        $crate::confinement::MediumString::try_from(s!($lit))
            .expect("static string for medium_s literal cis too long")
    };
}

/// Helper macro to construct confined blob
#[macro_export]
#[deprecated(since = "4.7.0", note = "use size-specific macros")]
macro_rules! confined_blob {
    () => {
        $crate::confinement::ConfinedBlob::new()
    };
    ($elem:expr; $n:expr) => (
        $crate::confinement::ConfinedBlob::try_from(vec![$elem; $n])
            .expect("inline confined_blob contains invalid number of items")
    );
    ($($x:expr),+ $(,)?) => (
        $crate::confinement::ConfinedBlob::try_from(vec![$($x,)+])
            .expect("inline confined_blob contains invalid number of items")
            .into()
    )
}

/// Helper macro to construct confined blob of a [`TinyBlob`] type
#[macro_export]
macro_rules! tiny_blob {
    () => {
        $crate::confinement::TinyBlob::new()
    };
    ($elem:expr; $n:expr) => (
        $crate::confinement::TinyBlob::try_from(vec![$elem; $n])
            .expect("inline tiny_blob contains invalid number of items")
    );
    ($($x:expr),+ $(,)?) => (
        $crate::confinement::TinyBlob::try_from(vec![$($x,)+])
            .expect("inline tiny_blob contains invalid number of items")
    )
}

/// Helper macro to construct confined blob of a [`SmallBlob`] type
#[macro_export]
macro_rules! small_blob {
    () => {
        $crate::confinement::SmallBlob::new()
    };
    ($elem:expr; $n:expr) => (
        $crate::confinement::SmallBlob::try_from(vec![$elem; $n])
            .expect("inline small_blob contains invalid number of items")
    );
    ($($x:expr),+ $(,)?) => (
        $crate::confinement::SmallBlob::try_from(vec![$($x,)+])
            .expect("inline small_blob contains invalid number of items")
    )
}

/// Helper macro to construct confined blob of a [`MediumBlob`] type
#[macro_export]
macro_rules! medium_blob {
    () => {
        $crate::confinement::MediumBlob::new()
    };
    ($elem:expr; $n:expr) => (
        $crate::confinement::MediumBlob::try_from(vec![$elem; $n])
            .expect("inline medium_blob contains invalid number of items")
    );
    ($($x:expr),+ $(,)?) => (
        $crate::confinement::MediumBlob::try_from(vec![$($x,)+])
            .expect("inline medium_blob contains invalid number of items")
    )
}

/// Helper macro to construct confined vector of a given type
#[macro_export]
#[deprecated(since = "4.7.0", note = "use size-specific macros")]
macro_rules! confined_vec {
    () => {
        $crate::confinement::Confined::<Vec<_>>::new()
    };
    ($elem:expr; $n:expr) => (
        $crate::confinement::Confined::try_from(vec![$elem; $n])
            .expect("inline confined_vec literal contains invalid number of items")
    );
    ($($x:expr),+ $(,)?) => (
        $crate::confinement::Confined::try_from(vec![$($x,)+])
            .expect("inline confined_vec literal contains invalid number of items")
            .into()
    )
}

/// Helper macro to construct confined vector of a [`TinyVec`] type
#[macro_export]
macro_rules! tiny_vec {
    () => {
        $crate::confinement::TinyVec::new()
    };
    ($elem:expr; $n:expr) => (
        $crate::confinement::TinyVec::try_from(vec![$elem; $n])
            .expect("inline tiny_vec literal contains invalid number of items")
    );
    ($($x:expr),+ $(,)?) => (
        $crate::confinement::TinyVec::try_from(vec![$($x,)+])
            .expect("inline tiny_vec literal contains invalid number of items")
    )
}

/// Helper macro to construct confined vector of a [`SmallVec`] type
#[macro_export]
macro_rules! small_vec {
    () => {
        $crate::confinement::SmallVec::new()
    };
    ($elem:expr; $n:expr) => (
        $crate::confinement::SmallVec::try_from(vec![$elem; $n])
            .expect("inline small_vec literal contains invalid number of items")
    );
    ($($x:expr),+ $(,)?) => (
        $crate::confinement::SmallVec::try_from(vec![$($x,)+])
            .expect("inline small_vec literal contains invalid number of items")
    )
}

/// Helper macro to construct confined vector of a [`MediumVec`] type
#[macro_export]
macro_rules! medium_vec {
    () => {
        $crate::confinement::MediumVec::new()
    };
    ($elem:expr; $n:expr) => (
        $crate::confinement::MediumVec::try_from(vec![$elem; $n])
            .expect("inline medium_vec literal contains invalid number of items")
    );
    ($($x:expr),+ $(,)?) => (
        $crate::confinement::MediumVec::try_from(vec![$($x,)+])
            .expect("inline medium_vec literal contains invalid number of items")
    )
}

/// Helper macro to construct confined [`HashSet`] of a given type
#[macro_export]
#[deprecated(since = "4.7.0", note = "use size-specific macros")]
macro_rules! confined_set {
    () => {
        $crate::confinement::Confined::<HashSet<_>>::new()
    };
    ($($x:expr),+ $(,)?) => (
        $crate::confinement::Confined::try_from(set![$($x,)+])
            .expect("inline confined_set literal contains invalid number of items")
            .into()
    )
}

/// Helper macro to construct confined [`HashSet`] of a [`TinyHashSet`] type
#[macro_export]
macro_rules! tiny_set {
    () => {
        $crate::confinement::TinyHashSet::new()
    };
    ($($x:expr),+ $(,)?) => (
        $crate::confinement::TinyHashSet::try_from(set![$($x,)+])
            .expect("inline tiny_set literal contains invalid number of items")
    )
}

/// Helper macro to construct confined [`HashSet`] of a [`SmallHashSet`] type
#[macro_export]
macro_rules! small_set {
    () => {
        $crate::confinement::SmallHashSet::new()
    };
    ($($x:expr),+ $(,)?) => (
        $crate::confinement::SmallHashSet::try_from(set![$($x,)+])
            .expect("inline small_set literal contains invalid number of items")
    )
}

/// Helper macro to construct confined [`HashSet`] of a [`MediumHashSet`] type
#[macro_export]
macro_rules! medium_set {
    () => {
        $crate::confinement::MediumHashSet::new()
    };
    ($($x:expr),+ $(,)?) => (
        $crate::confinement::MediumHashSet::try_from(set![$($x,)+])
            .expect("inline medium_set literal contains invalid number of items")
    )
}

/// Helper macro to construct confined [`BTreeSet`] of a given type
#[macro_export]
#[deprecated(since = "4.7.0", note = "use size-specific macros")]
macro_rules! confined_bset {
    () => {
        $crate::confinement::Confined::<BTreeSet<_>>::new()
    };
    ($($x:expr),+ $(,)?) => (
        $crate::confinement::Confined::try_from(bset![$($x,)+])
            .expect("inline confined_bset literal contains invalid number of items")
            .into()
    )
}

/// Helper macro to construct confined [`BTreeSet`] of a [`TinyOrdSet`] type
#[macro_export]
macro_rules! tiny_bset {
    () => {
        $crate::confinement::TinyOrdSet::new()
    };
    ($($x:expr),+ $(,)?) => (
        $crate::confinement::TinyOrdSet::try_from(bset![$($x,)+])
            .expect("inline tiny_bset literal contains invalid number of items")
    )
}

/// Helper macro to construct confined [`BTreeSet`] of a [`SmallOrdSet`] type
#[macro_export]
macro_rules! small_bset {
    () => {
        $crate::confinement::SmallOrdSet::new()
    };
    ($($x:expr),+ $(,)?) => (
        $crate::confinement::SmallOrdSet::try_from(bset![$($x,)+])
            .expect("inline small_bset literal contains invalid number of items")
    )
}

/// Helper macro to construct confined [`BTreeSet`] of a [`MediumOrdSet`] type
#[macro_export]
macro_rules! medium_bset {
    () => {
        $crate::confinement::MediumOrdSet::new()
    };
    ($($x:expr),+ $(,)?) => (
        $crate::confinement::MediumOrdSet::try_from(bset![$($x,)+])
            .expect("inline medium_bset literal contains invalid number of items")
    )
}

/// Helper macro to construct confined [`HashMap`] of a given type
#[macro_export]
#[deprecated(since = "4.7.0", note = "use size-specific macros")]
macro_rules! confined_map {
    () => {
        $crate::confinement::Confined::<HashMap<_, _>>::new()
    };
    ($($key:expr => $value:expr),+ $(,)?) => (
        $crate::confinement::Confined::try_from(map!{ $($key => $value),+ })
            .expect("inline confined_map literal contains invalid number of items")
            .into()
    )
}

/// Helper macro to construct confined [`HashMap`] of a [`TinyHashMap`] type
#[macro_export]
macro_rules! tiny_map {
    () => {
        $crate::confinement::TinyHashMap::new()
    };
    { $($key:expr => $value:expr),+ $(,)? } => {
        $crate::confinement::TinyHashMap::try_from(map!{ $($key => $value,)+ })
            .expect("inline tiny_map literal contains invalid number of items")
    }
}

/// Helper macro to construct confined [`HashMap`] of a [`SmallHashMap`] type
#[macro_export]
macro_rules! small_map {
    () => {
        $crate::confinement::SmallHashMap::new()
    };
    { $($key:expr => $value:expr),+ $(,)? } => {
        $crate::confinement::SmallHashMap::try_from(map!{ $($key => $value,)+ })
            .expect("inline small_map literal contains invalid number of items")
    }
}

/// Helper macro to construct confined [`HashMap`] of a [`MediumHashMap`] type
#[macro_export]
macro_rules! medium_map {
    () => {
        $crate::confinement::MediumHashMap::new()
    };
    { $($key:expr => $value:expr),+ $(,)? } => {
        $crate::confinement::MediumHashMap::try_from(map!{ $($key => $value,)+ })
            .expect("inline medium_map literal contains invalid number of items")
    }
}

/// Helper macro to construct confined [`BTreeMap`] of a given type
#[macro_export]
#[deprecated(since = "4.7.0", note = "use size-specific macros")]
macro_rules! confined_bmap {
    () => {
        $crate::confinement::Confined::<BTreeMap<_, _>>::new()
    };
    ($($key:expr => $value:expr),+ $(,)?) => (
        $crate::confinement::Confined::try_from(bmap!{ $($key => $value),+ })
            .expect("inline confined_bmap literal contains invalid number of items")
            .into()
    )
}

/// Helper macro to construct confined [`BTreeMap`] of a [`TinyOrdMap`] type
#[macro_export]
macro_rules! tiny_bmap {
    () => {
        $crate::confinement::TinyOrdMap::new()
    };
    { $($key:expr => $value:expr),+ $(,)? } => {
        $crate::confinement::TinyOrdMap::try_from(bmap!{ $($key => $value,)+ })
            .expect("inline tiny_bmap literal contains invalid number of items")
    }
}

/// Helper macro to construct confined [`BTreeMap`] of a [`SmallOrdMap`] type
#[macro_export]
macro_rules! small_bmap {
    () => {
        $crate::confinement::SmallOrdMap::new()
    };
    { $($key:expr => $value:expr),+ $(,)? } => {
        $crate::confinement::SmallOrdMap::try_from(bmap!{ $($key => $value,)+ })
            .expect("inline small_bmap literal contains invalid number of items")
    }
}

/// Helper macro to construct confined [`BTreeMap`] of a [`MediumOrdMap`] type
#[macro_export]
macro_rules! medium_bmap {
    () => {
        $crate::confinement::MediumOrdMap::new()
    };
    { $($key:expr => $value:expr),+ $(,)? } => {
        $crate::confinement::MediumOrdMap::try_from(bmap!{ $($key => $value,)+ })
            .expect("inline medium_bmap literal contains invalid number of items")
    }
}

/// Helper macro to construct confined [`indexmap::IndexMap`] of a
/// [`TinyIndexMap`] type
#[macro_export]
#[cfg(feature = "indexmap")]
macro_rules! tiny_imap {
    () => {
        $crate::confinement::TinyIndexMap::new()
    };
    { $($key:expr => $value:expr),+ $(,)? } => {
        $crate::confinement::TinyIndexMap::try_from(::core::iter::FromIterator::from_iter([$(($key, $value)),+]))
            .expect("inline tiny_imap literal contains invalid number of items")
    }
}

/// Helper macro to construct confined [`indexmap::IndexMap`] of a
/// [`SmallIndexMap`] type
#[macro_export]
#[cfg(feature = "indexmap")]
macro_rules! small_imap {
    () => {
        $crate::confinement::SmallIndexMap::new()
    };
    { $($key:expr => $value:expr),+ $(,)? } => {
        $crate::confinement::SmallIndexMap::try_from(::core::iter::FromIterator::from_iter([$(($key, $value)),+]))
            .expect("inline small_imap literal contains invalid number of items")
    }
}

/// Helper macro to construct confined [`indexmap::IndexMap`] of a
/// [`MediumIndexMap`] type
#[macro_export]
#[cfg(feature = "indexmap")]
macro_rules! medium_imap {
    () => {
        $crate::confinement::MediumIndexMap::new()
    };
    { $($key:expr => $value:expr),+ $(,)? } => {
        $crate::confinement::MediumIndexMap::try_from(::core::iter::FromIterator::from_iter([$(($key, $value)),+]))
            .expect("inline medium_imap literal contains invalid number of items")
    }
}

/// Helper macro to construct confined [`indexmap::IndexSet`] of a
/// [`TinyIndexSet`] type
#[macro_export]
#[cfg(feature = "indexmap")]
macro_rules! tiny_iset {
    () => {
        $crate::confinement::TinyIndexSet::new()
    };
    ($($x:expr),+ $(,)?) => (
        $crate::confinement::TinyIndexSet::try_from(::core::iter::FromIterator::from_iter([$($x,)+]))
            .expect("inline tiny_iset literal contains invalid number of items")
    )
}

/// Helper macro to construct confined [`indexmap::IndexSet`] of a
/// [`SmallIndexSet`] type
#[macro_export]
#[cfg(feature = "indexmap")]
macro_rules! small_iset {
    () => {
        $crate::confinement::SmallIndexSet::new()
    };
    ($($x:expr),+ $(,)?) => (
        $crate::confinement::SmallIndexSet::try_from(::core::iter::FromIterator::from_iter([$($x,)+]))
            .expect("inline small_iset literal contains invalid number of items")
    )
}

/// Helper macro to construct confined [`indexmap::IndexSet`] of a
/// [`MediumIndexSet`] type
#[macro_export]
#[cfg(feature = "indexmap")]
macro_rules! medium_iset {
    () => {
        $crate::confinement::MediumIndexSet::new()
    };
    ($($x:expr),+ $(,)?) => (
        $crate::confinement::MediumIndexSet::try_from(::core::iter::FromIterator::from_iter([$($x,)+]))
            .expect("inline medium_iset literal contains invalid number of items")
    )
}

/// Helper macro to construct confined [`smallvec::SmallVec`] of a
/// [`TinySmallVec`] type
#[macro_export]
#[cfg(feature = "smallvec")]
macro_rules! tiny_svec {
    () => {
        $crate::confinement::TinySmallVec::new()
    };
    ($($x:expr),+ $(,)?) => (
        $crate::confinement::TinySmallVec::try_from(::core::iter::FromIterator::from_iter([$($x,)+]))
            .expect("inline tiny_svec literal contains invalid number of items")
    )
}

/// Helper macro to construct confined [`smallvec::SmallVec`] of a
/// [`SmallSmallVec`] type
#[macro_export]
#[cfg(feature = "smallvec")]
macro_rules! small_svec {
    () => {
        $crate::confinement::SmallSmallVec::new()
    };
    ($($x:expr),+ $(,)?) => (
        $crate::confinement::SmallSmallVec::try_from(::core::iter::FromIterator::from_iter([$($x,)+]))
            .expect("inline small_svec literal contains invalid number of items")
    )
}

/// Helper macro to construct confined [`smallvec::SmallVec`] of a
/// [`MediumSmallVec`] type
#[macro_export]
#[cfg(feature = "smallvec")]
macro_rules! medium_svec {
    () => {
        $crate::confinement::MediumSmallVec::new()
    };
    ($($x:expr),+ $(,)?) => (
        $crate::confinement::MediumSmallVec::try_from(::core::iter::FromIterator::from_iter([$($x,)+]))
            .expect("inline medium_svec literal contains invalid number of items")
    )
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn fits_max() {
        let mut s = TinyString::new();
        assert!(s.is_empty());
        for _ in 1..=255 {
            s.push('a').unwrap();
        }
        assert_eq!(s.len_u8(), u8::MAX);
        assert_eq!(s.len_u8(), s.len() as u8);
        assert!(!s.is_empty());

        let mut vec = TinyVec::new();
        let mut deque = TinyDeque::new();
        let mut set = TinyHashSet::new();
        let mut bset = TinyOrdSet::new();
        #[cfg(feature = "smallvec")]
        let mut svec = TinySmallVec::<[u8; 8]>::new();
        let mut map = TinyHashMap::new();
        let mut bmap = TinyOrdMap::new();
        #[cfg(feature = "indexmap")]
        let mut imap = TinyIndexMap::new();
        #[cfg(feature = "indexmap")]
        let mut iset = TinyIndexSet::new();
        assert!(vec.is_empty());
        assert!(deque.is_empty());
        assert!(set.is_empty());
        assert!(bset.is_empty());
        assert!(map.is_empty());
        assert!(bmap.is_empty());
        #[cfg(feature = "indexmap")]
        assert!(imap.is_empty());
        #[cfg(feature = "indexmap")]
        assert!(iset.is_empty());
        for index in 1..=255 {
            vec.push(5u8).unwrap();
            deque.push(5u8).unwrap();
            set.push(index).unwrap();
            bset.push(5u8).unwrap();
            #[cfg(feature = "smallvec")]
            svec.push(5u8).unwrap();
            map.insert(5u8, 'a').unwrap();
            bmap.insert(index, 'a').unwrap();
            #[cfg(feature = "indexmap")]
            imap.insert(index, 'a').unwrap();
            #[cfg(feature = "indexmap")]
            iset.push(index).unwrap();
        }
        assert_eq!(vec.len_u8(), u8::MAX);
        assert_eq!(deque.len_u8(), u8::MAX);
        assert_eq!(set.len_u8(), u8::MAX);
        assert_eq!(bset.len_u8(), 1);
        #[cfg(feature = "smallvec")]
        assert_eq!(svec.len_u8(), 255);
        assert_eq!(map.len_u8(), 1);
        assert_eq!(bmap.len_u8(), u8::MAX);
        #[cfg(feature = "indexmap")]
        assert_eq!(imap.len_u8(), u8::MAX);
        #[cfg(feature = "indexmap")]
        assert_eq!(iset.len_u8(), u8::MAX);

        vec.clear();
        assert!(vec.is_empty());
    }

    #[test]
    fn cant_go_above_max() {
        let mut s = TinyString::new();
        for _ in 1..=255 {
            s.push('a').unwrap();
        }
        assert!(matches!(
            s.push('a'),
            Err(Error::Oversize {
                len: 255,
                max_len: 255
            })
        ));

        let mut v = TinyVec::<u8>::new();
        for _ in 1..=255 {
            v.push(1).unwrap();
        }
        assert!(matches!(
            v.push(1),
            Err(Error::Oversize {
                len: 255,
                max_len: 255
            })
        ));

        let mut set = TinyOrdSet::<u8>::new();
        for i in 1..=255 {
            set.push(i).unwrap();
        }
        assert!(matches!(
            set.push(255),
            Err(Error::Oversize {
                len: 255,
                max_len: 255
            })
        ));
        assert!(matches!(
            set.push(0),
            Err(Error::Oversize {
                len: 255,
                max_len: 255
            })
        ));

        let mut map = TinyOrdMap::<u8, u8>::new();
        for i in 1..=255 {
            map.insert(i, i).unwrap();
        }
        // Replacing an existing key should succeed
        assert_eq!(map.insert(255, 0), Ok(Some(255)));
        assert!(matches!(
            map.insert(0, 0),
            Err(Error::Oversize {
                len: 255,
                max_len: 255
            })
        ));
    }

    #[test]
    fn boundary() {
        let mut s = TinyString::new();
        s.push('a').unwrap();
        assert!(matches!(
            s.remove(1),
            Err(Error::OutOfBoundary { index: 1, len: 1 })
        ));

        let mut v = TinyVec::<u8>::new();
        v.push(1).unwrap();
        assert!(matches!(
            v.remove(1),
            Err(Error::OutOfBoundary { index: 1, len: 1 })
        ));

        let mut d = TinyDeque::<u8>::new();
        d.push_back(1).unwrap();
        assert!(matches!(
            d.remove(1),
            Err(Error::OutOfBoundary { index: 1, len: 1 })
        ));
    }

    #[test]
    fn cant_go_below_min() {
        let mut s = TinyString::new();
        assert!(matches!(
            s.remove(0),
            Err(Error::OutOfBoundary { index: 0, len: 0 })
        ));

        let mut s = TinyVec::<u8>::new();
        assert!(matches!(
            s.remove(0),
            Err(Error::OutOfBoundary { index: 0, len: 0 })
        ));

        let mut s = TinyOrdSet::<u8>::new();
        assert!(matches!(s.remove(&0), Ok(false)));

        let mut s = TinyOrdMap::<u8, u8>::new();
        assert!(matches!(s.remove(&0), Ok(None)));

        let mut s = NonEmptyString::<U8>::with('a');
        assert!(matches!(
            s.remove(0),
            Err(Error::Undersize { len: 1, min_len: 1 })
        ));
        assert!(matches!(
            s.remove(1),
            Err(Error::OutOfBoundary { index: 1, len: 1 })
        ));

        let mut v = NonEmptyVec::<u8>::with(1);
        assert!(matches!(
            v.remove(0),
            Err(Error::Undersize { len: 1, min_len: 1 })
        ));
        assert!(matches!(
            v.remove(1),
            Err(Error::OutOfBoundary { index: 1, len: 1 })
        ));

        let mut set = NonEmptyOrdSet::<u8>::with(1);
        assert!(matches!(
            set.remove(&1),
            Err(Error::Undersize { len: 1, min_len: 1 })
        ));

        let mut map = NonEmptyOrdMap::<u8, u8>::with_key_value(1, 1);
        assert!(matches!(
            map.remove(&1),
            Err(Error::Undersize { len: 1, min_len: 1 })
        ));

        #[cfg(feature = "smallvec")]
        {
            let mut v = NonEmptySmallVec::<[u8; 1]>::with(1);
            assert!(matches!(
                v.remove(0),
                Err(Error::Undersize { len: 1, min_len: 1 })
            ));
        }
    }

    #[test]
    fn entry() {
        let mut col = tiny_bmap!(0u16 => 'a');
        assert!(matches!(
            col.entry(0).unwrap(),
            btree_map::Entry::Occupied(_)
        ));
        assert!(matches!(col.entry(1).unwrap(), btree_map::Entry::Vacant(_)));
        for idx in 1..u8::MAX {
            col.insert(idx as u16, 'b').unwrap();
        }
        assert!(matches!(
            col.entry(2).unwrap(),
            btree_map::Entry::Occupied(_)
        ));
        assert!(col.entry(256).is_err());
    }

    #[test]
    fn macros() {
        tiny_vec!() as TinyVec<&str>;
        tiny_vec!("a", "b", "c");
        small_vec!("a", "b", "c");
        #[cfg(feature = "smallvec")]
        {
            tiny_svec!() as TinySmallVec<[&str; 3]>;
            tiny_svec!("a", "b", "c") as TinySmallVec<[&str; 3]>;
            small_svec!("a", "b", "c") as SmallSmallVec<[&str; 3]>;
        }

        tiny_set!() as TinyHashSet<&str>;
        tiny_set!("a", "b", "c");
        tiny_bset!() as TinyOrdSet<&str>;
        tiny_bset!("a", "b", "c");
        tiny_map!() as TinyHashMap<&str, u8>;
        tiny_map!("a" => 1, "b" => 2, "c" => 3);
        tiny_bmap!() as TinyOrdMap<&str, u8>;
        tiny_bmap!("a" => 1, "b" => 2, "c" => 3);

        small_set!("a", "b", "c");
        small_bset!("a", "b", "c");
        small_map!("a" => 1, "b" => 2, "c" => 3);
        small_bmap!("a" => 1, "b" => 2, "c" => 3);

        #[cfg(feature = "indexmap")]
        {
            tiny_imap!() as TinyIndexMap<&str, u8>;
            tiny_imap!("a" => 1, "b" => 2, "c" => 3);
            small_imap!("a" => 1, "b" => 2, "c" => 3);

            tiny_iset!() as TinyIndexSet<&str>;
            tiny_iset!("a", "b", "c");
            small_iset!("a", "b", "c");
        }
    }

    #[test]
    fn iter_mut_btree() {
        let mut coll = tiny_bmap!(1 => "one");
        for (_index, item) in &mut coll {
            *item = "two";
        }
        assert_eq!(coll.get(&1), Some(&"two"));
        for (_index, item) in coll.keyed_values_mut() {
            *item = "three";
        }
        assert_eq!(coll.get(&1), Some(&"three"));
        for item in coll.values_mut() {
            *item = "four";
        }
        assert_eq!(coll.get(&1), Some(&"four"));
        *coll.get_mut(&1).unwrap() = "five";
        assert_eq!(coll.get(&1), Some(&"five"));
    }

    #[test]
    #[cfg(feature = "indexmap")]
    fn iter_mut_indexmap() {
        let mut coll = tiny_imap!(1 => "one");
        for (_index, item) in &mut coll {
            *item = "two";
        }
        assert_eq!(coll.get(&1), Some(&"two"));
        for (_index, item) in coll.keyed_values_mut() {
            *item = "three";
        }
        assert_eq!(coll.get(&1), Some(&"three"));
        for item in coll.values_mut() {
            *item = "four";
        }
        assert_eq!(coll.get(&1), Some(&"four"));
        *coll.get_mut(&1).unwrap() = "five";
        assert_eq!(coll.get(&1), Some(&"five"));
    }

    #[test]
    fn first_last_non_empty_vec() {
        let coll = NonEmptyVec::<u8, U8>::try_from(vec![1, 2, 3]).unwrap();
        assert_eq!(*coll.first(), 1);
        assert_eq!(*coll.last(), 3);
    }

    #[test]
    fn first_last_non_empty_deque() {
        let coll = NonEmptyDeque::<u8, U8>::try_from_iter([1, 2, 3]).unwrap();
        assert_eq!(*coll.first(), 1);
        assert_eq!(*coll.last(), 3);
    }

    #[test]
    fn test_index() {
        let mut v = TinyVec::<u8>::new();
        v.push(1u8).unwrap();
        v.push(2u8).unwrap();
        assert_eq!(v[0], 1);
        assert_eq!(v[1], 2);
        v[0] = 3;
        assert_eq!(v[0], 3);
        assert_eq!(&v[0..1], &[3]);

        let mut m = TinyOrdMap::<u8, u8>::new();
        m.insert(1u8, 10u8).unwrap();
        m.insert(2u8, 20u8).unwrap();
        assert_eq!(m[&1], 10);
        assert_eq!(m[&2], 20);
        // m[&1] = 11; // BTreeMap doesn't support IndexMut
    }

    #[test]
    fn btree_pop_first() {
        let mut m = Confined::<BTreeMap<u8, u8>, 1, 3>::try_from_iter([(1, 10), (2, 20)]).unwrap();
        assert_eq!(m.pop_first().unwrap(), Some((1, 10)));
        assert_eq!(m.len(), 1);
        assert!(m.pop_first().is_err()); // Underflow 1 -> 0
    }

    #[test]
    fn btree_pop_last() {
        let mut m = Confined::<BTreeMap<u8, u8>, 1, 3>::try_from_iter([(1, 10), (2, 20)]).unwrap();
        assert_eq!(m.pop_last().unwrap(), Some((2, 20)));
        assert_eq!(m.len(), 1);
        assert!(m.pop_last().is_err()); // Underflow 1 -> 0
    }

    #[test]
    fn btree_remove_entry() {
        let mut m = Confined::<BTreeMap<u8, u8>, 1, 3>::try_from_iter([(1, 10), (2, 20)]).unwrap();
        assert_eq!(m.remove_entry(&2).unwrap(), Some((2, 20)));
        assert_eq!(m.len(), 1);
        assert!(m.remove_entry(&1).is_err()); // Underflow 1 -> 0
        assert_eq!(m.remove_entry(&3).unwrap(), None);
    }

    #[test]
    fn btree_append() {
        let mut m1 = Confined::<BTreeMap<u8, u8>, 0, 3>::try_from_iter([(1, 10)]).unwrap();
        let mut m2 = Confined::<BTreeMap<u8, u8>, 0, 3>::try_from_iter([(2, 20), (3, 30)]).unwrap();
        m1.append(&mut m2).unwrap();
        assert_eq!(m1.len(), 3);
        assert_eq!(m2.len(), 0);

        let mut m3 = Confined::<BTreeMap<u8, u8>, 0, 3>::try_from_iter([(4, 40)]).unwrap();
        assert!(m1.append(&mut m3).is_err()); // Overflow 3 + 1 -> 4
    }

    #[test]
    fn btree_range_mut() {
        let mut m =
            Confined::<BTreeMap<u8, u8>, 0, 3>::try_from_iter([(1, 10), (2, 20), (3, 30)]).unwrap();
        for (_, v) in m.range_mut(2..) {
            *v += 1;
        }
        assert_eq!(m[&2], 21);
        assert_eq!(m[&3], 31);
    }

    #[test]
    fn btree_split_off() {
        let mut m =
            Confined::<BTreeMap<u8, u8>, 1, 5>::try_from_iter([(1, 10), (2, 20), (3, 30), (4, 40)])
                .unwrap();
        let m2: Confined<BTreeMap<u8, u8>, 1, 5> = m.split_off(&3).unwrap();
        assert_eq!(m.len(), 2);
        assert_eq!(m2.len(), 2);
        assert!(m.contains_key(&1));
        assert!(m.contains_key(&2));
        assert!(m2.contains_key(&3));
        assert!(m2.contains_key(&4));

        let mut m =
            Confined::<BTreeMap<u8, u8>, 1, 5>::try_from_iter([(1, 10), (2, 20), (3, 30), (4, 40)])
                .unwrap();
        assert!(m.split_off::<1, 5>(&1).is_err()); // Underflow m: 4 -> 0 (min 1 is NOT OK)
        assert!(m.split_off::<1, 5>(&5).is_err()); // Underflow m2: 4 -> 0 (min
        // 1 is NOT OK)
    }

    #[test]
    fn btree_try_retain() {
        let mut m =
            Confined::<BTreeMap<u8, u8>, 1, 5>::try_from_iter([(1, 10), (2, 20), (3, 30)]).unwrap();
        m.try_retain(|k, _v| *k < 3).unwrap();
        assert_eq!(m.len(), 2);
        assert!(m.try_retain(|k, _v| *k < 1).is_err()); // Underflow 2 -> 0
    }

    #[test]
    fn btree_retain() {
        let mut m =
            Confined::<BTreeMap<u8, u8>, 0, 5>::try_from_iter([(1, 10), (2, 20), (3, 30)]).unwrap();
        m.retain(|k, _v| *k < 2);
        assert_eq!(m.len(), 1);
    }

    #[test]
    #[cfg(feature = "std")]
    fn hash_remove_entry() {
        let mut m = Confined::<HashMap<u8, u8>, 1, 3>::try_from_iter([(1, 10), (2, 20)]).unwrap();
        assert_eq!(m.remove_entry(&2).unwrap(), Some((2, 20)));
        assert_eq!(m.len(), 1);
        assert!(m.remove_entry(&1).is_err()); // Underflow 1 -> 0
        assert_eq!(m.remove_entry(&3).unwrap(), None);
    }

    #[test]
    #[cfg(feature = "std")]
    fn hash_drain() {
        let mut m = Confined::<HashMap<u8, u8>, 0, 3>::try_from_iter([(1, 10), (2, 20)]).unwrap();
        let drained: Vec<_> = m.drain().collect();
        assert_eq!(drained.len(), 2);
        assert_eq!(m.len(), 0);
    }

    #[test]
    #[cfg(feature = "std")]
    fn hash_reserve() {
        let mut m = Confined::<HashMap<u8, u8>, 0, 3>::try_from_iter([(1, 10)]).unwrap();
        m.reserve(2);
        assert!(m.capacity() >= 2);
        m.reserve(3);
        let capacity = m.capacity();
        m.reserve(4);
        assert_eq!(m.capacity(), capacity);
    }

    #[test]
    #[cfg(feature = "std")]
    fn hash_shrink_to_fit() {
        let mut m = Confined::<HashMap<u8, u8>, 0, 10>::try_from_iter([(1, 10)]).unwrap();
        m.reserve(5);
        m.shrink_to_fit();
        assert!(m.capacity() >= 1);
    }

    #[test]
    #[cfg(feature = "std")]
    fn hash_shrink_to() {
        let mut m = Confined::<HashMap<u8, u8>, 0, 10>::try_from_iter([(1, 10)]).unwrap();
        m.reserve(5);
        m.shrink_to(2);
        assert!(m.capacity() >= 2);
    }
}
