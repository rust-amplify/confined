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

//! Methods of [`Confined`] which are shared by all collections
//! (`src/base.rs` and `src/specifics.rs`).
//!
//! A `Vec<u8>` stands for the sequences and a `BTreeMap<u8, char>` for the
//! maps; the methods specific to a collection have a test file of their own.
//! Each module covers one method, and each test covers one case of it:
//!
//! - construction: `try_from`, `from_checked`, `try_from_iter`,
//!   `from_iter_checked`, `new`, `with_capacity`, `try_with_capacity`, `with`,
//!   `with_key_value`, `widen`;
//! - reading: `len`, `unconfined`, `first_last`, `len_integers`;
//! - growth: `push`, `extend`, `try_reserve`, `insert`, `entry`;
//! - changing items: `mutable_access`;
//! - removal: `clear`, `try_retain`, `retain`;
//! - re-validation: `with_mut`, `as_mut_checked`.

#![cfg(feature = "alloc")]

use std::collections::BTreeMap;

use confined::{Confined, Error};

type Bytes<const MIN: usize, const MAX: usize> = Confined<Vec<u8>, MIN, MAX>;
type Map<const MIN: usize, const MAX: usize> = Confined<BTreeMap<u8, char>, MIN, MAX>;

fn map_of<const MIN: usize, const MAX: usize>(
    entries: impl IntoIterator<Item = (u8, char)>,
) -> Map<MIN, MAX> {
    Map::try_from_iter(entries).unwrap()
}

fn entries<const MIN: usize, const MAX: usize>(map: &Map<MIN, MAX>) -> Vec<(u8, char)> {
    map.iter().map(|(key, value)| (*key, *value)).collect()
}

mod try_from {
    use super::*;

    #[test]
    fn collection_at_the_minimum_is_accepted() {
        let bytes = Bytes::<2, 4>::try_from(vec![1, 2]).unwrap();
        assert_eq!(bytes.as_slice(), [1, 2]);
    }

    #[test]
    fn collection_at_the_maximum_is_accepted() {
        let bytes = Bytes::<2, 4>::try_from(vec![1, 2, 3, 4]).unwrap();
        assert_eq!(bytes.as_slice(), [1, 2, 3, 4]);
    }

    #[test]
    fn collection_below_the_minimum_is_rejected() {
        assert_eq!(
            Bytes::<2, 4>::try_from(vec![1]),
            Err(Error::Undersize { len: 1, min_len: 2 })
        );
    }

    #[test]
    fn collection_above_the_maximum_is_rejected() {
        assert_eq!(
            Bytes::<2, 4>::try_from(vec![1, 2, 3, 4, 5]),
            Err(Error::Oversize { len: 5, max_len: 4 })
        );
    }

    #[test]
    fn empty_collection_fits_a_confinement_without_a_minimum() {
        assert!(Bytes::<0, 4>::try_from(vec![]).unwrap().is_empty());
    }

    #[test]
    fn equal_bounds_accept_a_single_length() {
        assert!(Bytes::<2, 2>::try_from(vec![1]).is_err());
        assert!(Bytes::<2, 2>::try_from(vec![1, 2]).is_ok());
        assert!(Bytes::<2, 2>::try_from(vec![1, 2, 3]).is_err());
    }

    #[test]
    fn zero_maximum_accepts_only_an_empty_collection() {
        assert!(Bytes::<0, 0>::try_from(vec![]).is_ok());
        assert_eq!(
            Bytes::<0, 0>::try_from(vec![1]),
            Err(Error::Oversize { len: 1, max_len: 0 })
        );
    }
}

mod from_checked {
    use super::*;

    #[test]
    fn fitting_collection_is_returned() {
        assert_eq!(Bytes::<1, 2>::from_checked(vec![1]).as_slice(), [1]);
    }

    #[test]
    #[should_panic(expected = "collection size mismatch")]
    fn collection_below_the_minimum_is_a_panic() {
        Bytes::<1, 2>::from_checked(vec![]);
    }

    #[test]
    #[should_panic(expected = "collection size mismatch")]
    fn collection_above_the_maximum_is_a_panic() {
        Bytes::<1, 2>::from_checked(vec![1, 2, 3]);
    }
}

mod try_from_iter {
    use super::*;

    #[test]
    fn fitting_items_are_collected_in_order() {
        let bytes = Bytes::<1, 3>::try_from_iter([3, 1, 2]).unwrap();
        assert_eq!(bytes.as_slice(), [3, 1, 2]);
    }

    #[test]
    fn too_few_items_are_reported_with_their_number() {
        assert_eq!(
            Bytes::<2, 3>::try_from_iter([1]),
            Err(Error::Undersize { len: 1, min_len: 2 })
        );
    }

    #[test]
    fn item_above_the_maximum_is_reported_with_the_length_it_makes() {
        // The length is the one reached by the rejected item, and not the
        // number of items of the iterator.
        assert_eq!(
            Bytes::<0, 3>::try_from_iter(0..100),
            Err(Error::Oversize { len: 4, max_len: 3 })
        );
    }

    #[test]
    fn empty_iterator_fits_a_confinement_without_a_minimum() {
        assert!(Bytes::<0, 3>::try_from_iter([]).unwrap().is_empty());
    }

    #[test]
    fn repeated_keys_of_a_map_are_counted_once() {
        let map = Map::<1, 2>::try_from_iter([(1, 'a'), (2, 'b'), (1, 'c'), (2, 'd')]).unwrap();
        assert_eq!(entries(&map), [(1, 'c'), (2, 'd')]);
    }
}

mod from_iter_checked {
    use super::*;

    #[test]
    fn fitting_items_are_collected() {
        assert_eq!(Bytes::<1, 2>::from_iter_checked([1, 2]).as_slice(), [1, 2]);
    }

    #[test]
    #[should_panic(expected = "collection size mismatch")]
    fn too_few_items_are_a_panic() {
        Bytes::<1, 2>::from_iter_checked([]);
    }

    #[test]
    #[should_panic(expected = "collection size mismatch")]
    fn too_many_items_are_a_panic() {
        Bytes::<1, 2>::from_iter_checked([1, 2, 3]);
    }
}

mod new {
    use super::*;

    #[test]
    fn new_collection_is_empty() {
        assert!(Bytes::<0, 3>::new().is_empty());
        assert!(Map::<0, 3>::new().is_empty());
    }

    #[test]
    fn default_collection_is_empty() {
        assert!(Bytes::<0, 3>::default().is_empty());
        assert_eq!(Bytes::<0, 3>::default(), Bytes::<0, 3>::new());
    }

    #[test]
    fn collection_with_a_zero_maximum_can_be_created() {
        assert!(Bytes::<0, 0>::new().is_empty());
    }
}

mod with_capacity {
    use super::*;

    #[test]
    fn requested_capacity_is_reserved() {
        let bytes = Bytes::<0, 100>::with_capacity(10);
        assert!(bytes.capacity() >= 10);
        assert!(bytes.is_empty());
    }

    #[test]
    fn request_above_the_maximum_is_capped() {
        // An uncapped request of this size would be a capacity overflow.
        let bytes = Bytes::<0, 4>::with_capacity(usize::MAX);
        assert!(bytes.capacity() >= 4);
        assert!(bytes.is_empty());
    }
}

mod try_with_capacity {
    use super::*;

    #[test]
    fn requested_capacity_is_reserved() {
        let bytes = Bytes::<0, 100>::try_with_capacity(10).unwrap();
        assert!(bytes.capacity() >= 10);
        assert!(bytes.is_empty());
    }

    #[test]
    fn request_above_the_maximum_is_capped() {
        let bytes = Bytes::<0, 4>::try_with_capacity(usize::MAX).unwrap();
        assert!(bytes.capacity() >= 4);
    }

    #[test]
    fn capacity_which_can_not_be_reserved_is_an_error() {
        // Neither the request nor the maximum fit the address space.
        let result = Confined::<Vec<u64>, 0, { usize::MAX }>::try_with_capacity(usize::MAX);
        assert_eq!(result, Err(Error::CapacityExceeded { len: 0 }));
    }
}

mod with {
    use super::*;

    #[test]
    fn single_item_fits_a_confinement_with_the_minimum_of_one() {
        assert_eq!(Bytes::<1, 3>::with(7).as_slice(), [7]);
    }

    #[test]
    fn single_item_fits_a_confinement_without_a_minimum() {
        assert_eq!(Bytes::<0, 1>::with(7).as_slice(), [7]);
    }

    #[test]
    fn single_item_fits_a_confinement_of_exactly_one_item() {
        assert_eq!(Bytes::<1, 1>::with(7).as_slice(), [7]);
    }
}

mod with_key_value {
    use super::*;

    #[test]
    fn single_entry_is_stored() {
        let map = Map::<1, 3>::with_key_value(1, 'a');
        assert_eq!(entries(&map), [(1, 'a')]);
    }

    #[test]
    fn single_entry_fits_a_confinement_without_a_minimum() {
        let map = Map::<0, 1>::with_key_value(1, 'a');
        assert_eq!(entries(&map), [(1, 'a')]);
    }
}

mod widen {
    use super::*;

    #[test]
    fn wider_bounds_hold_the_same_collection() {
        let wide: Bytes<0, 255> = Bytes::<1, 3>::from_checked(vec![1, 2]).widen();
        assert_eq!(wide.as_slice(), [1, 2]);
    }

    #[test]
    fn the_same_bounds_are_allowed() {
        let same: Bytes<1, 3> = Bytes::<1, 3>::from_checked(vec![1, 2]).widen();
        assert_eq!(same.as_slice(), [1, 2]);
    }

    #[test]
    fn widened_collection_accepts_more_items() {
        let mut wide: Bytes<0, 3> = Bytes::<1, 1>::with(1).widen();
        assert_eq!(wide.extend([2, 3]), Ok(()));
        assert_eq!(wide.pop(), Some(3));
    }
}

mod len {
    use super::*;

    #[test]
    fn length_is_the_number_of_items() {
        assert_eq!(Bytes::<0, 3>::from_checked(vec![1, 2]).len(), 2);
        assert_eq!(map_of::<0, 3>([(1, 'a')]).len(), 1);
    }

    #[test]
    fn empty_collection_is_empty() {
        let bytes = Bytes::<0, 3>::new();
        assert_eq!(bytes.len(), 0);
        assert!(bytes.is_empty());
    }

    #[test]
    fn collection_with_items_is_not_empty() {
        assert!(!Bytes::<0, 3>::from_checked(vec![1]).is_empty());
    }
}

mod unconfined {
    use super::*;

    #[test]
    fn inner_collection_is_borrowed() {
        let bytes = Bytes::<0, 3>::from_checked(vec![1, 2]);
        let inner: &Vec<u8> = bytes.as_unconfined();
        assert_eq!(inner, &[1, 2]);
    }

    #[test]
    fn inner_collection_is_cloned() {
        let bytes = Bytes::<0, 3>::from_checked(vec![1, 2]);
        let mut inner: Vec<u8> = bytes.to_unconfined();
        inner.push(3);
        assert_eq!(inner, [1, 2, 3]);
        assert_eq!(bytes.as_slice(), [1, 2]);
    }

    #[test]
    fn inner_collection_is_released() {
        let inner: Vec<u8> = Bytes::<0, 3>::from_checked(vec![1, 2]).release();
        assert_eq!(inner, [1, 2]);
    }
}

mod first_last {
    use super::*;

    #[test]
    fn first_and_last_items_are_returned() {
        let bytes = Bytes::<1, 3>::from_checked(vec![1, 2, 3]);
        assert_eq!(*bytes.first(), 1);
        assert_eq!(*bytes.last(), 3);
    }

    #[test]
    fn single_item_is_both_the_first_and_the_last() {
        let bytes = Bytes::<1, 3>::with(7);
        assert_eq!(*bytes.first(), 7);
        assert_eq!(*bytes.last(), 7);
    }

    #[test]
    fn any_positive_minimum_gives_the_access() {
        let bytes = Bytes::<2, 3>::from_checked(vec![1, 2]);
        assert_eq!((*bytes.first(), *bytes.last()), (1, 2));
    }

    #[test]
    fn first_and_last_items_are_changed_in_place() {
        let mut bytes = Bytes::<1, 3>::from_checked(vec![1, 2, 3]);
        *bytes.first_mut() = 10;
        *bytes.last_mut() = 30;
        assert_eq!(bytes.as_slice(), [10, 2, 30]);
    }

    #[test]
    fn single_item_is_changed_through_both_ends() {
        let mut bytes = Bytes::<1, 3>::with(1);
        *bytes.first_mut() += 1;
        *bytes.last_mut() += 1;
        assert_eq!(bytes.as_slice(), [3]);
    }
}

mod len_integers {
    use amplify_num::u24;
    use confined::{U8MAX, U16MAX, U24MAX, U32MAX};

    use super::*;

    #[test]
    fn length_is_returned_as_one_byte() {
        let bytes = Bytes::<0, U8MAX>::from_checked(vec![0; 255]);
        assert_eq!(bytes.len_u8(), 255);
    }

    #[test]
    fn length_is_returned_as_two_bytes() {
        let bytes = Bytes::<0, U16MAX>::from_checked(vec![0; 65_535]);
        assert_eq!(bytes.len_u16(), 65_535);
    }

    #[test]
    fn length_is_returned_as_three_bytes() {
        let bytes = Bytes::<0, U24MAX>::from_checked(vec![0; 70_000]);
        assert_eq!(bytes.len_u24(), u24::try_from(70_000u32).unwrap());
    }

    #[test]
    fn length_is_returned_as_four_bytes() {
        let bytes = Bytes::<0, U32MAX>::from_checked(vec![0; 70_000]);
        assert_eq!(bytes.len_u32(), 70_000);
    }

    #[test]
    fn narrower_maximum_gives_every_wider_integer() {
        let bytes = Bytes::<0, 10>::from_checked(vec![1, 2]);
        assert_eq!(bytes.len_u8(), 2);
        assert_eq!(bytes.len_u16(), 2);
        assert_eq!(bytes.len_u24(), u24::try_from(2u32).unwrap());
        assert_eq!(bytes.len_u32(), 2);
    }
}

mod push {
    use super::*;

    #[test]
    fn item_is_added_below_the_maximum() {
        let mut bytes = Bytes::<0, 2>::from_checked(vec![1]);
        assert_eq!(bytes.push(2), Ok(()));
        assert_eq!(bytes.as_slice(), [1, 2]);
    }

    #[test]
    fn item_is_refused_at_the_maximum() {
        let mut bytes = Bytes::<0, 2>::from_checked(vec![1, 2]);
        assert_eq!(bytes.push(3), Err(Error::Oversize { len: 2, max_len: 2 }));
        assert_eq!(bytes.as_slice(), [1, 2]);
    }

    #[test]
    fn item_is_refused_by_a_zero_maximum() {
        let mut bytes = Bytes::<0, 0>::new();
        assert_eq!(bytes.push(1), Err(Error::Oversize { len: 0, max_len: 0 }));
        assert!(bytes.is_empty());
    }
}

mod extend {
    use super::*;

    #[test]
    fn fitting_items_are_added() {
        let mut bytes = Bytes::<0, 3>::from_checked(vec![1]);
        assert_eq!(bytes.extend([2, 3]), Ok(()));
        assert_eq!(bytes.as_slice(), [1, 2, 3]);
    }

    #[test]
    fn items_before_the_one_which_does_not_fit_are_kept() {
        let mut bytes = Bytes::<0, 3>::from_checked(vec![1]);
        assert_eq!(
            bytes.extend([2, 3, 4, 5]),
            Err(Error::Oversize { len: 3, max_len: 3 })
        );
        assert_eq!(bytes.as_slice(), [1, 2, 3]);
    }

    #[test]
    fn iterator_is_not_read_past_the_item_which_does_not_fit() {
        let mut bytes = Bytes::<0, 2>::new();
        let mut source = 1..=10;
        assert!(bytes.extend(&mut source).is_err());
        assert_eq!(source.next(), Some(4));
    }

    #[test]
    fn empty_iterator_adds_nothing_at_the_maximum() {
        let mut bytes = Bytes::<0, 1>::from_checked(vec![1]);
        assert_eq!(bytes.extend([]), Ok(()));
        assert_eq!(bytes.as_slice(), [1]);
    }
}

mod try_reserve {
    use super::*;

    #[test]
    fn room_within_the_allowance_is_reserved() {
        let mut bytes = Bytes::<0, 100>::from_checked(vec![1, 2]);
        assert_eq!(bytes.try_reserve(50), Ok(()));
        assert!(bytes.capacity() >= 52);
        assert_eq!(bytes.as_slice(), [1, 2]);
    }

    #[test]
    fn the_whole_allowance_can_be_reserved() {
        let mut bytes = Bytes::<0, 4>::from_checked(vec![1, 2]);
        assert_eq!(bytes.try_reserve(2), Ok(()));
        assert!(bytes.capacity() >= 4);
    }

    #[test]
    fn room_above_the_allowance_is_refused() {
        let mut bytes = Bytes::<0, 4>::from_checked(vec![1, 2]);
        assert_eq!(
            bytes.try_reserve(3),
            Err(Error::Oversize { len: 2, max_len: 4 })
        );
    }

    #[test]
    fn largest_request_is_refused_without_an_overflow() {
        let mut bytes = Bytes::<0, 4>::from_checked(vec![1, 2]);
        assert_eq!(
            bytes.try_reserve(usize::MAX),
            Err(Error::Oversize { len: 2, max_len: 4 })
        );
    }

    #[test]
    fn nothing_can_be_reserved_at_the_maximum() {
        let mut bytes = Bytes::<0, 2>::from_checked(vec![1, 2]);
        assert_eq!(bytes.try_reserve(0), Ok(()));
        assert_eq!(
            bytes.try_reserve(1),
            Err(Error::Oversize { len: 2, max_len: 2 })
        );
    }

    #[test]
    fn room_which_can_not_be_allocated_is_a_capacity_error() {
        // The allowance has no limit, but the address space does.
        let mut items = Confined::<Vec<u64>, 0, { usize::MAX }>::from_checked(vec![1]);
        assert_eq!(
            items.try_reserve(usize::MAX - 1),
            Err(Error::CapacityExceeded { len: 1 })
        );
        assert_eq!(items.as_slice(), [1]);
    }
}

mod insert {
    use std::cmp::Ordering;

    use super::*;

    #[test]
    fn new_key_is_added_below_the_maximum() {
        let mut map = map_of::<0, 2>([(1, 'a')]);
        assert_eq!(map.insert(2, 'b'), Ok(None));
        assert_eq!(entries(&map), [(1, 'a'), (2, 'b')]);
    }

    #[test]
    fn value_of_an_existing_key_is_replaced_and_returned() {
        let mut map = map_of::<0, 2>([(1, 'a')]);
        assert_eq!(map.insert(1, 'x'), Ok(Some('a')));
        assert_eq!(entries(&map), [(1, 'x')]);
    }

    #[test]
    fn value_of_an_existing_key_is_replaced_at_the_maximum() {
        let mut map = map_of::<0, 2>([(1, 'a'), (2, 'b')]);
        assert_eq!(map.insert(2, 'x'), Ok(Some('b')));
        assert_eq!(entries(&map), [(1, 'a'), (2, 'x')]);
    }

    #[test]
    fn new_key_is_refused_at_the_maximum() {
        let mut map = map_of::<0, 2>([(1, 'a'), (2, 'b')]);
        assert_eq!(
            map.insert(3, 'c'),
            Err(Error::Oversize { len: 2, max_len: 2 })
        );
        assert_eq!(entries(&map), [(1, 'a'), (2, 'b')]);
    }

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
    fn stored_key_is_kept_when_its_value_is_replaced() {
        let mut map =
            Confined::<BTreeMap<Labelled, char>, 0, 1>::with_key_value(Labelled(1, "stored"), 'a');
        assert_eq!(map.insert(Labelled(1, "inserted"), 'x'), Ok(Some('a')));
        let (key, value) = map.first_key_value().unwrap();
        assert_eq!((key.1, *value), ("stored", 'x'));
        assert_eq!(map.len(), 1);
    }
}

mod entry {
    use super::*;

    #[test]
    fn vacant_entry_is_given_out_below_the_maximum() {
        let mut map = map_of::<0, 2>([(1, 'a')]);
        assert_eq!(*map.entry(2).unwrap().or_insert('b'), 'b');
        assert_eq!(entries(&map), [(1, 'a'), (2, 'b')]);
    }

    #[test]
    fn vacant_entry_is_refused_at_the_maximum() {
        let mut map = map_of::<0, 2>([(1, 'a'), (2, 'b')]);
        assert_eq!(
            map.entry(3).err(),
            Some(Error::Oversize { len: 2, max_len: 2 })
        );
        assert_eq!(entries(&map), [(1, 'a'), (2, 'b')]);
    }

    #[test]
    fn occupied_entry_is_given_out_at_the_maximum() {
        let mut map = map_of::<0, 2>([(1, 'a'), (2, 'b')]);
        assert_eq!(*map.entry(2).unwrap().or_insert('x'), 'b');
    }

    #[test]
    fn unused_vacant_entry_adds_nothing() {
        let mut map = map_of::<0, 2>([(1, 'a')]);
        let _ = map.entry(2).unwrap();
        assert_eq!(entries(&map), [(1, 'a')]);
    }

    #[test]
    fn key_of_the_entry_is_returned() {
        let mut map = map_of::<0, 2>([(1, 'a')]);
        assert_eq!(*map.entry(1).unwrap().key(), 1);
        assert_eq!(*map.entry(2).unwrap().key(), 2);
    }

    #[test]
    fn or_insert_keeps_the_value_of_an_occupied_entry() {
        let mut map = map_of::<0, 2>([(1, 'a')]);
        assert_eq!(*map.entry(1).unwrap().or_insert('x'), 'a');
        assert_eq!(entries(&map), [(1, 'a')]);
    }

    #[test]
    fn or_insert_with_calls_the_default_for_a_vacant_entry_only() {
        let mut map = map_of::<0, 2>([(1, 'a')]);
        assert_eq!(
            *map.entry(1).unwrap().or_insert_with(|| unreachable!()),
            'a'
        );
        assert_eq!(*map.entry(2).unwrap().or_insert_with(|| 'b'), 'b');
        assert_eq!(entries(&map), [(1, 'a'), (2, 'b')]);
    }

    #[test]
    fn or_insert_with_key_gets_the_key_of_a_vacant_entry() {
        let mut map = map_of::<0, 2>([(1, 'a')]);
        let letter = |key: &u8| char::from(b'a' + key - 1);
        assert_eq!(*map.entry(2).unwrap().or_insert_with_key(letter), 'b');
        assert_eq!(*map.entry(1).unwrap().or_insert_with_key(|_| 'x'), 'a');
    }

    #[test]
    fn or_default_inserts_the_default_value_for_a_vacant_entry() {
        let mut map = map_of::<0, 2>([(1, 'a')]);
        assert_eq!(*map.entry(2).unwrap().or_default(), char::default());
        assert_eq!(*map.entry(1).unwrap().or_default(), 'a');
    }

    #[test]
    fn and_modify_changes_the_value_of_an_occupied_entry() {
        let mut map = map_of::<0, 2>([(1, 'a')]);
        let entry = map.entry(1).unwrap().and_modify(|value| *value = 'x');
        assert_eq!(*entry.or_insert('y'), 'x');
    }

    #[test]
    fn and_modify_leaves_a_vacant_entry_vacant() {
        let mut map = map_of::<0, 2>([(1, 'a')]);
        let entry = map.entry(2).unwrap().and_modify(|_| unreachable!());
        assert_eq!(*entry.or_insert('b'), 'b');
    }

    #[test]
    fn value_is_changed_through_the_returned_reference() {
        let mut map = map_of::<0, 2>([(1, 'a')]);
        *map.entry(1).unwrap().or_insert('x') = 'z';
        assert_eq!(entries(&map), [(1, 'z')]);
    }

    #[test]
    fn entry_is_printed_for_debugging() {
        let mut map = map_of::<0, 2>([(1, 'a')]);
        let text = format!("{:?}", map.entry(1).unwrap());
        assert!(text.starts_with("Entry("), "unexpected output: {text}");
    }
}

mod mutable_access {
    use super::*;

    #[test]
    fn value_is_changed_through_get_mut() {
        let mut map = map_of::<1, 2>([(1, 'a')]);
        *map.get_mut(&1).unwrap() = 'x';
        assert_eq!(entries(&map), [(1, 'x')]);
    }

    #[test]
    fn absent_key_has_no_mutable_value() {
        let mut map = map_of::<1, 2>([(1, 'a')]);
        assert_eq!(map.get_mut(&2), None);
    }

    #[test]
    fn values_are_changed_through_values_mut() {
        let mut map = map_of::<1, 2>([(1, 'a'), (2, 'b')]);
        for value in map.values_mut() {
            *value = value.to_ascii_uppercase();
        }
        assert_eq!(entries(&map), [(1, 'A'), (2, 'B')]);
    }

    #[test]
    fn values_are_changed_with_their_keys_known() {
        let mut map = map_of::<1, 2>([(1, 'a'), (2, 'b')]);
        for (key, value) in map.keyed_values_mut() {
            if *key == 2 {
                *value = 'x';
            }
        }
        assert_eq!(entries(&map), [(1, 'a'), (2, 'x')]);
    }

    #[test]
    fn items_are_changed_through_iter_mut() {
        let mut bytes = Bytes::<1, 3>::from_checked(vec![1, 2, 3]);
        for item in bytes.iter_mut() {
            *item *= 2;
        }
        assert_eq!(bytes.as_slice(), [2, 4, 6]);
    }

    #[test]
    fn changing_items_keeps_the_length() {
        let mut bytes = Bytes::<3, 3>::from_checked(vec![1, 2, 3]);
        bytes.iter_mut().for_each(|item| *item = 0);
        assert_eq!(bytes.len(), 3);
    }
}

mod clear {
    use super::*;

    #[test]
    fn all_items_are_removed() {
        let mut bytes = Bytes::<0, 3>::from_checked(vec![1, 2]);
        bytes.clear();
        assert!(bytes.is_empty());
    }

    #[test]
    fn cleared_collection_accepts_items_again() {
        let mut bytes = Bytes::<0, 1>::from_checked(vec![1]);
        bytes.clear();
        assert_eq!(bytes.push(2), Ok(()));
        assert_eq!(bytes.as_slice(), [2]);
    }
}

mod try_retain {
    use super::*;

    fn letters<const MIN: usize>() -> Map<MIN, 4> {
        map_of([(1, 'a'), (2, 'b'), (3, 'c'), (4, 'd')])
    }

    #[test]
    fn accepted_entries_are_all_kept() {
        let mut map = letters::<2>();
        assert_eq!(map.try_retain(|_, _| true), Ok(()));
        assert_eq!(map.len(), 4);
    }

    #[test]
    fn rejected_entries_are_removed_above_the_minimum() {
        let mut map = letters::<2>();
        assert_eq!(map.try_retain(|key, _| key % 2 == 0), Ok(()));
        assert_eq!(entries(&map), [(2, 'b'), (4, 'd')]);
    }

    #[test]
    fn removal_stops_at_the_minimum() {
        // Three entries are rejected, and only two of them may be removed.
        let mut map = letters::<2>();
        assert_eq!(
            map.try_retain(|key, _| *key == 4),
            Err(Error::Undersize { len: 1, min_len: 2 })
        );
        assert_eq!(entries(&map), [(3, 'c'), (4, 'd')]);
    }

    #[test]
    fn nothing_is_removed_at_the_minimum() {
        let mut map = letters::<4>();
        assert_eq!(
            map.try_retain(|_, _| false),
            Err(Error::Undersize { len: 0, min_len: 4 })
        );
        assert_eq!(map.len(), 4);
    }

    #[test]
    fn everything_is_removed_without_a_minimum() {
        let mut map = letters::<0>();
        assert_eq!(map.try_retain(|_, _| false), Ok(()));
        assert!(map.is_empty());
    }

    #[test]
    fn predicate_is_called_once_for_each_entry() {
        let mut map = letters::<2>();
        let mut visited = vec![];
        let _ = map.try_retain(|key, _| {
            visited.push(*key);
            false
        });
        assert_eq!(visited, [1, 2, 3, 4]);
    }

    #[test]
    fn predicate_may_change_the_values_it_keeps() {
        let mut map = letters::<2>();
        let result = map.try_retain(|_, value| {
            *value = value.to_ascii_uppercase();
            true
        });
        assert_eq!(result, Ok(()));
        assert_eq!(entries(&map), [(1, 'A'), (2, 'B'), (3, 'C'), (4, 'D')]);
    }
}

mod retain {
    use super::*;

    #[test]
    fn rejected_entries_are_removed() {
        let mut map = map_of::<0, 4>([(1, 'a'), (2, 'b'), (3, 'c')]);
        map.retain(|key, _| *key != 2);
        assert_eq!(entries(&map), [(1, 'a'), (3, 'c')]);
    }

    #[test]
    fn every_entry_may_be_removed() {
        let mut map = map_of::<0, 4>([(1, 'a'), (2, 'b')]);
        map.retain(|_, _| false);
        assert!(map.is_empty());
    }
}

mod with_mut {
    use super::*;

    #[test]
    fn collection_is_returned_after_the_mutation() {
        let bytes = Bytes::<1, 3>::with(1);
        let bytes = bytes.with_mut(|bytes| bytes.push(2).unwrap()).unwrap();
        assert_eq!(bytes.as_slice(), [1, 2]);
    }

    #[test]
    fn refused_mutation_leaves_the_collection_within_the_bounds() {
        let bytes = Bytes::<1, 1>::with(1);
        let bytes = bytes
            .with_mut(|bytes| assert!(bytes.push(2).is_err()))
            .unwrap();
        assert_eq!(bytes.as_slice(), [1]);
    }
}

mod as_mut_checked {
    use super::*;

    #[test]
    fn mutation_is_applied() {
        let mut bytes = Bytes::<1, 3>::with(1);
        bytes.as_mut_checked(|bytes| bytes.push(2).unwrap());
        assert_eq!(bytes.as_slice(), [1, 2]);
    }

    #[test]
    fn mutation_is_called_once() {
        let mut bytes = Bytes::<1, 3>::with(1);
        let mut calls = 0;
        bytes.as_mut_checked(|_| calls += 1);
        assert_eq!(calls, 1);
    }
}
