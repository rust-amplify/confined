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

//! Confined fixed-capacity collections of `heapless` (`src/fixed.rs`).
//!
//! Such a collection has two limits: the upper bound of the confinement,
//! reported as [`Error::Oversize`], and the capacity of its storage, reported
//! as [`Error::CapacityExceeded`]. Neither of them may cause a panic in a
//! checked method. The storage of every collection below holds four items
//! (eight bytes for the string).
//!
//! Each module covers one collection, and each test covers one case:
//! `vec`, `string`, `deque`, `heap`, `index_set`, `index_map`, `linear_map`,
//! `reservation`, `with` and `aliases`.

#![cfg(feature = "heapless")]

use std::hash::RandomState;

use confined::{Confined, Error, heapless as fixed};

mod vec {
    use super::*;

    type Bytes<const MIN: usize, const MAX: usize> = Confined<fixed::Vec<u8, 4>, MIN, MAX>;

    fn bytes_of<const MIN: usize, const MAX: usize>(items: &[u8]) -> Bytes<MIN, MAX> {
        Bytes::try_from_iter(items.iter().copied()).unwrap()
    }

    #[test]
    fn push_adds_an_item_within_both_limits() {
        let mut bytes = bytes_of::<0, 3>(&[1, 2]);
        assert_eq!(bytes.push(3), Ok(()));
        assert_eq!(bytes.as_slice(), [1, 2, 3]);
    }

    #[test]
    fn push_is_refused_at_the_maximum() {
        let mut bytes = bytes_of::<0, 2>(&[1, 2]);
        assert_eq!(bytes.push(3), Err(Error::Oversize { len: 2, max_len: 2 }));
        assert_eq!(bytes.as_slice(), [1, 2]);
    }

    #[test]
    fn push_is_refused_by_a_full_storage() {
        let mut bytes = bytes_of::<0, 8>(&[1, 2, 3, 4]);
        assert_eq!(bytes.push(5), Err(Error::CapacityExceeded { len: 4 }));
        assert_eq!(bytes.as_slice(), [1, 2, 3, 4]);
    }

    #[test]
    fn extend_keeps_the_items_which_fit_the_storage() {
        let mut bytes = bytes_of::<0, 8>(&[1, 2]);
        assert_eq!(
            bytes.extend([3, 4, 5, 6]),
            Err(Error::CapacityExceeded { len: 4 })
        );
        assert_eq!(bytes.as_slice(), [1, 2, 3, 4]);
    }

    #[test]
    fn construction_reports_a_full_storage() {
        assert_eq!(
            Bytes::<0, 8>::try_from_iter([1, 2, 3, 4, 5]),
            Err(Error::CapacityExceeded { len: 4 })
        );
    }

    #[test]
    fn construction_reports_the_maximum_before_the_storage() {
        assert_eq!(
            Bytes::<0, 2>::try_from_iter([1, 2, 3, 4, 5]),
            Err(Error::Oversize { len: 3, max_len: 2 })
        );
    }

    #[test]
    fn remove_returns_the_item_at_the_index() {
        let mut bytes = bytes_of::<1, 4>(&[1, 2, 3]);
        assert_eq!(bytes.remove(1), Ok(2));
        assert_eq!(bytes.as_slice(), [1, 3]);
    }

    #[test]
    fn remove_refuses_an_index_past_the_end() {
        let mut bytes = bytes_of::<1, 4>(&[1, 2, 3]);
        assert_eq!(
            bytes.remove(3),
            Err(Error::OutOfBoundary { index: 3, len: 3 })
        );
        assert_eq!(bytes.as_slice(), [1, 2, 3]);
    }

    #[test]
    fn remove_keeps_the_items_at_the_minimum() {
        let mut bytes = bytes_of::<2, 4>(&[1, 2]);
        assert_eq!(
            bytes.remove(0),
            Err(Error::Undersize { len: 2, min_len: 2 })
        );
        assert_eq!(bytes.as_slice(), [1, 2]);
    }

    #[test]
    fn pop_returns_the_last_item() {
        let mut bytes = bytes_of::<0, 4>(&[1, 2]);
        assert_eq!(bytes.pop(), Some(2));
        assert_eq!(bytes.as_slice(), [1]);
    }

    #[test]
    fn pop_of_an_empty_vector_returns_nothing() {
        let mut bytes = Bytes::<0, 4>::new();
        assert_eq!(bytes.pop(), None);
    }
}

mod string {
    use super::*;

    type Text<const MIN: usize, const MAX: usize> = Confined<fixed::String<8>, MIN, MAX>;

    fn text_of<const MIN: usize, const MAX: usize>(text: &str) -> Text<MIN, MAX> {
        Text::try_from_iter(text.chars()).unwrap()
    }

    #[test]
    fn length_is_measured_in_bytes() {
        let text = text_of::<0, 8>("a€");
        assert_eq!(text.len(), 4);
        assert_eq!(text.as_str(), "a€");
    }

    #[test]
    fn push_adds_a_character_within_both_limits() {
        let mut text = text_of::<0, 8>("ab");
        assert_eq!(text.push('€'), Ok(()));
        assert_eq!(text.as_str(), "ab€");
        assert_eq!(text.len(), 5);
    }

    #[test]
    fn push_counts_the_bytes_of_a_character_for_the_maximum() {
        // One more character, but three more bytes.
        let mut text = text_of::<0, 4>("ab");
        assert_eq!(text.push('€'), Err(Error::Oversize { len: 2, max_len: 4 }));
        assert_eq!(text.as_str(), "ab");
    }

    #[test]
    fn push_counts_the_bytes_of_a_character_for_the_storage() {
        let mut text = text_of::<0, 16>("abcdef");
        assert_eq!(text.push('€'), Err(Error::CapacityExceeded { len: 6 }));
        assert_eq!(text.as_str(), "abcdef");
    }

    #[test]
    fn push_fills_the_storage_exactly() {
        let mut text = text_of::<0, 16>("abcde");
        assert_eq!(text.push('€'), Ok(()));
        assert_eq!(text.len(), 8);
        assert_eq!(text.push('!'), Err(Error::CapacityExceeded { len: 8 }));
    }

    #[test]
    fn construction_checks_the_minimum_in_bytes() {
        assert_eq!(
            Text::<4, 8>::try_from_iter("€".chars()),
            Err(Error::Undersize { len: 3, min_len: 4 })
        );
    }

    #[test]
    fn pop_returns_the_last_character() {
        let mut text = text_of::<0, 8>("a€");
        assert_eq!(text.pop(), Some('€'));
        assert_eq!(text.as_str(), "a");
        assert_eq!(text.len(), 1);
    }

    #[test]
    fn pop_of_an_empty_string_returns_nothing() {
        let mut text = Text::<0, 8>::new();
        assert_eq!(text.pop(), None);
    }
}

mod deque {
    use super::*;

    type Queue<const MIN: usize, const MAX: usize> = Confined<fixed::Deque<u8, 4>, MIN, MAX>;

    fn queue_of<const MIN: usize, const MAX: usize>(items: &[u8]) -> Queue<MIN, MAX> {
        Queue::try_from_iter(items.iter().copied()).unwrap()
    }

    fn items<const MIN: usize, const MAX: usize>(queue: &Queue<MIN, MAX>) -> Vec<u8> {
        queue.iter().copied().collect()
    }

    #[test]
    fn push_front_adds_the_first_item() {
        let mut queue = queue_of::<0, 3>(&[2, 3]);
        assert_eq!(queue.push_front(1), Ok(()));
        assert_eq!(items(&queue), [1, 2, 3]);
    }

    #[test]
    fn push_front_is_refused_at_the_maximum() {
        let mut queue = queue_of::<0, 2>(&[2, 3]);
        assert_eq!(
            queue.push_front(1),
            Err(Error::Oversize { len: 2, max_len: 2 })
        );
        assert_eq!(items(&queue), [2, 3]);
    }

    #[test]
    fn push_front_is_refused_by_a_full_storage() {
        let mut queue = queue_of::<0, 8>(&[1, 2, 3, 4]);
        assert_eq!(queue.push_front(0), Err(Error::CapacityExceeded { len: 4 }));
        assert_eq!(items(&queue), [1, 2, 3, 4]);
    }

    #[test]
    fn push_back_adds_the_last_item() {
        let mut queue = queue_of::<0, 3>(&[1, 2]);
        assert_eq!(queue.push_back(3), Ok(()));
        assert_eq!(items(&queue), [1, 2, 3]);
    }

    #[test]
    fn push_back_is_refused_at_the_maximum() {
        let mut queue = queue_of::<0, 2>(&[1, 2]);
        assert_eq!(
            queue.push_back(3),
            Err(Error::Oversize { len: 2, max_len: 2 })
        );
        assert_eq!(items(&queue), [1, 2]);
    }

    #[test]
    fn push_back_is_refused_by_a_full_storage() {
        let mut queue = queue_of::<0, 8>(&[1, 2, 3, 4]);
        assert_eq!(queue.push_back(5), Err(Error::CapacityExceeded { len: 4 }));
        assert_eq!(items(&queue), [1, 2, 3, 4]);
    }

    #[test]
    fn pop_front_returns_the_first_item() {
        let mut queue = queue_of::<0, 4>(&[1, 2]);
        assert_eq!(queue.pop_front(), Some(1));
        assert_eq!(items(&queue), [2]);
    }

    #[test]
    fn pop_back_returns_the_last_item() {
        let mut queue = queue_of::<0, 4>(&[1, 2]);
        assert_eq!(queue.pop_back(), Some(2));
        assert_eq!(items(&queue), [1]);
    }

    #[test]
    fn pops_of_an_empty_deque_return_nothing() {
        let mut queue = Queue::<0, 4>::new();
        assert_eq!(queue.pop_front(), None);
        assert_eq!(queue.pop_back(), None);
    }
}

mod heap {
    use fixed::binary_heap::{Max, Min};

    use super::*;

    type Heap<const MIN: usize, const MAX: usize> =
        Confined<fixed::BinaryHeap<u8, Max, 4>, MIN, MAX>;

    fn heap_of<const MIN: usize, const MAX: usize>(items: &[u8]) -> Heap<MIN, MAX> {
        Heap::try_from_iter(items.iter().copied()).unwrap()
    }

    #[test]
    fn push_adds_an_item_within_both_limits() {
        let mut heap = heap_of::<0, 3>(&[1, 3]);
        assert_eq!(heap.push(2), Ok(()));
        assert_eq!(heap.len(), 3);
        assert_eq!(heap.peek(), Some(&3));
    }

    #[test]
    fn push_is_refused_at_the_maximum() {
        let mut heap = heap_of::<0, 2>(&[1, 3]);
        assert_eq!(heap.push(2), Err(Error::Oversize { len: 2, max_len: 2 }));
        assert_eq!(heap.len(), 2);
    }

    #[test]
    fn push_is_refused_by_a_full_storage() {
        let mut heap = heap_of::<0, 8>(&[1, 2, 3, 4]);
        assert_eq!(heap.push(5), Err(Error::CapacityExceeded { len: 4 }));
        assert_eq!(heap.len(), 4);
        assert_eq!(heap.peek(), Some(&4));
    }

    #[test]
    fn pop_returns_the_greatest_item_of_a_max_heap() {
        let mut heap = heap_of::<0, 4>(&[1, 3, 2]);
        assert_eq!(heap.pop(), Some(3));
        assert_eq!(heap.pop(), Some(2));
        assert_eq!(heap.pop(), Some(1));
        assert_eq!(heap.pop(), None);
    }

    #[test]
    fn pop_returns_the_least_item_of_a_min_heap() {
        let mut heap =
            Confined::<fixed::BinaryHeap<u8, Min, 4>, 0, 4>::try_from_iter([2, 1, 3]).unwrap();
        assert_eq!(heap.pop(), Some(1));
        assert_eq!(heap.pop(), Some(2));
        assert_eq!(heap.pop(), Some(3));
        assert_eq!(heap.pop(), None);
    }
}

mod index_set {
    use super::*;

    type Set<const MIN: usize, const MAX: usize> =
        Confined<fixed::IndexSet<u8, RandomState, 4>, MIN, MAX>;

    fn set_of<const MIN: usize, const MAX: usize>(items: &[u8]) -> Set<MIN, MAX> {
        Set::try_from_iter(items.iter().copied()).unwrap()
    }

    fn items<const MIN: usize, const MAX: usize>(set: &Set<MIN, MAX>) -> Vec<u8> {
        set.iter().copied().collect()
    }

    #[test]
    fn push_adds_a_new_item_within_both_limits() {
        let mut set = set_of::<0, 3>(&[1, 2]);
        assert_eq!(set.push(3), Ok(()));
        assert_eq!(items(&set), [1, 2, 3]);
    }

    #[test]
    fn push_refuses_a_new_item_at_the_maximum() {
        let mut set = set_of::<0, 2>(&[1, 2]);
        assert_eq!(set.push(3), Err(Error::Oversize { len: 2, max_len: 2 }));
        assert_eq!(items(&set), [1, 2]);
    }

    #[test]
    fn push_accepts_a_present_item_at_the_maximum() {
        let mut set = set_of::<0, 2>(&[1, 2]);
        assert_eq!(set.push(2), Ok(()));
        assert_eq!(items(&set), [1, 2]);
    }

    #[test]
    fn push_refuses_a_new_item_when_the_storage_is_full() {
        let mut set = set_of::<0, 8>(&[1, 2, 3, 4]);
        assert_eq!(set.push(5), Err(Error::CapacityExceeded { len: 4 }));
        assert_eq!(items(&set), [1, 2, 3, 4]);
    }

    #[test]
    fn push_accepts_a_present_item_when_the_storage_is_full() {
        let mut set = set_of::<0, 8>(&[1, 2, 3, 4]);
        assert_eq!(set.push(4), Ok(()));
        assert_eq!(items(&set), [1, 2, 3, 4]);
    }

    #[test]
    fn remove_removes_a_present_item() {
        let mut set = set_of::<1, 4>(&[1, 2]);
        assert_eq!(set.remove(&2), Ok(true));
        assert_eq!(items(&set), [1]);
    }

    #[test]
    fn remove_of_an_absent_item_is_not_an_error_at_the_minimum() {
        let mut set = set_of::<2, 4>(&[1, 2]);
        assert_eq!(set.remove(&3), Ok(false));
        assert_eq!(items(&set), [1, 2]);
    }

    #[test]
    fn remove_keeps_a_present_item_at_the_minimum() {
        let mut set = set_of::<2, 4>(&[1, 2]);
        assert_eq!(set.remove(&2), Err(Error::Undersize { len: 2, min_len: 2 }));
        assert_eq!(items(&set), [1, 2]);
    }

    #[test]
    fn remove_moves_the_last_item_into_the_place_of_the_removed_one() {
        // Unlike `indexmap`, `heapless` removes by swapping, and has no
        // operation which keeps the order.
        let mut set = set_of::<1, 4>(&[1, 2, 3, 4]);
        assert_eq!(set.remove(&1), Ok(true));
        assert_eq!(items(&set), [4, 2, 3]);
        assert!(set.contains(&4) && set.contains(&2) && set.contains(&3));
    }
}

mod index_map {
    use super::*;

    type Map<const MIN: usize, const MAX: usize> =
        Confined<fixed::IndexMap<u8, char, RandomState, 4>, MIN, MAX>;

    fn map_of<const MIN: usize, const MAX: usize>(keys: &[u8]) -> Map<MIN, MAX> {
        Map::try_from_iter(keys.iter().map(|key| (*key, char::from(b'a' + key - 1)))).unwrap()
    }

    fn entries<const MIN: usize, const MAX: usize>(map: &Map<MIN, MAX>) -> Vec<(u8, char)> {
        map.iter().map(|(key, value)| (*key, *value)).collect()
    }

    #[test]
    fn insert_adds_a_new_key_within_both_limits() {
        let mut map = map_of::<0, 3>(&[1, 2]);
        assert_eq!(map.insert(3, 'c'), Ok(None));
        assert_eq!(entries(&map), [(1, 'a'), (2, 'b'), (3, 'c')]);
    }

    #[test]
    fn insert_replaces_the_value_of_an_existing_key_at_the_maximum() {
        let mut map = map_of::<0, 2>(&[1, 2]);
        assert_eq!(map.insert(2, 'x'), Ok(Some('b')));
        assert_eq!(entries(&map), [(1, 'a'), (2, 'x')]);
    }

    #[test]
    fn insert_refuses_a_new_key_at_the_maximum() {
        let mut map = map_of::<0, 2>(&[1, 2]);
        assert_eq!(
            map.insert(3, 'c'),
            Err(Error::Oversize { len: 2, max_len: 2 })
        );
        assert_eq!(entries(&map), [(1, 'a'), (2, 'b')]);
    }

    #[test]
    fn insert_refuses_a_new_key_when_the_storage_is_full() {
        let mut map = map_of::<0, 8>(&[1, 2, 3, 4]);
        assert_eq!(map.insert(5, 'e'), Err(Error::CapacityExceeded { len: 4 }));
        assert_eq!(map.len(), 4);
    }

    #[test]
    fn insert_replaces_the_value_of_an_existing_key_when_the_storage_is_full() {
        let mut map = map_of::<0, 8>(&[1, 2, 3, 4]);
        assert_eq!(map.insert(4, 'x'), Ok(Some('d')));
        assert_eq!(map.get(&4), Some(&'x'));
    }

    #[test]
    fn push_refuses_a_new_key_when_the_storage_is_full() {
        let mut map = map_of::<0, 8>(&[1, 2, 3, 4]);
        assert_eq!(map.push((5, 'e')), Err(Error::CapacityExceeded { len: 4 }));
        assert_eq!(map.len(), 4);
    }

    #[test]
    fn vacant_entry_is_given_out_within_both_limits() {
        let mut map = map_of::<0, 3>(&[1, 2]);
        assert_eq!(*map.entry(3).unwrap().or_insert('c'), 'c');
        assert_eq!(entries(&map), [(1, 'a'), (2, 'b'), (3, 'c')]);
    }

    #[test]
    fn vacant_entry_is_refused_at_the_maximum() {
        let mut map = map_of::<0, 2>(&[1, 2]);
        assert_eq!(
            map.entry(3).err(),
            Some(Error::Oversize { len: 2, max_len: 2 })
        );
    }

    #[test]
    fn vacant_entry_is_refused_when_the_storage_is_full() {
        // The entry itself would panic on the insertion.
        let mut map = map_of::<0, 8>(&[1, 2, 3, 4]);
        assert_eq!(map.entry(5).err(), Some(Error::CapacityExceeded { len: 4 }));
        assert_eq!(map.len(), 4);
    }

    #[test]
    fn occupied_entry_is_given_out_when_the_storage_is_full() {
        let mut map = map_of::<0, 8>(&[1, 2, 3, 4]);
        *map.entry(4).unwrap().or_insert('x') = 'y';
        assert_eq!(map.get(&4), Some(&'y'));
    }

    #[test]
    fn remove_returns_the_value_of_a_present_key() {
        let mut map = map_of::<1, 4>(&[1, 2]);
        assert_eq!(map.remove(&2), Ok(Some('b')));
        assert_eq!(entries(&map), [(1, 'a')]);
    }

    #[test]
    fn remove_moves_the_last_entry_into_the_place_of_the_removed_one() {
        // Unlike `indexmap`, `heapless` removes by swapping, and has no
        // operation which keeps the order.
        let mut map = map_of::<1, 4>(&[1, 2, 3, 4]);
        assert_eq!(map.remove(&1), Ok(Some('a')));
        assert_eq!(entries(&map), [(4, 'd'), (2, 'b'), (3, 'c')]);
        assert_eq!(map.get(&4), Some(&'d'));
    }

    #[test]
    fn remove_of_an_absent_key_is_not_an_error_at_the_minimum() {
        let mut map = map_of::<2, 4>(&[1, 2]);
        assert_eq!(map.remove(&3), Ok(None));
        assert_eq!(entries(&map), [(1, 'a'), (2, 'b')]);
    }

    #[test]
    fn remove_keeps_a_present_key_at_the_minimum() {
        let mut map = map_of::<2, 4>(&[1, 2]);
        assert_eq!(map.remove(&2), Err(Error::Undersize { len: 2, min_len: 2 }));
        assert_eq!(entries(&map), [(1, 'a'), (2, 'b')]);
    }

    #[test]
    fn try_retain_stops_at_the_minimum() {
        let mut map = map_of::<2, 4>(&[1, 2, 3, 4]);
        assert_eq!(
            map.try_retain(|_, _| false),
            Err(Error::Undersize { len: 0, min_len: 2 })
        );
        assert_eq!(entries(&map), [(3, 'c'), (4, 'd')]);
        assert_eq!(map.get(&3), Some(&'c'));
        assert_eq!(map.get(&1), None);
    }

    #[test]
    fn retain_removes_the_rejected_entries() {
        let mut map = map_of::<0, 4>(&[1, 2, 3, 4]);
        map.retain(|key, _| key % 2 == 0);
        assert_eq!(entries(&map), [(2, 'b'), (4, 'd')]);
        assert_eq!(map.get(&4), Some(&'d'));
    }
}

mod linear_map {
    use super::*;

    type Map<const MIN: usize, const MAX: usize> =
        Confined<fixed::LinearMap<u8, char, 4>, MIN, MAX>;

    fn map_of<const MIN: usize, const MAX: usize>(keys: &[u8]) -> Map<MIN, MAX> {
        Map::try_from_iter(keys.iter().map(|key| (*key, char::from(b'a' + key - 1)))).unwrap()
    }

    #[test]
    fn insert_adds_a_new_key_within_both_limits() {
        let mut map = map_of::<0, 3>(&[1, 2]);
        assert_eq!(map.insert(3, 'c'), Ok(None));
        assert_eq!(map.get(&3), Some(&'c'));
        assert_eq!(map.len(), 3);
    }

    #[test]
    fn insert_replaces_the_value_of_an_existing_key_at_the_maximum() {
        let mut map = map_of::<0, 2>(&[1, 2]);
        assert_eq!(map.insert(2, 'x'), Ok(Some('b')));
        assert_eq!(map.get(&2), Some(&'x'));
        assert_eq!(map.len(), 2);
    }

    #[test]
    fn insert_refuses_a_new_key_at_the_maximum() {
        let mut map = map_of::<0, 2>(&[1, 2]);
        assert_eq!(
            map.insert(3, 'c'),
            Err(Error::Oversize { len: 2, max_len: 2 })
        );
        assert_eq!(map.len(), 2);
    }

    #[test]
    fn insert_refuses_a_new_key_when_the_storage_is_full() {
        let mut map = map_of::<0, 8>(&[1, 2, 3, 4]);
        assert_eq!(map.insert(5, 'e'), Err(Error::CapacityExceeded { len: 4 }));
        assert_eq!(map.len(), 4);
    }

    #[test]
    fn insert_replaces_the_value_of_an_existing_key_when_the_storage_is_full() {
        let mut map = map_of::<0, 8>(&[1, 2, 3, 4]);
        assert_eq!(map.insert(4, 'x'), Ok(Some('d')));
        assert_eq!(map.get(&4), Some(&'x'));
    }

    #[test]
    fn remove_returns_the_value_of_a_present_key() {
        let mut map = map_of::<1, 4>(&[1, 2]);
        assert_eq!(map.remove(&2), Ok(Some('b')));
        assert_eq!(map.len(), 1);
    }

    #[test]
    fn remove_of_an_absent_key_is_not_an_error_at_the_minimum() {
        let mut map = map_of::<2, 4>(&[1, 2]);
        assert_eq!(map.remove(&3), Ok(None));
        assert_eq!(map.len(), 2);
    }

    #[test]
    fn remove_keeps_a_present_key_at_the_minimum() {
        let mut map = map_of::<2, 4>(&[1, 2]);
        assert_eq!(map.remove(&2), Err(Error::Undersize { len: 2, min_len: 2 }));
        assert_eq!(map.len(), 2);
    }
}

mod reservation {
    use super::*;

    type Bytes<const MAX: usize> = Confined<fixed::Vec<u8, 4>, 0, MAX>;

    #[test]
    fn room_within_both_limits_is_available() {
        let mut bytes = Bytes::<8>::try_from_iter([1, 2]).unwrap();
        assert_eq!(bytes.try_reserve(2), Ok(()));
    }

    #[test]
    fn room_above_the_allowance_is_refused_first() {
        let mut bytes = Bytes::<3>::try_from_iter([1, 2]).unwrap();
        assert_eq!(
            bytes.try_reserve(2),
            Err(Error::Oversize { len: 2, max_len: 3 })
        );
    }

    #[test]
    fn room_above_the_storage_is_refused() {
        let mut bytes = Bytes::<8>::try_from_iter([1, 2]).unwrap();
        assert_eq!(
            bytes.try_reserve(3),
            Err(Error::CapacityExceeded { len: 2 })
        );
    }

    #[test]
    fn requested_capacity_is_checked_against_the_storage() {
        assert!(Bytes::<4>::try_with_capacity(100).is_ok());
        assert_eq!(
            Bytes::<8>::try_with_capacity(100),
            Err(Error::CapacityExceeded { len: 0 })
        );
    }
}

mod with {
    use super::*;

    #[test]
    fn single_item_is_stored() {
        let bytes = Confined::<fixed::Vec<u8, 4>, 1, 1>::with(7);
        assert_eq!(bytes.as_slice(), [7]);
    }

    #[test]
    fn single_character_is_stored() {
        let text = Confined::<fixed::String<8>, 1, 4>::with('€');
        assert_eq!(text.as_str(), "€");
    }

    #[test]
    #[should_panic(expected = "collection has no capacity")]
    fn storage_without_room_is_a_panic() {
        let _ = Confined::<fixed::Vec<u8, 0>, 0, 1>::with(7);
    }
}

mod aliases {
    use std::marker::PhantomData;

    use confined::{
        HeaplessDeque, HeaplessHeap, HeaplessIndexMap, HeaplessIndexSet, HeaplessLinearMap,
        HeaplessString, HeaplessVec,
    };
    use fixed::binary_heap::Max;

    use super::*;

    /// Builds only if both types are the same.
    fn same_type<T>(_: PhantomData<T>, _: PhantomData<T>) {}

    const ANY: usize = usize::MAX;

    #[test]
    fn aliases_default_to_the_widest_bounds() {
        same_type(
            PhantomData::<HeaplessVec<u8, 4>>,
            PhantomData::<Confined<fixed::Vec<u8, 4>, 0, ANY>>,
        );
        same_type(
            PhantomData::<HeaplessString<8>>,
            PhantomData::<Confined<fixed::String<8>, 0, ANY>>,
        );
        same_type(
            PhantomData::<HeaplessDeque<u8, 4>>,
            PhantomData::<Confined<fixed::Deque<u8, 4>, 0, ANY>>,
        );
        same_type(
            PhantomData::<HeaplessHeap<u8, Max, 4>>,
            PhantomData::<Confined<fixed::BinaryHeap<u8, Max, 4>, 0, ANY>>,
        );
        same_type(
            PhantomData::<HeaplessIndexSet<u8, RandomState, 4>>,
            PhantomData::<Confined<fixed::IndexSet<u8, RandomState, 4>, 0, ANY>>,
        );
        same_type(
            PhantomData::<HeaplessIndexMap<u8, char, RandomState, 4>>,
            PhantomData::<Confined<fixed::IndexMap<u8, char, RandomState, 4>, 0, ANY>>,
        );
        same_type(
            PhantomData::<HeaplessLinearMap<u8, char, 4>>,
            PhantomData::<Confined<fixed::LinearMap<u8, char, 4>, 0, ANY>>,
        );
    }

    #[test]
    fn aliases_take_the_bounds_after_the_capacity() {
        same_type(
            PhantomData::<HeaplessVec<u8, 4, 1, 3>>,
            PhantomData::<Confined<fixed::Vec<u8, 4>, 1, 3>>,
        );
        same_type(
            PhantomData::<HeaplessString<8, 1, 3>>,
            PhantomData::<Confined<fixed::String<8>, 1, 3>>,
        );
        same_type(
            PhantomData::<HeaplessLinearMap<u8, char, 4, 1, 3>>,
            PhantomData::<Confined<fixed::LinearMap<u8, char, 4>, 1, 3>>,
        );
    }

    #[test]
    fn vector_and_string_aliases_take_the_length_type_last() {
        same_type(
            PhantomData::<HeaplessVec<u8, 4, 0, 3, u8>>,
            PhantomData::<Confined<fixed::Vec<u8, 4, u8>, 0, 3>>,
        );
        same_type(
            PhantomData::<HeaplessString<8, 0, 3, u8>>,
            PhantomData::<Confined<fixed::String<8, u8>, 0, 3>>,
        );
    }
}
