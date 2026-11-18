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

//! Checked methods of the confined `BTreeSet` and `BTreeMap`
//! (`src/imp/btree.rs`).
//!
//! Each module covers one method, and each test covers one case of it:
//!
//! - sets: `set_push`, `set_remove`, `set_take`;
//! - maps: `map_remove`, `map_remove_entry`, `map_pop_first`, `map_pop_last`,
//!   `map_append`, `map_split_off`, `map_range_mut`, `map_into_keys`,
//!   `map_into_values`.
//!
//! The panic safety of `append` and `split_off` is covered by
//! `tests/panic_safety.rs` and `tests/audit.rs`.

#![cfg(feature = "alloc")]

use std::collections::{BTreeMap, BTreeSet};

use confined::{Confined, Error};

type Set<const MIN: usize, const MAX: usize> = Confined<BTreeSet<u8>, MIN, MAX>;
type Map<const MIN: usize, const MAX: usize> = Confined<BTreeMap<u8, char>, MIN, MAX>;

fn set_of<const MIN: usize, const MAX: usize>(
    items: impl IntoIterator<Item = u8>,
) -> Set<MIN, MAX> {
    Set::try_from_iter(items).unwrap()
}

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

fn entries<const MIN: usize, const MAX: usize>(map: &Map<MIN, MAX>) -> Vec<(u8, char)> {
    map.iter().map(|(key, value)| (*key, *value)).collect()
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
}

mod map_pop_first {
    use super::*;

    #[test]
    fn first_entry_is_removed() {
        let mut map = map_of::<1, 3>([1, 2]);
        assert_eq!(map.pop_first(), Ok(Some((1, 'a'))));
        assert_eq!(entries(&map), [(2, 'b')]);
    }

    #[test]
    fn entry_is_kept_at_the_minimum() {
        let mut map = map_of::<2, 3>([1, 2]);
        assert_eq!(
            map.pop_first(),
            Err(Error::Undersize { len: 2, min_len: 2 })
        );
        assert_eq!(entries(&map), [(1, 'a'), (2, 'b')]);
    }

    #[test]
    fn empty_map_has_nothing_to_remove() {
        let mut map = Map::<0, 3>::new();
        assert_eq!(map.pop_first(), Ok(None));
        assert!(map.is_empty());
    }
}

mod map_pop_last {
    use super::*;

    #[test]
    fn last_entry_is_removed() {
        let mut map = map_of::<1, 3>([1, 2]);
        assert_eq!(map.pop_last(), Ok(Some((2, 'b'))));
        assert_eq!(entries(&map), [(1, 'a')]);
    }

    #[test]
    fn entry_is_kept_at_the_minimum() {
        let mut map = map_of::<2, 3>([1, 2]);
        assert_eq!(map.pop_last(), Err(Error::Undersize { len: 2, min_len: 2 }));
        assert_eq!(entries(&map), [(1, 'a'), (2, 'b')]);
    }

    #[test]
    fn empty_map_has_nothing_to_remove() {
        let mut map = Map::<0, 3>::new();
        assert_eq!(map.pop_last(), Ok(None));
        assert!(map.is_empty());
    }
}

mod map_append {
    use std::cmp::Ordering;

    use super::*;

    /// A key compared by its number only. The label tells equal keys apart.
    #[derive(Debug)]
    struct Labelled(u8, &'static str);

    impl PartialEq for Labelled {
        fn eq(&self, other: &Self) -> bool {
            self.0 == other.0
        }
    }

    impl Eq for Labelled {}

    impl PartialOrd for Labelled {
        fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
            Some(self.cmp(other))
        }
    }

    impl Ord for Labelled {
        fn cmp(&self, other: &Self) -> Ordering {
            self.0.cmp(&other.0)
        }
    }

    #[test]
    fn stored_key_is_kept_for_a_shared_key() {
        type Labels<const MIN: usize> = Confined<BTreeMap<Labelled, char>, MIN, 4>;

        let mut map = Labels::<1>::try_from_iter([(Labelled(1, "stored"), 'a')]).unwrap();
        let mut other = Labels::<0>::try_from_iter([(Labelled(1, "appended"), 'x')]).unwrap();
        assert_eq!(map.append(&mut other), Ok(()));
        let (key, value) = map.first_key_value().unwrap();
        assert_eq!((key.1, *value), ("stored", 'x'));
        assert_eq!(map.len(), 1);
        assert!(other.is_empty());
    }

    #[test]
    fn entries_are_moved_and_the_source_is_emptied() {
        let mut map = map_of::<1, 4>([1, 3]);
        let mut other = map_of::<0, 4>([2, 4]);
        assert_eq!(map.append(&mut other), Ok(()));
        assert_eq!(entries(&map), [(1, 'a'), (2, 'b'), (3, 'c'), (4, 'd')]);
        assert!(other.is_empty());
    }

    #[test]
    fn value_of_a_shared_key_is_overwritten() {
        let mut map = map_of::<1, 4>([1, 2]);
        let mut other = Map::<0, 4>::try_from_iter([(2, 'x'), (3, 'y')]).unwrap();
        assert_eq!(map.append(&mut other), Ok(()));
        assert_eq!(entries(&map), [(1, 'a'), (2, 'x'), (3, 'y')]);
        assert!(other.is_empty());
    }

    #[test]
    fn shared_keys_do_not_count_towards_the_maximum() {
        // Two entries and three entries fit the maximum of three, because two
        // of the keys are shared.
        let mut map = map_of::<1, 3>([1, 2]);
        let mut other = Map::<0, 4>::try_from_iter([(1, 'x'), (2, 'y'), (3, 'z')]).unwrap();
        assert_eq!(map.append(&mut other), Ok(()));
        assert_eq!(entries(&map), [(1, 'x'), (2, 'y'), (3, 'z')]);
        assert!(other.is_empty());
    }

    #[test]
    fn empty_map_takes_the_whole_source() {
        let mut map = Map::<0, 4>::new();
        let mut other = map_of::<0, 4>([1, 2]);
        assert_eq!(map.append(&mut other), Ok(()));
        assert_eq!(entries(&map), [(1, 'a'), (2, 'b')]);
        assert!(other.is_empty());
    }

    #[test]
    fn empty_source_changes_nothing() {
        let mut map = map_of::<1, 4>([1, 2]);
        let mut other = Map::<0, 4>::new();
        assert_eq!(map.append(&mut other), Ok(()));
        assert_eq!(entries(&map), [(1, 'a'), (2, 'b')]);
        assert!(other.is_empty());
    }

    #[test]
    fn map_can_be_filled_up_to_the_maximum() {
        let mut map = map_of::<1, 3>([1]);
        let mut other = map_of::<0, 2>([2, 3]);
        assert_eq!(map.append(&mut other), Ok(()));
        assert_eq!(map.len(), 3);
    }

    #[test]
    fn source_may_have_a_wider_maximum() {
        let mut map = map_of::<1, 3>([1]);
        let mut other = map_of::<0, 200>([2]);
        assert_eq!(map.append(&mut other), Ok(()));
        assert_eq!(entries(&map), [(1, 'a'), (2, 'b')]);
    }

    #[test]
    fn new_keys_above_the_maximum_change_neither_map() {
        let mut map = map_of::<1, 3>([1, 2]);
        let mut other = Map::<0, 4>::try_from_iter([(2, 'x'), (3, 'y'), (4, 'z')]).unwrap();
        assert_eq!(
            map.append(&mut other),
            Err(Error::Oversize { len: 4, max_len: 3 })
        );
        assert_eq!(entries(&map), [(1, 'a'), (2, 'b')]);
        assert_eq!(entries(&other), [(2, 'x'), (3, 'y'), (4, 'z')]);
    }

    #[test]
    fn full_map_accepts_no_new_key() {
        let mut map = map_of::<1, 2>([1, 2]);
        let mut other = map_of::<0, 2>([3]);
        assert_eq!(
            map.append(&mut other),
            Err(Error::Oversize { len: 3, max_len: 2 })
        );
        assert_eq!(entries(&map), [(1, 'a'), (2, 'b')]);
        assert_eq!(entries(&other), [(3, 'c')]);
    }
}

mod map_split_off {
    use std::cmp::Ordering;

    use super::*;

    #[test]
    fn entries_from_the_key_are_moved() {
        let mut map = map_of::<1, 4>([1, 2, 3, 4]);
        let right: Map<1, 4> = map.split_off(&3).unwrap();
        assert_eq!(entries(&map), [(1, 'a'), (2, 'b')]);
        assert_eq!(entries(&right), [(3, 'c'), (4, 'd')]);
    }

    #[test]
    fn absent_key_splits_at_the_next_one() {
        let mut map = map_of::<1, 4>([1, 2, 4]);
        let right: Map<1, 4> = map.split_off(&3).unwrap();
        assert_eq!(entries(&map), [(1, 'a'), (2, 'b')]);
        assert_eq!(entries(&right), [(4, 'd')]);
    }

    #[test]
    fn key_above_all_moves_nothing() {
        let mut map = map_of::<1, 4>([1, 2]);
        let right: Map<0, 4> = map.split_off(&9).unwrap();
        assert_eq!(entries(&map), [(1, 'a'), (2, 'b')]);
        assert!(right.is_empty());
    }

    #[test]
    fn key_below_all_moves_everything() {
        let mut map = map_of::<0, 4>([2, 3]);
        let right: Map<2, 2> = map.split_off(&1).unwrap();
        assert!(map.is_empty());
        assert_eq!(entries(&right), [(2, 'b'), (3, 'c')]);
    }

    #[test]
    fn left_part_below_its_minimum_changes_nothing() {
        let mut map = map_of::<2, 4>([1, 2, 3]);
        let result = map.split_off::<0, 4>(&2);
        assert_eq!(result.unwrap_err(), Error::Undersize { len: 1, min_len: 2 });
        assert_eq!(entries(&map), [(1, 'a'), (2, 'b'), (3, 'c')]);
    }

    #[test]
    fn right_part_below_its_minimum_changes_nothing() {
        let mut map = map_of::<1, 4>([1, 2, 3]);
        let result = map.split_off::<2, 4>(&3);
        assert_eq!(result.unwrap_err(), Error::Undersize { len: 1, min_len: 2 });
        assert_eq!(entries(&map), [(1, 'a'), (2, 'b'), (3, 'c')]);
    }

    #[test]
    fn right_part_above_its_maximum_changes_nothing() {
        let mut map = map_of::<1, 4>([1, 2, 3]);
        let result = map.split_off::<0, 1>(&2);
        assert_eq!(result.unwrap_err(), Error::Oversize { len: 2, max_len: 1 });
        assert_eq!(entries(&map), [(1, 'a'), (2, 'b'), (3, 'c')]);
    }

    /// A number ordered with [`f64::total_cmp`], which tells `-0.0` from
    /// `0.0`, and compared with `==`, which doesn't. The inconsistency is
    /// deliberate: a set of such keys can hold both zeroes, while a map
    /// collected from them holds only one.
    #[derive(Clone, Copy, Debug)]
    struct Number(f64);

    impl PartialEq for Number {
        fn eq(&self, other: &Self) -> bool {
            self.0 == other.0
        }
    }

    impl Eq for Number {}

    impl PartialOrd for Number {
        fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
            Some(self.cmp(other))
        }
    }

    impl Ord for Number {
        fn cmp(&self, other: &Self) -> Ordering {
            self.0.total_cmp(&other.0)
        }
    }

    type Numbers<const MIN: usize, const MAX: usize> = Confined<BTreeMap<Number, u8>, MIN, MAX>;

    fn numbers() -> Numbers<0, 4> {
        let numbers = [-1.0, -0.0, 0.0, 1.0];
        let map = Numbers::try_from_iter(numbers.map(|number| (Number(number), 0))).unwrap();
        assert_eq!(map.len(), 4, "both zeroes are distinct keys of the map");
        map
    }

    #[test]
    fn right_part_is_checked_as_built_and_not_as_counted() {
        // Three entries are split off. The standard library merges the keys
        // which are equal for `==` when it collects a map, so the map built
        // from them has two entries, which is below its minimum of three.
        // Should the library stop merging them, the split has to succeed.
        let mut map = numbers();
        match map.split_off::<3, 4>(&Number(-0.0)) {
            Ok(right) => {
                assert_eq!(right.len(), 3, "the returned map is out of its bounds");
                assert_eq!(right.iter().count(), 3);
                assert_eq!(map.len(), 1);
            }
            Err(error) => {
                assert_eq!(error, Error::Undersize { len: 2, min_len: 3 });
                assert_eq!(map.len(), 4, "the original map must stay unchanged");
            }
        }
    }

    #[test]
    fn right_part_within_its_bounds_is_returned_as_built() {
        let mut map = numbers();
        let right = map.split_off::<2, 4>(&Number(-0.0)).unwrap();
        assert!((2..=3).contains(&right.len()));
        assert_eq!(right.len(), right.iter().count());
        assert_eq!(map.len(), 1);
    }
}

mod map_range_mut {
    use super::*;

    #[test]
    fn values_in_the_range_are_changed() {
        let mut map = map_of::<1, 4>([1, 2, 3, 4]);
        for (_, value) in map.range_mut(2..=3) {
            *value = value.to_ascii_uppercase();
        }
        assert_eq!(entries(&map), [(1, 'a'), (2, 'B'), (3, 'C'), (4, 'd')]);
    }

    #[test]
    fn empty_range_changes_nothing() {
        let mut map = map_of::<1, 4>([1, 2]);
        assert_eq!(map.range_mut(5..).count(), 0);
        assert_eq!(entries(&map), [(1, 'a'), (2, 'b')]);
    }
}

mod map_into_keys {
    use super::*;

    #[test]
    fn keys_are_returned_in_their_order() {
        let map = map_of::<1, 4>([3, 1, 2]);
        assert_eq!(map.into_keys().collect::<Vec<_>>(), [1, 2, 3]);
    }
}

mod map_into_values {
    use super::*;

    #[test]
    fn values_are_returned_in_the_order_of_their_keys() {
        let map = map_of::<1, 4>([3, 1, 2]);
        assert_eq!(map.into_values().collect::<Vec<_>>(), ['a', 'b', 'c']);
    }
}
