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

//! Checked methods of the confined `VecDeque` (`src/imp/deque.rs`).
//!
//! Each module covers one method, and each test covers one case of it:
//!
//! - the methods of the deques without a lower bound: `pop_front`, `pop_back`,
//!   `drain`, `truncate`;
//! - the checked methods of any deque: `push_front`, `push_back`, `remove`;
//! - `first_last`: the access to the ends of a deque with a lower bound.
//!
//! The positions of a deque are logical: they don't depend on where its ring
//! buffer starts. The tests named `..._of_a_wrapped_deque` use a deque which
//! was filled from both ends, so that the standard library stores its items
//! in two separate parts of the buffer.
//!
//! The methods shared by all collections are covered by `tests/generic.rs`,
//! and the refused allocations by `tests/oom.rs`.

#![cfg(feature = "alloc")]

use std::collections::VecDeque;

use confined::{Confined, Error};

type Deque<const MIN: usize, const MAX: usize> = Confined<VecDeque<u8>, MIN, MAX>;

fn deque_of<const MIN: usize, const MAX: usize>(items: &[u8]) -> Deque<MIN, MAX> {
    Deque::try_from_iter(items.iter().copied()).unwrap()
}

/// The deque `[1, 2, 3, 4]`, with the first two items pushed to the front
/// after the last two were pushed to the back. The standard deque puts such
/// items before the start of its buffer, which means at its end.
fn wrapped<const MIN: usize, const MAX: usize>() -> Deque<MIN, MAX> {
    let mut deque = VecDeque::with_capacity(4);
    deque.push_back(3);
    deque.push_back(4);
    deque.push_front(2);
    deque.push_front(1);
    assert!(deque.iter().eq(&[1, 2, 3, 4]));
    Deque::try_from(deque).unwrap()
}

fn items<const MIN: usize, const MAX: usize>(deque: &Deque<MIN, MAX>) -> Vec<u8> {
    deque.iter().copied().collect()
}

mod pop_front {
    use super::*;

    #[test]
    fn first_item_is_returned() {
        let mut deque = deque_of::<0, 3>(&[1, 2, 3]);
        assert_eq!(deque.pop_front(), Some(1));
        assert_eq!(items(&deque), [2, 3]);
    }

    #[test]
    fn items_are_returned_from_the_front_until_the_deque_is_empty() {
        let mut deque = deque_of::<0, 3>(&[1, 2]);
        assert_eq!(deque.pop_front(), Some(1));
        assert_eq!(deque.pop_front(), Some(2));
        assert_eq!(deque.pop_front(), None);
        assert!(deque.is_empty());
    }

    #[test]
    fn empty_deque_returns_nothing() {
        let mut deque = Deque::<0, 3>::new();
        assert_eq!(deque.pop_front(), None);
        assert!(deque.is_empty());
    }

    #[test]
    fn first_item_of_a_wrapped_deque_is_returned() {
        let mut deque = wrapped::<0, 4>();
        assert_eq!(deque.pop_front(), Some(1));
        assert_eq!(items(&deque), [2, 3, 4]);
    }
}

mod pop_back {
    use super::*;

    #[test]
    fn last_item_is_returned() {
        let mut deque = deque_of::<0, 3>(&[1, 2, 3]);
        assert_eq!(deque.pop_back(), Some(3));
        assert_eq!(items(&deque), [1, 2]);
    }

    #[test]
    fn items_are_returned_from_the_back_until_the_deque_is_empty() {
        let mut deque = deque_of::<0, 3>(&[1, 2]);
        assert_eq!(deque.pop_back(), Some(2));
        assert_eq!(deque.pop_back(), Some(1));
        assert_eq!(deque.pop_back(), None);
        assert!(deque.is_empty());
    }

    #[test]
    fn empty_deque_returns_nothing() {
        let mut deque = Deque::<0, 3>::new();
        assert_eq!(deque.pop_back(), None);
        assert!(deque.is_empty());
    }

    #[test]
    fn last_item_of_a_wrapped_deque_is_returned() {
        let mut deque = wrapped::<0, 4>();
        assert_eq!(deque.pop_back(), Some(4));
        assert_eq!(items(&deque), [1, 2, 3]);
    }
}

mod drain {
    use super::*;

    #[test]
    fn full_range_yields_every_item_and_empties_the_deque() {
        let mut deque = deque_of::<0, 3>(&[1, 2, 3]);
        assert_eq!(deque.drain(..).collect::<Vec<_>>(), [1, 2, 3]);
        assert!(deque.is_empty());
    }

    #[test]
    fn inner_range_keeps_the_rest_in_order() {
        let mut deque = deque_of::<0, 4>(&[1, 2, 3, 4]);
        assert_eq!(deque.drain(1..3).collect::<Vec<_>>(), [2, 3]);
        assert_eq!(items(&deque), [1, 4]);
    }

    #[test]
    fn range_from_the_start_keeps_the_tail() {
        let mut deque = deque_of::<0, 4>(&[1, 2, 3, 4]);
        assert_eq!(deque.drain(..2).collect::<Vec<_>>(), [1, 2]);
        assert_eq!(items(&deque), [3, 4]);
    }

    #[test]
    fn range_to_the_end_keeps_the_head() {
        let mut deque = deque_of::<0, 4>(&[1, 2, 3, 4]);
        assert_eq!(deque.drain(2..).collect::<Vec<_>>(), [3, 4]);
        assert_eq!(items(&deque), [1, 2]);
    }

    #[test]
    fn inclusive_range_includes_its_end() {
        let mut deque = deque_of::<0, 4>(&[1, 2, 3, 4]);
        assert_eq!(deque.drain(1..=2).collect::<Vec<_>>(), [2, 3]);
        assert_eq!(items(&deque), [1, 4]);
    }

    #[test]
    fn empty_range_removes_nothing() {
        let mut deque = deque_of::<0, 3>(&[1, 2, 3]);
        assert_eq!(deque.drain(1..1).count(), 0);
        assert_eq!(deque.drain(3..).count(), 0);
        assert_eq!(items(&deque), [1, 2, 3]);
    }

    #[test]
    fn range_is_removed_when_the_iterator_is_dropped_unused() {
        let mut deque = deque_of::<0, 3>(&[1, 2, 3]);
        drop(deque.drain(..2));
        assert_eq!(items(&deque), [3]);
    }

    #[test]
    fn range_of_a_wrapped_deque_follows_the_logical_order() {
        let mut deque = wrapped::<0, 4>();
        assert_eq!(deque.drain(1..3).collect::<Vec<_>>(), [2, 3]);
        assert_eq!(items(&deque), [1, 4]);
    }

    #[test]
    fn drained_deque_accepts_items_up_to_the_maximum_again() {
        let mut deque = deque_of::<0, 2>(&[1, 2]);
        assert_eq!(deque.drain(..).count(), 2);
        assert_eq!(deque.extend([3, 4]), Ok(()));
        assert_eq!(
            deque.push_back(5),
            Err(Error::Oversize { len: 2, max_len: 2 })
        );
    }

    // The two panics below come from the wrapped deque. Their messages belong
    // to the standard library and are not checked.

    #[test]
    #[should_panic]
    fn range_past_the_end_is_a_panic() {
        let mut deque = deque_of::<0, 3>(&[1, 2, 3]);
        let _ = deque.drain(..4);
    }

    #[test]
    #[should_panic]
    fn range_which_starts_after_its_end_is_a_panic() {
        let mut deque = deque_of::<0, 3>(&[1, 2, 3]);
        #[allow(clippy::reversed_empty_ranges)]
        let _ = deque.drain(2..1);
    }
}

mod truncate {
    use super::*;

    #[test]
    fn shorter_length_drops_the_tail() {
        let mut deque = deque_of::<0, 3>(&[1, 2, 3]);
        deque.truncate(1);
        assert_eq!(items(&deque), [1]);
    }

    #[test]
    fn current_length_changes_nothing() {
        let mut deque = deque_of::<0, 3>(&[1, 2, 3]);
        deque.truncate(3);
        assert_eq!(items(&deque), [1, 2, 3]);
    }

    #[test]
    fn longer_length_changes_nothing() {
        let mut deque = deque_of::<0, 3>(&[1, 2, 3]);
        deque.truncate(usize::MAX);
        assert_eq!(items(&deque), [1, 2, 3]);
    }

    #[test]
    fn zero_length_empties_the_deque() {
        let mut deque = deque_of::<0, 3>(&[1, 2, 3]);
        deque.truncate(0);
        assert!(deque.is_empty());
    }

    #[test]
    fn tail_of_a_wrapped_deque_is_dropped() {
        let mut deque = wrapped::<0, 4>();
        deque.truncate(3);
        assert_eq!(items(&deque), [1, 2, 3]);
    }
}

mod push_front {
    use super::*;

    #[test]
    fn item_becomes_the_first_one_below_the_maximum() {
        let mut deque = deque_of::<0, 3>(&[2, 3]);
        assert_eq!(deque.push_front(1), Ok(()));
        assert_eq!(items(&deque), [1, 2, 3]);
    }

    #[test]
    fn item_is_added_to_an_empty_deque() {
        let mut deque = Deque::<0, 3>::new();
        assert_eq!(deque.push_front(1), Ok(()));
        assert_eq!(items(&deque), [1]);
    }

    #[test]
    fn item_is_refused_at_the_maximum() {
        let mut deque = deque_of::<0, 2>(&[2, 3]);
        assert_eq!(
            deque.push_front(1),
            Err(Error::Oversize { len: 2, max_len: 2 })
        );
        assert_eq!(items(&deque), [2, 3]);
    }

    #[test]
    fn item_is_refused_by_a_zero_maximum() {
        let mut deque = Deque::<0, 0>::new();
        assert_eq!(
            deque.push_front(1),
            Err(Error::Oversize { len: 0, max_len: 0 })
        );
        assert!(deque.is_empty());
    }

    #[test]
    fn deque_with_a_minimum_is_filled_up_to_the_maximum() {
        let mut deque = deque_of::<1, 3>(&[3]);
        assert_eq!(deque.push_front(2), Ok(()));
        assert_eq!(deque.push_front(1), Ok(()));
        assert_eq!(
            deque.push_front(0),
            Err(Error::Oversize { len: 3, max_len: 3 })
        );
        assert_eq!(items(&deque), [1, 2, 3]);
    }
}

mod push_back {
    use super::*;

    #[test]
    fn item_becomes_the_last_one_below_the_maximum() {
        let mut deque = deque_of::<0, 3>(&[1, 2]);
        assert_eq!(deque.push_back(3), Ok(()));
        assert_eq!(items(&deque), [1, 2, 3]);
    }

    #[test]
    fn item_is_added_to_an_empty_deque() {
        let mut deque = Deque::<0, 3>::new();
        assert_eq!(deque.push_back(1), Ok(()));
        assert_eq!(items(&deque), [1]);
    }

    #[test]
    fn item_is_refused_at_the_maximum() {
        let mut deque = deque_of::<0, 2>(&[1, 2]);
        assert_eq!(
            deque.push_back(3),
            Err(Error::Oversize { len: 2, max_len: 2 })
        );
        assert_eq!(items(&deque), [1, 2]);
    }

    #[test]
    fn item_is_refused_by_a_zero_maximum() {
        let mut deque = Deque::<0, 0>::new();
        assert_eq!(
            deque.push_back(1),
            Err(Error::Oversize { len: 0, max_len: 0 })
        );
        assert!(deque.is_empty());
    }

    #[test]
    fn generic_push_adds_to_the_back_as_well() {
        let mut deque = deque_of::<0, 3>(&[1, 2]);
        assert_eq!(deque.push(3), Ok(()));
        assert_eq!(items(&deque), [1, 2, 3]);
    }

    #[test]
    fn both_ends_are_filled_up_to_the_maximum() {
        let mut deque = Deque::<0, 4>::new();
        assert_eq!(deque.push_back(3), Ok(()));
        assert_eq!(deque.push_front(2), Ok(()));
        assert_eq!(deque.push_back(4), Ok(()));
        assert_eq!(deque.push_front(1), Ok(()));
        assert_eq!(items(&deque), [1, 2, 3, 4]);
        assert!(deque.push_back(5).is_err());
        assert!(deque.push_front(0).is_err());
        assert_eq!(items(&deque), [1, 2, 3, 4]);
    }
}

mod remove {
    use super::*;

    #[test]
    fn front_item_is_removed() {
        let mut deque = deque_of::<1, 3>(&[1, 2, 3]);
        assert_eq!(deque.remove(0), Ok(1));
        assert_eq!(items(&deque), [2, 3]);
    }

    #[test]
    fn middle_item_is_removed() {
        let mut deque = deque_of::<1, 3>(&[1, 2, 3]);
        assert_eq!(deque.remove(1), Ok(2));
        assert_eq!(items(&deque), [1, 3]);
    }

    #[test]
    fn back_item_is_removed() {
        let mut deque = deque_of::<1, 3>(&[1, 2, 3]);
        assert_eq!(deque.remove(2), Ok(3));
        assert_eq!(items(&deque), [1, 2]);
    }

    #[test]
    fn item_of_a_wrapped_deque_is_removed_by_its_logical_position() {
        for (index, rest) in [
            (0, [2, 3, 4]),
            (1, [1, 3, 4]),
            (2, [1, 2, 4]),
            (3, [1, 2, 3]),
        ] {
            let mut deque = wrapped::<1, 4>();
            assert_eq!(deque.remove(index), Ok(u8::try_from(index + 1).unwrap()));
            assert_eq!(items(&deque), rest);
        }
    }

    #[test]
    fn index_equal_to_the_length_is_out_of_boundary() {
        let mut deque = deque_of::<1, 3>(&[1, 2, 3]);
        assert_eq!(
            deque.remove(3),
            Err(Error::OutOfBoundary { index: 3, len: 3 })
        );
        assert_eq!(items(&deque), [1, 2, 3]);
    }

    #[test]
    fn largest_index_is_out_of_boundary() {
        let mut deque = deque_of::<1, 3>(&[1, 2, 3]);
        assert_eq!(
            deque.remove(usize::MAX),
            Err(Error::OutOfBoundary {
                index: usize::MAX,
                len: 3
            })
        );
        assert_eq!(items(&deque), [1, 2, 3]);
    }

    #[test]
    fn item_is_kept_at_the_minimum() {
        let mut deque = deque_of::<2, 3>(&[1, 2]);
        assert_eq!(
            deque.remove(0),
            Err(Error::Undersize { len: 2, min_len: 2 })
        );
        assert_eq!(items(&deque), [1, 2]);
    }

    #[test]
    fn index_is_checked_before_the_minimum() {
        let mut deque = deque_of::<2, 3>(&[1, 2]);
        assert_eq!(
            deque.remove(2),
            Err(Error::OutOfBoundary { index: 2, len: 2 })
        );
        assert_eq!(items(&deque), [1, 2]);
    }

    #[test]
    fn items_are_removed_down_to_the_minimum_and_not_further() {
        let mut deque = deque_of::<1, 3>(&[1, 2, 3]);
        assert_eq!(deque.remove(2), Ok(3));
        assert_eq!(deque.remove(0), Ok(1));
        assert_eq!(
            deque.remove(0),
            Err(Error::Undersize { len: 1, min_len: 1 })
        );
        assert_eq!(items(&deque), [2]);
    }

    #[test]
    fn only_item_is_removed_without_a_minimum() {
        let mut deque = deque_of::<0, 3>(&[1]);
        assert_eq!(deque.remove(0), Ok(1));
        assert!(deque.is_empty());
    }

    #[test]
    fn empty_deque_has_no_item_to_remove() {
        let mut deque = Deque::<0, 3>::new();
        assert_eq!(
            deque.remove(0),
            Err(Error::OutOfBoundary { index: 0, len: 0 })
        );
        assert!(deque.is_empty());
    }
}

mod first_last {
    use super::*;

    #[test]
    fn ends_of_a_deque_are_returned() {
        let deque = deque_of::<1, 3>(&[1, 2, 3]);
        assert_eq!(*deque.first(), 1);
        assert_eq!(*deque.last(), 3);
    }

    #[test]
    fn ends_of_a_wrapped_deque_are_returned() {
        let deque = wrapped::<1, 4>();
        assert_eq!(*deque.first(), 1);
        assert_eq!(*deque.last(), 4);
    }

    #[test]
    fn ends_follow_the_items_pushed_to_them() {
        let mut deque = deque_of::<1, 3>(&[2]);
        deque.push_front(1).unwrap();
        deque.push_back(3).unwrap();
        assert_eq!(*deque.first(), 1);
        assert_eq!(*deque.last(), 3);
    }

    #[test]
    fn ends_are_changed_in_place() {
        let mut deque = wrapped::<1, 4>();
        *deque.first_mut() = 10;
        *deque.last_mut() = 40;
        assert_eq!(items(&deque), [10, 2, 3, 40]);
    }
}
