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

//! Traits implemented by [`Confined`] (`src/base.rs`), and the traits which
//! it must not implement.
//!
//! - `unchecked_access`: the build fails if `Confined` gets an implementation
//!   of a trait which changes the size of the collection without a check, or
//!   hands the collection out mutably;
//! - `deref`, `slices`, `iteration`, `indexing`, `display`, `derived`: the
//!   behaviour of the implemented traits;
//! - `layout`: the wrapper adds nothing to the size of the collection.

#![cfg(feature = "alloc")]

use std::collections::BTreeMap;

use confined::Confined;

type Bytes<const MIN: usize, const MAX: usize> = Confined<Vec<u8>, MIN, MAX>;
type Text<const MIN: usize, const MAX: usize> = Confined<String, MIN, MAX>;
type Map<const MIN: usize, const MAX: usize> = Confined<BTreeMap<u8, char>, MIN, MAX>;

fn bytes<const MIN: usize, const MAX: usize>(items: &[u8]) -> Bytes<MIN, MAX> {
    Bytes::try_from(items.to_vec()).unwrap()
}

fn letters<const MIN: usize, const MAX: usize>() -> Map<MIN, MAX> {
    Map::try_from_iter([(1, 'a'), (2, 'b')]).unwrap()
}

mod unchecked_access {
    use std::borrow::BorrowMut;
    use std::fmt;
    use std::ops::DerefMut;

    use super::*;

    /// Fails to build if the type implements the trait.
    ///
    /// A helper trait is implemented for every type with the parameter `()`,
    /// and once more with the parameter `Implemented` for the types which
    /// implement the trait in question. The parameter of the helper trait
    /// can be inferred only when the second implementation doesn't apply.
    macro_rules! assert_not_impl {
        ($type:ty: $($trait:tt)+) => {{
            trait AmbiguousIfImpl<A> {
                fn some_item() {}
            }
            impl<T: ?Sized> AmbiguousIfImpl<()> for T {}
            #[allow(dead_code)]
            struct Implemented;
            impl<T: ?Sized + $($trait)+> AmbiguousIfImpl<Implemented> for T {}
            let _ = <$type as AmbiguousIfImpl<_>>::some_item;
        }};
    }

    fn assert_impl<T: Default>() {}

    #[test]
    fn collection_is_never_dereferenced_mutably() {
        assert_not_impl!(Bytes<1, 3>: DerefMut);
        assert_not_impl!(Bytes<0, 3>: DerefMut);
        assert_not_impl!(Text<1, 3>: DerefMut);
        assert_not_impl!(Map<1, 3>: DerefMut);
    }

    #[test]
    fn collection_is_never_lent_mutably() {
        assert_not_impl!(Bytes<1, 3>: AsMut<Vec<u8>>);
        assert_not_impl!(Bytes<1, 3>: BorrowMut<Vec<u8>>);
        assert_not_impl!(Text<1, 3>: AsMut<String>);
        assert_not_impl!(Text<1, 3>: BorrowMut<String>);
        assert_not_impl!(Map<1, 3>: AsMut<BTreeMap<u8, char>>);
        assert_not_impl!(Map<1, 3>: BorrowMut<BTreeMap<u8, char>>);
    }

    #[test]
    fn collection_with_a_minimum_has_no_default() {
        assert_not_impl!(Bytes<1, 3>: Default);
        assert_not_impl!(Text<1, 3>: Default);
        assert_not_impl!(Map<1, 3>: Default);
    }

    #[test]
    fn collection_without_a_minimum_has_a_default() {
        assert_impl::<Bytes<0, 3>>();
        assert_impl::<Text<0, 3>>();
        assert_impl::<Map<0, 3>>();
    }

    #[test]
    fn collection_is_never_extended_without_a_check() {
        assert_not_impl!(Bytes<0, 3>: Extend<u8>);
        assert_not_impl!(Text<0, 3>: Extend<char>);
        assert_not_impl!(Map<0, 3>: Extend<(u8, char)>);
    }

    #[test]
    fn collection_is_never_collected_without_a_check() {
        assert_not_impl!(Bytes<0, 3>: FromIterator<u8>);
        assert_not_impl!(Text<0, 3>: FromIterator<char>);
        assert_not_impl!(Map<0, 3>: FromIterator<(u8, char)>);
    }

    #[test]
    fn collection_is_never_converted_without_a_check() {
        assert_not_impl!(Bytes<0, 3>: From<Vec<u8>>);
        assert_not_impl!(Bytes<0, 3>: From<&'static [u8]>);
        assert_not_impl!(Text<0, 3>: From<String>);
        assert_not_impl!(Text<0, 3>: From<&'static str>);
        assert_not_impl!(Map<0, 3>: From<BTreeMap<u8, char>>);
    }

    #[test]
    fn string_is_never_written_without_a_check() {
        assert_not_impl!(Text<0, 3>: fmt::Write);
    }
}

mod deref {
    use super::*;

    #[test]
    fn read_only_methods_of_a_vector_are_available() {
        let bytes = bytes::<1, 3>(&[1, 2, 3]);
        assert!(bytes.contains(&2));
        assert_eq!(bytes.binary_search(&3), Ok(2));
        assert_eq!(bytes.iter().max(), Some(&3));
    }

    #[test]
    fn read_only_methods_of_a_map_are_available() {
        let map = letters::<1, 3>();
        assert_eq!(map.get(&1), Some(&'a'));
        assert!(map.contains_key(&2));
        assert_eq!(map.first_key_value(), Some((&1, &'a')));
    }

    #[test]
    fn read_only_methods_of_a_string_are_available() {
        let text: Text<1, 8> = "confined".parse().unwrap();
        assert!(text.starts_with("con"));
        assert_eq!(text.as_str(), "confined");
    }
}

mod slices {
    use std::borrow::{Borrow, BorrowMut};
    use std::collections::HashSet;
    use std::hash::{BuildHasher, RandomState};

    use super::*;

    #[test]
    fn items_are_referenced_as_a_slice() {
        let bytes = bytes::<1, 3>(&[1, 2, 3]);
        let slice: &[u8] = bytes.as_ref();
        assert_eq!(slice, [1, 2, 3]);
    }

    #[test]
    fn items_are_changed_through_a_mutable_slice() {
        let mut bytes = bytes::<3, 3>(&[3, 1, 2]);
        bytes.as_mut().sort_unstable();
        assert_eq!(bytes.as_slice(), [1, 2, 3]);
    }

    #[test]
    fn items_are_borrowed_as_a_slice() {
        let bytes = bytes::<1, 3>(&[1, 2, 3]);
        let slice: &[u8] = bytes.borrow();
        assert_eq!(slice, [1, 2, 3]);
    }

    #[test]
    fn items_are_changed_through_a_mutable_borrow() {
        let mut bytes = bytes::<3, 3>(&[1, 2, 3]);
        let slice: &mut [u8] = bytes.borrow_mut();
        slice.reverse();
        assert_eq!(bytes.as_slice(), [3, 2, 1]);
    }

    #[test]
    fn borrowed_slice_hashes_like_the_collection() {
        let bytes = bytes::<1, 3>(&[1, 2, 3]);
        let state = RandomState::new();
        let slice: &[u8] = bytes.borrow();
        assert_eq!(state.hash_one(&bytes), state.hash_one(slice));
    }

    #[test]
    fn collection_in_a_set_is_found_by_a_slice() {
        let set = HashSet::from([bytes::<1, 3>(&[1, 2])]);
        assert!(set.contains(&[1u8, 2][..]));
        assert!(!set.contains(&[2u8, 1][..]));
    }
}

mod iteration {
    use super::*;

    #[test]
    fn items_are_iterated_by_value() {
        let items = bytes::<1, 3>(&[1, 2, 3]).into_iter().collect::<Vec<u8>>();
        assert_eq!(items, [1, 2, 3]);
    }

    #[test]
    fn items_are_iterated_by_reference() {
        let bytes = bytes::<1, 3>(&[1, 2, 3]);
        let mut sum = 0;
        for item in &bytes {
            sum += *item;
        }
        assert_eq!(sum, 6);
        assert_eq!(bytes.len(), 3);
    }

    #[test]
    fn items_are_changed_by_a_mutable_iteration() {
        let mut bytes = bytes::<3, 3>(&[1, 2, 3]);
        for item in &mut bytes {
            *item += 10;
        }
        assert_eq!(bytes.as_slice(), [11, 12, 13]);
    }

    #[test]
    fn entries_of_a_map_are_iterated_by_value() {
        let entries = letters::<1, 3>().into_iter().collect::<Vec<_>>();
        assert_eq!(entries, [(1, 'a'), (2, 'b')]);
    }

    #[test]
    fn entries_of_a_map_are_iterated_by_reference() {
        let map = letters::<1, 3>();
        let mut keys = vec![];
        for (key, _) in &map {
            keys.push(*key);
        }
        assert_eq!(keys, [1, 2]);
        assert_eq!(map.len(), 2);
    }

    #[test]
    fn values_of_a_map_are_changed_by_a_mutable_iteration() {
        let mut map = letters::<2, 2>();
        for (_, value) in &mut map {
            *value = value.to_ascii_uppercase();
        }
        assert_eq!(map.values().collect::<String>(), "AB");
        assert_eq!(map.len(), 2);
    }
}

mod indexing {
    use super::*;

    #[test]
    fn item_is_read_by_its_position() {
        let bytes = bytes::<1, 3>(&[1, 2, 3]);
        assert_eq!(bytes[0], 1);
        assert_eq!(bytes[2], 3);
    }

    #[test]
    fn items_are_read_by_a_range() {
        let bytes = bytes::<1, 3>(&[1, 2, 3]);
        assert_eq!(bytes[1..], [2, 3]);
        assert_eq!(bytes[..], [1, 2, 3]);
    }

    #[test]
    fn item_is_changed_by_its_position() {
        let mut bytes = bytes::<3, 3>(&[1, 2, 3]);
        bytes[1] = 20;
        assert_eq!(bytes.as_slice(), [1, 20, 3]);
    }

    #[test]
    fn items_are_changed_by_a_range() {
        let mut bytes = bytes::<3, 3>(&[1, 2, 3]);
        bytes[..2].fill(0);
        assert_eq!(bytes.as_slice(), [0, 0, 3]);
    }

    #[test]
    #[should_panic(expected = "index out of bounds")]
    fn position_past_the_end_is_a_panic_of_the_collection() {
        let bytes = bytes::<1, 3>(&[1, 2, 3]);
        let _ = bytes[3];
    }

    #[test]
    fn value_of_a_map_is_read_by_its_key() {
        let map = letters::<1, 3>();
        assert_eq!(map[&2], 'b');
    }

    #[test]
    fn part_of_a_string_is_read_by_a_range_of_bytes() {
        let text: Text<1, 8> = "confined".parse().unwrap();
        assert_eq!(&text[..3], "con");
    }
}

mod display {
    use super::*;

    #[test]
    fn string_is_displayed_as_its_content() {
        let text: Text<1, 8> = "confined".parse().unwrap();
        assert_eq!(text.to_string(), "confined");
    }

    #[test]
    fn formatting_options_are_passed_to_the_content() {
        let text: Text<1, 8> = "ab".parse().unwrap();
        assert_eq!(format!("{text:>4}"), "  ab");
    }
}

mod derived {
    use std::hash::{BuildHasher, RandomState};

    use super::*;

    #[test]
    fn clone_is_an_independent_copy() {
        let original = bytes::<0, 3>(&[1, 2]);
        let mut clone = original.clone();
        clone.push(3).unwrap();
        assert_eq!(original.as_slice(), [1, 2]);
        assert_eq!(clone.as_slice(), [1, 2, 3]);
    }

    #[test]
    fn equal_collections_are_equal() {
        assert_eq!(bytes::<0, 3>(&[1, 2]), bytes::<0, 3>(&[1, 2]));
        assert_ne!(bytes::<0, 3>(&[1, 2]), bytes::<0, 3>(&[2, 1]));
        assert_ne!(bytes::<0, 3>(&[1, 2]), bytes::<0, 3>(&[1]));
    }

    #[test]
    fn collections_are_ordered_like_their_content() {
        assert!(bytes::<0, 3>(&[1, 2]) < bytes::<0, 3>(&[1, 3]));
        assert!(bytes::<0, 3>(&[1, 2]) < bytes::<0, 3>(&[1, 2, 0]));
        assert!(bytes::<0, 3>(&[2]) > bytes::<0, 3>(&[1, 9, 9]));
        assert_eq!(
            bytes::<0, 3>(&[1, 2]).cmp(&bytes::<0, 3>(&[1, 2])),
            std::cmp::Ordering::Equal
        );
    }

    #[test]
    fn collection_hashes_like_its_content() {
        let state = RandomState::new();
        let bytes = bytes::<0, 3>(&[1, 2]);
        assert_eq!(state.hash_one(&bytes), state.hash_one(vec![1u8, 2]));
    }

    #[test]
    fn collection_is_printed_for_debugging() {
        assert_eq!(format!("{:?}", bytes::<0, 3>(&[1, 2])), "Confined([1, 2])");
    }

    #[test]
    fn default_collection_is_empty() {
        assert!(Bytes::<0, 3>::default().is_empty());
        assert!(Text::<0, 3>::default().is_empty());
        assert!(Map::<0, 3>::default().is_empty());
    }
}

mod layout {
    use std::mem::size_of;

    use super::*;

    #[test]
    fn wrapper_has_the_size_of_the_collection() {
        assert_eq!(size_of::<Bytes<1, 3>>(), size_of::<Vec<u8>>());
        assert_eq!(size_of::<Text<1, 3>>(), size_of::<String>());
        assert_eq!(size_of::<Map<1, 3>>(), size_of::<BTreeMap<u8, char>>());
    }

    #[test]
    fn optional_wrapper_has_the_size_of_the_optional_collection() {
        assert_eq!(
            size_of::<Option<Bytes<1, 3>>>(),
            size_of::<Option<Vec<u8>>>()
        );
    }

    #[test]
    fn wrapper_is_sent_and_shared_between_threads_like_the_collection() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<Bytes<1, 3>>();
        assert_send_sync::<Text<1, 3>>();
        assert_send_sync::<Map<1, 3>>();
    }
}
