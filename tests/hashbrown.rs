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

#![cfg(feature = "hashbrown")]

use core::alloc::{AllocError, Allocator, Layout};
use core::ptr::NonNull;
use std::alloc::System;
use std::hash::{BuildHasher, BuildHasherDefault, DefaultHasher, Hasher};

use confined::hashbrown::{HashMap, HashSet};
use confined::{Confined, Error};

type HashState = BuildHasherDefault<DefaultHasher>;
type Map = HashMap<u8, u8, HashState>;
type Set = HashSet<u8, HashState>;

#[test]
fn map_push_extend_insert_and_entries_replace_at_maximum() {
    let mut map = Confined::<Map, 1, 2>::with_key_value(1, 10);
    assert_eq!(map.insert(2, 20), Ok(None));
    assert_eq!(map.insert(2, 21), Ok(Some(20)));
    assert_eq!(map.push((2, 22)), Ok(()));
    assert_eq!(map.extend([(1, 11), (2, 23)]), Ok(()));
    assert_eq!(
        *map.entry(2).unwrap().and_modify(|v| *v += 1).or_default(),
        24
    );
    assert_eq!(
        *map.entry(1).unwrap().or_insert_with(|| panic!("occupied")),
        11
    );
    assert_eq!(map.len(), 2);
    assert_eq!(
        map.insert(3, 30),
        Err(Error::Oversize { len: 2, max_len: 2 })
    );
    assert_eq!(
        map.push((3, 30)),
        Err(Error::Oversize { len: 2, max_len: 2 })
    );
    assert_eq!(
        map.extend([(3, 30)]),
        Err(Error::Oversize { len: 2, max_len: 2 })
    );
    assert!(matches!(
        map.entry(3),
        Err(Error::Oversize { len: 2, max_len: 2 })
    ));
    assert_eq!(map.remove(&3), Ok(None));
    assert_eq!(map.remove(&2), Ok(Some(24)));
    assert_eq!(map.remove(&1), Err(Error::Undersize { len: 1, min_len: 1 }));
    assert_eq!(map.remove(&3), Ok(None));
    assert_eq!(
        *map.entry(3).unwrap().or_insert_with_key(|key| key * 10),
        30
    );
    assert_eq!(map.len(), 2);
}

#[test]
fn set_duplicates_do_not_consume_confinement_room() {
    let mut set = Confined::<Set, 1, 2>::try_from_iter([1, 1, 2, 2]).unwrap();
    assert_eq!(set.push(2), Ok(()));
    assert_eq!(set.extend([1, 2, 1]), Ok(()));
    assert_eq!(set.len(), 2);
    assert_eq!(set.push(3), Err(Error::Oversize { len: 2, max_len: 2 }));
    assert_eq!(set.extend([3]), Err(Error::Oversize { len: 2, max_len: 2 }));
    assert_eq!(set.remove(&3), Ok(false));
    assert_eq!(set.take(&2), Ok(Some(2)));
    assert_eq!(set.remove(&1), Err(Error::Undersize { len: 1, min_len: 1 }));
    assert_eq!(set.take(&1), Err(Error::Undersize { len: 1, min_len: 1 }));
    assert_eq!(set.take(&3), Ok(None));
}

#[derive(Debug)]
struct ExplicitHasher(u64);

impl BuildHasher for ExplicitHasher {
    type Hasher = DefaultHasher;

    fn build_hasher(&self) -> Self::Hasher {
        let mut hasher = DefaultHasher::new();
        hasher.write_u64(self.0);
        hasher
    }
}

#[derive(Debug)]
struct ExplicitAllocator(u8);

// SAFETY: Every allocation and deallocation is forwarded unchanged to System.
unsafe impl Allocator for ExplicitAllocator {
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        System.allocate(layout)
    }

    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout) {
        // SAFETY: The caller supplies the original pointer and layout.
        unsafe { System.deallocate(ptr, layout) };
    }
}

fn mutate_map<S: BuildHasher, A: Allocator>(map: &mut Confined<HashMap<u8, u8, S, A>, 0, 2>) {
    map.push((1, 10)).unwrap();
    map.extend([(2, 20), (1, 11)]).unwrap();
    assert_eq!(map.insert(2, 21), Ok(Some(20)));
    assert_eq!(*map.entry(1).unwrap().or_default(), 11);
    *map.get_mut(&1).unwrap() += 1;
    for value in map.values_mut() {
        *value += 1;
    }
    assert_eq!(map.get(&1), Some(&13));
    assert_eq!(map.get(&2), Some(&22));
    assert_eq!(map.remove(&2), Ok(Some(22)));
    assert_eq!(map.try_retain(|_, _| false), Ok(()));
    assert!(map.is_empty());
}

fn mutate_set<S: BuildHasher, A: Allocator>(set: &mut Confined<HashSet<u8, S, A>, 0, 2>) {
    set.push(1).unwrap();
    set.extend([2, 1, 2]).unwrap();
    assert_eq!(set.len(), 2);
    assert_eq!(set.push(3), Err(Error::Oversize { len: 2, max_len: 2 }));
    assert_eq!(set.remove(&1), Ok(true));
    assert_eq!(set.take(&2), Ok(Some(2)));
    assert!(set.is_empty());
}

#[test]
fn generic_operations_need_neither_default_nor_clone_for_hasher_and_allocator() {
    let inner = HashMap::with_hasher_in(ExplicitHasher(7), ExplicitAllocator(9));
    let mut map = Confined::<_, 0, 2>::try_from(inner).unwrap();
    mutate_map(&mut map);
    assert_eq!(map.release().allocator().0, 9);

    let inner = HashSet::with_hasher_in(ExplicitHasher(7), ExplicitAllocator(9));
    let mut set = Confined::<_, 0, 2>::try_from(inner).unwrap();
    mutate_set(&mut set);
    assert_eq!(set.release().allocator().0, 9);
}

#[test]
fn allocator_aware_constructors_accept_non_default_allocators() {
    type AllocMap = HashMap<u8, u8, HashState, ExplicitAllocator>;
    type AllocSet = HashSet<u8, HashState, ExplicitAllocator>;

    let mut map = Confined::<AllocMap, 0, 2>::new_in(ExplicitAllocator(1));
    mutate_map(&mut map);
    assert_eq!(map.release().allocator().0, 1);
    let mut set = Confined::<AllocSet, 0, 2>::new_in(ExplicitAllocator(2));
    mutate_set(&mut set);
    assert_eq!(set.release().allocator().0, 2);

    let mut map = Confined::<AllocMap, 1, 2>::with_in((1, 10), ExplicitAllocator(3));
    assert_eq!(*map.entry(2).unwrap().or_insert(20), 20);
    assert_eq!(map.remove(&2), Ok(Some(20)));
    assert_eq!(map.remove(&1), Err(Error::Undersize { len: 1, min_len: 1 }));
    assert_eq!(map.release().allocator().0, 3);
    let set = Confined::<AllocSet, 1, 2>::with_in(1, ExplicitAllocator(4));
    assert!(set.contains(&1));
    assert_eq!(set.release().allocator().0, 4);

    let map = Confined::<AllocMap, 2, 2>::try_from_iter_in(
        [(1, 10), (2, 20), (1, 11)],
        ExplicitAllocator(5),
    )
    .unwrap();
    assert_eq!(map.get(&1), Some(&11));
    assert_eq!(map.release().allocator().0, 5);
    let set =
        Confined::<AllocSet, 2, 2>::try_from_iter_in([1, 2, 1], ExplicitAllocator(6)).unwrap();
    assert_eq!(set.len(), 2);
    assert_eq!(set.release().allocator().0, 6);
    assert!(matches!(
        Confined::<AllocMap, 2, 2>::try_from_iter_in([(1, 10)], ExplicitAllocator(0)),
        Err(Error::Undersize { len: 1, min_len: 2 })
    ));
    assert!(matches!(
        Confined::<AllocSet, 0, 2>::try_from_iter_in([1, 2, 3], ExplicitAllocator(0)),
        Err(Error::Oversize { len: 3, max_len: 2 })
    ));
}

#[test]
fn fallible_capacity_constructors_cap_requests_and_reservations_check_bounds() {
    let mut map = Confined::<Map, 0, 3>::try_with_capacity(usize::MAX).unwrap();
    assert!(map.capacity() >= 3);
    assert!(map.is_empty());
    map.insert(1, 10).unwrap();
    assert_eq!(map.try_reserve(2), Ok(()));
    assert!(matches!(map.try_reserve(3), Err(Error::Oversize { .. })));
    assert!(matches!(
        map.try_reserve(usize::MAX),
        Err(Error::Oversize { .. })
    ));
    assert_eq!(map.get(&1), Some(&10));

    let mut set = Confined::<Set, 0, 3>::try_with_capacity(usize::MAX).unwrap();
    set.push(1).unwrap();
    assert_eq!(set.try_reserve(2), Ok(()));
    assert!(matches!(set.try_reserve(3), Err(Error::Oversize { .. })));
    assert!(matches!(
        set.try_reserve(usize::MAX),
        Err(Error::Oversize { .. })
    ));
    assert_eq!(set.len(), 1);

    let map =
        Confined::<HashMap<u8, u8, HashState, ExplicitAllocator>, 0, 3>::try_with_capacity_in(
            usize::MAX,
            ExplicitAllocator(7),
        )
        .unwrap();
    assert!(map.capacity() >= 3);
    assert_eq!(map.release().allocator().0, 7);
    let set = Confined::<HashSet<u8, HashState, ExplicitAllocator>, 0, 0>::try_with_capacity_in(
        usize::MAX,
        ExplicitAllocator(8),
    )
    .unwrap();
    assert_eq!(set.capacity(), 0);
    assert_eq!(set.release().allocator().0, 8);
}
