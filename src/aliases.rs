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

use alloc::alloc::Global;
use alloc::collections::{BTreeMap, BTreeSet, BinaryHeap, LinkedList, VecDeque};
use alloc::string::String;
use alloc::vec::Vec;
#[cfg(feature = "std")]
use std::collections::{HashMap, HashSet};

#[cfg(feature = "ascii")]
use ascii::AsciiString;

use crate::{Confined, ONE, U8MAX, U16MAX, U24MAX, U32MAX, U64MAX, ZERO};

/// [`String`] with maximum 255 bytes in UTF-8 encoding.
pub type TinyString = Confined<String, ZERO, U8MAX>;
/// [`String`] with maximum 2^16-1 bytes in UTF-8 encoding.
pub type SmallString = Confined<String, ZERO, U16MAX>;
/// [`String`] with maximum 2^24-1 bytes in UTF-8 encoding.
pub type MediumString = Confined<String, ZERO, U24MAX>;
/// [`String`] with maximum 2^32-1 bytes in UTF-8 encoding.
pub type LargeString = Confined<String, ZERO, U32MAX>;
/// Confined [`String`]. The limits are set in bytes of UTF-8 encoding, not in
/// characters.
pub type ConfinedString<const MIN: usize = 0, const MAX: usize = U64MAX> =
    Confined<String, MIN, MAX>;
/// [`String`] which contains at least a single byte.
pub type NonEmptyString<const MAX: usize = U64MAX> = Confined<String, ONE, MAX>;

/// [`AsciiString`] with maximum 255 characters.
#[cfg(feature = "ascii")]
pub type TinyAscii = Confined<AsciiString, ZERO, U8MAX>;
/// [`AsciiString`] with maximum 2^16-1 characters.
#[cfg(feature = "ascii")]
pub type SmallAscii = Confined<AsciiString, ZERO, U16MAX>;
/// [`AsciiString`] with maximum 2^24-1 characters.
#[cfg(feature = "ascii")]
pub type MediumAscii = Confined<AsciiString, ZERO, U24MAX>;
/// [`AsciiString`] with maximum 2^32-1 characters.
#[cfg(feature = "ascii")]
pub type LargeAscii = Confined<AsciiString, ZERO, U32MAX>;
/// Confined [`AsciiString`].
#[cfg(feature = "ascii")]
pub type ConfinedAscii<const MIN: usize = 0, const MAX: usize = U64MAX> =
    Confined<AsciiString, MIN, MAX>;
/// [`AsciiString`] which contains at least a single character.
#[cfg(feature = "ascii")]
pub type NonEmptyAscii<const MAX: usize = U64MAX> = Confined<AsciiString, ONE, MAX>;

/// [`Vec<u8>`] with maximum 255 bytes.
pub type TinyBlob<A = Global> = Confined<Vec<u8, A>, ZERO, U8MAX>;
/// [`Vec<u8>`] with maximum 2^16-1 bytes.
pub type SmallBlob<A = Global> = Confined<Vec<u8, A>, ZERO, U16MAX>;
/// [`Vec<u8>`] with maximum 2^24-1 bytes.
pub type MediumBlob<A = Global> = Confined<Vec<u8, A>, ZERO, U24MAX>;
/// [`Vec<u8>`] with maximum 2^32-1 bytes.
pub type LargeBlob<A = Global> = Confined<Vec<u8, A>, ZERO, U32MAX>;
/// Confined [`Vec<u8>`].
pub type ConfinedBlob<const MIN: usize = 0, const MAX: usize = U64MAX, A = Global> =
    Confined<Vec<u8, A>, MIN, MAX>;
/// [`Vec<u8>`] which contains at least a single byte.
pub type NonEmptyBlob<const MAX: usize = U64MAX, A = Global> = Confined<Vec<u8, A>, ONE, MAX>;

/// [`Vec`] with maximum 255 items of type `T`.
pub type TinyVec<T, A = Global> = Confined<Vec<T, A>, ZERO, U8MAX>;
/// [`Vec`] with maximum 2^16-1 items of type `T`.
pub type SmallVec<T, A = Global> = Confined<Vec<T, A>, ZERO, U16MAX>;
/// [`Vec`] with maximum 2^24-1 items of type `T`.
pub type MediumVec<T, A = Global> = Confined<Vec<T, A>, ZERO, U24MAX>;
/// [`Vec`] with maximum 2^32-1 items of type `T`.
pub type LargeVec<T, A = Global> = Confined<Vec<T, A>, ZERO, U32MAX>;
/// Confined [`Vec`].
pub type ConfinedVec<T, const MIN: usize = 0, const MAX: usize = U64MAX, A = Global> =
    Confined<Vec<T, A>, MIN, MAX>;
/// [`Vec`] which contains at least a single item.
pub type NonEmptyVec<T, const MAX: usize = U64MAX, A = Global> = Confined<Vec<T, A>, ONE, MAX>;

/// [`VecDeque`] with maximum 255 items of type `T`.
pub type TinyDeque<T> = Confined<VecDeque<T>, ZERO, U8MAX>;
/// [`VecDeque`] with maximum 2^16-1 items of type `T`.
pub type SmallDeque<T> = Confined<VecDeque<T>, ZERO, U16MAX>;
/// [`VecDeque`] with maximum 2^24-1 items of type `T`.
pub type MediumDeque<T> = Confined<VecDeque<T>, ZERO, U24MAX>;
/// [`VecDeque`] with maximum 2^32-1 items of type `T`.
pub type LargeDeque<T> = Confined<VecDeque<T>, ZERO, U32MAX>;
/// Confined [`VecDeque`].
pub type ConfinedDeque<T, const MIN: usize = 0, const MAX: usize = U64MAX> =
    Confined<VecDeque<T>, MIN, MAX>;
/// [`VecDeque`] which contains at least a single item.
pub type NonEmptyDeque<T, const MAX: usize = U64MAX> = Confined<VecDeque<T>, ONE, MAX>;

/// [`LinkedList`] with maximum 255 items of type `T`.
pub type TinyList<T> = Confined<LinkedList<T>, ZERO, U8MAX>;
/// [`LinkedList`] with maximum 2^16-1 items of type `T`.
pub type SmallList<T> = Confined<LinkedList<T>, ZERO, U16MAX>;
/// [`LinkedList`] with maximum 2^24-1 items of type `T`.
pub type MediumList<T> = Confined<LinkedList<T>, ZERO, U24MAX>;
/// [`LinkedList`] with maximum 2^32-1 items of type `T`.
pub type LargeList<T> = Confined<LinkedList<T>, ZERO, U32MAX>;
/// Confined [`LinkedList`].
pub type ConfinedList<T, const MIN: usize = 0, const MAX: usize = U64MAX> =
    Confined<LinkedList<T>, MIN, MAX>;
/// [`LinkedList`] which contains at least a single item.
pub type NonEmptyList<T, const MAX: usize = U64MAX> = Confined<LinkedList<T>, ONE, MAX>;

/// [`BinaryHeap`] with maximum 255 items of type `T`.
pub type TinyHeap<T> = Confined<BinaryHeap<T>, ZERO, U8MAX>;
/// [`BinaryHeap`] with maximum 2^16-1 items of type `T`.
pub type SmallHeap<T> = Confined<BinaryHeap<T>, ZERO, U16MAX>;
/// [`BinaryHeap`] with maximum 2^24-1 items of type `T`.
pub type MediumHeap<T> = Confined<BinaryHeap<T>, ZERO, U24MAX>;
/// [`BinaryHeap`] with maximum 2^32-1 items of type `T`.
pub type LargeHeap<T> = Confined<BinaryHeap<T>, ZERO, U32MAX>;
/// Confined [`BinaryHeap`].
pub type ConfinedHeap<T, const MIN: usize = 0, const MAX: usize = U64MAX> =
    Confined<BinaryHeap<T>, MIN, MAX>;
/// [`BinaryHeap`] which contains at least a single item.
pub type NonEmptyHeap<T, const MAX: usize = U64MAX> = Confined<BinaryHeap<T>, ONE, MAX>;

/// [`HashSet`] with maximum 255 items of type `T`.
#[cfg(feature = "std")]
pub type TinyHashSet<T> = Confined<HashSet<T>, ZERO, U8MAX>;
/// [`HashSet`] with maximum 2^16-1 items of type `T`.
#[cfg(feature = "std")]
pub type SmallHashSet<T> = Confined<HashSet<T>, ZERO, U16MAX>;
/// [`HashSet`] with maximum 2^24-1 items of type `T`.
#[cfg(feature = "std")]
pub type MediumHashSet<T> = Confined<HashSet<T>, ZERO, U24MAX>;
/// [`HashSet`] with maximum 2^32-1 items of type `T`.
#[cfg(feature = "std")]
pub type LargeHashSet<T> = Confined<HashSet<T>, ZERO, U32MAX>;
/// Confined [`HashSet`].
#[cfg(feature = "std")]
pub type ConfinedHashSet<T, const MIN: usize = 0, const MAX: usize = U64MAX> =
    Confined<HashSet<T>, MIN, MAX>;
/// [`HashSet`] which contains at least a single item.
#[cfg(feature = "std")]
pub type NonEmptyHashSet<T, const MAX: usize = U64MAX> = Confined<HashSet<T>, ONE, MAX>;

/// [`BTreeSet`] with maximum 255 items of type `T`.
pub type TinyOrdSet<T> = Confined<BTreeSet<T>, ZERO, U8MAX>;
/// [`BTreeSet`] with maximum 2^16-1 items of type `T`.
pub type SmallOrdSet<T> = Confined<BTreeSet<T>, ZERO, U16MAX>;
/// [`BTreeSet`] with maximum 2^24-1 items of type `T`.
pub type MediumOrdSet<T> = Confined<BTreeSet<T>, ZERO, U24MAX>;
/// [`BTreeSet`] with maximum 2^32-1 items of type `T`.
pub type LargeOrdSet<T> = Confined<BTreeSet<T>, ZERO, U32MAX>;
/// Confined [`BTreeSet`].
pub type ConfinedOrdSet<T, const MIN: usize = 0, const MAX: usize = U64MAX> =
    Confined<BTreeSet<T>, MIN, MAX>;
/// [`BTreeSet`] which contains at least a single item.
pub type NonEmptyOrdSet<T, const MAX: usize = U64MAX> = Confined<BTreeSet<T>, ONE, MAX>;

/// [`HashMap`] with maximum 255 items.
#[cfg(feature = "std")]
pub type TinyHashMap<K, V> = Confined<HashMap<K, V>, ZERO, U8MAX>;
/// [`HashMap`] with maximum 2^16-1 items.
#[cfg(feature = "std")]
pub type SmallHashMap<K, V> = Confined<HashMap<K, V>, ZERO, U16MAX>;
/// [`HashMap`] with maximum 2^24-1 items.
#[cfg(feature = "std")]
pub type MediumHashMap<K, V> = Confined<HashMap<K, V>, ZERO, U24MAX>;
/// [`HashMap`] with maximum 2^32-1 items.
#[cfg(feature = "std")]
pub type LargeHashMap<K, V> = Confined<HashMap<K, V>, ZERO, U32MAX>;
/// Confined [`HashMap`].
#[cfg(feature = "std")]
pub type ConfinedHashMap<K, V, const MIN: usize = 0, const MAX: usize = U64MAX> =
    Confined<HashMap<K, V>, MIN, MAX>;
/// [`HashMap`] which contains at least a single item.
#[cfg(feature = "std")]
pub type NonEmptyHashMap<K, V, const MAX: usize = U64MAX> = Confined<HashMap<K, V>, ONE, MAX>;

/// [`BTreeMap`] with maximum 255 items.
pub type TinyOrdMap<K, V> = Confined<BTreeMap<K, V>, ZERO, U8MAX>;
/// [`BTreeMap`] with maximum 2^16-1 items.
pub type SmallOrdMap<K, V> = Confined<BTreeMap<K, V>, ZERO, U16MAX>;
/// [`BTreeMap`] with maximum 2^24-1 items.
pub type MediumOrdMap<K, V> = Confined<BTreeMap<K, V>, ZERO, U24MAX>;
/// [`BTreeMap`] with maximum 2^32-1 items.
pub type LargeOrdMap<K, V> = Confined<BTreeMap<K, V>, ZERO, U32MAX>;
/// Confined [`BTreeMap`].
pub type ConfinedOrdMap<K, V, const MIN: usize = 0, const MAX: usize = U64MAX> =
    Confined<BTreeMap<K, V>, MIN, MAX>;
/// [`BTreeMap`] which contains at least a single item.
pub type NonEmptyOrdMap<K, V, const MAX: usize = U64MAX> = Confined<BTreeMap<K, V>, ONE, MAX>;

/// [`indexmap::IndexSet`] with maximum 255 items of type `T`.
#[cfg(all(feature = "indexmap", feature = "std"))]
pub type TinyIndexSet<T> = Confined<indexmap::IndexSet<T>, ZERO, U8MAX>;
/// [`indexmap::IndexSet`] with maximum 2^16-1 items of type `T`.
#[cfg(all(feature = "indexmap", feature = "std"))]
pub type SmallIndexSet<T> = Confined<indexmap::IndexSet<T>, ZERO, U16MAX>;
/// [`indexmap::IndexSet`] with maximum 2^24-1 items of type `T`.
#[cfg(all(feature = "indexmap", feature = "std"))]
pub type MediumIndexSet<T> = Confined<indexmap::IndexSet<T>, ZERO, U24MAX>;
/// [`indexmap::IndexSet`] with maximum 2^32-1 items of type `T`.
#[cfg(all(feature = "indexmap", feature = "std"))]
pub type LargeIndexSet<T> = Confined<indexmap::IndexSet<T>, ZERO, U32MAX>;
/// Confined [`indexmap::IndexSet`].
#[cfg(all(feature = "indexmap", feature = "std"))]
pub type ConfinedIndexSet<T, const MIN: usize = 0, const MAX: usize = U64MAX> =
    Confined<indexmap::IndexSet<T>, MIN, MAX>;
/// [`indexmap::IndexSet`] which contains at least a single item.
#[cfg(all(feature = "indexmap", feature = "std"))]
pub type NonEmptyIndexSet<T, const MAX: usize = U64MAX> = Confined<indexmap::IndexSet<T>, ONE, MAX>;

/// [`indexmap::IndexMap`] with maximum 255 items.
#[cfg(all(feature = "indexmap", feature = "std"))]
pub type TinyIndexMap<K, V> = Confined<indexmap::IndexMap<K, V>, ZERO, U8MAX>;
/// [`indexmap::IndexMap`] with maximum 2^16-1 items.
#[cfg(all(feature = "indexmap", feature = "std"))]
pub type SmallIndexMap<K, V> = Confined<indexmap::IndexMap<K, V>, ZERO, U16MAX>;
/// [`indexmap::IndexMap`] with maximum 2^24-1 items.
#[cfg(all(feature = "indexmap", feature = "std"))]
pub type MediumIndexMap<K, V> = Confined<indexmap::IndexMap<K, V>, ZERO, U24MAX>;
/// [`indexmap::IndexMap`] with maximum 2^32-1 items.
#[cfg(all(feature = "indexmap", feature = "std"))]
pub type LargeIndexMap<K, V> = Confined<indexmap::IndexMap<K, V>, ZERO, U32MAX>;
/// Confined [`indexmap::IndexMap`].
#[cfg(all(feature = "indexmap", feature = "std"))]
pub type ConfinedIndexMap<K, V, const MIN: usize = 0, const MAX: usize = U64MAX> =
    Confined<indexmap::IndexMap<K, V>, MIN, MAX>;
/// [`indexmap::IndexMap`] which contains at least a single item.
#[cfg(all(feature = "indexmap", feature = "std"))]
pub type NonEmptyIndexMap<K, V, const MAX: usize = U64MAX> =
    Confined<indexmap::IndexMap<K, V>, ONE, MAX>;
