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

//! Checked methods of the confined `Vec` (`src/imp/vec.rs`).
//!
//! Each module covers one method, and each test covers one case of it:
//! `from_array`, `try_from_slice`, `from_slice_checked`, `pop`, `remove`,
//! `iter`, `as_slice` and `into_vec`.
//!
//! The module `largest_length` runs the checked methods on a vector of
//! `usize::MAX` zero-sized items, where an unchecked addition would overflow.
//!
//! The methods shared by all collections are covered by `tests/generic.rs`,
//! the vectors with a custom allocator by `tests/allocator.rs`, and the
//! refused allocations by `tests/oom.rs`.

#![cfg(feature = "alloc")]

use confined::{Confined, Error};

type Bytes<const MIN: usize, const MAX: usize> = Confined<Vec<u8>, MIN, MAX>;

fn vec_of<const MIN: usize, const MAX: usize>(items: &[u8]) -> Bytes<MIN, MAX> {
    Bytes::try_from(items.to_vec()).unwrap()
}

mod from_array {
    use super::*;

    #[test]
    fn array_of_the_minimal_length_is_accepted() {
        let vec = Bytes::<2, 4>::from_array([1, 2]);
        assert_eq!(vec.as_slice(), [1, 2]);
    }

    #[test]
    fn array_of_the_maximal_length_is_accepted() {
        let vec = Bytes::<2, 4>::from_array([1, 2, 3, 4]);
        assert_eq!(vec.as_slice(), [1, 2, 3, 4]);
    }

    #[test]
    fn items_keep_the_order_of_the_array() {
        let vec = Bytes::<1, 5>::from_array([3, 1, 2]);
        assert_eq!(vec.as_slice(), [3, 1, 2]);
    }

    #[test]
    fn empty_array_fits_a_confinement_without_a_minimum() {
        let vec = Bytes::<0, 4>::from_array([]);
        assert!(vec.is_empty());
    }

    #[test]
    fn array_fits_a_confinement_of_a_single_length() {
        let vec = Bytes::<3, 3>::from_array([1, 2, 3]);
        assert_eq!(vec.as_slice(), [1, 2, 3]);
    }

    #[test]
    fn items_are_moved_and_need_not_be_cloned() {
        struct Unique(u8);

        let vec = Confined::<Vec<Unique>, 1, 2>::from_array([Unique(1), Unique(2)]);
        assert_eq!(vec.iter().map(|item| item.0).collect::<Vec<_>>(), [1, 2]);
    }
}

mod try_from_slice {
    use std::cell::Cell;

    use super::*;

    #[test]
    fn slice_at_the_minimum_is_accepted() {
        let vec = Bytes::<2, 4>::try_from_slice(&[1, 2]).unwrap();
        assert_eq!(vec.as_slice(), [1, 2]);
    }

    #[test]
    fn slice_at_the_maximum_is_accepted() {
        let vec = Bytes::<2, 4>::try_from_slice(&[1, 2, 3, 4]).unwrap();
        assert_eq!(vec.as_slice(), [1, 2, 3, 4]);
    }

    #[test]
    fn slice_below_the_minimum_is_rejected() {
        assert_eq!(
            Bytes::<2, 4>::try_from_slice(&[1]),
            Err(Error::Undersize { len: 1, min_len: 2 })
        );
    }

    #[test]
    fn slice_above_the_maximum_is_rejected() {
        assert_eq!(
            Bytes::<2, 4>::try_from_slice(&[1, 2, 3, 4, 5]),
            Err(Error::Oversize { len: 5, max_len: 4 })
        );
    }

    #[test]
    fn empty_slice_fits_a_confinement_without_a_minimum() {
        assert!(Bytes::<0, 4>::try_from_slice(&[]).unwrap().is_empty());
    }

    #[test]
    fn empty_slice_is_rejected_by_a_minimum() {
        assert_eq!(
            Bytes::<1, 4>::try_from_slice(&[]),
            Err(Error::Undersize { len: 0, min_len: 1 })
        );
    }

    #[test]
    fn items_are_cloned_in_order_and_the_slice_is_left_intact() {
        let source = [String::from("b"), String::from("a")];
        let vec = Confined::<Vec<String>, 1, 2>::try_from_slice(&source).unwrap();
        assert_eq!(vec.as_slice(), ["b", "a"]);
        assert_eq!(source, ["b", "a"]);
    }

    /// An item which counts the clones made of it and of its clones.
    struct Counted<'a>(&'a Cell<usize>);

    impl Clone for Counted<'_> {
        fn clone(&self) -> Self {
            self.0.set(self.0.get() + 1);
            Counted(self.0)
        }
    }

    #[test]
    fn each_item_of_an_accepted_slice_is_cloned_once() {
        let clones = Cell::new(0);
        let source = [Counted(&clones), Counted(&clones)];
        let vec = Confined::<Vec<Counted<'_>>, 1, 2>::try_from_slice(&source).unwrap();
        assert_eq!(vec.len(), 2);
        assert_eq!(clones.get(), 2);
    }

    #[test]
    fn no_item_of_a_rejected_slice_is_cloned() {
        let clones = Cell::new(0);
        let source = [Counted(&clones), Counted(&clones), Counted(&clones)];
        assert_eq!(
            Confined::<Vec<Counted<'_>>, 1, 2>::try_from_slice(&source).err(),
            Some(Error::Oversize { len: 3, max_len: 2 })
        );
        assert_eq!(
            Confined::<Vec<Counted<'_>>, 4, 5>::try_from_slice(&source).err(),
            Some(Error::Undersize { len: 3, min_len: 4 })
        );
        assert_eq!(clones.get(), 0);
    }
}

mod from_slice_checked {
    use super::*;

    #[test]
    fn fitting_slice_is_returned() {
        let vec = Bytes::<1, 3>::from_slice_checked(&[1, 2]);
        assert_eq!(vec.as_slice(), [1, 2]);
    }

    #[test]
    fn slices_at_both_bounds_are_returned() {
        assert_eq!(Bytes::<1, 3>::from_slice_checked(&[1]).len(), 1);
        assert_eq!(Bytes::<1, 3>::from_slice_checked(&[1, 2, 3]).len(), 3);
    }

    #[test]
    #[should_panic(expected = "slice size mismatch")]
    fn slice_below_the_minimum_is_a_panic() {
        Bytes::<1, 3>::from_slice_checked(&[]);
    }

    #[test]
    #[should_panic(expected = "slice size mismatch")]
    fn slice_above_the_maximum_is_a_panic() {
        Bytes::<1, 3>::from_slice_checked(&[1, 2, 3, 4]);
    }
}

mod pop {
    use super::*;

    #[test]
    fn last_item_is_returned() {
        let mut vec = vec_of::<0, 3>(&[1, 2, 3]);
        assert_eq!(vec.pop(), Some(3));
        assert_eq!(vec.as_slice(), [1, 2]);
    }

    #[test]
    fn items_are_returned_from_the_end_until_the_vector_is_empty() {
        let mut vec = vec_of::<0, 3>(&[1, 2]);
        assert_eq!(vec.pop(), Some(2));
        assert_eq!(vec.pop(), Some(1));
        assert_eq!(vec.pop(), None);
        assert!(vec.is_empty());
    }

    #[test]
    fn empty_vector_returns_nothing() {
        let mut vec = Bytes::<0, 3>::new();
        assert_eq!(vec.pop(), None);
        assert!(vec.is_empty());
    }

    #[test]
    fn full_vector_accepts_an_item_after_a_pop() {
        let mut vec = vec_of::<0, 2>(&[1, 2]);
        assert_eq!(vec.pop(), Some(2));
        assert_eq!(vec.push(3), Ok(()));
        assert_eq!(vec.as_slice(), [1, 3]);
    }
}

mod remove {
    use super::*;

    #[test]
    fn item_at_the_index_is_returned_and_the_rest_is_shifted() {
        let mut vec = vec_of::<1, 4>(&[1, 2, 3, 4]);
        assert_eq!(vec.remove(1), Ok(2));
        assert_eq!(vec.as_slice(), [1, 3, 4]);
    }

    #[test]
    fn first_item_is_removed() {
        let mut vec = vec_of::<1, 3>(&[1, 2, 3]);
        assert_eq!(vec.remove(0), Ok(1));
        assert_eq!(vec.as_slice(), [2, 3]);
    }

    #[test]
    fn last_item_is_removed() {
        let mut vec = vec_of::<1, 3>(&[1, 2, 3]);
        assert_eq!(vec.remove(2), Ok(3));
        assert_eq!(vec.as_slice(), [1, 2]);
    }

    #[test]
    fn index_equal_to_the_length_is_out_of_boundary() {
        let mut vec = vec_of::<1, 3>(&[1, 2, 3]);
        assert_eq!(
            vec.remove(3),
            Err(Error::OutOfBoundary { index: 3, len: 3 })
        );
        assert_eq!(vec.as_slice(), [1, 2, 3]);
    }

    #[test]
    fn largest_index_is_out_of_boundary() {
        let mut vec = vec_of::<1, 3>(&[1, 2, 3]);
        assert_eq!(
            vec.remove(usize::MAX),
            Err(Error::OutOfBoundary {
                index: usize::MAX,
                len: 3
            })
        );
        assert_eq!(vec.as_slice(), [1, 2, 3]);
    }

    #[test]
    fn item_is_kept_at_the_minimum() {
        let mut vec = vec_of::<2, 3>(&[1, 2]);
        assert_eq!(vec.remove(0), Err(Error::Undersize { len: 2, min_len: 2 }));
        assert_eq!(vec.as_slice(), [1, 2]);
    }

    #[test]
    fn index_is_checked_before_the_minimum() {
        let mut vec = vec_of::<2, 3>(&[1, 2]);
        assert_eq!(
            vec.remove(2),
            Err(Error::OutOfBoundary { index: 2, len: 2 })
        );
        assert_eq!(vec.as_slice(), [1, 2]);
    }

    #[test]
    fn items_are_removed_down_to_the_minimum_and_not_further() {
        let mut vec = vec_of::<1, 3>(&[1, 2, 3]);
        assert_eq!(vec.remove(0), Ok(1));
        assert_eq!(vec.remove(0), Ok(2));
        assert_eq!(vec.remove(0), Err(Error::Undersize { len: 1, min_len: 1 }));
        assert_eq!(vec.as_slice(), [3]);
    }

    #[test]
    fn only_item_is_removed_without_a_minimum() {
        let mut vec = vec_of::<0, 3>(&[1]);
        assert_eq!(vec.remove(0), Ok(1));
        assert!(vec.is_empty());
    }

    #[test]
    fn empty_vector_has_no_item_to_remove() {
        let mut vec = Bytes::<0, 3>::new();
        assert_eq!(
            vec.remove(0),
            Err(Error::OutOfBoundary { index: 0, len: 0 })
        );
        assert!(vec.is_empty());
    }
}

mod iter {
    use super::*;

    #[test]
    fn items_are_yielded_from_start_to_end() {
        let vec = vec_of::<1, 3>(&[3, 1, 2]);
        assert_eq!(vec.iter().copied().collect::<Vec<_>>(), [3, 1, 2]);
    }

    #[test]
    fn iterator_knows_its_length_and_runs_backwards() {
        let vec = vec_of::<1, 3>(&[3, 1, 2]);
        assert_eq!(vec.iter().len(), 3);
        assert_eq!(vec.iter().rev().copied().collect::<Vec<_>>(), [2, 1, 3]);
    }

    #[test]
    fn empty_vector_yields_nothing() {
        assert_eq!(Bytes::<0, 3>::new().iter().next(), None);
    }
}

mod as_slice {
    use super::*;

    #[test]
    fn slice_holds_the_items_in_order() {
        let vec = vec_of::<1, 3>(&[3, 1, 2]);
        let slice: &[u8] = vec.as_slice();
        assert_eq!(slice, [3, 1, 2]);
    }

    #[test]
    fn empty_vector_is_an_empty_slice() {
        assert!(Bytes::<0, 3>::new().as_slice().is_empty());
    }
}

mod into_vec {
    use super::*;

    #[test]
    fn inner_vector_is_returned_with_its_items() {
        let vec: Vec<u8> = vec_of::<1, 3>(&[1, 2, 3]).into_vec();
        assert_eq!(vec, [1, 2, 3]);
    }

    #[test]
    fn storage_is_handed_over_without_a_copy() {
        let confined = vec_of::<1, 3>(&[1, 2, 3]);
        let storage = confined.as_slice().as_ptr();
        let vec = confined.into_vec();
        assert_eq!(vec.as_ptr(), storage);
    }

    #[test]
    fn returned_vector_is_no_longer_confined() {
        let mut vec = vec_of::<1, 3>(&[1, 2, 3]).into_vec();
        vec.push(4);
        assert_eq!(vec, [1, 2, 3, 4]);
        vec.clear();
        assert!(vec.is_empty());
    }
}

mod largest_length {
    use super::*;

    /// A vector of zero-sized items takes no memory, whatever its length is.
    type Units<const MIN: usize, const MAX: usize> = Confined<Vec<()>, MIN, MAX>;

    const LARGEST: usize = usize::MAX;

    /// A vector with the largest length a collection can have.
    fn largest<const MIN: usize>() -> Units<MIN, LARGEST> {
        Units::from_array([(); LARGEST])
    }

    /// A vector with room for exactly one more item.
    fn almost_largest() -> Units<0, LARGEST> {
        Units::from_array([(); LARGEST - 1])
    }

    #[test]
    fn length_is_reported_without_a_truncation() {
        assert_eq!(largest::<0>().len(), LARGEST);
        assert!(!largest::<0>().is_empty());
    }

    #[test]
    fn push_reaches_the_largest_length() {
        let mut units = almost_largest();
        assert_eq!(units.push(()), Ok(()));
        assert_eq!(units.len(), LARGEST);
    }

    #[test]
    fn push_at_the_largest_length_is_refused_without_an_overflow() {
        let mut units = largest::<0>();
        assert_eq!(
            units.push(()),
            Err(Error::Oversize {
                len: LARGEST,
                max_len: LARGEST
            })
        );
        assert_eq!(units.len(), LARGEST);
    }

    #[test]
    fn extend_stops_at_the_largest_length() {
        let mut units = almost_largest();
        assert_eq!(
            units.extend([(), (), ()]),
            Err(Error::Oversize {
                len: LARGEST,
                max_len: LARGEST
            })
        );
        assert_eq!(units.len(), LARGEST);
    }

    #[test]
    fn reservation_at_the_largest_length_is_refused_without_an_overflow() {
        let mut units = largest::<0>();
        assert_eq!(units.try_reserve(0), Ok(()));
        for additional in [1, 2, LARGEST] {
            assert_eq!(
                units.try_reserve(additional),
                Err(Error::Oversize {
                    len: LARGEST,
                    max_len: LARGEST
                })
            );
        }
    }

    #[test]
    fn reservation_of_the_room_left_is_accepted() {
        let mut units = almost_largest();
        assert_eq!(units.try_reserve(1), Ok(()));
        assert_eq!(
            units.try_reserve(2),
            Err(Error::Oversize {
                len: LARGEST - 1,
                max_len: LARGEST
            })
        );
    }

    #[test]
    fn length_above_a_smaller_maximum_is_rejected() {
        assert_eq!(
            Units::<0, 10>::try_from(vec_of_units()),
            Err(Error::Oversize {
                len: LARGEST,
                max_len: 10
            })
        );
    }

    fn vec_of_units() -> Vec<()> {
        Vec::from([(); LARGEST])
    }

    #[test]
    fn slice_of_the_largest_length_is_checked_against_the_bounds() {
        let units = vec_of_units();
        assert_eq!(
            Units::<0, 10>::try_from_slice(&units),
            Err(Error::Oversize {
                len: LARGEST,
                max_len: 10
            })
        );
        assert_eq!(
            Units::<0, LARGEST>::try_from_slice(&units).map(|units| units.len()),
            Ok(LARGEST)
        );
    }

    #[test]
    fn items_are_removed_from_the_largest_length() {
        let mut units = largest::<0>();
        assert_eq!(units.pop(), Some(()));
        assert_eq!(units.remove(0), Ok(()));
        assert_eq!(units.remove(LARGEST - 3), Ok(()));
        assert_eq!(units.len(), LARGEST - 3);
    }

    #[test]
    fn index_at_the_largest_length_is_out_of_boundary() {
        let mut units = almost_largest();
        assert_eq!(
            units.remove(LARGEST - 1),
            Err(Error::OutOfBoundary {
                index: LARGEST - 1,
                len: LARGEST - 1
            })
        );
        assert_eq!(
            units.remove(LARGEST),
            Err(Error::OutOfBoundary {
                index: LARGEST,
                len: LARGEST - 1
            })
        );
    }

    #[test]
    fn confinement_of_the_largest_length_only_can_not_change_its_length() {
        let mut units = largest::<LARGEST>();
        assert_eq!(
            units.push(()),
            Err(Error::Oversize {
                len: LARGEST,
                max_len: LARGEST
            })
        );
        assert_eq!(
            units.remove(0),
            Err(Error::Undersize {
                len: LARGEST,
                min_len: LARGEST
            })
        );
        assert_eq!(units.len(), LARGEST);
    }

    #[test]
    fn shorter_vector_is_rejected_by_the_largest_minimum() {
        assert_eq!(
            Units::<LARGEST, LARGEST>::try_from(Vec::from([(); LARGEST - 1])),
            Err(Error::Undersize {
                len: LARGEST - 1,
                min_len: LARGEST
            })
        );
    }
}
