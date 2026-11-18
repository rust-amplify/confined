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

//! Checked methods of the confined `indexmap::IndexSet` and
//! `indexmap::IndexMap` (`src/imp/indexed.rs`).
//!
//! These collections keep the order of insertion, and the confinement must
//! keep it too: a removal shifts the items after the removed one and never
//! swaps them. The helpers `items` and `entries` return the content in the
//! order of iteration, so each test checks the order together with the
//! content.
//!
//! Each module covers one method, and each test covers one case of it:
//!
//! - sets: `set_push`, `set_remove`, `set_take`, `set_first_last`;
//! - maps: `map_insert`, `map_remove`, `map_entry`, `map_entry_methods`,
//!   `map_try_retain`, `map_retain`, `map_into_keys`, `map_into_values`,
//!   `map_indexing`.
//!
//! The collections are given a hasher explicitly, so the file doesn't need
//! the `std` feature of the crate. The native capacity, the refused
//! allocations and the panic safety are covered by `tests/capacity.rs`,
//! `tests/oom.rs` and `tests/panic_safety.rs`.

#![cfg(feature = "indexmap")]

use std::hash::RandomState;

use confined::{Confined, Error};
use indexmap::{IndexMap, IndexSet};

type Set<const MIN: usize, const MAX: usize> = Confined<IndexSet<u8, RandomState>, MIN, MAX>;
type Map<const MIN: usize, const MAX: usize> = Confined<IndexMap<u8, char, RandomState>, MIN, MAX>;

fn set_of<const MIN: usize, const MAX: usize>(
    items: impl IntoIterator<Item = u8>,
) -> Set<MIN, MAX> {
    Set::try_from_iter(items).unwrap()
}

/// The items of the set in the order of the iteration.
fn items<const MIN: usize, const MAX: usize>(set: &Set<MIN, MAX>) -> Vec<u8> {
    set.iter().copied().collect()
}

/// The value stored under a key: `1 => 'a'`, `2 => 'b'` and so on.
fn letter(key: u8) -> char {
    char::from(b'a' + key - 1)
}

fn map_of<const MIN: usize, const MAX: usize>(keys: impl IntoIterator<Item = u8>) -> Map<MIN, MAX> {
    Map::try_from_iter(keys.into_iter().map(|key| (key, letter(key)))).unwrap()
}

/// The entries of the map in the order of the iteration.
fn entries<const MIN: usize, const MAX: usize>(map: &Map<MIN, MAX>) -> Vec<(u8, char)> {
    map.iter().map(|(key, value)| (*key, *value)).collect()
}

mod set_push {
    use super::*;

    #[test]
    fn new_item_is_added_at_the_end() {
        let mut set = set_of::<0, 3>([3, 1]);
        assert_eq!(set.push(2), Ok(()));
        assert_eq!(items(&set), [3, 1, 2]);
    }

    #[test]
    fn new_item_is_refused_at_the_maximum() {
        let mut set = set_of::<0, 2>([3, 1]);
        assert_eq!(set.push(2), Err(Error::Oversize { len: 2, max_len: 2 }));
        assert_eq!(items(&set), [3, 1]);
    }

    #[test]
    fn present_item_keeps_its_position() {
        let mut set = set_of::<0, 3>([3, 1, 2]);
        assert_eq!(set.push(3), Ok(()));
        assert_eq!(items(&set), [3, 1, 2]);
    }

    #[test]
    fn present_item_is_accepted_at_the_maximum() {
        let mut set = set_of::<0, 2>([3, 1]);
        assert_eq!(set.push(1), Ok(()));
        assert_eq!(items(&set), [3, 1]);
    }

    #[test]
    fn item_is_refused_by_a_zero_maximum() {
        let mut set = Set::<0, 0>::new();
        assert_eq!(set.push(1), Err(Error::Oversize { len: 0, max_len: 0 }));
        assert!(set.is_empty());
    }

    #[test]
    fn extension_keeps_the_order_of_the_first_occurrences() {
        let mut set = set_of::<0, 4>([3]);
        assert_eq!(set.extend([1, 3, 2, 1, 4]), Ok(()));
        assert_eq!(items(&set), [3, 1, 2, 4]);
    }

    #[test]
    fn extension_stops_at_the_first_new_item_which_does_not_fit() {
        let mut set = set_of::<0, 2>([3]);
        assert_eq!(
            set.extend([3, 1, 1, 2, 4]),
            Err(Error::Oversize { len: 2, max_len: 2 })
        );
        assert_eq!(items(&set), [3, 1]);
    }
}

mod set_remove {
    use super::*;

    #[test]
    fn first_item_is_removed_and_the_rest_keeps_its_order() {
        let mut set = set_of::<1, 4>([3, 1, 4, 2]);
        assert_eq!(set.remove(&3), Ok(true));
        assert_eq!(items(&set), [1, 4, 2]);
    }

    #[test]
    fn inner_item_is_removed_and_the_rest_keeps_its_order() {
        let mut set = set_of::<1, 4>([3, 1, 4, 2]);
        assert_eq!(set.remove(&1), Ok(true));
        assert_eq!(items(&set), [3, 4, 2]);
    }

    #[test]
    fn last_item_is_removed() {
        let mut set = set_of::<1, 4>([3, 1, 4, 2]);
        assert_eq!(set.remove(&2), Ok(true));
        assert_eq!(items(&set), [3, 1, 4]);
    }

    #[test]
    fn remaining_items_are_found_at_their_new_positions() {
        let mut set = set_of::<1, 4>([3, 1, 4, 2]);
        assert_eq!(set.remove(&3), Ok(true));
        assert_eq!(set.get_index_of(&1), Some(0));
        assert_eq!(set.get_index_of(&4), Some(1));
        assert_eq!(set.get_index_of(&2), Some(2));
        assert_eq!(set.get_index_of(&3), None);
    }

    #[test]
    fn absent_item_is_not_an_error() {
        let mut set = set_of::<1, 3>([3, 1]);
        assert_eq!(set.remove(&2), Ok(false));
        assert_eq!(items(&set), [3, 1]);
    }

    #[test]
    fn absent_item_is_not_an_error_at_the_minimum() {
        let mut set = set_of::<2, 3>([3, 1]);
        assert_eq!(set.remove(&2), Ok(false));
        assert_eq!(items(&set), [3, 1]);
    }

    #[test]
    fn present_item_is_kept_at_the_minimum() {
        let mut set = set_of::<2, 3>([3, 1]);
        assert_eq!(set.remove(&3), Err(Error::Undersize { len: 2, min_len: 2 }));
        assert_eq!(items(&set), [3, 1]);
    }

    #[test]
    fn only_item_is_removed_without_a_minimum() {
        let mut set = set_of::<0, 3>([1]);
        assert_eq!(set.remove(&1), Ok(true));
        assert!(set.is_empty());
    }
}

mod set_take {
    use super::*;

    #[test]
    fn present_item_is_returned_and_the_rest_keeps_its_order() {
        let mut set = set_of::<1, 4>([3, 1, 4, 2]);
        assert_eq!(set.take(&1), Ok(Some(1)));
        assert_eq!(items(&set), [3, 4, 2]);
    }

    #[test]
    fn absent_item_is_not_an_error() {
        let mut set = set_of::<1, 3>([3, 1]);
        assert_eq!(set.take(&2), Ok(None));
        assert_eq!(items(&set), [3, 1]);
    }

    #[test]
    fn absent_item_is_not_an_error_at_the_minimum() {
        let mut set = set_of::<2, 3>([3, 1]);
        assert_eq!(set.take(&2), Ok(None));
        assert_eq!(items(&set), [3, 1]);
    }

    #[test]
    fn present_item_is_kept_at_the_minimum() {
        let mut set = set_of::<2, 3>([3, 1]);
        assert_eq!(set.take(&3), Err(Error::Undersize { len: 2, min_len: 2 }));
        assert_eq!(items(&set), [3, 1]);
    }

    #[test]
    fn stored_item_is_returned_and_not_the_one_it_was_asked_with() {
        let stored = String::from("stored");
        let address = stored.as_ptr();
        let mut set = Confined::<IndexSet<String, RandomState>, 0, 1>::with(stored);
        let taken = set.take(&String::from("stored")).unwrap().unwrap();
        assert_eq!(taken.as_ptr(), address);
        assert!(set.is_empty());
    }
}

mod set_first_last {
    use super::*;

    #[test]
    fn ends_are_the_first_and_the_last_inserted_items() {
        let set = set_of::<1, 3>([3, 1, 2]);
        assert_eq!(*set.first(), 3);
        assert_eq!(*set.last(), 2);
    }

    #[test]
    fn single_item_is_at_both_ends() {
        let set = Set::<1, 3>::with(7);
        assert_eq!(*set.first(), 7);
        assert_eq!(*set.last(), 7);
    }

    #[test]
    fn ends_follow_the_removals() {
        let mut set = set_of::<1, 3>([3, 1, 2]);
        set.remove(&3).unwrap();
        assert_eq!(*set.first(), 1);
        set.remove(&2).unwrap();
        assert_eq!(*set.last(), 1);
    }
}

mod map_insert {
    use super::*;

    #[test]
    fn new_key_is_added_at_the_end() {
        let mut map = map_of::<0, 3>([3, 1]);
        assert_eq!(map.insert(2, 'b'), Ok(None));
        assert_eq!(entries(&map), [(3, 'c'), (1, 'a'), (2, 'b')]);
    }

    #[test]
    fn existing_key_keeps_its_position_when_its_value_is_replaced() {
        let mut map = map_of::<0, 3>([3, 1, 2]);
        assert_eq!(map.insert(3, 'x'), Ok(Some('c')));
        assert_eq!(entries(&map), [(3, 'x'), (1, 'a'), (2, 'b')]);
    }

    #[test]
    fn value_of_an_existing_key_is_replaced_at_the_maximum() {
        let mut map = map_of::<0, 2>([3, 1]);
        assert_eq!(map.insert(1, 'x'), Ok(Some('a')));
        assert_eq!(entries(&map), [(3, 'c'), (1, 'x')]);
    }

    #[test]
    fn new_key_is_refused_at_the_maximum() {
        let mut map = map_of::<0, 2>([3, 1]);
        assert_eq!(
            map.insert(2, 'b'),
            Err(Error::Oversize { len: 2, max_len: 2 })
        );
        assert_eq!(entries(&map), [(3, 'c'), (1, 'a')]);
    }

    #[test]
    fn push_of_an_existing_key_replaces_its_value_in_place() {
        let mut map = map_of::<0, 2>([3, 1]);
        assert_eq!(map.push((3, 'x')), Ok(()));
        assert_eq!(entries(&map), [(3, 'x'), (1, 'a')]);
    }

    #[test]
    fn push_of_a_new_key_is_refused_at_the_maximum() {
        let mut map = map_of::<0, 2>([3, 1]);
        assert_eq!(
            map.push((2, 'b')),
            Err(Error::Oversize { len: 2, max_len: 2 })
        );
        assert_eq!(entries(&map), [(3, 'c'), (1, 'a')]);
    }

    #[test]
    fn extension_keeps_the_positions_of_the_existing_keys() {
        let mut map = map_of::<0, 3>([3, 1]);
        assert_eq!(map.extend([(1, 'x'), (2, 'y'), (3, 'z')]), Ok(()));
        assert_eq!(entries(&map), [(3, 'z'), (1, 'x'), (2, 'y')]);
    }
}

mod map_remove {
    use super::*;

    #[test]
    fn first_entry_is_removed_and_the_rest_keeps_its_order() {
        let mut map = map_of::<1, 4>([3, 1, 4, 2]);
        assert_eq!(map.remove(&3), Ok(Some('c')));
        assert_eq!(entries(&map), [(1, 'a'), (4, 'd'), (2, 'b')]);
    }

    #[test]
    fn inner_entry_is_removed_and_the_rest_keeps_its_order() {
        let mut map = map_of::<1, 4>([3, 1, 4, 2]);
        assert_eq!(map.remove(&1), Ok(Some('a')));
        assert_eq!(entries(&map), [(3, 'c'), (4, 'd'), (2, 'b')]);
    }

    #[test]
    fn last_entry_is_removed() {
        let mut map = map_of::<1, 4>([3, 1, 4, 2]);
        assert_eq!(map.remove(&2), Ok(Some('b')));
        assert_eq!(entries(&map), [(3, 'c'), (1, 'a'), (4, 'd')]);
    }

    #[test]
    fn remaining_entries_are_found_at_their_new_positions() {
        let mut map = map_of::<1, 4>([3, 1, 4, 2]);
        assert_eq!(map.remove(&3), Ok(Some('c')));
        assert_eq!(map.get_index_of(&1), Some(0));
        assert_eq!(map.get_index_of(&4), Some(1));
        assert_eq!(map.get_index_of(&2), Some(2));
        assert_eq!(map.get(&2), Some(&'b'));
        assert_eq!(map.get(&3), None);
    }

    #[test]
    fn absent_key_is_not_an_error() {
        let mut map = map_of::<1, 3>([3, 1]);
        assert_eq!(map.remove(&2), Ok(None));
        assert_eq!(entries(&map), [(3, 'c'), (1, 'a')]);
    }

    #[test]
    fn absent_key_is_not_an_error_at_the_minimum() {
        let mut map = map_of::<2, 3>([3, 1]);
        assert_eq!(map.remove(&2), Ok(None));
        assert_eq!(entries(&map), [(3, 'c'), (1, 'a')]);
    }

    #[test]
    fn present_key_is_kept_at_the_minimum() {
        let mut map = map_of::<2, 3>([3, 1]);
        assert_eq!(map.remove(&3), Err(Error::Undersize { len: 2, min_len: 2 }));
        assert_eq!(entries(&map), [(3, 'c'), (1, 'a')]);
    }

    #[test]
    fn only_entry_is_removed_without_a_minimum() {
        let mut map = map_of::<0, 3>([1]);
        assert_eq!(map.remove(&1), Ok(Some('a')));
        assert!(map.is_empty());
    }
}

mod map_entry {
    use super::*;

    #[test]
    fn vacant_entry_is_filled_at_the_end() {
        let mut map = map_of::<0, 3>([3, 1]);
        assert_eq!(*map.entry(2).unwrap().or_insert('b'), 'b');
        assert_eq!(entries(&map), [(3, 'c'), (1, 'a'), (2, 'b')]);
    }

    #[test]
    fn vacant_entry_is_refused_at_the_maximum() {
        let mut map = map_of::<0, 2>([3, 1]);
        assert_eq!(
            map.entry(2).err(),
            Some(Error::Oversize { len: 2, max_len: 2 })
        );
        assert_eq!(entries(&map), [(3, 'c'), (1, 'a')]);
    }

    #[test]
    fn occupied_entry_is_given_out_at_the_maximum() {
        let mut map = map_of::<0, 2>([3, 1]);
        *map.entry(3).unwrap().or_insert('x') = 'y';
        assert_eq!(entries(&map), [(3, 'y'), (1, 'a')]);
    }

    #[test]
    fn unused_vacant_entry_adds_nothing() {
        let mut map = map_of::<0, 3>([3, 1]);
        let _ = map.entry(2).unwrap();
        assert_eq!(entries(&map), [(3, 'c'), (1, 'a')]);
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
        let mut map = map_of::<2, 4>([3, 1, 4, 2]);
        assert_eq!(map.try_retain(|_, _| true), Ok(()));
        assert_eq!(entries(&map), [(3, 'c'), (1, 'a'), (4, 'd'), (2, 'b')]);
    }

    #[test]
    fn rejected_entries_are_removed_and_the_rest_keeps_its_order() {
        let mut map = map_of::<2, 4>([3, 1, 4, 2]);
        assert_eq!(map.try_retain(|key, _| *key > 2), Ok(()));
        assert_eq!(entries(&map), [(3, 'c'), (4, 'd')]);
    }

    #[test]
    fn first_rejected_entries_are_removed_until_the_minimum() {
        // All the entries are rejected in the order of their insertion, and
        // only the first two of them may be removed.
        let mut map = map_of::<2, 4>([3, 1, 4, 2]);
        assert_eq!(
            map.try_retain(|_, _| false),
            Err(Error::Undersize { len: 0, min_len: 2 })
        );
        assert_eq!(entries(&map), [(4, 'd'), (2, 'b')]);
    }

    #[test]
    fn nothing_is_removed_at_the_minimum() {
        let mut map = map_of::<4, 4>([3, 1, 4, 2]);
        assert_eq!(
            map.try_retain(|key, _| *key != 1),
            Err(Error::Undersize { len: 3, min_len: 4 })
        );
        assert_eq!(entries(&map), [(3, 'c'), (1, 'a'), (4, 'd'), (2, 'b')]);
    }

    #[test]
    fn entries_are_visited_once_in_the_order_of_their_insertion() {
        let mut map = map_of::<2, 4>([3, 1, 4, 2]);
        let mut visited = vec![];
        let _ = map.try_retain(|key, _| {
            visited.push(*key);
            false
        });
        assert_eq!(visited, [3, 1, 4, 2]);
    }

    #[test]
    fn kept_entries_are_found_after_the_removal() {
        let mut map = map_of::<1, 4>([3, 1, 4, 2]);
        assert_eq!(map.try_retain(|key, _| *key % 2 == 0), Ok(()));
        assert_eq!(map.get(&4), Some(&'d'));
        assert_eq!(map.get(&2), Some(&'b'));
        assert_eq!(map.get(&3), None);
        assert_eq!(map.get_index_of(&2), Some(1));
    }
}

mod map_retain {
    use super::*;

    #[test]
    fn rejected_entries_are_removed_and_the_rest_keeps_its_order() {
        let mut map = map_of::<0, 4>([3, 1, 4, 2]);
        map.retain(|key, _| *key != 1);
        assert_eq!(entries(&map), [(3, 'c'), (4, 'd'), (2, 'b')]);
    }

    #[test]
    fn every_entry_may_be_removed() {
        let mut map = map_of::<0, 4>([3, 1, 4, 2]);
        map.retain(|_, _| false);
        assert!(map.is_empty());
    }
}

mod map_into_keys {
    use super::*;

    #[test]
    fn keys_are_returned_in_the_order_of_their_insertion() {
        let map = map_of::<1, 4>([3, 1, 4, 2]);
        assert_eq!(map.into_keys().collect::<Vec<_>>(), [3, 1, 4, 2]);
    }

    #[test]
    fn order_of_the_keys_survives_a_removal() {
        let mut map = map_of::<1, 4>([3, 1, 4, 2]);
        map.remove(&1).unwrap();
        assert_eq!(map.into_keys().collect::<Vec<_>>(), [3, 4, 2]);
    }
}

mod map_into_values {
    use super::*;

    #[test]
    fn values_are_returned_in_the_order_of_the_insertion_of_their_keys() {
        let map = map_of::<1, 4>([3, 1, 4, 2]);
        assert_eq!(map.into_values().collect::<Vec<_>>(), ['c', 'a', 'd', 'b']);
    }

    #[test]
    fn order_of_the_values_survives_a_removal() {
        let mut map = map_of::<1, 4>([3, 1, 4, 2]);
        map.remove(&1).unwrap();
        assert_eq!(map.into_values().collect::<Vec<_>>(), ['c', 'd', 'b']);
    }
}

mod map_indexing {
    use super::*;

    #[test]
    fn value_is_read_by_its_key_and_by_its_position() {
        let map = map_of::<1, 3>([3, 1, 2]);
        assert_eq!(map[&1], 'a');
        assert_eq!(map[0], 'c');
    }

    #[test]
    fn value_is_changed_by_its_key_and_by_its_position() {
        let mut map = map_of::<3, 3>([3, 1, 2]);
        map[&1] = 'x';
        map[0] = 'y';
        assert_eq!(entries(&map), [(3, 'y'), (1, 'x'), (2, 'b')]);
    }
}
