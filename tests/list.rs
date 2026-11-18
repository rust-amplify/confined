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

//! Checked methods of the confined `LinkedList` (`src/imp/list.rs`).
//!
//! Each module covers one method, and each test covers one case of it:
//!
//! - the checked methods of any list: `push_front`, `push_back`, `remove`;
//! - the methods of the lists without a lower bound: `pop_front`, `pop_back`.
//!
//! `remove` takes the list apart at the position of the item and joins it
//! again, so its tests also check the list in both directions.
//!
//! The methods shared by all collections are covered by `tests/generic.rs`.

#![cfg(feature = "alloc")]

use std::collections::LinkedList;

use confined::{Confined, Error};

type List<const MIN: usize, const MAX: usize> = Confined<LinkedList<u8>, MIN, MAX>;

fn list_of<const MIN: usize, const MAX: usize>(items: &[u8]) -> List<MIN, MAX> {
    List::try_from_iter(items.iter().copied()).unwrap()
}

fn items<const MIN: usize, const MAX: usize>(list: &List<MIN, MAX>) -> Vec<u8> {
    list.iter().copied().collect()
}

/// Checks that the list holds the expected items when it is walked from its
/// head, from its tail, and when its length and both of its ends are read.
fn assert_linked<const MIN: usize, const MAX: usize>(list: &List<MIN, MAX>, expected: &[u8]) {
    assert_eq!(items(list), expected);
    let mut backwards = list.iter().rev().copied().collect::<Vec<_>>();
    backwards.reverse();
    assert_eq!(backwards, expected);
    assert_eq!(list.len(), expected.len());
    assert_eq!(list.front(), expected.first());
    assert_eq!(list.back(), expected.last());
}

mod push_front {
    use super::*;

    #[test]
    fn item_becomes_the_first_one_below_the_maximum() {
        let mut list = list_of::<0, 3>(&[2, 3]);
        assert_eq!(list.push_front(1), Ok(()));
        assert_linked(&list, &[1, 2, 3]);
    }

    #[test]
    fn item_is_added_to_an_empty_list() {
        let mut list = List::<0, 3>::new();
        assert_eq!(list.push_front(1), Ok(()));
        assert_linked(&list, &[1]);
    }

    #[test]
    fn item_is_refused_at_the_maximum() {
        let mut list = list_of::<0, 2>(&[2, 3]);
        assert_eq!(
            list.push_front(1),
            Err(Error::Oversize { len: 2, max_len: 2 })
        );
        assert_linked(&list, &[2, 3]);
    }

    #[test]
    fn item_is_refused_by_a_zero_maximum() {
        let mut list = List::<0, 0>::new();
        assert_eq!(
            list.push_front(1),
            Err(Error::Oversize { len: 0, max_len: 0 })
        );
        assert!(list.is_empty());
    }

    #[test]
    fn list_with_a_minimum_is_filled_up_to_the_maximum() {
        let mut list = list_of::<1, 3>(&[3]);
        assert_eq!(list.push_front(2), Ok(()));
        assert_eq!(list.push_front(1), Ok(()));
        assert_eq!(
            list.push_front(0),
            Err(Error::Oversize { len: 3, max_len: 3 })
        );
        assert_linked(&list, &[1, 2, 3]);
    }
}

mod push_back {
    use super::*;

    #[test]
    fn item_becomes_the_last_one_below_the_maximum() {
        let mut list = list_of::<0, 3>(&[1, 2]);
        assert_eq!(list.push_back(3), Ok(()));
        assert_linked(&list, &[1, 2, 3]);
    }

    #[test]
    fn item_is_added_to_an_empty_list() {
        let mut list = List::<0, 3>::new();
        assert_eq!(list.push_back(1), Ok(()));
        assert_linked(&list, &[1]);
    }

    #[test]
    fn item_is_refused_at_the_maximum() {
        let mut list = list_of::<0, 2>(&[1, 2]);
        assert_eq!(
            list.push_back(3),
            Err(Error::Oversize { len: 2, max_len: 2 })
        );
        assert_linked(&list, &[1, 2]);
    }

    #[test]
    fn item_is_refused_by_a_zero_maximum() {
        let mut list = List::<0, 0>::new();
        assert_eq!(
            list.push_back(1),
            Err(Error::Oversize { len: 0, max_len: 0 })
        );
        assert!(list.is_empty());
    }

    #[test]
    fn generic_push_adds_to_the_back_as_well() {
        let mut list = list_of::<0, 3>(&[1, 2]);
        assert_eq!(list.push(3), Ok(()));
        assert_linked(&list, &[1, 2, 3]);
    }

    #[test]
    fn both_ends_are_filled_up_to_the_maximum() {
        let mut list = List::<0, 4>::new();
        assert_eq!(list.push_back(3), Ok(()));
        assert_eq!(list.push_front(2), Ok(()));
        assert_eq!(list.push_back(4), Ok(()));
        assert_eq!(list.push_front(1), Ok(()));
        assert!(list.push_back(5).is_err());
        assert!(list.push_front(0).is_err());
        assert_linked(&list, &[1, 2, 3, 4]);
    }
}

mod remove {
    use super::*;

    #[test]
    fn head_is_removed() {
        let mut list = list_of::<1, 4>(&[1, 2, 3, 4]);
        assert_eq!(list.remove(0), Ok(1));
        assert_linked(&list, &[2, 3, 4]);
    }

    #[test]
    fn inner_item_is_removed() {
        let mut list = list_of::<1, 4>(&[1, 2, 3, 4]);
        assert_eq!(list.remove(1), Ok(2));
        assert_linked(&list, &[1, 3, 4]);
    }

    #[test]
    fn item_before_the_tail_is_removed() {
        let mut list = list_of::<1, 4>(&[1, 2, 3, 4]);
        assert_eq!(list.remove(2), Ok(3));
        assert_linked(&list, &[1, 2, 4]);
    }

    #[test]
    fn tail_is_removed() {
        let mut list = list_of::<1, 4>(&[1, 2, 3, 4]);
        assert_eq!(list.remove(3), Ok(4));
        assert_linked(&list, &[1, 2, 3]);
    }

    #[test]
    fn list_accepts_items_at_both_ends_after_a_removal() {
        let mut list = list_of::<1, 4>(&[1, 2, 3, 4]);
        assert_eq!(list.remove(1), Ok(2));
        assert_eq!(list.remove(1), Ok(3));
        assert_eq!(list.push_front(0), Ok(()));
        assert_eq!(list.push_back(5), Ok(()));
        assert_linked(&list, &[0, 1, 4, 5]);
    }

    #[test]
    fn index_equal_to_the_length_is_out_of_boundary() {
        let mut list = list_of::<1, 3>(&[1, 2, 3]);
        assert_eq!(
            list.remove(3),
            Err(Error::OutOfBoundary { index: 3, len: 3 })
        );
        assert_linked(&list, &[1, 2, 3]);
    }

    #[test]
    fn largest_index_is_out_of_boundary() {
        let mut list = list_of::<1, 3>(&[1, 2, 3]);
        assert_eq!(
            list.remove(usize::MAX),
            Err(Error::OutOfBoundary {
                index: usize::MAX,
                len: 3
            })
        );
        assert_linked(&list, &[1, 2, 3]);
    }

    #[test]
    fn item_is_kept_at_the_minimum() {
        let mut list = list_of::<2, 3>(&[1, 2]);
        assert_eq!(list.remove(0), Err(Error::Undersize { len: 2, min_len: 2 }));
        assert_linked(&list, &[1, 2]);
    }

    #[test]
    fn index_is_checked_before_the_minimum() {
        let mut list = list_of::<2, 3>(&[1, 2]);
        assert_eq!(
            list.remove(2),
            Err(Error::OutOfBoundary { index: 2, len: 2 })
        );
        assert_linked(&list, &[1, 2]);
    }

    #[test]
    fn items_are_removed_down_to_the_minimum_and_not_further() {
        let mut list = list_of::<1, 3>(&[1, 2, 3]);
        assert_eq!(list.remove(2), Ok(3));
        assert_eq!(list.remove(0), Ok(1));
        assert_eq!(list.remove(0), Err(Error::Undersize { len: 1, min_len: 1 }));
        assert_linked(&list, &[2]);
    }

    #[test]
    fn only_item_is_removed_without_a_minimum() {
        let mut list = list_of::<0, 3>(&[1]);
        assert_eq!(list.remove(0), Ok(1));
        assert_linked(&list, &[]);
    }

    #[test]
    fn empty_list_has_no_item_to_remove() {
        let mut list = List::<0, 3>::new();
        assert_eq!(
            list.remove(0),
            Err(Error::OutOfBoundary { index: 0, len: 0 })
        );
        assert!(list.is_empty());
    }

    #[test]
    fn removed_item_is_returned_and_the_others_are_not_dropped() {
        use std::rc::Rc;

        let shared = [Rc::new(1), Rc::new(2), Rc::new(3)];
        let mut list = Confined::<LinkedList<Rc<u8>>, 1, 3>::try_from_iter(shared.clone()).unwrap();
        let removed = list.remove(1).unwrap();
        assert!(Rc::ptr_eq(&removed, &shared[1]));
        drop(removed);
        assert_eq!(shared.each_ref().map(Rc::strong_count), [2, 1, 2]);
    }
}

mod pop_front {
    use super::*;

    #[test]
    fn first_item_is_returned() {
        let mut list = list_of::<0, 3>(&[1, 2, 3]);
        assert_eq!(list.pop_front(), Some(1));
        assert_linked(&list, &[2, 3]);
    }

    #[test]
    fn items_are_returned_from_the_front_until_the_list_is_empty() {
        let mut list = list_of::<0, 3>(&[1, 2]);
        assert_eq!(list.pop_front(), Some(1));
        assert_eq!(list.pop_front(), Some(2));
        assert_eq!(list.pop_front(), None);
        assert!(list.is_empty());
    }

    #[test]
    fn empty_list_returns_nothing() {
        let mut list = List::<0, 3>::new();
        assert_eq!(list.pop_front(), None);
        assert!(list.is_empty());
    }
}

mod pop_back {
    use super::*;

    #[test]
    fn last_item_is_returned() {
        let mut list = list_of::<0, 3>(&[1, 2, 3]);
        assert_eq!(list.pop_back(), Some(3));
        assert_linked(&list, &[1, 2]);
    }

    #[test]
    fn items_are_returned_from_the_back_until_the_list_is_empty() {
        let mut list = list_of::<0, 3>(&[1, 2]);
        assert_eq!(list.pop_back(), Some(2));
        assert_eq!(list.pop_back(), Some(1));
        assert_eq!(list.pop_back(), None);
        assert!(list.is_empty());
    }

    #[test]
    fn empty_list_returns_nothing() {
        let mut list = List::<0, 3>::new();
        assert_eq!(list.pop_back(), None);
        assert!(list.is_empty());
    }

    #[test]
    fn full_list_accepts_an_item_after_a_pop() {
        let mut list = list_of::<0, 2>(&[1, 2]);
        assert_eq!(list.pop_back(), Some(2));
        assert_eq!(list.push_front(0), Ok(()));
        assert_linked(&list, &[0, 1]);
    }
}
