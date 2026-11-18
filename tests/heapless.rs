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

#![cfg(feature = "heapless")]

use std::hash::{BuildHasherDefault, DefaultHasher};

use collections::CollectionMap;
use confined::heapless::binary_heap::{Max, Min};
use confined::heapless::{BinaryHeap, Deque, IndexMap, IndexSet, LinearMap, String, Vec};
use confined::{Confined, Error, ItemLen};

type Hasher = BuildHasherDefault<DefaultHasher>;

fn native_capacity<C: ItemLen<Item = u8>>(mut col: Confined<C, 0, 3>) {
    assert_eq!(col.extend([1, 2]), Ok(()));
    assert!(matches!(col.push(3), Err(Error::CapacityExceeded { .. })));
    assert_eq!(col.len(), 2);
    assert!(matches!(
        col.extend([3, 4]),
        Err(Error::CapacityExceeded { .. })
    ));
    assert_eq!(col.len(), 2);
    col.clear();
    assert!(col.is_empty());
    assert_eq!(col.push(4), Ok(()));
    assert_eq!(col.len(), 1);
}

#[test]
fn sequences_and_sets_report_native_capacity_without_panicking() {
    native_capacity(Confined::<Vec<u8, 2>, 0, 3>::new());
    native_capacity(Confined::<Vec<u8, 2, u8>, 0, 3>::new());
    native_capacity(Confined::<Deque<u8, 2>, 0, 3>::new());
    native_capacity(Confined::<BinaryHeap<u8, Max, 2>, 0, 3>::new());
    native_capacity(Confined::<BinaryHeap<u8, Min, 2>, 0, 3>::new());
    native_capacity(Confined::<IndexSet<u8, Hasher, 2>, 0, 3>::new());
}

#[test]
fn vec_constructor_and_removal_preserve_bounds() {
    type Bounded = Confined<Vec<u8, 4, u8>, 1, 2>;
    assert!(matches!(
        Bounded::try_from_iter([]),
        Err(Error::Undersize { len: 0, min_len: 1 })
    ));
    assert!(matches!(
        Bounded::try_from_iter([1, 2, 3]),
        Err(Error::Oversize { len: 3, max_len: 2 })
    ));
    assert!(matches!(
        Confined::<Vec<u8, 2>, 0, 4>::try_from_iter([1, 2, 3]),
        Err(Error::CapacityExceeded { .. })
    ));
    let mut vec = Bounded::try_from_iter([1, 2]).unwrap();
    assert_eq!(vec.push(3), Err(Error::Oversize { len: 2, max_len: 2 }));
    assert_eq!(
        vec.remove(2),
        Err(Error::OutOfBoundary { index: 2, len: 2 })
    );
    assert_eq!(vec.remove(0), Ok(1));
    assert_eq!(vec.remove(0), Err(Error::Undersize { len: 1, min_len: 1 }));
    assert_eq!(
        vec.remove(1),
        Err(Error::OutOfBoundary { index: 1, len: 1 })
    );
    assert_eq!(vec.as_slice(), &[2]);

    let mut vec = Confined::<Vec<u8, 2>, 0, 2>::try_from_iter([1, 2]).unwrap();
    assert_eq!(vec.pop(), Some(2));
    assert_eq!(vec.pop(), Some(1));
    assert_eq!(vec.pop(), None);
}

#[test]
fn strings_count_bytes_for_confinement_and_native_capacity() {
    let mut s = Confined::<String<4, u8>, 0, 3>::new();
    assert_eq!(s.push('€'), Ok(()));
    assert_eq!(s.len(), 3);
    assert_eq!(s.push('a'), Err(Error::Oversize { len: 3, max_len: 3 }));
    assert_eq!(s.as_str(), "€");

    let mut s = Confined::<String<4>, 0, 8>::new();
    assert_eq!(s.extend(['a', '€']), Ok(()));
    assert_eq!(s.len(), 4);
    assert!(matches!(s.push('b'), Err(Error::CapacityExceeded { .. })));
    assert_eq!(s.as_str(), "a€");
    s.clear();
    assert_eq!(s.push('a'), Ok(()));
    assert!(matches!(
        s.extend(['b', '€']),
        Err(Error::CapacityExceeded { .. })
    ));
    assert_eq!(s.as_str(), "ab");

    assert!(matches!(
        Confined::<String<8>, 4, 8>::try_from_iter(['€']),
        Err(Error::Undersize { len: 3, min_len: 4 })
    ));
    assert!(matches!(
        Confined::<String<8>, 0, 3>::try_from_iter(['a', '€']),
        Err(Error::Oversize { len: 4, max_len: 3 })
    ));
    assert!(matches!(
        Confined::<String<2>, 0, 8>::try_from_iter(['€']),
        Err(Error::CapacityExceeded { .. })
    ));
}

#[test]
fn deque_ends_and_heap_priority_work_without_a_minimum() {
    let mut deque = Confined::<Deque<u8, 2>, 0, 3>::new();
    assert_eq!(deque.push_front(2), Ok(()));
    assert_eq!(deque.push_back(3), Ok(()));
    assert!(matches!(
        deque.push_front(1),
        Err(Error::CapacityExceeded { .. })
    ));
    assert!(matches!(
        deque.push_back(4),
        Err(Error::CapacityExceeded { .. })
    ));
    assert_eq!(deque.pop_front(), Some(2));
    assert_eq!(deque.pop_back(), Some(3));
    assert_eq!(deque.pop_front(), None);
    assert_eq!(deque.pop_back(), None);

    let mut bounded = Confined::<Deque<u8, 4>, 0, 1>::new();
    bounded.push_back(1).unwrap();
    assert_eq!(
        bounded.push_front(0),
        Err(Error::Oversize { len: 1, max_len: 1 })
    );
    assert_eq!(
        bounded.push_back(2),
        Err(Error::Oversize { len: 1, max_len: 1 })
    );

    let mut heap = Confined::<BinaryHeap<u8, Max, 4>, 0, 3>::try_from_iter([2, 1, 3]).unwrap();
    assert_eq!(heap.push(4), Err(Error::Oversize { len: 3, max_len: 3 }));
    assert_eq!(heap.pop(), Some(3));
    assert_eq!(heap.pop(), Some(2));
    assert_eq!(heap.pop(), Some(1));
    assert_eq!(heap.pop(), None);
}

#[test]
fn set_duplicates_succeed_at_both_limits_and_removal_checks_minimum() {
    let mut set = Confined::<IndexSet<u8, Hasher, 2>, 1, 2>::try_from_iter([1, 2]).unwrap();
    assert_eq!(set.push(2), Ok(()));
    assert_eq!(set.extend([1, 2, 1]), Ok(()));
    assert_eq!(set.len(), 2);
    assert_eq!(set.push(3), Err(Error::Oversize { len: 2, max_len: 2 }));
    assert_eq!(set.remove(&3), Ok(false));
    assert_eq!(set.remove(&2), Ok(true));
    assert_eq!(set.remove(&1), Err(Error::Undersize { len: 1, min_len: 1 }));
    assert_eq!(set.remove(&3), Ok(false));

    let mut native = Confined::<IndexSet<u8, Hasher, 2>, 0, 3>::try_from_iter([1, 2]).unwrap();
    assert_eq!(native.extend([2, 1]), Ok(()));
    assert!(matches!(
        native.push(3),
        Err(Error::CapacityExceeded { .. })
    ));
    assert_eq!(native.len(), 2);
}

fn map_without_entries<C: ItemLen + CollectionMap<Key = u8, Value = u8>>(
    map: &mut Confined<C, 1, 2>,
) {
    assert_eq!(map.insert(2, 20), Ok(None));
    assert_eq!(map.insert(2, 21), Ok(Some(20)));
    assert_eq!(map.push((2, 22)), Ok(()));
    assert_eq!(map.extend([(1, 11), (2, 23)]), Ok(()));
    assert_eq!(map.len(), 2);
    assert_eq!(
        map.insert(3, 30),
        Err(Error::Oversize { len: 2, max_len: 2 })
    );
    assert_eq!(
        map.push((3, 30)),
        Err(Error::Oversize { len: 2, max_len: 2 })
    );
    *map.get_mut(&1).unwrap() += 1;
    for value in map.values_mut() {
        *value += 1;
    }
    assert_eq!(map.get(&1), Some(&13));
    assert_eq!(map.get(&2), Some(&24));
    assert_eq!(map.try_retain(|key, _| *key == 1), Ok(()));
    assert_eq!(map.len(), 1);
    assert_eq!(map.get(&1), Some(&13));
    assert_eq!(
        map.try_retain(|_, _| false),
        Err(Error::Undersize { len: 0, min_len: 1 })
    );
    assert_eq!(map.get(&1), Some(&13));
}

fn native_map_capacity<C: ItemLen + CollectionMap<Key = u8, Value = u8>>(
    map: &mut Confined<C, 0, 3>,
) {
    assert_eq!(map.extend([(1, 10), (2, 20)]), Ok(()));
    assert_eq!(map.insert(2, 21), Ok(Some(20)));
    assert_eq!(map.push((2, 22)), Ok(()));
    assert_eq!(map.extend([(1, 11), (2, 23)]), Ok(()));
    assert!(matches!(
        map.insert(3, 30),
        Err(Error::CapacityExceeded { .. })
    ));
    assert!(matches!(
        map.push((3, 30)),
        Err(Error::CapacityExceeded { .. })
    ));
    assert!(matches!(
        map.extend([(3, 30)]),
        Err(Error::CapacityExceeded { .. })
    ));
    assert_eq!(map.len(), 2);
    assert_eq!(map.get(&1), Some(&11));
    assert_eq!(map.get(&2), Some(&23));
}

#[test]
fn linear_and_index_maps_do_not_need_entries_for_generic_operations() {
    let mut inner = LinearMap::<u8, u8, 2>::new();
    inner.insert(1, 10).unwrap();
    let mut linear = Confined::<_, 1, 2>::try_from(inner).unwrap();
    map_without_entries(&mut linear);
    assert_eq!(linear.remove(&2), Ok(None));
    assert_eq!(
        linear.remove(&1),
        Err(Error::Undersize { len: 1, min_len: 1 })
    );
    linear.insert(2, 20).unwrap();
    assert_eq!(linear.remove(&2), Ok(Some(20)));

    let mut indexed = Confined::<IndexMap<u8, u8, Hasher, 2>, 1, 2>::with_key_value(1, 10);
    map_without_entries(&mut indexed);
    assert_eq!(indexed.remove(&2), Ok(None));
    assert_eq!(
        indexed.remove(&1),
        Err(Error::Undersize { len: 1, min_len: 1 })
    );
    indexed.insert(2, 20).unwrap();
    assert_eq!(indexed.remove(&2), Ok(Some(20)));

    let mut linear = Confined::<_, 0, 3>::try_from(LinearMap::<u8, u8, 2>::new()).unwrap();
    native_map_capacity(&mut linear);
    let mut indexed = Confined::<IndexMap<u8, u8, Hasher, 2>, 0, 3>::new();
    native_map_capacity(&mut indexed);
}

#[test]
fn index_map_entries_check_native_capacity_before_insertion() {
    let mut map = Confined::<IndexMap<u8, u8, Hasher, 2>, 0, 3>::new();
    assert_eq!(*map.entry(1).unwrap().or_insert(10), 10);
    assert_eq!(*map.entry(2).unwrap().or_insert_with(|| 20), 20);
    assert!(matches!(map.entry(3), Err(Error::CapacityExceeded { .. })));
    assert_eq!(
        *map.entry(2).unwrap().and_modify(|v| *v += 1).or_default(),
        21
    );
    assert_eq!(map.len(), 2);

    let mut map = Confined::<IndexMap<u8, u8, Hasher, 2>, 0, 1>::new();
    *map.entry(1).unwrap().or_default() = 10;
    assert!(matches!(
        map.entry(2),
        Err(Error::Oversize { len: 1, max_len: 1 })
    ));
    assert_eq!(*map.entry(1).unwrap().or_insert(0), 10);
}

#[test]
fn reservations_check_confinement_before_native_capacity() {
    let mut vec = Confined::<Vec<u8, 2>, 0, 3>::new();
    assert_eq!(vec.try_reserve(2), Ok(()));
    assert!(matches!(
        vec.try_reserve(3),
        Err(Error::CapacityExceeded { .. })
    ));
    assert!(matches!(vec.try_reserve(4), Err(Error::Oversize { .. })));
    vec.push(1).unwrap();
    assert_eq!(vec.try_reserve(1), Ok(()));
    assert!(matches!(
        vec.try_reserve(2),
        Err(Error::CapacityExceeded { .. })
    ));
    assert!(matches!(
        vec.try_reserve(usize::MAX),
        Err(Error::Oversize { .. })
    ));
    assert_eq!(vec.as_slice(), &[1]);
}
