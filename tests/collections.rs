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

#![cfg(feature = "alloc")]

use std::assert_matches;
use std::collections::{BTreeMap, BinaryHeap, LinkedList, btree_map};
#[cfg(any(feature = "std", feature = "indexmap"))]
use std::hash::{BuildHasherDefault, DefaultHasher};

use collections::{Collection, CollectionBase, CollectionEntry, CollectionMap};
use confined::{Confined, Error, ItemLen};

#[test]
fn linked_list_preserves_bounds_and_order() {
    let mut list = Confined::<LinkedList<u8>, 1, 4>::with(2);
    list.push_front(1).unwrap();
    list.push_back(3).unwrap();
    list.push(4).unwrap();
    assert_matches!(
        list.push_front(0),
        Err(Error::Oversize { len: 4, max_len: 4 })
    );
    assert_matches!(
        list.push_back(5),
        Err(Error::Oversize { len: 4, max_len: 4 })
    );
    assert_matches!(
        list.remove(4),
        Err(Error::OutOfBoundary { index: 4, len: 4 })
    );
    assert_eq!(list.remove(1), Ok(2));
    assert_eq!(list.iter().copied().collect::<Vec<_>>(), [1, 3, 4]);
    assert_eq!(list.remove(2), Ok(4));
    assert_eq!(list.remove(0), Ok(1));
    assert_matches!(list.remove(0), Err(Error::Undersize { len: 1, min_len: 1 }));
    for item in &mut list {
        *item += 1;
    }
    assert_eq!(list.into_iter().collect::<Vec<_>>(), [4]);
}

#[test]
fn linked_list_without_lower_bound_can_be_emptied() {
    let mut list = confined::tiny_list![1, 2, 3];
    assert_eq!(list.pop_front(), Some(1));
    assert_eq!(list.pop_back(), Some(3));
    assert_eq!(list.pop_back(), Some(2));
    assert_eq!(list.pop_front(), None);
    assert_eq!(list.pop_back(), None);
    list.extend([4, 5]).unwrap();
    list.clear();
    assert!(list.is_empty());
}

#[test]
fn heap_preserves_bounds_and_priority() {
    let mut heap = Confined::<BinaryHeap<u8>, 0, 3>::with_capacity(3);
    heap.extend([2, 1, 3]).unwrap();
    assert_matches!(heap.push(4), Err(Error::Oversize { len: 3, max_len: 3 }));
    assert_eq!(heap.pop(), Some(3));
    assert_eq!(heap.pop(), Some(2));
    assert_eq!(heap.pop(), Some(1));
    assert_eq!(heap.pop(), None);
    heap.push(5).unwrap();
    heap.clear();
    assert!(heap.is_empty());
    assert_matches!(
        Confined::<BinaryHeap<u8>, 2, 3>::try_from_iter([1]),
        Err(Error::Undersize { len: 1, min_len: 2 })
    );
    assert_matches!(
        Confined::<BinaryHeap<u8>, 0, 2>::try_from_iter([1, 2, 3]),
        Err(Error::Oversize { len: 3, max_len: 2 })
    );
    let heap = confined::NonEmptyHeap::<u8>::with(1);
    assert_eq!(heap.peek(), Some(&1));
}

#[test]
fn new_aliases_and_macros() {
    let _: confined::TinyDeque<u8> = confined::tiny_deque![];
    let _: confined::SmallDeque<u8> = confined::small_deque![];
    let _: confined::MediumDeque<u8> = confined::medium_deque![];
    assert_eq!(confined::tiny_deque![const { 1 }, 2,].len(), 2);
    assert_eq!(confined::small_deque![1, 2].len(), 2);
    assert_eq!(confined::medium_deque![1, 2].len(), 2);
    assert_eq!(confined::tiny_deque![1; 3].len(), 3);
    assert_eq!(confined::small_deque![1; 3].len(), 3);
    assert_eq!(confined::medium_deque![1; 3].len(), 3);

    let _: confined::TinyList<u8> = confined::tiny_list![];
    let _: confined::SmallList<u8> = confined::small_list![];
    let _: confined::MediumList<u8> = confined::medium_list![];
    let _: confined::LargeList<u8> = confined::LargeList::new();
    let _: confined::ConfinedList<u8, 1, 3> = Confined::with(1);
    let _: confined::NonEmptyList<u8> = confined::NonEmptyList::with(1);
    assert_eq!(confined::tiny_list![const { 1 }, 2,].len(), 2);
    assert_eq!(confined::small_list![1, 2].len(), 2);
    assert_eq!(confined::medium_list![1, 2].len(), 2);

    let _: confined::TinyHeap<u8> = confined::tiny_heap![];
    let _: confined::SmallHeap<u8> = confined::small_heap![];
    let _: confined::MediumHeap<u8> = confined::medium_heap![];
    let _: confined::LargeHeap<u8> = confined::LargeHeap::new();
    let _: confined::ConfinedHeap<u8, 1, 3> = Confined::with(1);
    assert_eq!(confined::tiny_heap![const { 1 }, 2,].peek(), Some(&2));
    assert_eq!(confined::small_heap![1, 2].len(), 2);
    assert_eq!(confined::medium_heap![1, 2].len(), 2);

    assert_eq!(confined::tiny_vec![const { 1 }].as_slice(), &[1]);
    assert_eq!(confined::tiny_bmap![const { 1 } => const { 2 }][&1], 2);
}

#[test]
#[should_panic(expected = "invalid number of items")]
fn deque_macro_rejects_overflow() {
    let _ = confined::tiny_deque![0; 256];
}

#[test]
fn btree_keys_need_not_implement_hash() {
    #[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
    struct Key(u8);

    let mut map = Confined::<BTreeMap<Key, u8>, 1, 2>::with_key_value(Key(1), 10);
    *map.entry(Key(2)).unwrap().or_insert_with_key(|key| key.0) += 1;
    assert_eq!(map.remove(&Key(2)), Ok(Some(3)));
    assert_matches!(map.remove(&Key(1)), Err(Error::Undersize { .. }));
}

#[cfg(any(feature = "std", feature = "indexmap"))]
macro_rules! custom_hasher_tests {
    ($name:ident, $map:ident, $set:ident) => {
        #[test]
        fn $name() {
            type Map = Confined<$map<u8, u8, BuildHasherDefault<DefaultHasher>>, 1, 2>;
            let mut map = Map::with_key_value(1, 10);
            assert_eq!(map.insert(2, 20), Ok(None));
            assert_eq!(map.insert(2, 21), Ok(Some(20)));
            assert_eq!(
                *map.entry(2).unwrap().and_modify(|v| *v += 1).or_default(),
                22
            );
            assert_matches!(map.entry(3), Err(Error::Oversize { .. }));
            assert_eq!(map.remove(&2), Ok(Some(22)));
            assert_eq!(map.remove(&3), Ok(None));
            assert_matches!(map.remove(&1), Err(Error::Undersize { .. }));

            type Set = Confined<$set<u8, BuildHasherDefault<DefaultHasher>>, 1, 2>;
            let mut set = Set::try_from_iter([1, 2]).unwrap();
            assert_eq!(set.take(&2), Ok(Some(2)));
            assert_eq!(set.remove(&3), Ok(false));
            assert_eq!(set.take(&3), Ok(None));
            assert_matches!(set.remove(&1), Err(Error::Undersize { .. }));
            assert_matches!(set.take(&1), Err(Error::Undersize { .. }));
            set.push(3).unwrap();
            assert_matches!(set.push(4), Err(Error::Oversize { .. }));
            assert_eq!(set.remove(&3), Ok(true));
        }
    };
}

#[cfg(feature = "std")]
mod hashed {
    use std::collections::{HashMap, HashSet};

    use super::*;

    custom_hasher_tests!(custom_hashers, HashMap, HashSet);
}

#[cfg(feature = "indexmap")]
mod indexed {
    use confined::indexmap::{IndexMap, IndexSet};

    use super::*;

    custom_hasher_tests!(custom_hashers_without_requiring_std, IndexMap, IndexSet);

    #[test]
    fn removals_preserve_insertion_order() {
        type Map = Confined<IndexMap<u8, u8, BuildHasherDefault<DefaultHasher>>, 1, 3>;
        let mut map = Map::try_from_iter([(3, 30), (1, 10), (2, 20)]).unwrap();
        assert_eq!(map.remove(&1), Ok(Some(10)));
        assert_eq!(map.keys().copied().collect::<Vec<_>>(), [3, 2]);
        type Set = Confined<IndexSet<u8, BuildHasherDefault<DefaultHasher>>, 1, 3>;
        let mut set = Set::try_from_iter([3, 1, 2]).unwrap();
        assert_eq!(set.take(&1), Ok(Some(1)));
        assert_eq!(set.iter().copied().collect::<Vec<_>>(), [3, 2]);
    }
}

#[derive(Default, Debug)]
struct OwnMap(BTreeMap<u8, u8>);

impl FromIterator<(u8, u8)> for OwnMap {
    fn from_iter<I: IntoIterator<Item = (u8, u8)>>(iter: I) -> Self {
        Self(iter.into_iter().collect())
    }
}

impl Extend<(u8, u8)> for OwnMap {
    fn extend<I: IntoIterator<Item = (u8, u8)>>(&mut self, iter: I) {
        self.0.extend(iter);
    }
}

impl CollectionBase for OwnMap {
    type Item = (u8, u8);

    fn item_count(&self) -> usize {
        self.0.len()
    }
    fn push(&mut self, (key, value): Self::Item) {
        self.0.insert(key, value);
    }
    fn clear(&mut self) {
        self.0.clear();
    }
}

impl Collection for OwnMap {
    fn with_capacity(_capacity: usize) -> Self {
        Self::default()
    }
}

impl CollectionMap for OwnMap {
    type Key = u8;
    type Value = u8;

    fn contains_key(&self, key: &u8) -> bool {
        self.0.contains_key(key)
    }
    fn get(&self, key: &u8) -> Option<&u8> {
        self.0.get(key)
    }
    fn get_mut(&mut self, key: &u8) -> Option<&mut u8> {
        self.0.get_mut(key)
    }
    fn iter(&self) -> impl Iterator<Item = (&u8, &u8)> {
        self.0.iter()
    }
    fn iter_mut(&mut self) -> impl Iterator<Item = (&u8, &mut u8)> {
        self.0.iter_mut()
    }
    fn values_mut(&mut self) -> impl Iterator<Item = &mut u8> {
        self.0.values_mut()
    }
    fn insert(&mut self, key: u8, value: u8) -> Option<u8> {
        self.0.insert(key, value)
    }
    fn remove(&mut self, key: &u8) -> Option<u8> {
        self.0.remove(key)
    }
    fn retain(&mut self, f: impl FnMut(&u8, &mut u8) -> bool) {
        self.0.retain(f);
    }
}

impl CollectionEntry for OwnMap {
    type Entry<'a> = btree_map::Entry<'a, u8, u8>;

    fn entry(&mut self, key: u8) -> Self::Entry<'_> {
        self.0.entry(key)
    }
}

impl ItemLen for OwnMap {}

#[test]
fn entries_work_for_user_defined_maps() {
    let mut map = Confined::<OwnMap, 1, 3>::with_key_value(1, 10);
    assert_eq!(map.entry(2).unwrap().key(), &2);
    assert_eq!(*map.entry(2).unwrap().or_insert_with(|| 20), 20);
    assert_eq!(
        *map.entry(1).unwrap().and_modify(|v| *v += 1).or_insert(0),
        11
    );
    assert_eq!(*map.entry(3).unwrap().or_default(), 0);
    assert_eq!(
        *map.entry(3).unwrap().or_insert_with(|| panic!("occupied")),
        0
    );
    assert_matches!(map.entry(4), Err(Error::Oversize { len: 3, max_len: 3 }));
    assert_eq!(map.len(), 3);
    assert_eq!(
        map.try_retain(|_, _| false),
        Err(Error::Undersize { len: 0, min_len: 1 })
    );
    assert_eq!(map.len(), 1);
}

#[test]
fn item_len_is_sufficient_for_generic_operations() {
    fn fill<C: ItemLen>(col: &mut Confined<C, 0, 2>, items: impl IntoIterator<Item = C::Item>) {
        assert!(col.is_empty());
        col.extend(items).unwrap();
        assert_eq!(col.len(), 2);
        col.clear();
        assert!(col.is_empty());
    }

    fill(&mut Confined::<Vec<u8>, 0, 2>::new(), [1, 2]);
    fill(&mut Confined::<LinkedList<u8>, 0, 2>::new(), [1, 2]);
    fill(&mut Confined::<BinaryHeap<u8>, 0, 2>::new(), [1, 2]);
    fill(&mut Confined::<OwnMap, 0, 2>::new(), [(1, 1), (2, 2)]);
}

#[derive(Default, Debug)]
struct BaseOnly(Vec<u8>);

impl CollectionBase for BaseOnly {
    type Item = u8;

    fn item_count(&self) -> usize {
        self.0.len()
    }
    fn push(&mut self, item: Self::Item) {
        self.0.push(item);
    }
    fn clear(&mut self) {
        self.0.clear();
    }
}

impl ItemLen for BaseOnly {}

#[test]
fn try_from_iter_needs_only_item_len_and_default() {
    fn construct<C: ItemLen + Default>(
        items: impl IntoIterator<Item = C::Item>,
    ) -> Result<Confined<C, 1, 2>, Error> {
        Confined::try_from_iter(items)
    }

    let mut col = construct::<BaseOnly>([1, 2]).unwrap();
    assert_eq!(col.len(), 2);
    assert_eq!(col.push(3), Err(Error::Oversize { len: 2, max_len: 2 }));
    assert_eq!(col.release().0, [1, 2]);
    assert!(matches!(
        construct::<BaseOnly>([]),
        Err(Error::Undersize { len: 0, min_len: 1 })
    ));
    assert!(matches!(
        construct::<BaseOnly>([1, 2, 3]),
        Err(Error::Oversize { len: 3, max_len: 2 })
    ));
}

#[test]
fn errors_implement_core_error_without_std() {
    fn assert_error<T: core::error::Error>() {}

    assert_error::<Error>();
    #[cfg(feature = "ascii")]
    assert_error::<confined::AsciiError>();
}
