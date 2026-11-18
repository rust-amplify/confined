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

//! Checked methods of the confined `HashSet` and `HashMap`
//! (`src/imp/hashed.rs`).
//!
//! Each module covers one method, and each test covers one case of it:
//!
//! - sets: `set_push`, `set_remove`, `set_take`;
//! - maps: `map_insert`, `map_remove`, `map_remove_entry`, `map_entry`,
//!   `map_entry_methods`, `map_try_retain`, `map_retain`, `map_reserve`,
//!   `map_shrink_to_fit`, `map_shrink_to`, `map_into_keys`, `map_into_values`,
//!   `map_drain`;
//! - `custom_hasher`: the same methods of the collections which don't use the
//!   default hasher.
//!
//! A hash collection iterates in an arbitrary order, so the tests compare
//! sorted contents. The native capacity of the hash tables, the refused
//! allocations and the panic safety are covered by `tests/capacity.rs`,
//! `tests/oom.rs` and `tests/panic_safety.rs`.

#![cfg(feature = "std")]

use std::collections::{HashMap, HashSet};

use confined::{Confined, Error};

type Set<const MIN: usize, const MAX: usize> = Confined<HashSet<u8>, MIN, MAX>;
type Map<const MIN: usize, const MAX: usize> = Confined<HashMap<u8, char>, MIN, MAX>;

fn set_of<const MIN: usize, const MAX: usize>(
    items: impl IntoIterator<Item = u8>,
) -> Set<MIN, MAX> {
    Set::try_from_iter(items).unwrap()
}

/// Collects what an iterator yields and sorts it.
fn sorted<T: Ord>(iter: impl IntoIterator<Item = T>) -> Vec<T> {
    let mut items = iter.into_iter().collect::<Vec<_>>();
    items.sort_unstable();
    items
}

/// The items of the set in the ascending order.
fn items<const MIN: usize, const MAX: usize>(set: &Set<MIN, MAX>) -> Vec<u8> {
    sorted(set.iter().copied())
}

/// The value stored under a key: `1 => 'a'`, `2 => 'b'` and so on.
fn letter(key: u8) -> char {
    char::from(b'a' + key - 1)
}

fn map_of<const MIN: usize, const MAX: usize>(keys: impl IntoIterator<Item = u8>) -> Map<MIN, MAX> {
    Map::try_from_iter(keys.into_iter().map(|key| (key, letter(key)))).unwrap()
}

/// The entries of the map in the ascending order of their keys.
fn entries<const MIN: usize, const MAX: usize>(map: &Map<MIN, MAX>) -> Vec<(u8, char)> {
    sorted(map.iter().map(|(key, value)| (*key, *value)))
}

mod set_push {
    use super::*;

    #[test]
    fn new_item_is_added_below_the_maximum() {
        let mut set = set_of::<0, 3>([1, 2]);
        assert_eq!(set.push(3), Ok(()));
        assert_eq!(items(&set), [1, 2, 3]);
    }

    #[test]
    fn new_item_is_refused_at_the_maximum() {
        let mut set = set_of::<0, 2>([1, 2]);
        assert_eq!(set.push(3), Err(Error::Oversize { len: 2, max_len: 2 }));
        assert_eq!(items(&set), [1, 2]);
    }

    #[test]
    fn present_item_is_accepted_at_the_maximum() {
        let mut set = set_of::<0, 2>([1, 2]);
        assert_eq!(set.push(2), Ok(()));
        assert_eq!(items(&set), [1, 2]);
    }

    #[test]
    fn present_item_is_accepted_below_the_maximum() {
        let mut set = set_of::<0, 3>([1, 2]);
        assert_eq!(set.push(1), Ok(()));
        assert_eq!(items(&set), [1, 2]);
    }

    #[test]
    fn item_is_refused_by_a_zero_maximum() {
        let mut set = Set::<0, 0>::new();
        assert_eq!(set.push(1), Err(Error::Oversize { len: 0, max_len: 0 }));
        assert!(set.is_empty());
    }

    #[test]
    fn extension_counts_each_item_once() {
        let mut set = set_of::<0, 3>([1]);
        assert_eq!(set.extend([2, 1, 2, 3, 3]), Ok(()));
        assert_eq!(items(&set), [1, 2, 3]);
    }

    #[test]
    fn extension_stops_at_the_first_new_item_which_does_not_fit() {
        let mut set = set_of::<0, 2>([1]);
        assert_eq!(
            set.extend([1, 2, 2, 3, 4]),
            Err(Error::Oversize { len: 2, max_len: 2 })
        );
        assert_eq!(items(&set), [1, 2]);
    }
}

mod set_remove {
    use super::*;

    #[test]
    fn present_item_is_removed() {
        let mut set = set_of::<1, 3>([1, 2]);
        assert_eq!(set.remove(&2), Ok(true));
        assert_eq!(items(&set), [1]);
    }

    #[test]
    fn absent_item_is_not_an_error() {
        let mut set = set_of::<1, 3>([1, 2]);
        assert_eq!(set.remove(&3), Ok(false));
        assert_eq!(items(&set), [1, 2]);
    }

    #[test]
    fn absent_item_is_not_an_error_at_the_minimum() {
        let mut set = set_of::<2, 3>([1, 2]);
        assert_eq!(set.remove(&3), Ok(false));
        assert_eq!(items(&set), [1, 2]);
    }

    #[test]
    fn present_item_is_kept_at_the_minimum() {
        let mut set = set_of::<2, 3>([1, 2]);
        assert_eq!(set.remove(&2), Err(Error::Undersize { len: 2, min_len: 2 }));
        assert_eq!(items(&set), [1, 2]);
    }

    #[test]
    fn last_item_is_removed_without_a_minimum() {
        let mut set = set_of::<0, 3>([1]);
        assert_eq!(set.remove(&1), Ok(true));
        assert!(set.is_empty());
    }

    #[test]
    fn empty_set_has_nothing_to_remove() {
        let mut set = Set::<0, 3>::new();
        assert_eq!(set.remove(&1), Ok(false));
        assert!(set.is_empty());
    }
}

mod set_take {
    use super::*;

    #[test]
    fn present_item_is_returned() {
        let mut set = set_of::<1, 3>([1, 2]);
        assert_eq!(set.take(&2), Ok(Some(2)));
        assert_eq!(items(&set), [1]);
    }

    #[test]
    fn absent_item_is_not_an_error() {
        let mut set = set_of::<1, 3>([1, 2]);
        assert_eq!(set.take(&3), Ok(None));
        assert_eq!(items(&set), [1, 2]);
    }

    #[test]
    fn absent_item_is_not_an_error_at_the_minimum() {
        let mut set = set_of::<2, 3>([1, 2]);
        assert_eq!(set.take(&3), Ok(None));
        assert_eq!(items(&set), [1, 2]);
    }

    #[test]
    fn present_item_is_kept_at_the_minimum() {
        let mut set = set_of::<2, 3>([1, 2]);
        assert_eq!(set.take(&2), Err(Error::Undersize { len: 2, min_len: 2 }));
        assert_eq!(items(&set), [1, 2]);
    }

    #[test]
    fn stored_item_is_returned_and_not_the_one_it_was_asked_with() {
        let stored = String::from("stored");
        let address = stored.as_ptr();
        let mut set = Confined::<HashSet<String>, 0, 1>::with(stored);
        let taken = set.take(&String::from("stored")).unwrap().unwrap();
        assert_eq!(taken.as_ptr(), address);
        assert!(set.is_empty());
    }
}

mod map_insert {
    use std::hash::{Hash, Hasher};

    use super::*;

    #[test]
    fn new_key_is_added_below_the_maximum() {
        let mut map = map_of::<0, 3>([1, 2]);
        assert_eq!(map.insert(3, 'c'), Ok(None));
        assert_eq!(entries(&map), [(1, 'a'), (2, 'b'), (3, 'c')]);
    }

    #[test]
    fn value_of_an_existing_key_is_replaced_and_returned() {
        let mut map = map_of::<0, 3>([1, 2]);
        assert_eq!(map.insert(2, 'x'), Ok(Some('b')));
        assert_eq!(entries(&map), [(1, 'a'), (2, 'x')]);
    }

    #[test]
    fn value_of_an_existing_key_is_replaced_at_the_maximum() {
        let mut map = map_of::<0, 2>([1, 2]);
        assert_eq!(map.insert(2, 'x'), Ok(Some('b')));
        assert_eq!(entries(&map), [(1, 'a'), (2, 'x')]);
    }

    #[test]
    fn new_key_is_refused_at_the_maximum() {
        let mut map = map_of::<0, 2>([1, 2]);
        assert_eq!(
            map.insert(3, 'c'),
            Err(Error::Oversize { len: 2, max_len: 2 })
        );
        assert_eq!(entries(&map), [(1, 'a'), (2, 'b')]);
    }

    #[test]
    fn new_key_is_refused_by_a_zero_maximum() {
        let mut map = Map::<0, 0>::new();
        assert_eq!(
            map.insert(1, 'a'),
            Err(Error::Oversize { len: 0, max_len: 0 })
        );
        assert!(map.is_empty());
    }

    #[test]
    fn push_of_an_existing_key_replaces_its_value_at_the_maximum() {
        let mut map = map_of::<0, 2>([1, 2]);
        assert_eq!(map.push((2, 'x')), Ok(()));
        assert_eq!(entries(&map), [(1, 'a'), (2, 'x')]);
    }

    #[test]
    fn push_of_a_new_key_is_refused_at_the_maximum() {
        let mut map = map_of::<0, 2>([1, 2]);
        assert_eq!(
            map.push((3, 'c')),
            Err(Error::Oversize { len: 2, max_len: 2 })
        );
        assert_eq!(entries(&map), [(1, 'a'), (2, 'b')]);
    }

    /// A key hashed and compared by its number only. The label tells equal
    /// keys apart.
    #[derive(Debug)]
    struct Labelled(u8, &'static str);

    impl PartialEq for Labelled {
        fn eq(&self, other: &Self) -> bool {
            self.0 == other.0
        }
    }

    impl Eq for Labelled {}

    impl Hash for Labelled {
        fn hash<H: Hasher>(&self, state: &mut H) {
            self.0.hash(state);
        }
    }

    #[test]
    fn stored_key_is_kept_when_its_value_is_replaced() {
        let mut map =
            Confined::<HashMap<Labelled, char>, 0, 1>::with_key_value(Labelled(1, "stored"), 'a');
        assert_eq!(map.insert(Labelled(1, "inserted"), 'x'), Ok(Some('a')));
        let (key, value) = map.iter().next().unwrap();
        assert_eq!((key.1, *value), ("stored", 'x'));
        assert_eq!(map.len(), 1);
    }
}

mod map_remove {
    use super::*;

    #[test]
    fn present_key_is_removed() {
        let mut map = map_of::<1, 3>([1, 2]);
        assert_eq!(map.remove(&2), Ok(Some('b')));
        assert_eq!(entries(&map), [(1, 'a')]);
    }

    #[test]
    fn absent_key_is_not_an_error() {
        let mut map = map_of::<1, 3>([1, 2]);
        assert_eq!(map.remove(&3), Ok(None));
        assert_eq!(entries(&map), [(1, 'a'), (2, 'b')]);
    }

    #[test]
    fn absent_key_is_not_an_error_at_the_minimum() {
        let mut map = map_of::<2, 3>([1, 2]);
        assert_eq!(map.remove(&3), Ok(None));
        assert_eq!(entries(&map), [(1, 'a'), (2, 'b')]);
    }

    #[test]
    fn present_key_is_kept_at_the_minimum() {
        let mut map = map_of::<2, 3>([1, 2]);
        assert_eq!(map.remove(&2), Err(Error::Undersize { len: 2, min_len: 2 }));
        assert_eq!(entries(&map), [(1, 'a'), (2, 'b')]);
    }

    #[test]
    fn last_entry_is_removed_without_a_minimum() {
        let mut map = map_of::<0, 3>([1]);
        assert_eq!(map.remove(&1), Ok(Some('a')));
        assert!(map.is_empty());
    }
}

mod map_remove_entry {
    use super::*;

    #[test]
    fn present_key_is_removed_with_its_value() {
        let mut map = map_of::<1, 3>([1, 2]);
        assert_eq!(map.remove_entry(&2), Ok(Some((2, 'b'))));
        assert_eq!(entries(&map), [(1, 'a')]);
    }

    #[test]
    fn absent_key_is_not_an_error() {
        let mut map = map_of::<1, 3>([1, 2]);
        assert_eq!(map.remove_entry(&3), Ok(None));
        assert_eq!(entries(&map), [(1, 'a'), (2, 'b')]);
    }

    #[test]
    fn absent_key_is_not_an_error_at_the_minimum() {
        let mut map = map_of::<2, 3>([1, 2]);
        assert_eq!(map.remove_entry(&3), Ok(None));
        assert_eq!(entries(&map), [(1, 'a'), (2, 'b')]);
    }

    #[test]
    fn present_key_is_kept_at_the_minimum() {
        let mut map = map_of::<2, 3>([1, 2]);
        assert_eq!(
            map.remove_entry(&2),
            Err(Error::Undersize { len: 2, min_len: 2 })
        );
        assert_eq!(entries(&map), [(1, 'a'), (2, 'b')]);
    }

    #[test]
    fn stored_key_is_returned_and_not_the_one_it_was_asked_with() {
        let stored = String::from("stored");
        let address = stored.as_ptr();
        let mut map = Confined::<HashMap<String, char>, 0, 1>::with_key_value(stored, 'a');
        let (key, value) = map.remove_entry(&String::from("stored")).unwrap().unwrap();
        assert_eq!((key.as_ptr(), value), (address, 'a'));
        assert!(map.is_empty());
    }
}

mod map_entry {
    use super::*;

    #[test]
    fn vacant_entry_is_given_out_below_the_maximum() {
        let mut map = map_of::<0, 3>([1, 2]);
        assert_eq!(*map.entry(3).unwrap().or_insert('c'), 'c');
        assert_eq!(entries(&map), [(1, 'a'), (2, 'b'), (3, 'c')]);
    }

    #[test]
    fn vacant_entry_is_refused_at_the_maximum() {
        let mut map = map_of::<0, 2>([1, 2]);
        assert_eq!(
            map.entry(3).err(),
            Some(Error::Oversize { len: 2, max_len: 2 })
        );
        assert_eq!(entries(&map), [(1, 'a'), (2, 'b')]);
    }

    #[test]
    fn vacant_entry_is_refused_by_a_zero_maximum() {
        let mut map = Map::<0, 0>::new();
        assert_eq!(
            map.entry(1).err(),
            Some(Error::Oversize { len: 0, max_len: 0 })
        );
        assert!(map.is_empty());
    }

    #[test]
    fn occupied_entry_is_given_out_at_the_maximum() {
        let mut map = map_of::<0, 2>([1, 2]);
        *map.entry(2).unwrap().or_insert('x') = 'y';
        assert_eq!(entries(&map), [(1, 'a'), (2, 'y')]);
    }

    #[test]
    fn unused_vacant_entry_adds_nothing() {
        let mut map = map_of::<0, 3>([1, 2]);
        let _ = map.entry(3).unwrap();
        assert_eq!(entries(&map), [(1, 'a'), (2, 'b')]);
    }

    #[test]
    fn entries_fill_the_map_up_to_the_maximum() {
        let mut map = Map::<0, 2>::new();
        for key in [1, 2, 1, 2] {
            map.entry(key).unwrap().or_insert(letter(key));
        }
        assert!(map.entry(3).is_err());
        assert_eq!(entries(&map), [(1, 'a'), (2, 'b')]);
    }
}

mod map_entry_methods {
    use super::*;

    #[test]
    fn key_of_a_vacant_entry_is_returned() {
        let mut map = map_of::<0, 3>([1]);
        assert_eq!(*map.entry(2).unwrap().key(), 2);
    }

    #[test]
    fn key_of_an_occupied_entry_is_returned() {
        let mut map = map_of::<0, 3>([1]);
        assert_eq!(*map.entry(1).unwrap().key(), 1);
    }

    #[test]
    fn or_insert_fills_a_vacant_entry() {
        let mut map = map_of::<0, 3>([1]);
        assert_eq!(*map.entry(2).unwrap().or_insert('x'), 'x');
        assert_eq!(entries(&map), [(1, 'a'), (2, 'x')]);
    }

    #[test]
    fn or_insert_keeps_the_value_of_an_occupied_entry() {
        let mut map = map_of::<0, 3>([1]);
        assert_eq!(*map.entry(1).unwrap().or_insert('x'), 'a');
        assert_eq!(entries(&map), [(1, 'a')]);
    }

    #[test]
    fn or_insert_with_fills_a_vacant_entry() {
        let mut map = map_of::<0, 3>([1]);
        assert_eq!(*map.entry(2).unwrap().or_insert_with(|| 'x'), 'x');
        assert_eq!(entries(&map), [(1, 'a'), (2, 'x')]);
    }

    #[test]
    fn or_insert_with_does_not_call_the_default_for_an_occupied_entry() {
        let mut map = map_of::<0, 3>([1]);
        assert_eq!(
            *map.entry(1).unwrap().or_insert_with(|| unreachable!()),
            'a'
        );
    }

    #[test]
    fn or_insert_with_key_gives_the_key_to_the_default_of_a_vacant_entry() {
        let mut map = map_of::<0, 3>([1]);
        assert_eq!(
            *map.entry(2).unwrap().or_insert_with_key(|key| letter(*key)),
            'b'
        );
        assert_eq!(entries(&map), [(1, 'a'), (2, 'b')]);
    }

    #[test]
    fn or_insert_with_key_does_not_call_the_default_for_an_occupied_entry() {
        let mut map = map_of::<0, 3>([1]);
        assert_eq!(
            *map.entry(1).unwrap().or_insert_with_key(|_| unreachable!()),
            'a'
        );
    }

    #[test]
    fn or_default_fills_a_vacant_entry_with_the_default_value() {
        let mut map = map_of::<0, 3>([1]);
        assert_eq!(*map.entry(2).unwrap().or_default(), char::default());
        assert_eq!(map.len(), 2);
    }

    #[test]
    fn or_default_keeps_the_value_of_an_occupied_entry() {
        let mut map = map_of::<0, 3>([1]);
        assert_eq!(*map.entry(1).unwrap().or_default(), 'a');
    }

    #[test]
    fn and_modify_changes_the_value_of_an_occupied_entry() {
        let mut map = map_of::<0, 3>([1]);
        let entry = map.entry(1).unwrap().and_modify(|value| *value = 'x');
        assert_eq!(*entry.or_insert('y'), 'x');
        assert_eq!(entries(&map), [(1, 'x')]);
    }

    #[test]
    fn and_modify_leaves_a_vacant_entry_vacant() {
        let mut map = map_of::<0, 3>([1]);
        let entry = map.entry(2).unwrap().and_modify(|_| unreachable!());
        assert_eq!(*entry.or_insert('y'), 'y');
        assert_eq!(entries(&map), [(1, 'a'), (2, 'y')]);
    }
}

mod map_try_retain {
    use super::*;

    #[test]
    fn accepted_entries_are_all_kept() {
        let mut map = map_of::<2, 4>([1, 2, 3, 4]);
        assert_eq!(map.try_retain(|_, _| true), Ok(()));
        assert_eq!(map.len(), 4);
    }

    #[test]
    fn rejected_entries_are_removed_above_the_minimum() {
        let mut map = map_of::<2, 4>([1, 2, 3, 4]);
        assert_eq!(map.try_retain(|key, _| key % 2 == 0), Ok(()));
        assert_eq!(entries(&map), [(2, 'b'), (4, 'd')]);
    }

    #[test]
    fn removal_stops_at_the_minimum() {
        // Three entries are rejected, and only two of them may be removed.
        // Which of them stays depends on the order of the iteration.
        let mut map = map_of::<2, 4>([1, 2, 3, 4]);
        assert_eq!(
            map.try_retain(|key, _| *key == 4),
            Err(Error::Undersize { len: 1, min_len: 2 })
        );
        assert_eq!(map.len(), 2);
        assert_eq!(map.get(&4), Some(&'d'));
    }

    #[test]
    fn nothing_is_removed_at_the_minimum() {
        let mut map = map_of::<4, 4>([1, 2, 3, 4]);
        assert_eq!(
            map.try_retain(|_, _| false),
            Err(Error::Undersize { len: 0, min_len: 4 })
        );
        assert_eq!(entries(&map), [(1, 'a'), (2, 'b'), (3, 'c'), (4, 'd')]);
    }

    #[test]
    fn everything_is_removed_without_a_minimum() {
        let mut map = map_of::<0, 4>([1, 2, 3, 4]);
        assert_eq!(map.try_retain(|_, _| false), Ok(()));
        assert!(map.is_empty());
    }

    #[test]
    fn predicate_is_called_once_for_each_entry() {
        let mut map = map_of::<2, 4>([1, 2, 3, 4]);
        let mut visited = vec![];
        let _ = map.try_retain(|key, _| {
            visited.push(*key);
            false
        });
        assert_eq!(sorted(visited), [1, 2, 3, 4]);
    }

    #[test]
    fn predicate_may_change_the_values_it_keeps() {
        let mut map = map_of::<2, 4>([1, 2]);
        let result = map.try_retain(|_, value| {
            *value = value.to_ascii_uppercase();
            true
        });
        assert_eq!(result, Ok(()));
        assert_eq!(entries(&map), [(1, 'A'), (2, 'B')]);
    }
}

mod map_retain {
    use super::*;

    #[test]
    fn rejected_entries_are_removed() {
        let mut map = map_of::<0, 4>([1, 2, 3]);
        map.retain(|key, _| *key != 2);
        assert_eq!(entries(&map), [(1, 'a'), (3, 'c')]);
    }

    #[test]
    fn every_entry_may_be_removed() {
        let mut map = map_of::<0, 4>([1, 2, 3]);
        map.retain(|_, _| false);
        assert!(map.is_empty());
    }

    #[test]
    fn accepted_entries_are_all_kept() {
        let mut map = map_of::<0, 4>([1, 2, 3]);
        map.retain(|_, _| true);
        assert_eq!(entries(&map), [(1, 'a'), (2, 'b'), (3, 'c')]);
    }
}

mod map_reserve {
    use super::*;

    #[test]
    fn room_for_the_requested_entries_is_reserved() {
        let mut map = map_of::<0, 100>([1, 2]);
        map.reserve(10);
        assert!(map.capacity() >= 12);
        assert_eq!(entries(&map), [(1, 'a'), (2, 'b')]);
    }

    #[test]
    fn request_is_capped_at_the_room_left_below_the_maximum() {
        // An uncapped request of this size would be a capacity overflow.
        let mut map = map_of::<0, 4>([1, 2]);
        map.reserve(usize::MAX);
        assert!(map.capacity() >= 4);
        assert_eq!(entries(&map), [(1, 'a'), (2, 'b')]);
    }

    #[test]
    fn nothing_is_reserved_at_the_maximum() {
        let mut map = map_of::<0, 2>([1, 2]);
        let capacity = map.capacity();
        map.reserve(usize::MAX);
        assert_eq!(map.capacity(), capacity);
    }

    #[test]
    fn zero_request_changes_nothing() {
        let mut map = map_of::<0, 100>([1, 2]);
        let capacity = map.capacity();
        map.reserve(0);
        assert_eq!(map.capacity(), capacity);
    }

    #[test]
    fn reserved_room_is_filled_without_another_reservation() {
        let mut map = Map::<0, 8>::new();
        map.reserve(8);
        let capacity = map.capacity();
        for key in 1..=8 {
            assert_eq!(map.insert(key, letter(key)), Ok(None));
        }
        assert_eq!(map.capacity(), capacity);
    }
}

mod map_shrink_to_fit {
    use super::*;

    #[test]
    fn spare_capacity_is_released() {
        let mut map = Map::<0, 100>::with_capacity(64);
        map.insert(1, 'a').unwrap();
        let capacity = map.capacity();
        map.shrink_to_fit();
        assert!(map.capacity() < capacity);
        assert!(map.capacity() >= map.len());
        assert_eq!(entries(&map), [(1, 'a')]);
    }

    #[test]
    fn empty_map_releases_its_capacity() {
        let mut map = Map::<0, 100>::with_capacity(64);
        let capacity = map.capacity();
        map.shrink_to_fit();
        assert!(map.capacity() < capacity);
        assert!(map.is_empty());
    }
}

mod map_shrink_to {
    use super::*;

    #[test]
    fn capacity_is_reduced_down_to_the_requested_one() {
        let mut map = Map::<0, 100>::with_capacity(64);
        map.insert(1, 'a').unwrap();
        let capacity = map.capacity();
        map.shrink_to(10);
        assert!(map.capacity() >= 10);
        assert!(map.capacity() < capacity);
        assert_eq!(entries(&map), [(1, 'a')]);
    }

    #[test]
    fn capacity_is_never_reduced_below_the_length() {
        let mut map = Map::<0, 100>::with_capacity(64);
        for key in 1..=20 {
            map.insert(key, letter(key)).unwrap();
        }
        map.shrink_to(0);
        assert!(map.capacity() >= 20);
        assert_eq!(map.len(), 20);
    }

    #[test]
    fn request_above_the_capacity_changes_nothing() {
        let mut map = map_of::<0, 100>([1, 2]);
        let capacity = map.capacity();
        map.shrink_to(capacity + 50);
        assert_eq!(map.capacity(), capacity);
    }
}

mod map_into_keys {
    use super::*;

    #[test]
    fn every_key_is_returned_once() {
        let map = map_of::<1, 3>([3, 1, 2]);
        assert_eq!(sorted(map.into_keys()), [1, 2, 3]);
    }

    #[test]
    fn empty_map_has_no_keys() {
        assert_eq!(Map::<0, 3>::new().into_keys().count(), 0);
    }
}

mod map_into_values {
    use super::*;

    #[test]
    fn every_value_is_returned_once() {
        let map = map_of::<1, 3>([3, 1, 2]);
        assert_eq!(sorted(map.into_values()), ['a', 'b', 'c']);
    }

    #[test]
    fn empty_map_has_no_values() {
        assert_eq!(Map::<0, 3>::new().into_values().count(), 0);
    }
}

mod map_drain {
    use super::*;

    #[test]
    fn every_entry_is_yielded_and_the_map_is_emptied() {
        let mut map = map_of::<0, 3>([1, 2, 3]);
        assert_eq!(sorted(map.drain()), [(1, 'a'), (2, 'b'), (3, 'c')]);
        assert!(map.is_empty());
    }

    #[test]
    fn map_is_emptied_when_the_iterator_is_dropped_unused() {
        let mut map = map_of::<0, 3>([1, 2, 3]);
        drop(map.drain());
        assert!(map.is_empty());
    }

    #[test]
    fn allocated_memory_is_kept_for_reuse() {
        let mut map = map_of::<0, 3>([1, 2, 3]);
        let capacity = map.capacity();
        assert_eq!(map.drain().count(), 3);
        assert_eq!(map.capacity(), capacity);
    }

    #[test]
    fn drained_map_is_filled_up_to_the_maximum_again() {
        let mut map = map_of::<0, 2>([1, 2]);
        assert_eq!(map.drain().count(), 2);
        assert_eq!(map.insert(3, 'c'), Ok(None));
        assert_eq!(map.insert(4, 'd'), Ok(None));
        assert!(map.insert(5, 'e').is_err());
        assert_eq!(entries(&map), [(3, 'c'), (4, 'd')]);
    }

    #[test]
    fn empty_map_yields_nothing() {
        let mut map = Map::<0, 3>::new();
        assert_eq!(map.drain().count(), 0);
    }
}

mod custom_hasher {
    use std::hash::{BuildHasherDefault, DefaultHasher};

    use super::*;

    type Hasher = BuildHasherDefault<DefaultHasher>;
    type Set<const MIN: usize, const MAX: usize> = Confined<HashSet<u8, Hasher>, MIN, MAX>;
    type Map<const MIN: usize, const MAX: usize> = Confined<HashMap<u8, char, Hasher>, MIN, MAX>;

    fn letters<const MIN: usize, const MAX: usize>() -> Map<MIN, MAX> {
        Map::try_from_iter([(1, 'a'), (2, 'b')]).unwrap()
    }

    #[test]
    fn set_is_filled_and_emptied_within_its_bounds() {
        let mut set = Set::<1, 2>::try_from_iter([1]).unwrap();
        assert_eq!(set.push(2), Ok(()));
        assert_eq!(set.push(2), Ok(()));
        assert_eq!(set.push(3), Err(Error::Oversize { len: 2, max_len: 2 }));
        assert_eq!(set.take(&2), Ok(Some(2)));
        assert_eq!(set.remove(&1), Err(Error::Undersize { len: 1, min_len: 1 }));
        assert_eq!(sorted(set.iter().copied()), [1]);
    }

    #[test]
    fn map_is_filled_within_its_upper_bound() {
        let mut map = letters::<0, 3>();
        assert_eq!(map.insert(2, 'x'), Ok(Some('b')));
        assert_eq!(
            *map.entry(3).unwrap().or_insert_with_key(|key| letter(*key)),
            'c'
        );
        assert_eq!(
            map.insert(4, 'd'),
            Err(Error::Oversize { len: 3, max_len: 3 })
        );
        assert!(map.entry(4).is_err());
        assert_eq!(sorted(map.into_values()), ['a', 'c', 'x']);
    }

    #[test]
    fn map_is_emptied_within_its_lower_bound() {
        let mut map = letters::<1, 3>();
        assert_eq!(map.remove_entry(&2), Ok(Some((2, 'b'))));
        assert_eq!(map.remove(&1), Err(Error::Undersize { len: 1, min_len: 1 }));
        assert_eq!(
            map.try_retain(|_, _| false),
            Err(Error::Undersize { len: 0, min_len: 1 })
        );
        assert_eq!(sorted(map.into_keys()), [1]);
    }

    #[test]
    fn capacity_of_a_map_is_managed() {
        let mut map = letters::<0, 4>();
        map.reserve(usize::MAX);
        assert!(map.capacity() >= 4);
        map.shrink_to(2);
        map.shrink_to_fit();
        assert!(map.capacity() >= 2);
        assert_eq!(map.drain().count(), 2);
        assert!(map.is_empty());
    }
}
