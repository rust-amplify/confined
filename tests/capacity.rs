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

//! Native capacity of the hash tables behind the confined sets and maps.
//!
//! An item which a set already holds and a key which a map already holds add
//! nothing to the collection, so an operation with them must not ask the
//! table for room. Otherwise a table without spare capacity is reallocated
//! for nothing, and the process is aborted when the allocator refuses.
//!
//! The tests fill a table up to its native capacity and then check, for each
//! operation accepting a duplicate:
//!
//! - `std_hash_map`, `std_hash_set`, `hashbrown_map`, `hashbrown_set`,
//!   `index_map`, `index_set`: the capacity of the table is unchanged;
//! - `hashbrown_allocator::duplicates_make_no_allocation`: the allocator is not
//!   called;
//! - `hashbrown_allocator::duplicates_do_not_need_the_allocator`: the operation
//!   succeeds with an allocator which refuses every request;
//! - `hashbrown_allocator::new_items_report_the_refused_allocation`: a new item
//!   is reported as an error in the same situation, and nothing changes.

#![cfg(any(feature = "std", feature = "hashbrown", feature = "indexmap"))]

use collections::{CollectionCapacity, CollectionEntry, CollectionMap, CollectionSet};
use confined::{Confined, ItemLen};

/// The upper bound of the confinement, far above the capacity of the tables:
/// the tests are about the native capacity and never reach the bound.
const MAX: usize = 1000;

/// The capacity requested for a table. A table rounds it up to its real one.
const REQUESTED: usize = 20;

fn capacity_of<C: CollectionCapacity + ItemLen>(col: &Confined<C, 0, MAX>) -> usize {
    col.as_unconfined().capacity()
}

/// Adds new keys `0..capacity` until the table has no spare capacity left.
fn fill_map<C>(map: &mut Confined<C, 0, MAX>) -> usize
where
    C: CollectionMap<Key = u32, Value = u32> + CollectionCapacity + ItemLen,
{
    let capacity = capacity_of(map);
    assert!((REQUESTED..MAX).contains(&capacity));
    for key in 0..u32::try_from(capacity).unwrap() {
        assert_eq!(map.insert(key, key), Ok(None));
    }
    assert_eq!(map.len(), capacity);
    assert_eq!(capacity_of(map), capacity, "the table must be exactly full");
    capacity
}

/// Adds new items `0..capacity` until the table has no spare capacity left.
fn fill_set<C>(set: &mut Confined<C, 0, MAX>) -> usize
where
    C: CollectionSet<Item = u32> + CollectionCapacity + ItemLen,
{
    let capacity = capacity_of(set);
    assert!((REQUESTED..MAX).contains(&capacity));
    for item in 0..u32::try_from(capacity).unwrap() {
        assert_eq!(set.push(item), Ok(()));
    }
    assert_eq!(set.len(), capacity);
    assert_eq!(capacity_of(set), capacity, "the table must be exactly full");
    capacity
}

fn insert_of_an_existing_key<C>(map: &mut Confined<C, 0, MAX>)
where
    C: CollectionMap<Key = u32, Value = u32> + CollectionCapacity + ItemLen,
{
    let capacity = capacity_of(map);
    assert_eq!(map.insert(0, 500), Ok(Some(0)));
    assert_eq!(capacity_of(map), capacity);
    assert_eq!(map.len(), capacity);
    assert_eq!(map.as_unconfined().get(&0), Some(&500));
}

fn push_of_an_existing_key<C>(map: &mut Confined<C, 0, MAX>)
where
    C: CollectionMap<Key = u32, Value = u32> + CollectionCapacity + ItemLen,
{
    let capacity = capacity_of(map);
    assert_eq!(map.push((1, 501)), Ok(()));
    assert_eq!(capacity_of(map), capacity);
    assert_eq!(map.len(), capacity);
    assert_eq!(map.as_unconfined().get(&1), Some(&501));
}

fn extend_with_existing_keys<C>(map: &mut Confined<C, 0, MAX>)
where
    C: CollectionMap<Key = u32, Value = u32> + CollectionCapacity + ItemLen,
{
    let capacity = capacity_of(map);
    assert_eq!(map.extend([(2, 502), (3, 503)]), Ok(()));
    assert_eq!(capacity_of(map), capacity);
    assert_eq!(map.len(), capacity);
    assert_eq!(map.as_unconfined().get(&2), Some(&502));
    assert_eq!(map.as_unconfined().get(&3), Some(&503));
}

fn occupied_entry<C>(map: &mut Confined<C, 0, MAX>)
where
    C: CollectionEntry<Key = u32, Value = u32> + CollectionCapacity + ItemLen,
{
    let capacity = capacity_of(map);
    *map.entry(4).unwrap().or_insert(0) += 500;
    assert_eq!(capacity_of(map), capacity);
    assert_eq!(map.len(), capacity);
    assert_eq!(map.as_unconfined().get(&4), Some(&504));
}

fn push_of_an_existing_item<C>(set: &mut Confined<C, 0, MAX>)
where
    C: CollectionSet<Item = u32> + CollectionCapacity + ItemLen,
{
    let capacity = capacity_of(set);
    assert_eq!(set.push(0), Ok(()));
    assert_eq!(capacity_of(set), capacity);
    assert_eq!(set.len(), capacity);
    assert!(set.as_unconfined().contains(&0));
}

fn extend_with_existing_items<C>(set: &mut Confined<C, 0, MAX>)
where
    C: CollectionSet<Item = u32> + CollectionCapacity + ItemLen,
{
    let capacity = capacity_of(set);
    assert_eq!(set.extend([1, 2, 3]), Ok(()));
    assert_eq!(capacity_of(set), capacity);
    assert_eq!(set.len(), capacity);
}

macro_rules! map_keeps_the_capacity {
    ($name:ident, $map:ty) => {
        mod $name {
            use super::*;

            fn full() -> Confined<$map, 0, MAX> {
                let mut map = Confined::try_with_capacity(REQUESTED).unwrap();
                fill_map(&mut map);
                map
            }

            #[test]
            fn insert_of_an_existing_key_keeps_the_capacity() {
                insert_of_an_existing_key(&mut full());
            }

            #[test]
            fn push_of_an_existing_key_keeps_the_capacity() {
                push_of_an_existing_key(&mut full());
            }

            #[test]
            fn extend_with_existing_keys_keeps_the_capacity() {
                extend_with_existing_keys(&mut full());
            }

            #[test]
            fn occupied_entry_keeps_the_capacity() {
                occupied_entry(&mut full());
            }
        }
    };
}

macro_rules! set_keeps_the_capacity {
    ($name:ident, $set:ty) => {
        mod $name {
            use super::*;

            fn full() -> Confined<$set, 0, MAX> {
                let mut set = Confined::try_with_capacity(REQUESTED).unwrap();
                fill_set(&mut set);
                set
            }

            #[test]
            fn push_of_an_existing_item_keeps_the_capacity() {
                push_of_an_existing_item(&mut full());
            }

            #[test]
            fn extend_with_existing_items_keeps_the_capacity() {
                extend_with_existing_items(&mut full());
            }
        }
    };
}

#[cfg(feature = "std")]
map_keeps_the_capacity!(std_hash_map, std::collections::HashMap<u32, u32>);

#[cfg(feature = "std")]
set_keeps_the_capacity!(std_hash_set, std::collections::HashSet<u32>);

#[cfg(feature = "hashbrown")]
map_keeps_the_capacity!(
    hashbrown_map,
    confined::hashbrown::HashMap<u32, u32, std::hash::RandomState>
);

#[cfg(feature = "hashbrown")]
set_keeps_the_capacity!(
    hashbrown_set,
    confined::hashbrown::HashSet<u32, std::hash::RandomState>
);

#[cfg(feature = "indexmap")]
map_keeps_the_capacity!(index_map, indexmap::IndexMap<u32, u32, std::hash::RandomState>);

#[cfg(feature = "indexmap")]
set_keeps_the_capacity!(index_set, indexmap::IndexSet<u32, std::hash::RandomState>);

#[cfg(feature = "hashbrown")]
mod hashbrown_allocator {
    use core::alloc::{AllocError, Allocator, Layout};
    use core::ptr::NonNull;
    use std::alloc::System;
    use std::cell::Cell;
    use std::hash::RandomState;
    use std::rc::Rc;

    use confined::Error;
    use confined::hashbrown::{HashMap, HashSet};

    use super::*;

    /// An allocator which counts the requests and can be told to refuse them.
    #[derive(Clone, Debug, Default)]
    struct Meter {
        allocations: Rc<Cell<usize>>,
        exhausted: Rc<Cell<bool>>,
    }

    // SAFETY: Every granted allocation and every deallocation is forwarded
    // unchanged to `System`.
    unsafe impl Allocator for Meter {
        fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
            if self.exhausted.get() {
                return Err(AllocError);
            }
            self.allocations.set(self.allocations.get() + 1);
            System.allocate(layout)
        }

        unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout) {
            // SAFETY: The caller supplies a pointer and a layout of an
            // allocation which was granted by `System`.
            unsafe { System.deallocate(ptr, layout) };
        }
    }

    type Map = Confined<HashMap<u32, u32, RandomState, Meter>, 0, MAX>;
    type Set = Confined<HashSet<u32, RandomState, Meter>, 0, MAX>;

    fn full_map(meter: &Meter) -> Map {
        let mut map = Map::try_with_capacity_in(REQUESTED, meter.clone()).unwrap();
        fill_map(&mut map);
        map
    }

    fn full_set(meter: &Meter) -> Set {
        let mut set = Set::try_with_capacity_in(REQUESTED, meter.clone()).unwrap();
        fill_set(&mut set);
        set
    }

    mod duplicates_make_no_allocation {
        use super::*;

        fn check_map(operation: impl FnOnce(&mut Map)) {
            let meter = Meter::default();
            let mut map = full_map(&meter);
            let allocations = meter.allocations.get();
            operation(&mut map);
            assert_eq!(meter.allocations.get(), allocations);
        }

        fn check_set(operation: impl FnOnce(&mut Set)) {
            let meter = Meter::default();
            let mut set = full_set(&meter);
            let allocations = meter.allocations.get();
            operation(&mut set);
            assert_eq!(meter.allocations.get(), allocations);
        }

        #[test]
        fn map_insert() {
            check_map(insert_of_an_existing_key);
        }

        #[test]
        fn map_push() {
            check_map(push_of_an_existing_key);
        }

        #[test]
        fn map_extend() {
            check_map(extend_with_existing_keys);
        }

        #[test]
        fn map_entry() {
            check_map(occupied_entry);
        }

        #[test]
        fn set_push() {
            check_set(push_of_an_existing_item);
        }

        #[test]
        fn set_extend() {
            check_set(extend_with_existing_items);
        }
    }

    mod duplicates_do_not_need_the_allocator {
        use super::*;

        fn check_map(operation: impl FnOnce(&mut Map)) {
            let meter = Meter::default();
            let mut map = full_map(&meter);
            meter.exhausted.set(true);
            operation(&mut map);
        }

        fn check_set(operation: impl FnOnce(&mut Set)) {
            let meter = Meter::default();
            let mut set = full_set(&meter);
            meter.exhausted.set(true);
            operation(&mut set);
        }

        #[test]
        fn map_insert() {
            check_map(insert_of_an_existing_key);
        }

        #[test]
        fn map_push() {
            check_map(push_of_an_existing_key);
        }

        #[test]
        fn map_extend() {
            check_map(extend_with_existing_keys);
        }

        #[test]
        fn map_entry() {
            check_map(occupied_entry);
        }

        #[test]
        fn set_push() {
            check_set(push_of_an_existing_item);
        }

        #[test]
        fn set_extend() {
            check_set(extend_with_existing_items);
        }
    }

    mod new_items_report_the_refused_allocation {
        use super::*;

        /// A key which is not in a filled table.
        const NEW: u32 = 5000;

        fn check_map(operation: impl FnOnce(&mut Map) -> Result<(), Error>) {
            let meter = Meter::default();
            let mut map = full_map(&meter);
            let len = map.len();
            meter.exhausted.set(true);
            assert_eq!(operation(&mut map), Err(Error::CapacityExceeded { len }));
            assert_eq!(map.len(), len);
            assert_eq!(capacity_of(&map), len);
            assert!(!map.contains_key(&NEW));
        }

        fn check_set(operation: impl FnOnce(&mut Set) -> Result<(), Error>) {
            let meter = Meter::default();
            let mut set = full_set(&meter);
            let len = set.len();
            meter.exhausted.set(true);
            assert_eq!(operation(&mut set), Err(Error::CapacityExceeded { len }));
            assert_eq!(set.len(), len);
            assert_eq!(capacity_of(&set), len);
            assert!(!set.contains(&NEW));
        }

        #[test]
        fn map_insert() {
            check_map(|map| map.insert(NEW, 0).map(|_| ()));
        }

        #[test]
        fn map_push() {
            check_map(|map| map.push((NEW, 0)));
        }

        #[test]
        fn map_extend() {
            check_map(|map| map.extend([(NEW, 0)]));
        }

        #[test]
        fn map_entry() {
            check_map(|map| map.entry(NEW).map(|_| ()));
        }

        #[test]
        fn set_push() {
            check_set(|set| set.push(NEW));
        }

        #[test]
        fn set_extend() {
            check_set(|set| set.extend([NEW]));
        }
    }
}
