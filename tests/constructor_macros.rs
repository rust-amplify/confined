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

//! Constructor macros (`src/macros.rs`).
//!
//! Each module covers one family of macros, and each test covers one form of
//! their input: the empty one, the list, the repeated item. The result of
//! each macro is bound to the alias which the macro has to build, so a macro
//! which builds a collection of another size fails to compile.
//!
//! The module `overflow` gives the macros of the smallest size a content
//! which doesn't fit, and checks the message of the panic. Its tests with
//! 65 536 items are skipped by Miri, which needs too long for them.
//!
//! The expansion of the macros without the standard prelude is covered by
//! `tests/macros.rs`.

#![cfg(feature = "alloc")]

mod strings {
    use confined::{MediumString, SmallString, TinyString, medium_s, small_s, tiny_s};

    #[test]
    fn empty_form_builds_an_empty_string() {
        let tiny: TinyString = tiny_s!();
        let small: SmallString = small_s!();
        let medium: MediumString = medium_s!();
        assert!(tiny.is_empty());
        assert!(small.is_empty());
        assert!(medium.is_empty());
    }

    #[test]
    fn literal_is_copied_into_the_string() {
        let tiny: TinyString = tiny_s!("añ€");
        let small: SmallString = small_s!("añ€");
        let medium: MediumString = medium_s!("añ€");
        assert_eq!(tiny.as_str(), "añ€");
        assert_eq!(small.as_str(), "añ€");
        assert_eq!(medium.as_str(), "añ€");
    }

    #[test]
    fn empty_literal_builds_an_empty_string() {
        let tiny: TinyString = tiny_s!("");
        assert!(tiny.is_empty());
    }

    #[test]
    fn literal_of_the_maximal_length_fits() {
        let tiny: TinyString = tiny_s!(
            "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcde"
        );
        assert_eq!(tiny.len(), 255);
    }
}

mod blobs {
    use confined::{MediumBlob, SmallBlob, TinyBlob, medium_blob, small_blob, tiny_blob};

    #[test]
    fn empty_form_builds_an_empty_blob() {
        let tiny: TinyBlob = tiny_blob![];
        let small: SmallBlob = small_blob![];
        let medium: MediumBlob = medium_blob![];
        assert!(tiny.is_empty());
        assert!(small.is_empty());
        assert!(medium.is_empty());
    }

    #[test]
    fn list_keeps_the_order_of_its_bytes() {
        let tiny: TinyBlob = tiny_blob![3, 1, 2];
        let small: SmallBlob = small_blob![3, 1, 2];
        let medium: MediumBlob = medium_blob![3, 1, 2];
        assert_eq!(tiny.as_slice(), [3, 1, 2]);
        assert_eq!(small.as_slice(), [3, 1, 2]);
        assert_eq!(medium.as_slice(), [3, 1, 2]);
    }

    #[test]
    fn list_may_end_with_a_comma() {
        let tiny: TinyBlob = tiny_blob![1, 2,];
        let small: SmallBlob = small_blob![1, 2,];
        let medium: MediumBlob = medium_blob![1, 2,];
        assert_eq!(tiny.as_slice(), [1, 2]);
        assert_eq!(small.as_slice(), [1, 2]);
        assert_eq!(medium.as_slice(), [1, 2]);
    }

    #[test]
    fn byte_is_repeated_the_given_number_of_times() {
        let tiny: TinyBlob = tiny_blob![7; 3];
        let small: SmallBlob = small_blob![7; 3];
        let medium: MediumBlob = medium_blob![7; 3];
        assert_eq!(tiny.as_slice(), [7, 7, 7]);
        assert_eq!(small.as_slice(), [7, 7, 7]);
        assert_eq!(medium.as_slice(), [7, 7, 7]);
    }

    #[test]
    fn byte_repeated_zero_times_builds_an_empty_blob() {
        let tiny: TinyBlob = tiny_blob![7; 0];
        assert!(tiny.is_empty());
    }

    #[test]
    fn content_of_the_maximal_length_fits() {
        let tiny: TinyBlob = tiny_blob![7; 255];
        assert_eq!(tiny.len_u8(), u8::MAX);
    }
}

mod vecs {
    use confined::{MediumVec, SmallVec, TinyVec, medium_vec, small_vec, tiny_vec};

    #[test]
    fn empty_form_builds_an_empty_vector() {
        let tiny: TinyVec<u16> = tiny_vec![];
        let small: SmallVec<u16> = small_vec![];
        let medium: MediumVec<u16> = medium_vec![];
        assert!(tiny.is_empty());
        assert!(small.is_empty());
        assert!(medium.is_empty());
    }

    #[test]
    fn list_keeps_the_order_of_its_items() {
        let tiny: TinyVec<u16> = tiny_vec![3, 1, 2];
        let small: SmallVec<u16> = small_vec![3, 1, 2];
        let medium: MediumVec<u16> = medium_vec![3, 1, 2];
        assert_eq!(tiny.as_slice(), [3, 1, 2]);
        assert_eq!(small.as_slice(), [3, 1, 2]);
        assert_eq!(medium.as_slice(), [3, 1, 2]);
    }

    #[test]
    fn list_may_end_with_a_comma() {
        let tiny: TinyVec<u16> = tiny_vec![1, 2,];
        let small: SmallVec<u16> = small_vec![1, 2,];
        let medium: MediumVec<u16> = medium_vec![1, 2,];
        assert_eq!(tiny.as_slice(), [1, 2]);
        assert_eq!(small.as_slice(), [1, 2]);
        assert_eq!(medium.as_slice(), [1, 2]);
    }

    #[test]
    fn item_is_cloned_the_given_number_of_times() {
        let tiny: TinyVec<String> = tiny_vec![String::from("a"); 3];
        let small: SmallVec<String> = small_vec![String::from("a"); 3];
        let medium: MediumVec<String> = medium_vec![String::from("a"); 3];
        assert_eq!(tiny.as_slice(), ["a", "a", "a"]);
        assert_eq!(small.as_slice(), ["a", "a", "a"]);
        assert_eq!(medium.as_slice(), ["a", "a", "a"]);
    }

    #[test]
    fn item_repeated_zero_times_builds_an_empty_vector() {
        let tiny: TinyVec<u16> = tiny_vec![7; 0];
        assert!(tiny.is_empty());
    }

    #[test]
    fn content_of_the_maximal_length_fits() {
        let tiny: TinyVec<u16> = tiny_vec![7; 255];
        assert_eq!(tiny.len_u8(), u8::MAX);
    }
}

mod deques {
    use confined::{MediumDeque, SmallDeque, TinyDeque, medium_deque, small_deque, tiny_deque};

    #[test]
    fn empty_form_builds_an_empty_deque() {
        let tiny: TinyDeque<u16> = tiny_deque![];
        let small: SmallDeque<u16> = small_deque![];
        let medium: MediumDeque<u16> = medium_deque![];
        assert!(tiny.is_empty());
        assert!(small.is_empty());
        assert!(medium.is_empty());
    }

    #[test]
    fn list_goes_from_the_front_to_the_back() {
        let tiny: TinyDeque<u16> = tiny_deque![3, 1, 2];
        let small: SmallDeque<u16> = small_deque![3, 1, 2];
        let medium: MediumDeque<u16> = medium_deque![3, 1, 2,];
        assert!(tiny.iter().eq(&[3, 1, 2]));
        assert!(small.iter().eq(&[3, 1, 2]));
        assert!(medium.iter().eq(&[3, 1, 2]));
    }

    #[test]
    fn item_is_cloned_the_given_number_of_times() {
        let tiny: TinyDeque<u16> = tiny_deque![7; 3];
        let small: SmallDeque<u16> = small_deque![7; 3];
        let medium: MediumDeque<u16> = medium_deque![7; 3];
        assert!(tiny.iter().eq(&[7, 7, 7]));
        assert!(small.iter().eq(&[7, 7, 7]));
        assert!(medium.iter().eq(&[7, 7, 7]));
    }

    #[test]
    fn content_of_the_maximal_length_fits() {
        let tiny: TinyDeque<u16> = tiny_deque![7; 255];
        assert_eq!(tiny.len_u8(), u8::MAX);
    }
}

mod lists {
    use confined::{MediumList, SmallList, TinyList, medium_list, small_list, tiny_list};

    #[test]
    fn empty_form_builds_an_empty_list() {
        let tiny: TinyList<u16> = tiny_list![];
        let small: SmallList<u16> = small_list![];
        let medium: MediumList<u16> = medium_list![];
        assert!(tiny.is_empty());
        assert!(small.is_empty());
        assert!(medium.is_empty());
    }

    #[test]
    fn list_goes_from_the_head_to_the_tail() {
        let tiny: TinyList<u16> = tiny_list![3, 1, 2];
        let small: SmallList<u16> = small_list![3, 1, 2];
        let medium: MediumList<u16> = medium_list![3, 1, 2,];
        assert!(tiny.iter().eq(&[3, 1, 2]));
        assert!(small.iter().eq(&[3, 1, 2]));
        assert!(medium.iter().eq(&[3, 1, 2]));
    }
}

mod heaps {
    use confined::{MediumHeap, SmallHeap, TinyHeap, medium_heap, small_heap, tiny_heap};

    #[test]
    fn empty_form_builds_an_empty_heap() {
        let tiny: TinyHeap<u16> = tiny_heap![];
        let small: SmallHeap<u16> = small_heap![];
        let medium: MediumHeap<u16> = medium_heap![];
        assert!(tiny.is_empty());
        assert!(small.is_empty());
        assert!(medium.is_empty());
    }

    #[test]
    fn greatest_item_of_the_list_is_at_the_top() {
        let tiny: TinyHeap<u16> = tiny_heap![1, 3, 2];
        let small: SmallHeap<u16> = small_heap![1, 3, 2];
        let medium: MediumHeap<u16> = medium_heap![1, 3, 2,];
        assert_eq!((tiny.peek(), tiny.len()), (Some(&3), 3));
        assert_eq!((small.peek(), small.len()), (Some(&3), 3));
        assert_eq!((medium.peek(), medium.len()), (Some(&3), 3));
    }

    #[test]
    fn equal_items_of_the_list_are_all_kept() {
        let tiny: TinyHeap<u16> = tiny_heap![2, 2, 2];
        assert_eq!(tiny.len(), 3);
    }
}

mod ordered_sets {
    use confined::{MediumOrdSet, SmallOrdSet, TinyOrdSet, medium_bset, small_bset, tiny_bset};

    #[test]
    fn empty_form_builds_an_empty_set() {
        let tiny: TinyOrdSet<u16> = tiny_bset![];
        let small: SmallOrdSet<u16> = small_bset![];
        let medium: MediumOrdSet<u16> = medium_bset![];
        assert!(tiny.is_empty());
        assert!(small.is_empty());
        assert!(medium.is_empty());
    }

    #[test]
    fn items_of_the_list_are_sorted() {
        let tiny: TinyOrdSet<u16> = tiny_bset![3, 1, 2];
        let small: SmallOrdSet<u16> = small_bset![3, 1, 2];
        let medium: MediumOrdSet<u16> = medium_bset![3, 1, 2,];
        assert!(tiny.iter().eq(&[1, 2, 3]));
        assert!(small.iter().eq(&[1, 2, 3]));
        assert!(medium.iter().eq(&[1, 2, 3]));
    }

    #[test]
    fn repeated_items_of_the_list_are_kept_once() {
        let tiny: TinyOrdSet<u16> = tiny_bset![2, 1, 2, 1];
        assert!(tiny.iter().eq(&[1, 2]));
    }
}

mod ordered_maps {
    use confined::{MediumOrdMap, SmallOrdMap, TinyOrdMap, medium_bmap, small_bmap, tiny_bmap};

    #[test]
    fn empty_form_builds_an_empty_map() {
        let tiny: TinyOrdMap<u16, char> = tiny_bmap! {};
        let small: SmallOrdMap<u16, char> = small_bmap! {};
        let medium: MediumOrdMap<u16, char> = medium_bmap! {};
        assert!(tiny.is_empty());
        assert!(small.is_empty());
        assert!(medium.is_empty());
    }

    #[test]
    fn entries_of_the_list_are_sorted_by_their_keys() {
        let tiny: TinyOrdMap<u16, char> = tiny_bmap! { 2 => 'b', 1 => 'a' };
        let small: SmallOrdMap<u16, char> = small_bmap! { 2 => 'b', 1 => 'a' };
        let medium: MediumOrdMap<u16, char> = medium_bmap! { 2 => 'b', 1 => 'a', };
        assert!(tiny.keys().eq(&[1, 2]) && tiny.values().eq(&['a', 'b']));
        assert!(small.keys().eq(&[1, 2]) && small.values().eq(&['a', 'b']));
        assert!(medium.keys().eq(&[1, 2]) && medium.values().eq(&['a', 'b']));
    }

    #[test]
    fn last_value_of_a_repeated_key_is_kept() {
        let tiny: TinyOrdMap<u16, char> = tiny_bmap! { 1 => 'a', 2 => 'b', 1 => 'x' };
        assert_eq!(tiny.len(), 2);
        assert_eq!(tiny.get(&1), Some(&'x'));
    }
}

#[cfg(feature = "std")]
mod hash_sets {
    use confined::{MediumHashSet, SmallHashSet, TinyHashSet, medium_set, small_set, tiny_set};

    #[test]
    fn empty_form_builds_an_empty_set() {
        let tiny: TinyHashSet<u16> = tiny_set![];
        let small: SmallHashSet<u16> = small_set![];
        let medium: MediumHashSet<u16> = medium_set![];
        assert!(tiny.is_empty());
        assert!(small.is_empty());
        assert!(medium.is_empty());
    }

    #[test]
    fn items_of_the_list_are_in_the_set() {
        let tiny: TinyHashSet<u16> = tiny_set![3, 1, 2];
        let small: SmallHashSet<u16> = small_set![3, 1, 2];
        let medium: MediumHashSet<u16> = medium_set![3, 1, 2,];
        for item in [1, 2, 3] {
            assert!(tiny.contains(&item) && small.contains(&item) && medium.contains(&item));
        }
        assert_eq!((tiny.len(), small.len(), medium.len()), (3, 3, 3));
    }

    #[test]
    fn repeated_items_of_the_list_are_kept_once() {
        let tiny: TinyHashSet<u16> = tiny_set![2, 1, 2, 1];
        assert_eq!(tiny.len(), 2);
    }
}

#[cfg(feature = "std")]
mod hash_maps {
    use confined::{MediumHashMap, SmallHashMap, TinyHashMap, medium_map, small_map, tiny_map};

    #[test]
    fn empty_form_builds_an_empty_map() {
        let tiny: TinyHashMap<u16, char> = tiny_map! {};
        let small: SmallHashMap<u16, char> = small_map! {};
        let medium: MediumHashMap<u16, char> = medium_map! {};
        assert!(tiny.is_empty());
        assert!(small.is_empty());
        assert!(medium.is_empty());
    }

    #[test]
    fn entries_of_the_list_are_in_the_map() {
        let tiny: TinyHashMap<u16, char> = tiny_map! { 2 => 'b', 1 => 'a' };
        let small: SmallHashMap<u16, char> = small_map! { 2 => 'b', 1 => 'a' };
        let medium: MediumHashMap<u16, char> = medium_map! { 2 => 'b', 1 => 'a', };
        assert_eq!(
            (tiny.get(&1), tiny.get(&2), tiny.len()),
            (Some(&'a'), Some(&'b'), 2)
        );
        assert_eq!(
            (small.get(&1), small.get(&2), small.len()),
            (Some(&'a'), Some(&'b'), 2)
        );
        assert_eq!(
            (medium.get(&1), medium.get(&2), medium.len()),
            (Some(&'a'), Some(&'b'), 2)
        );
    }

    #[test]
    fn last_value_of_a_repeated_key_is_kept() {
        let tiny: TinyHashMap<u16, char> = tiny_map! { 1 => 'a', 2 => 'b', 1 => 'x' };
        assert_eq!(tiny.len(), 2);
        assert_eq!(tiny.get(&1), Some(&'x'));
    }
}

#[cfg(all(feature = "indexmap", feature = "std"))]
mod index_sets {
    use confined::{
        MediumIndexSet, SmallIndexSet, TinyIndexSet, medium_iset, small_iset, tiny_iset,
    };

    #[test]
    fn empty_form_builds_an_empty_set() {
        let tiny: TinyIndexSet<u16> = tiny_iset![];
        let small: SmallIndexSet<u16> = small_iset![];
        let medium: MediumIndexSet<u16> = medium_iset![];
        assert!(tiny.is_empty());
        assert!(small.is_empty());
        assert!(medium.is_empty());
    }

    #[test]
    fn items_keep_the_order_of_the_list() {
        let tiny: TinyIndexSet<u16> = tiny_iset![3, 1, 2];
        let small: SmallIndexSet<u16> = small_iset![3, 1, 2];
        let medium: MediumIndexSet<u16> = medium_iset![3, 1, 2,];
        assert!(tiny.iter().eq(&[3, 1, 2]));
        assert!(small.iter().eq(&[3, 1, 2]));
        assert!(medium.iter().eq(&[3, 1, 2]));
    }

    #[test]
    fn repeated_item_keeps_its_first_position() {
        let tiny: TinyIndexSet<u16> = tiny_iset![2, 1, 2, 3];
        assert!(tiny.iter().eq(&[2, 1, 3]));
    }
}

#[cfg(all(feature = "indexmap", feature = "std"))]
mod index_maps {
    use confined::{
        MediumIndexMap, SmallIndexMap, TinyIndexMap, medium_imap, small_imap, tiny_imap,
    };

    #[test]
    fn empty_form_builds_an_empty_map() {
        let tiny: TinyIndexMap<u16, char> = tiny_imap! {};
        let small: SmallIndexMap<u16, char> = small_imap! {};
        let medium: MediumIndexMap<u16, char> = medium_imap! {};
        assert!(tiny.is_empty());
        assert!(small.is_empty());
        assert!(medium.is_empty());
    }

    #[test]
    fn entries_keep_the_order_of_the_list() {
        let tiny: TinyIndexMap<u16, char> = tiny_imap! { 2 => 'b', 1 => 'a' };
        let small: SmallIndexMap<u16, char> = small_imap! { 2 => 'b', 1 => 'a' };
        let medium: MediumIndexMap<u16, char> = medium_imap! { 2 => 'b', 1 => 'a', };
        assert!(tiny.keys().eq(&[2, 1]) && tiny.values().eq(&['b', 'a']));
        assert!(small.keys().eq(&[2, 1]) && small.values().eq(&['b', 'a']));
        assert!(medium.keys().eq(&[2, 1]) && medium.values().eq(&['b', 'a']));
    }

    #[test]
    fn repeated_key_keeps_its_first_position_and_its_last_value() {
        let tiny: TinyIndexMap<u16, char> = tiny_imap! { 2 => 'b', 1 => 'a', 2 => 'x' };
        assert!(tiny.keys().eq(&[2, 1]) && tiny.values().eq(&['x', 'a']));
    }
}

mod overflow {
    use std::cell::Cell;

    use confined::{
        small_blob, small_deque, small_vec, tiny_blob, tiny_bmap, tiny_bset, tiny_deque, tiny_heap,
        tiny_list, tiny_s, tiny_vec,
    };

    thread_local! {
        /// The last number given out to the thread, which runs a single test.
        static LAST: Cell<u16> = const { Cell::new(0) };
    }

    /// A number which the test has not got before.
    fn fresh() -> u16 {
        LAST.set(LAST.get() + 1);
        LAST.get()
    }

    /// Calls the macro named last with a list where each item of the given
    /// one is written four times for each `quadruple` named before it.
    macro_rules! quadruple {
        ($target:ident; $($item:expr),+) => {
            $target![$($item, $item, $item, $item),+]
        };
        ($next:ident, $($rest:ident),+; $($item:expr),+) => {
            $next!($($rest),+; $($item, $item, $item, $item),+)
        };
    }

    /// The same as `quadruple`, for the lists of `key => value` pairs.
    macro_rules! quadruple_pairs {
        ($target:ident; $($key:expr => $value:expr),+) => {
            $target! { $($key => $value, $key => $value, $key => $value, $key => $value),+ }
        };
        ($next:ident, $($rest:ident),+; $($key:expr => $value:expr),+) => {
            $next!($($rest),+; $($key => $value, $key => $value, $key => $value, $key => $value),+)
        };
    }

    #[test]
    fn helper_writes_an_item_256_times() {
        fn count(items: [u16; 256]) -> usize {
            items.len()
        }

        macro_rules! array {
            ($($item:expr),+) => { [$($item),+] };
        }

        let items = quadruple!(quadruple, quadruple, quadruple, array; fresh());
        assert_eq!(count(items), 256);
        assert!(items.iter().copied().eq(1..=256));
    }

    #[test]
    #[should_panic(expected = "static string for tiny_s literal is too long")]
    fn string_literal_of_256_bytes_does_not_fit() {
        let _ = tiny_s!(
            "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
        );
    }

    #[test]
    #[should_panic(expected = "inline tiny_blob contains invalid number of items")]
    fn blob_of_256_bytes_does_not_fit() {
        let _ = tiny_blob![7; 256];
    }

    #[test]
    #[cfg_attr(miri, ignore = "65 536 items take too long under Miri")]
    #[should_panic(expected = "inline small_blob contains invalid number of items")]
    fn blob_of_65_536_bytes_does_not_fit_the_next_size() {
        let _ = small_blob![7; 65_536];
    }

    #[test]
    #[should_panic(expected = "inline tiny_vec literal contains invalid number of items")]
    fn vector_of_256_items_does_not_fit() {
        let _ = tiny_vec![(); 256];
    }

    #[test]
    #[should_panic(expected = "inline tiny_vec literal contains invalid number of items")]
    fn vector_of_256_listed_items_does_not_fit() {
        let _ = quadruple!(quadruple, quadruple, quadruple, tiny_vec; ());
    }

    #[test]
    #[cfg_attr(miri, ignore = "65 536 items take too long under Miri")]
    #[should_panic(expected = "inline small_vec literal contains invalid number of items")]
    fn vector_of_65_536_items_does_not_fit_the_next_size() {
        let _ = small_vec![(); 65_536];
    }

    #[test]
    #[should_panic(expected = "inline tiny_deque contains invalid number of items")]
    fn deque_of_256_items_does_not_fit() {
        let _ = tiny_deque![(); 256];
    }

    #[test]
    #[cfg_attr(miri, ignore = "65 536 items take too long under Miri")]
    #[should_panic(expected = "inline small_deque contains invalid number of items")]
    fn deque_of_65_536_items_does_not_fit_the_next_size() {
        let _ = small_deque![(); 65_536];
    }

    #[test]
    #[should_panic(expected = "inline tiny_list contains invalid number of items")]
    fn list_of_256_items_does_not_fit() {
        let _ = quadruple!(quadruple, quadruple, quadruple, tiny_list; ());
    }

    #[test]
    #[should_panic(expected = "inline tiny_heap contains invalid number of items")]
    fn heap_of_256_items_does_not_fit() {
        let _ = quadruple!(quadruple, quadruple, quadruple, tiny_heap; ());
    }

    #[test]
    #[should_panic(expected = "inline tiny_bset literal contains invalid number of items")]
    fn ordered_set_of_256_items_does_not_fit() {
        let _ = quadruple!(quadruple, quadruple, quadruple, tiny_bset; fresh());
    }

    #[test]
    fn ordered_set_counts_256_equal_items_once() {
        let set = quadruple!(quadruple, quadruple, quadruple, tiny_bset; 7u16);
        assert_eq!(set.len(), 1);
    }

    #[test]
    #[should_panic(expected = "inline tiny_bmap literal contains invalid number of items")]
    fn ordered_map_of_256_keys_does_not_fit() {
        let _ = quadruple_pairs!(
            quadruple_pairs, quadruple_pairs, quadruple_pairs, tiny_bmap; fresh() => 'a'
        );
    }

    #[cfg(feature = "std")]
    #[test]
    #[should_panic(expected = "inline tiny_set literal contains invalid number of items")]
    fn hash_set_of_256_items_does_not_fit() {
        use confined::tiny_set;

        let _ = quadruple!(quadruple, quadruple, quadruple, tiny_set; fresh());
    }

    #[cfg(feature = "std")]
    #[test]
    #[should_panic(expected = "inline tiny_map literal contains invalid number of items")]
    fn hash_map_of_256_keys_does_not_fit() {
        use confined::tiny_map;

        let _ = quadruple_pairs!(
            quadruple_pairs, quadruple_pairs, quadruple_pairs, tiny_map; fresh() => 'a'
        );
    }

    #[cfg(all(feature = "indexmap", feature = "std"))]
    #[test]
    #[should_panic(expected = "inline tiny_iset literal contains invalid number of items")]
    fn index_set_of_256_items_does_not_fit() {
        use confined::tiny_iset;

        let _ = quadruple!(quadruple, quadruple, quadruple, tiny_iset; fresh());
    }

    #[cfg(all(feature = "indexmap", feature = "std"))]
    #[test]
    #[should_panic(expected = "inline tiny_imap literal contains invalid number of items")]
    fn index_map_of_256_keys_does_not_fit() {
        use confined::tiny_imap;

        let _ = quadruple_pairs!(
            quadruple_pairs, quadruple_pairs, quadruple_pairs, tiny_imap; fresh() => 'a'
        );
    }
}
