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

use alloc::collections::{BTreeMap, BTreeSet, VecDeque};
use alloc::string::String;
use alloc::vec::Vec;
#[cfg(feature = "std")]
use std::collections::{HashMap, HashSet};

#[cfg(feature = "ascii")]
use ascii::AsciiString;

use crate::{Confined, ONE, U8, U16, U24, U32, U64, ZERO};

/// [`String`] with maximum 255 bytes in UTF-8 encoding.
pub type TinyString = Confined<String, ZERO, U8>;
/// [`String`] with maximum 2^16-1 bytes in UTF-8 encoding.
pub type SmallString = Confined<String, ZERO, U16>;
/// [`String`] with maximum 2^24-1 bytes in UTF-8 encoding.
pub type MediumString = Confined<String, ZERO, U24>;
/// [`String`] with maximum 2^32-1 bytes in UTF-8 encoding.
pub type LargeString = Confined<String, ZERO, U32>;
/// Confined [`String`]. The limits are set in bytes of UTF-8 encoding, not in
/// characters.
pub type ConfinedString<const MIN: usize = 0, const MAX: usize = U64> = Confined<String, MIN, MAX>;
/// [`String`] which contains at least a single byte.
pub type NonEmptyString<const MAX: usize = U64> = Confined<String, ONE, MAX>;

/// [`AsciiString`] with maximum 255 characters.
#[cfg(feature = "ascii")]
pub type TinyAscii = Confined<AsciiString, ZERO, U8>;
/// [`AsciiString`] with maximum 2^16-1 characters.
#[cfg(feature = "ascii")]
pub type SmallAscii = Confined<AsciiString, ZERO, U16>;
/// [`AsciiString`] with maximum 2^24-1 characters.
#[cfg(feature = "ascii")]
pub type MediumAscii = Confined<AsciiString, ZERO, U24>;
/// [`AsciiString`] with maximum 2^32-1 characters.
#[cfg(feature = "ascii")]
pub type LargeAscii = Confined<AsciiString, ZERO, U32>;
/// Confined [`AsciiString`].
#[cfg(feature = "ascii")]
pub type ConfinedAscii<const MIN: usize = 0, const MAX: usize = U64> =
    Confined<AsciiString, MIN, MAX>;
/// [`AsciiString`] which contains at least a single character.
#[cfg(feature = "ascii")]
pub type NonEmptyAscii<const MAX: usize = U64> = Confined<AsciiString, ONE, MAX>;

/// [`Vec<u8>`] with maximum 255 bytes.
pub type TinyBlob = Confined<Vec<u8>, ZERO, U8>;
/// [`Vec<u8>`] with maximum 2^16-1 bytes.
pub type SmallBlob = Confined<Vec<u8>, ZERO, U16>;
/// [`Vec<u8>`] with maximum 2^24-1 bytes.
pub type MediumBlob = Confined<Vec<u8>, ZERO, U24>;
/// [`Vec<u8>`] with maximum 2^32-1 bytes.
pub type LargeBlob = Confined<Vec<u8>, ZERO, U32>;
/// Confined [`Vec<u8>`].
pub type ConfinedBlob<const MIN: usize = 0, const MAX: usize = U64> = Confined<Vec<u8>, MIN, MAX>;
/// [`Vec<u8>`] which contains at least a single byte.
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
/// Confined [`HashSet`].
#[cfg(feature = "std")]
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
/// Confined [`HashMap`].
#[cfg(feature = "std")]
pub type ConfinedHashMap<K, V, const MIN: usize = 0, const MAX: usize = U64> =
    Confined<HashMap<K, V>, MIN, MAX>;
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
/// Confined [`indexmap::IndexSet`].
#[cfg(feature = "indexmap")]
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
/// Confined [`indexmap::IndexMap`].
#[cfg(feature = "indexmap")]
pub type ConfinedIndexMap<K, V, const MIN: usize = 0, const MAX: usize = U64> =
    Confined<indexmap::IndexMap<K, V>, MIN, MAX>;
/// [`indexmap::IndexMap`] which contains at least a single item.
#[cfg(feature = "indexmap")]
pub type NonEmptyIndexMap<K, V, const MAX: usize = U64> =
    Confined<indexmap::IndexMap<K, V>, ONE, MAX>;
