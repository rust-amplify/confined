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

//! Behaviour of the checked methods when the global allocator is out of
//! memory.
//!
//! The collections which can reserve storage fallibly (`String`, `Vec`,
//! `VecDeque`, `BinaryHeap` and the hash tables) must report a refused
//! allocation as [`Error::CapacityExceeded`] and stay unchanged. An infallible
//! allocation in such a method would abort the process instead; in this file
//! it aborts the test binary, which is how a regression shows up here.
//!
//! The binary installs a global allocator which refuses the requests of the
//! current thread while a switch is on. The switch is turned on only around
//! the operation under test, which therefore must neither panic nor print.
//!
//! - `construction`: the fallible constructors;
//! - `growth`: `push` to a collection without spare capacity;
//! - `reservation`: `try_reserve`;
//! - `maps`: `insert` and `entry` for a new key;
//! - `duplicates`: the operations which add nothing need no memory;
//! - `write`: `std::io::Write` of a confined byte vector.
//!
//! `BTreeMap`, `BTreeSet`, `LinkedList` and `AsciiString` have no fallible
//! reservation and are documented to abort; they are not tested here.

#![cfg(feature = "std")]

use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;
use std::collections::{BinaryHeap, HashMap, HashSet, VecDeque};
use std::ptr;

use collections::{Collection, CollectionCapacity};
use confined::{Confined, Error, ItemLen};

thread_local! {
    /// Whether the allocation requests of the current thread are refused.
    ///
    /// The cell is initialized when the program is built and has no
    /// destructor, so the allocator can read it without allocating anything
    /// on the platforms with native thread-local storage.
    static REFUSING: Cell<bool> = const { Cell::new(false) };
}

fn is_refusing() -> bool {
    REFUSING.try_with(Cell::get).unwrap_or(false)
}

/// The system allocator with a per-thread switch which makes it fail.
struct Switchable;

// SAFETY: Every request which is not refused is forwarded unchanged to
// `System`. A refused request returns the null pointer, which is how an
// allocator reports a failure, and leaves the existing allocation intact.
unsafe impl GlobalAlloc for Switchable {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if is_refusing() {
            return ptr::null_mut();
        }
        // SAFETY: The caller upholds the contract of `GlobalAlloc::alloc`.
        unsafe { System.alloc(layout) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        // SAFETY: The block was allocated by `System` with the same layout.
        unsafe { System.dealloc(ptr, layout) }
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        if is_refusing() {
            return ptr::null_mut();
        }
        // SAFETY: The caller upholds the contract of `GlobalAlloc::alloc`.
        unsafe { System.alloc_zeroed(layout) }
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        if is_refusing() {
            return ptr::null_mut();
        }
        // SAFETY: The block was allocated by `System` with the same layout,
        // and the caller upholds the rest of the contract.
        unsafe { System.realloc(ptr, layout, new_size) }
    }
}

#[global_allocator]
static ALLOCATOR: Switchable = Switchable;

/// Runs the operation while the allocator refuses the requests of the thread.
fn out_of_memory<T>(operation: impl FnOnce() -> T) -> T {
    struct Restore;

    impl Drop for Restore {
        fn drop(&mut self) {
            REFUSING.set(false);
        }
    }

    let _restore = Restore;
    REFUSING.set(true);
    operation()
}

/// The upper bound of the confinement, never reached by the tests.
const MAX: usize = 4096;

/// Adds items until the collection has no spare capacity left.
fn fill<C>(col: &mut Confined<C, 0, MAX>, item: fn(usize) -> C::Item) -> usize
where
    C: ItemLen + CollectionCapacity,
{
    let capacity = col.as_unconfined().capacity();
    assert!(capacity > 0 && capacity < MAX);
    while col.len() < capacity {
        col.push(item(col.len())).unwrap();
    }
    assert_eq!(col.as_unconfined().capacity(), capacity);
    col.len()
}

/// A collection without spare capacity.
fn full<C>(item: fn(usize) -> C::Item) -> Confined<C, 0, MAX>
where
    C: ItemLen + CollectionCapacity + Collection,
{
    let mut col = Confined::<C, 0, MAX>::try_with_capacity(16).unwrap();
    fill(&mut col, item);
    col
}

fn byte(index: usize) -> u8 {
    u8::try_from(index % 256).unwrap()
}

fn letter(_: usize) -> char {
    'a'
}

fn number(index: usize) -> u32 {
    u32::try_from(index).unwrap()
}

fn entry(index: usize) -> (u32, u32) {
    (number(index), 0)
}

mod self_test {
    use super::*;

    #[test]
    fn switch_makes_fallible_allocations_fail() {
        let mut vec = Vec::<u8>::new();
        assert!(out_of_memory(|| vec.try_reserve(8)).is_err());
        assert_eq!(vec.capacity(), 0);
    }

    #[test]
    fn switch_is_off_after_the_operation() {
        out_of_memory(|| ());
        assert!(!is_refusing());
        assert_eq!(vec![1u8, 2, 3].len(), 3);
    }
}

mod construction {
    use super::*;

    #[test]
    fn string_from_a_slice_reports_the_refused_allocation() {
        let result = out_of_memory(|| "abc".parse::<Confined<String, 0, 8>>());
        assert_eq!(result, Err(Error::CapacityExceeded { len: 0 }));
    }

    #[test]
    fn vector_from_a_slice_reports_the_refused_allocation() {
        let result = out_of_memory(|| Confined::<Vec<u8>, 0, 8>::try_from_slice(&[1, 2, 3]));
        assert_eq!(result, Err(Error::CapacityExceeded { len: 0 }));
    }

    #[test]
    fn vector_from_an_iterator_reports_the_refused_allocation() {
        let result = out_of_memory(|| Confined::<Vec<u8>, 0, 8>::try_from_iter([1, 2, 3]));
        assert_eq!(result, Err(Error::CapacityExceeded { len: 0 }));
    }

    #[test]
    fn string_from_an_iterator_reports_the_refused_allocation() {
        let result = out_of_memory(|| Confined::<String, 0, 8>::try_from_iter(['a', 'b']));
        assert_eq!(result, Err(Error::CapacityExceeded { len: 0 }));
    }

    #[test]
    fn hash_map_from_an_iterator_reports_the_refused_allocation() {
        let result = out_of_memory(|| Confined::<HashMap<u32, u32>, 0, 8>::try_from_iter([(1, 1)]));
        assert_eq!(result, Err(Error::CapacityExceeded { len: 0 }));
    }

    #[test]
    fn requested_capacity_reports_the_refused_allocation() {
        let result = out_of_memory(|| Confined::<Vec<u8>, 0, 8>::try_with_capacity(8));
        assert_eq!(result, Err(Error::CapacityExceeded { len: 0 }));
        let result = out_of_memory(|| Confined::<String, 0, 8>::try_with_capacity(8));
        assert_eq!(result, Err(Error::CapacityExceeded { len: 0 }));
        let result = out_of_memory(|| Confined::<HashSet<u32>, 0, 8>::try_with_capacity(8));
        assert_eq!(result, Err(Error::CapacityExceeded { len: 0 }));
    }

    #[test]
    fn empty_collection_needs_no_memory() {
        let vec = out_of_memory(Confined::<Vec<u8>, 0, 8>::new);
        assert!(vec.is_empty());
        let map = out_of_memory(Confined::<HashMap<u32, u32>, 0, 8>::new);
        assert!(map.is_empty());
    }
}

mod growth {
    use super::*;

    /// A push to a collection without spare capacity is refused, and the
    /// collection keeps its items.
    fn check<C>(item: fn(usize) -> C::Item)
    where
        C: ItemLen + CollectionCapacity + Collection,
    {
        let mut col = full::<C>(item);
        let len = col.len();
        let new = item(len);
        let result = out_of_memory(|| col.push(new));
        assert_eq!(result, Err(Error::CapacityExceeded { len }));
        assert_eq!(col.len(), len);
        assert_eq!(col.as_unconfined().capacity(), len);
    }

    #[test]
    fn vector_reports_the_refused_allocation() {
        check::<Vec<u8>>(byte);
    }

    #[test]
    fn string_reports_the_refused_allocation() {
        check::<String>(letter);
    }

    #[test]
    fn deque_reports_the_refused_allocation() {
        check::<VecDeque<u8>>(byte);
    }

    #[test]
    fn heap_reports_the_refused_allocation() {
        check::<BinaryHeap<u8>>(byte);
    }

    #[test]
    fn hash_set_reports_the_refused_allocation() {
        check::<HashSet<u32>>(number);
    }

    #[test]
    fn hash_map_reports_the_refused_allocation() {
        check::<HashMap<u32, u32>>(entry);
    }

    #[cfg(feature = "indexmap")]
    #[test]
    fn index_set_reports_the_refused_allocation() {
        check::<indexmap::IndexSet<u32>>(number);
    }

    #[cfg(feature = "indexmap")]
    #[test]
    fn index_map_reports_the_refused_allocation() {
        check::<indexmap::IndexMap<u32, u32>>(entry);
    }

    #[cfg(feature = "hashbrown")]
    #[test]
    fn hashbrown_set_reports_the_refused_allocation() {
        check::<confined::hashbrown::HashSet<u32, std::hash::RandomState>>(number);
    }

    #[cfg(feature = "hashbrown")]
    #[test]
    fn hashbrown_map_reports_the_refused_allocation() {
        check::<confined::hashbrown::HashMap<u32, u32, std::hash::RandomState>>(entry);
    }

    #[test]
    fn deque_front_reports_the_refused_allocation() {
        let mut deque = full::<VecDeque<u8>>(byte);
        let len = deque.len();
        let result = out_of_memory(|| deque.push_front(7));
        assert_eq!(result, Err(Error::CapacityExceeded { len }));
        assert_eq!(deque.len(), len);
    }

    #[test]
    fn extension_keeps_the_items_added_before_the_refused_allocation() {
        let mut vec = Confined::<Vec<u8>, 0, MAX>::try_with_capacity(16).unwrap();
        let capacity = vec.capacity();
        let result = out_of_memory(|| vec.extend((0..MAX).map(byte)));
        assert_eq!(result, Err(Error::CapacityExceeded { len: capacity }));
        assert_eq!(vec.len(), capacity);
    }
}

mod reservation {
    use super::*;

    fn check<C>(item: fn(usize) -> C::Item)
    where
        C: ItemLen + CollectionCapacity + Collection,
    {
        let mut col = full::<C>(item);
        let len = col.len();
        let result = out_of_memory(|| col.try_reserve(1));
        assert_eq!(result, Err(Error::CapacityExceeded { len }));
        assert_eq!(col.as_unconfined().capacity(), len);
    }

    #[test]
    fn vector_reports_the_refused_allocation() {
        check::<Vec<u8>>(byte);
    }

    #[test]
    fn string_reports_the_refused_allocation() {
        check::<String>(letter);
    }

    #[test]
    fn deque_reports_the_refused_allocation() {
        check::<VecDeque<u8>>(byte);
    }

    #[test]
    fn heap_reports_the_refused_allocation() {
        check::<BinaryHeap<u8>>(byte);
    }

    #[test]
    fn hash_set_reports_the_refused_allocation() {
        check::<HashSet<u32>>(number);
    }

    #[test]
    fn hash_map_reports_the_refused_allocation() {
        check::<HashMap<u32, u32>>(entry);
    }

    #[test]
    fn spare_capacity_needs_no_memory() {
        let mut vec = Confined::<Vec<u8>, 0, MAX>::try_with_capacity(16).unwrap();
        assert_eq!(out_of_memory(|| vec.try_reserve(16)), Ok(()));
        assert_eq!(out_of_memory(|| vec.push(1)), Ok(()));
    }
}

mod maps {
    use collections::{CollectionEntry, CollectionMap};

    use super::*;

    /// A key which is not in a filled map.
    const NEW: u32 = 100_000;

    fn insert<C>()
    where
        C: CollectionMap<Key = u32, Value = u32> + ItemLen + CollectionCapacity + Collection,
    {
        let mut map = full::<C>(entry);
        let len = map.len();
        let result = out_of_memory(|| map.insert(NEW, 1));
        assert_eq!(result, Err(Error::CapacityExceeded { len }));
        assert_eq!(map.len(), len);
        assert!(!map.as_unconfined().contains_key(&NEW));
    }

    fn vacant_entry<C>()
    where
        C: CollectionEntry<Key = u32, Value = u32> + ItemLen + CollectionCapacity + Collection,
    {
        let mut map = full::<C>(entry);
        let len = map.len();
        let result = out_of_memory(|| map.entry(NEW).err());
        assert_eq!(result, Some(Error::CapacityExceeded { len }));
        assert_eq!(map.len(), len);
        assert!(!map.as_unconfined().contains_key(&NEW));
    }

    #[test]
    fn hash_map_insert_reports_the_refused_allocation() {
        insert::<HashMap<u32, u32>>();
    }

    #[test]
    fn hash_map_entry_reports_the_refused_allocation() {
        vacant_entry::<HashMap<u32, u32>>();
    }

    #[cfg(feature = "indexmap")]
    #[test]
    fn index_map_insert_reports_the_refused_allocation() {
        insert::<indexmap::IndexMap<u32, u32>>();
    }

    #[cfg(feature = "indexmap")]
    #[test]
    fn index_map_entry_reports_the_refused_allocation() {
        vacant_entry::<indexmap::IndexMap<u32, u32>>();
    }

    #[cfg(feature = "hashbrown")]
    #[test]
    fn hashbrown_map_insert_reports_the_refused_allocation() {
        insert::<confined::hashbrown::HashMap<u32, u32, std::hash::RandomState>>();
    }

    #[cfg(feature = "hashbrown")]
    #[test]
    fn hashbrown_map_entry_reports_the_refused_allocation() {
        vacant_entry::<confined::hashbrown::HashMap<u32, u32, std::hash::RandomState>>();
    }
}

mod duplicates {
    use collections::{CollectionEntry, CollectionMap, CollectionSet};

    use super::*;

    fn set_push<C>()
    where
        C: CollectionSet<Item = u32> + ItemLen + CollectionCapacity + Collection,
    {
        let mut set = full::<C>(number);
        let len = set.len();
        assert_eq!(out_of_memory(|| set.push(0)), Ok(()));
        assert_eq!(out_of_memory(|| set.extend([1, 2])), Ok(()));
        assert_eq!(set.len(), len);
    }

    fn map_push<C>()
    where
        C: CollectionMap<Key = u32, Value = u32> + ItemLen + CollectionCapacity + Collection,
    {
        let mut map = full::<C>(entry);
        let len = map.len();
        assert_eq!(out_of_memory(|| map.push((0, 7))), Ok(()));
        assert_eq!(out_of_memory(|| map.extend([(1, 7), (2, 7)])), Ok(()));
        assert_eq!(map.len(), len);
        assert_eq!(map.as_unconfined().get(&2), Some(&7));
    }

    fn map_insert<C>()
    where
        C: CollectionMap<Key = u32, Value = u32> + ItemLen + CollectionCapacity + Collection,
    {
        let mut map = full::<C>(entry);
        let len = map.len();
        assert_eq!(out_of_memory(|| map.insert(0, 7)), Ok(Some(0)));
        assert_eq!(map.len(), len);
        assert_eq!(map.as_unconfined().get(&0), Some(&7));
    }

    fn occupied_entry<C>()
    where
        C: CollectionEntry<Key = u32, Value = u32> + ItemLen + CollectionCapacity + Collection,
    {
        let mut map = full::<C>(entry);
        let len = map.len();
        let value = out_of_memory(|| map.entry(0).map(|entry| *entry.or_insert(7)));
        assert_eq!(value, Ok(0));
        assert_eq!(map.len(), len);
    }

    #[test]
    fn hash_set_accepts_a_present_item() {
        set_push::<HashSet<u32>>();
    }

    #[test]
    fn hash_map_accepts_an_existing_key() {
        map_push::<HashMap<u32, u32>>();
    }

    #[test]
    fn hash_map_replaces_the_value_of_an_existing_key() {
        map_insert::<HashMap<u32, u32>>();
    }

    #[test]
    fn hash_map_gives_out_an_occupied_entry() {
        occupied_entry::<HashMap<u32, u32>>();
    }

    #[cfg(feature = "indexmap")]
    #[test]
    fn index_set_accepts_a_present_item() {
        set_push::<indexmap::IndexSet<u32>>();
    }

    #[cfg(feature = "indexmap")]
    #[test]
    fn index_map_accepts_an_existing_key() {
        map_push::<indexmap::IndexMap<u32, u32>>();
    }

    #[cfg(feature = "indexmap")]
    #[test]
    fn index_map_replaces_the_value_of_an_existing_key() {
        map_insert::<indexmap::IndexMap<u32, u32>>();
    }

    #[cfg(feature = "indexmap")]
    #[test]
    fn index_map_gives_out_an_occupied_entry() {
        occupied_entry::<indexmap::IndexMap<u32, u32>>();
    }

    #[cfg(feature = "hashbrown")]
    #[test]
    fn hashbrown_set_accepts_a_present_item() {
        set_push::<confined::hashbrown::HashSet<u32, std::hash::RandomState>>();
    }

    #[cfg(feature = "hashbrown")]
    #[test]
    fn hashbrown_map_accepts_an_existing_key() {
        map_push::<confined::hashbrown::HashMap<u32, u32, std::hash::RandomState>>();
    }

    #[cfg(feature = "hashbrown")]
    #[test]
    fn hashbrown_map_replaces_the_value_of_an_existing_key() {
        map_insert::<confined::hashbrown::HashMap<u32, u32, std::hash::RandomState>>();
    }

    #[cfg(feature = "hashbrown")]
    #[test]
    fn hashbrown_map_gives_out_an_occupied_entry() {
        occupied_entry::<confined::hashbrown::HashMap<u32, u32, std::hash::RandomState>>();
    }
}

mod write {
    use std::io::{ErrorKind, Write};

    use super::*;

    #[test]
    fn write_reports_the_refused_allocation() {
        let mut blob = full::<Vec<u8>>(byte);
        let len = blob.len();
        let result = out_of_memory(|| blob.write(&[1, 2, 3]).map_err(|error| error.kind()));
        assert_eq!(result, Err(ErrorKind::OutOfMemory));
        assert_eq!(blob.len(), len);
    }

    #[test]
    fn write_into_spare_capacity_needs_no_memory() {
        let mut blob = Confined::<Vec<u8>, 0, MAX>::try_with_capacity(16).unwrap();
        let result = out_of_memory(|| blob.write(&[1, 2, 3]).map_err(|error| error.kind()));
        assert_eq!(result, Ok(3));
        assert_eq!(blob.as_slice(), [1, 2, 3]);
    }
}
