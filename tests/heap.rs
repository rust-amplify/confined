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

//! Checked methods of the confined `BinaryHeap` (`src/imp/heap.rs`).
//!
//! A heap has a single method of its own, `pop`, which exists only without a
//! lower bound. It is filled with the methods shared by all collections,
//! which are covered here for a collection which reorders its items.
//!
//! Each module covers one method, and each test covers one case of it:
//! `pop`, `push`, `extend`, `construction` and `clear`.

#![cfg(feature = "alloc")]

use std::collections::BinaryHeap;

use confined::{Confined, Error};

type Heap<const MIN: usize, const MAX: usize> = Confined<BinaryHeap<u8>, MIN, MAX>;

fn heap_of<const MIN: usize, const MAX: usize>(items: &[u8]) -> Heap<MIN, MAX> {
    Heap::try_from_iter(items.iter().copied()).unwrap()
}

/// The items of the heap in the ascending order.
fn sorted<const MIN: usize, const MAX: usize>(heap: &Heap<MIN, MAX>) -> Vec<u8> {
    heap.to_unconfined().into_sorted_vec()
}

mod pop {
    use super::*;

    #[test]
    fn greatest_item_is_returned() {
        let mut heap = heap_of::<0, 3>(&[2, 3, 1]);
        assert_eq!(heap.pop(), Some(3));
        assert_eq!(sorted(&heap), [1, 2]);
    }

    #[test]
    fn items_are_returned_in_the_descending_order_until_the_heap_is_empty() {
        let mut heap = heap_of::<0, 5>(&[4, 1, 5, 3, 2]);
        let mut popped = vec![];
        while let Some(item) = heap.pop() {
            popped.push(item);
        }
        assert_eq!(popped, [5, 4, 3, 2, 1]);
        assert!(heap.is_empty());
    }

    #[test]
    fn equal_items_are_returned_one_by_one() {
        let mut heap = heap_of::<0, 3>(&[2, 2, 1]);
        assert_eq!(heap.pop(), Some(2));
        assert_eq!(heap.pop(), Some(2));
        assert_eq!(heap.pop(), Some(1));
        assert_eq!(heap.pop(), None);
    }

    #[test]
    fn empty_heap_returns_nothing() {
        let mut heap = Heap::<0, 3>::new();
        assert_eq!(heap.pop(), None);
        assert!(heap.is_empty());
    }

    #[test]
    fn full_heap_accepts_an_item_after_a_pop() {
        let mut heap = heap_of::<0, 2>(&[1, 2]);
        assert_eq!(heap.pop(), Some(2));
        assert_eq!(heap.push(3), Ok(()));
        assert_eq!(sorted(&heap), [1, 3]);
    }
}

mod push {
    use super::*;

    #[test]
    fn greater_item_becomes_the_top_one() {
        let mut heap = heap_of::<0, 3>(&[1, 2]);
        assert_eq!(heap.push(3), Ok(()));
        assert_eq!(heap.peek(), Some(&3));
        assert_eq!(sorted(&heap), [1, 2, 3]);
    }

    #[test]
    fn smaller_item_leaves_the_top_one() {
        let mut heap = heap_of::<0, 3>(&[2, 3]);
        assert_eq!(heap.push(1), Ok(()));
        assert_eq!(heap.peek(), Some(&3));
        assert_eq!(sorted(&heap), [1, 2, 3]);
    }

    #[test]
    fn item_is_added_to_an_empty_heap() {
        let mut heap = Heap::<0, 3>::new();
        assert_eq!(heap.push(1), Ok(()));
        assert_eq!(heap.peek(), Some(&1));
        assert_eq!(heap.len(), 1);
    }

    #[test]
    fn equal_item_takes_room_of_its_own() {
        let mut heap = heap_of::<0, 3>(&[1, 2]);
        assert_eq!(heap.push(2), Ok(()));
        assert_eq!(sorted(&heap), [1, 2, 2]);
    }

    #[test]
    fn item_is_refused_at_the_maximum() {
        let mut heap = heap_of::<0, 2>(&[1, 2]);
        assert_eq!(heap.push(3), Err(Error::Oversize { len: 2, max_len: 2 }));
        assert_eq!(heap.peek(), Some(&2));
        assert_eq!(sorted(&heap), [1, 2]);
    }

    #[test]
    fn equal_item_is_refused_at_the_maximum() {
        // Unlike a set, a heap keeps equal items apart.
        let mut heap = heap_of::<0, 2>(&[1, 2]);
        assert_eq!(heap.push(2), Err(Error::Oversize { len: 2, max_len: 2 }));
        assert_eq!(sorted(&heap), [1, 2]);
    }

    #[test]
    fn item_is_refused_by_a_zero_maximum() {
        let mut heap = Heap::<0, 0>::new();
        assert_eq!(heap.push(1), Err(Error::Oversize { len: 0, max_len: 0 }));
        assert!(heap.is_empty());
    }

    #[test]
    fn heap_with_a_minimum_is_filled_up_to_the_maximum() {
        let mut heap = heap_of::<1, 3>(&[2]);
        assert_eq!(heap.push(3), Ok(()));
        assert_eq!(heap.push(1), Ok(()));
        assert_eq!(heap.push(4), Err(Error::Oversize { len: 3, max_len: 3 }));
        assert_eq!(heap.peek(), Some(&3));
        assert_eq!(sorted(&heap), [1, 2, 3]);
    }
}

mod extend {
    use super::*;

    #[test]
    fn fitting_items_are_added() {
        let mut heap = heap_of::<0, 4>(&[2]);
        assert_eq!(heap.extend([4, 1, 3]), Ok(()));
        assert_eq!(heap.peek(), Some(&4));
        assert_eq!(sorted(&heap), [1, 2, 3, 4]);
    }

    #[test]
    fn items_before_the_one_which_does_not_fit_are_kept() {
        let mut heap = heap_of::<0, 3>(&[2]);
        assert_eq!(
            heap.extend([4, 1, 3, 5]),
            Err(Error::Oversize { len: 3, max_len: 3 })
        );
        assert_eq!(heap.peek(), Some(&4));
        assert_eq!(sorted(&heap), [1, 2, 4]);
    }

    #[test]
    fn empty_iterator_adds_nothing_at_the_maximum() {
        let mut heap = heap_of::<0, 2>(&[1, 2]);
        assert_eq!(heap.extend([]), Ok(()));
        assert_eq!(sorted(&heap), [1, 2]);
    }
}

mod construction {
    use super::*;

    #[test]
    fn items_of_an_iterator_are_collected_into_a_heap() {
        let heap = Heap::<1, 3>::try_from_iter([2, 3, 1]).unwrap();
        assert_eq!(heap.peek(), Some(&3));
        assert_eq!(sorted(&heap), [1, 2, 3]);
    }

    #[test]
    fn too_few_items_of_an_iterator_are_rejected() {
        assert_eq!(
            Heap::<2, 3>::try_from_iter([1]).err(),
            Some(Error::Undersize { len: 1, min_len: 2 })
        );
    }

    #[test]
    fn too_many_items_of_an_iterator_are_rejected() {
        assert_eq!(
            Heap::<2, 3>::try_from_iter([1, 2, 3, 4, 5]).err(),
            Some(Error::Oversize { len: 4, max_len: 3 })
        );
    }

    #[test]
    fn equal_items_of_an_iterator_are_counted_separately() {
        assert_eq!(
            Heap::<0, 2>::try_from_iter([1, 1, 1]).err(),
            Some(Error::Oversize { len: 3, max_len: 2 })
        );
        assert_eq!(Heap::<0, 3>::try_from_iter([1, 1, 1]).unwrap().len(), 3);
    }

    #[test]
    fn heap_within_the_bounds_is_wrapped() {
        let heap = Heap::<1, 3>::try_from(BinaryHeap::from([2, 3, 1])).unwrap();
        assert_eq!(heap.peek(), Some(&3));
        assert_eq!(heap.len(), 3);
    }

    #[test]
    fn heap_below_the_minimum_is_rejected() {
        assert_eq!(
            Heap::<2, 3>::try_from(BinaryHeap::from([1])).err(),
            Some(Error::Undersize { len: 1, min_len: 2 })
        );
    }

    #[test]
    fn heap_above_the_maximum_is_rejected() {
        assert_eq!(
            Heap::<2, 3>::try_from(BinaryHeap::from([1, 2, 3, 4])).err(),
            Some(Error::Oversize { len: 4, max_len: 3 })
        );
    }

    #[test]
    fn single_item_makes_a_heap_with_a_minimum() {
        let heap = Heap::<1, 3>::with(7);
        assert_eq!(heap.peek(), Some(&7));
        assert_eq!(heap.len(), 1);
    }

    #[test]
    fn released_heap_keeps_its_items() {
        let heap: BinaryHeap<u8> = heap_of::<1, 3>(&[2, 3, 1]).release();
        assert_eq!(heap.into_sorted_vec(), [1, 2, 3]);
    }
}

mod clear {
    use super::*;

    #[test]
    fn all_items_are_removed() {
        let mut heap = heap_of::<0, 3>(&[1, 2, 3]);
        heap.clear();
        assert!(heap.is_empty());
        assert_eq!(heap.peek(), None);
    }

    #[test]
    fn cleared_heap_is_filled_up_to_the_maximum_again() {
        let mut heap = heap_of::<0, 2>(&[1, 2]);
        heap.clear();
        assert_eq!(heap.extend([3, 4]), Ok(()));
        assert_eq!(heap.push(5), Err(Error::Oversize { len: 2, max_len: 2 }));
        assert_eq!(sorted(&heap), [3, 4]);
    }
}
