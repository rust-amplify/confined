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

//! Confinement of the collections defined outside of the crate.
//!
//! [`ItemLen`] lets a collection choose the units of its length. The generic
//! methods of [`Confined`] must uphold the bounds in these units, and not in
//! the number of items:
//!
//! - `hooks`: the defaults of the trait;
//! - `weighted_sequence`: a sequence which length is the total length of its
//!   items (`Words`);
//! - `weighted_map`: a map where an entry adds as much as its key (`Rack`);
//! - `native_capacity`: a collection which can refuse an item (`Pair`).

#![cfg(feature = "alloc")]

use std::collections::{BTreeMap, btree_map};
use std::ops::{Index, IndexMut};

use collections::{CollectionBase, CollectionEntry, CollectionMap};
use confined::{Confined, Error, ItemLen};

/// A list of words confined by their total length in bytes. A word may take
/// up to eight bytes.
#[derive(Default, Debug)]
struct Words(Vec<&'static str>);

impl CollectionBase for Words {
    type Item = &'static str;

    fn item_count(&self) -> usize {
        self.0.len()
    }

    fn push(&mut self, elem: Self::Item) {
        self.0.push(elem)
    }

    fn clear(&mut self) {
        self.0.clear()
    }
}

impl Index<usize> for Words {
    type Output = &'static str;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl IndexMut<usize> for Words {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.0[index]
    }
}

impl ItemLen for Words {
    const MAX_ITEM_LEN: usize = 8;

    fn confined_len(&self) -> usize {
        self.0.iter().map(|word| word.len()).sum()
    }

    fn item_len(item: &Self::Item) -> usize {
        item.len()
    }
}

/// A rack of modules. The key of a module is its height, from one to four
/// units, and the rack is confined by the total height of its modules.
#[derive(Default, Debug)]
struct Rack(BTreeMap<u8, u32>);

impl CollectionBase for Rack {
    type Item = (u8, u32);

    fn item_count(&self) -> usize {
        self.0.len()
    }

    fn push(&mut self, (key, value): Self::Item) {
        self.0.insert(key, value);
    }

    fn clear(&mut self) {
        self.0.clear()
    }
}

impl CollectionMap for Rack {
    type Key = u8;
    type Value = u32;

    fn contains_key(&self, key: &u8) -> bool {
        self.0.contains_key(key)
    }

    fn get(&self, key: &u8) -> Option<&u32> {
        self.0.get(key)
    }

    fn get_mut(&mut self, key: &u8) -> Option<&mut u32> {
        self.0.get_mut(key)
    }

    fn iter(&self) -> impl Iterator<Item = (&u8, &u32)> {
        self.0.iter()
    }

    fn iter_mut(&mut self) -> impl Iterator<Item = (&u8, &mut u32)> {
        self.0.iter_mut()
    }

    fn values_mut(&mut self) -> impl Iterator<Item = &mut u32> {
        self.0.values_mut()
    }

    fn insert(&mut self, key: u8, value: u32) -> Option<u32> {
        self.0.insert(key, value)
    }

    fn remove(&mut self, key: &u8) -> Option<u32> {
        self.0.remove(key)
    }

    fn retain(&mut self, f: impl FnMut(&u8, &mut u32) -> bool) {
        self.0.retain(f)
    }
}

impl CollectionEntry for Rack {
    type Entry<'a> = btree_map::Entry<'a, u8, u32>;

    fn entry(&mut self, key: u8) -> Self::Entry<'_> {
        self.0.entry(key)
    }
}

impl ItemLen for Rack {
    const MAX_ITEM_LEN: usize = 4;

    fn confined_len(&self) -> usize {
        self.0.keys().map(|key| usize::from(*key)).sum()
    }

    fn item_len(item: &Self::Item) -> usize {
        usize::from(item.0)
    }

    fn item_growth(&self, item: &Self::Item) -> usize {
        if self.0.contains_key(&item.0) {
            0
        } else {
            usize::from(item.0)
        }
    }
}

/// A stack with the native capacity of two items, which refuses the third one.
#[derive(Default, Debug)]
struct Pair(Vec<u8>);

impl CollectionBase for Pair {
    type Item = u8;

    fn item_count(&self) -> usize {
        self.0.len()
    }

    fn push(&mut self, elem: u8) {
        assert!(self.0.len() < 2, "the pair is full");
        self.0.push(elem)
    }

    fn clear(&mut self) {
        self.0.clear()
    }
}

impl ItemLen for Pair {
    fn try_push(&mut self, item: u8) -> Result<(), u8> {
        if self.0.len() == 2 {
            return Err(item);
        }
        self.0.push(item);
        Ok(())
    }
}

/// A stack which relies on every default of [`ItemLen`].
#[derive(Default, Debug)]
struct Stack(Vec<u8>);

impl CollectionBase for Stack {
    type Item = u8;

    fn item_count(&self) -> usize {
        self.0.len()
    }

    fn push(&mut self, elem: u8) {
        self.0.push(elem)
    }

    fn clear(&mut self) {
        self.0.clear()
    }
}

impl ItemLen for Stack {}

mod hooks {
    use super::*;

    #[test]
    fn largest_item_adds_one_by_default() {
        assert_eq!(Stack::MAX_ITEM_LEN, 1);
    }

    #[test]
    fn item_adds_one_by_default() {
        assert_eq!(Stack::item_len(&7), 1);
    }

    #[test]
    fn confined_length_is_the_number_of_items_by_default() {
        let stack = Stack(vec![1, 2, 3]);
        assert_eq!(stack.confined_len(), 3);
        assert_eq!(stack.confined_len(), stack.item_count());
    }

    #[test]
    fn growth_is_the_length_of_the_item_by_default() {
        let stack = Stack(vec![1, 2, 3]);
        assert_eq!(stack.item_growth(&3), 1);
        assert_eq!(Words::default().item_growth(&"alpha"), 5);
    }

    #[test]
    fn fallible_push_adds_the_item_by_default() {
        let mut stack = Stack::default();
        assert_eq!(ItemLen::try_push(&mut stack, 7), Ok(()));
        assert_eq!(stack.0, [7]);
    }

    #[test]
    fn entry_needs_no_preparation_by_default() {
        let mut rack = Rack::default();
        assert_eq!(rack.prepare_entry(), Ok(()));
        assert_eq!(rack.item_count(), 0);
    }

    #[test]
    fn overridden_length_is_used_by_the_confinement() {
        let words = Confined::<Words, 0, 16>::try_from(Words(vec!["alpha", "beta"])).unwrap();
        assert_eq!(words.len(), 9);
        assert_eq!(words.item_count(), 2);
        assert!(!words.is_empty());
    }

    #[test]
    fn empty_collection_has_no_length() {
        let words = Confined::<Words, 0, 16>::new();
        assert_eq!(words.len(), 0);
        assert!(words.is_empty());
    }
}

mod weighted_sequence {
    use super::*;

    /// Two words of nine bytes in total.
    fn words<const MIN: usize, const MAX: usize>() -> Confined<Words, MIN, MAX> {
        Confined::try_from_iter(["alpha", "beta"]).unwrap()
    }

    fn items<const MIN: usize, const MAX: usize>(
        words: &Confined<Words, MIN, MAX>,
    ) -> &[&'static str] {
        &words.as_unconfined().0
    }

    #[test]
    fn try_from_accepts_the_bounds_in_the_units_of_the_collection() {
        let words = Confined::<Words, 9, 9>::try_from(Words(vec!["alpha", "beta"])).unwrap();
        assert_eq!(items(&words), ["alpha", "beta"]);
    }

    #[test]
    fn try_from_rejects_a_length_below_the_minimum() {
        // Two items, but only nine bytes.
        let result = Confined::<Words, 10, 16>::try_from(Words(vec!["alpha", "beta"]));
        assert_eq!(result.unwrap_err(), Error::Undersize {
            len: 9,
            min_len: 10
        });
    }

    #[test]
    fn try_from_rejects_a_length_above_the_maximum() {
        // Two items, which is less than the maximum, but nine bytes.
        let result = Confined::<Words, 0, 8>::try_from(Words(vec!["alpha", "beta"]));
        assert_eq!(result.unwrap_err(), Error::Oversize { len: 9, max_len: 8 });
    }

    #[test]
    fn try_from_iter_reports_the_length_with_the_rejected_item() {
        let result = Confined::<Words, 0, 12>::try_from_iter(["alpha", "beta", "gamma"]);
        assert_eq!(result.unwrap_err(), Error::Oversize {
            len: 14,
            max_len: 12
        });
    }

    #[test]
    fn with_holds_a_single_item() {
        let words = Confined::<Words, 1, 8>::with("alpha");
        assert_eq!(words.len(), 5);
        assert_eq!(items(&words), ["alpha"]);
    }

    #[test]
    fn push_accepts_an_item_which_fits_the_room_left() {
        let mut words = words::<0, 12>();
        assert_eq!(words.push("abc"), Ok(()));
        assert_eq!(words.len(), 12);
        assert_eq!(items(&words), ["alpha", "beta", "abc"]);
    }

    #[test]
    fn push_rejects_an_item_longer_than_the_room_left() {
        let mut words = words::<0, 12>();
        assert_eq!(
            words.push("gamma"),
            Err(Error::Oversize {
                len: 9,
                max_len: 12
            })
        );
        assert_eq!(words.len(), 9);
        assert_eq!(items(&words), ["alpha", "beta"]);
    }

    #[test]
    fn extend_keeps_the_items_before_the_one_which_does_not_fit() {
        let mut words = words::<0, 12>();
        assert_eq!(
            words.extend(["a", "bb", "gamma", "c"]),
            Err(Error::Oversize {
                len: 12,
                max_len: 12
            })
        );
        assert_eq!(words.len(), 12);
        assert_eq!(items(&words), ["alpha", "beta", "a", "bb"]);
    }

    #[test]
    fn clear_empties_a_collection_without_a_minimum() {
        let mut words = words::<0, 12>();
        words.clear();
        assert!(words.is_empty());
        assert_eq!(words.item_count(), 0);
    }

    #[test]
    fn first_is_the_first_item() {
        assert_eq!(*words::<1, 16>().first(), "alpha");
    }

    #[test]
    fn last_is_the_last_item_and_not_the_item_at_the_confined_length() {
        // The length is nine, while the last item has the position one.
        assert_eq!(*words::<1, 16>().last(), "beta");
    }

    #[test]
    fn first_and_last_of_a_single_item_are_the_same() {
        let words = Confined::<Words, 1, 8>::with("alpha");
        assert_eq!(*words.first(), "alpha");
        assert_eq!(*words.last(), "alpha");
    }

    #[test]
    fn first_mut_changes_the_first_item() {
        let mut words = words::<1, 16>();
        // A word of the same length keeps the length of the collection.
        *words.first_mut() = "gamma";
        assert_eq!(items(&words), ["gamma", "beta"]);
        assert_eq!(words.len(), 9);
    }

    #[test]
    fn last_mut_changes_the_last_item() {
        let mut words = words::<1, 16>();
        // A word of the same length keeps the length of the collection.
        *words.last_mut() = "zeta";
        assert_eq!(items(&words), ["alpha", "zeta"]);
        assert_eq!(words.len(), 9);
    }
}

mod weighted_map {
    use super::*;

    fn rack<const MIN: usize, const MAX: usize>(
        heights: impl IntoIterator<Item = u8>,
    ) -> Confined<Rack, MIN, MAX> {
        Confined::try_from_iter(heights.into_iter().map(|height| (height, 0))).unwrap()
    }

    fn heights<const MIN: usize, const MAX: usize>(rack: &Confined<Rack, MIN, MAX>) -> Vec<u8> {
        rack.as_unconfined().0.keys().copied().collect()
    }

    fn value<const MIN: usize, const MAX: usize>(
        rack: &Confined<Rack, MIN, MAX>,
        height: u8,
    ) -> Option<u32> {
        rack.as_unconfined().0.get(&height).copied()
    }

    #[test]
    fn length_is_the_total_height() {
        let rack = rack::<0, 10>([1, 2, 4]);
        assert_eq!(rack.len(), 7);
        assert_eq!(rack.item_count(), 3);
    }

    #[test]
    fn with_key_value_holds_a_single_entry() {
        let rack = Confined::<Rack, 1, 4>::with_key_value(3, 30);
        assert_eq!(rack.len(), 3);
        assert_eq!(value(&rack, 3), Some(30));
    }

    #[test]
    fn push_of_a_new_key_adds_its_height() {
        let mut rack = rack::<0, 6>([1, 2]);
        assert_eq!(rack.push((3, 30)), Ok(()));
        assert_eq!(rack.len(), 6);
    }

    #[test]
    fn push_rejects_a_key_higher_than_the_room_left() {
        let mut rack = rack::<0, 6>([1, 2]);
        assert_eq!(
            rack.push((4, 40)),
            Err(Error::Oversize { len: 3, max_len: 6 })
        );
        assert_eq!(heights(&rack), [1, 2]);
    }

    #[test]
    fn insert_of_a_new_key_adds_its_height() {
        let mut rack = rack::<0, 6>([1, 2]);
        assert_eq!(rack.insert(3, 30), Ok(None));
        assert_eq!(rack.len(), 6);
        assert_eq!(value(&rack, 3), Some(30));
    }

    #[test]
    fn insert_rejects_a_key_higher_than_the_room_left() {
        // Two entries only, but a module of four units doesn't fit three.
        let mut rack = rack::<0, 6>([1, 2]);
        assert_eq!(
            rack.insert(4, 40),
            Err(Error::Oversize { len: 3, max_len: 6 })
        );
        assert_eq!(rack.len(), 3);
        assert_eq!(heights(&rack), [1, 2]);
    }

    #[test]
    fn insert_of_an_existing_key_replaces_the_value_at_the_maximum() {
        let mut rack = rack::<0, 6>([2, 4]);
        assert_eq!(rack.len(), 6);
        assert_eq!(rack.insert(4, 40), Ok(Some(0)));
        assert_eq!(rack.len(), 6);
        assert_eq!(value(&rack, 4), Some(40));
    }

    #[test]
    fn vacant_entry_is_given_out_when_the_largest_item_fits() {
        let mut rack = rack::<0, 6>([2]);
        assert_eq!(*rack.entry(3).unwrap().or_insert(30), 30);
        assert_eq!(rack.len(), 5);
    }

    #[test]
    fn vacant_entry_is_refused_unless_the_largest_item_fits() {
        // The value of the entry, and so the entry itself, is unknown when
        // the entry is requested: three units are not enough for any module.
        let mut rack = rack::<0, 6>([1, 2]);
        for height in [3, 4] {
            assert_eq!(
                rack.entry(height).err(),
                Some(Error::Oversize { len: 3, max_len: 6 })
            );
        }
        assert_eq!(heights(&rack), [1, 2]);
    }

    #[test]
    fn occupied_entry_is_given_out_at_the_maximum() {
        let mut rack = rack::<0, 6>([2, 4]);
        *rack.entry(4).unwrap().or_insert(1) += 40;
        assert_eq!(value(&rack, 4), Some(40));
        assert_eq!(rack.len(), 6);
    }

    #[test]
    fn try_retain_keeps_the_map_above_the_minimum() {
        // Ten units in four modules: counted in modules, all of them could
        // be removed with the minimum of three.
        let mut rack = rack::<3, 10>([1, 2, 3, 4]);
        let result = rack.try_retain(|_, _| false);
        assert!(matches!(result, Err(Error::Undersize { min_len: 3, .. })));
        assert!(rack.len() >= 3, "the lower bound is violated");
    }

    #[test]
    fn try_retain_counts_a_removal_as_the_largest_item() {
        // Nine units may be removed, and each removal is counted as four.
        let mut rack = rack::<1, 10>([1, 2, 3, 4]);
        assert_eq!(
            rack.try_retain(|_, _| false),
            Err(Error::Undersize { len: 0, min_len: 1 })
        );
        assert_eq!(heights(&rack), [3, 4]);
        assert_eq!(rack.len(), 7);
    }

    #[test]
    fn try_retain_keeps_the_accepted_entries() {
        let mut rack = rack::<1, 10>([1, 2, 3, 4]);
        assert_eq!(rack.try_retain(|height, _| height % 2 == 0), Ok(()));
        assert_eq!(heights(&rack), [2, 4]);
    }

    #[test]
    fn try_retain_without_a_minimum_removes_every_rejected_entry() {
        let mut rack = rack::<0, 10>([1, 2, 3, 4]);
        assert_eq!(rack.try_retain(|_, _| false), Ok(()));
        assert!(rack.is_empty());
    }

    #[test]
    fn retain_without_a_minimum_removes_every_rejected_entry() {
        let mut rack = rack::<0, 10>([1, 2, 3, 4]);
        rack.retain(|height, _| *height > 2);
        assert_eq!(heights(&rack), [3, 4]);
        assert_eq!(rack.len(), 7);
    }
}

mod native_capacity {
    use super::*;

    fn items<const MAX: usize>(pair: &Confined<Pair, 0, MAX>) -> &[u8] {
        &pair.as_unconfined().0
    }

    #[test]
    fn push_reports_the_refused_item() {
        let mut pair = Confined::<Pair, 0, 5>::try_from_iter([1, 2]).unwrap();
        assert_eq!(pair.push(3), Err(Error::CapacityExceeded { len: 2 }));
        assert_eq!(items(&pair), [1, 2]);
    }

    #[test]
    fn extend_reports_the_refused_item() {
        let mut pair = Confined::<Pair, 0, 5>::new();
        assert_eq!(
            pair.extend([1, 2, 3]),
            Err(Error::CapacityExceeded { len: 2 })
        );
        assert_eq!(items(&pair), [1, 2]);
    }

    #[test]
    fn try_from_iter_reports_the_refused_item() {
        let result = Confined::<Pair, 0, 5>::try_from_iter([1, 2, 3]);
        assert_eq!(result.unwrap_err(), Error::CapacityExceeded { len: 2 });
    }

    #[test]
    fn confinement_is_checked_before_the_native_capacity() {
        let mut pair = Confined::<Pair, 0, 2>::try_from_iter([1, 2]).unwrap();
        assert_eq!(pair.push(3), Err(Error::Oversize { len: 2, max_len: 2 }));
    }
}
