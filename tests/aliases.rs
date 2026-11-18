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

//! Type aliases of the confined collections (`src/aliases.rs`).
//!
//! An alias with a wrong bound would silently confine a collection to another
//! size, so each alias is compared with the type it has to stand for:
//!
//! - `definitions`: one test per family of aliases. A test fails to build if an
//!   alias of its family is not exactly the expected `Confined` type;
//! - `upper_bounds`: each prefix accepts a collection of its maximal length and
//!   nothing longer, and gives the length as the integer of its width;
//! - `lower_bounds`: the `NonEmpty` aliases refuse an empty collection.
//!
//! The upper bounds are reached with vectors of zero-sized items, which take
//! no memory whatever their length is.

#![cfg(feature = "alloc")]

use std::marker::PhantomData;

use confined::{Confined, Error};

/// Builds only if both parameters are the same type.
fn same_type<T>(_: PhantomData<T>, _: PhantomData<T>) {}

/// Fails to build unless the alias on the left is the type on the right.
macro_rules! assert_alias {
    ($alias:ty = $target:ty) => {
        same_type(PhantomData::<$alias>, PhantomData::<$target>)
    };
}

mod definitions {
    use std::alloc::System;
    use std::collections::{BTreeMap, BTreeSet, BinaryHeap, LinkedList, VecDeque};

    use confined::{
        ConfinedBlob, ConfinedDeque, ConfinedHeap, ConfinedList, ConfinedOrdMap, ConfinedOrdSet,
        ConfinedString, ConfinedVec, LargeBlob, LargeDeque, LargeHeap, LargeList, LargeOrdMap,
        LargeOrdSet, LargeString, LargeVec, MediumBlob, MediumDeque, MediumHeap, MediumList,
        MediumOrdMap, MediumOrdSet, MediumString, MediumVec, NonEmptyBlob, NonEmptyDeque,
        NonEmptyHeap, NonEmptyList, NonEmptyOrdMap, NonEmptyOrdSet, NonEmptyString, NonEmptyVec,
        SmallBlob, SmallDeque, SmallHeap, SmallList, SmallOrdMap, SmallOrdSet, SmallString,
        SmallVec, TinyBlob, TinyDeque, TinyHeap, TinyList, TinyOrdMap, TinyOrdSet, TinyString,
        TinyVec, U24MAX, U32MAX, U64MAX,
    };

    use super::*;

    #[test]
    fn string_aliases() {
        assert_alias!(TinyString = Confined<String, 0, 255>);
        assert_alias!(SmallString = Confined<String, 0, 65_535>);
        assert_alias!(MediumString = Confined<String, 0, U24MAX>);
        assert_alias!(LargeString = Confined<String, 0, U32MAX>);
        assert_alias!(ConfinedString = Confined<String, 0, U64MAX>);
        assert_alias!(ConfinedString<2> = Confined<String, 2, U64MAX>);
        assert_alias!(ConfinedString<2, 8> = Confined<String, 2, 8>);
        assert_alias!(NonEmptyString = Confined<String, 1, U64MAX>);
        assert_alias!(NonEmptyString<8> = Confined<String, 1, 8>);
    }

    #[test]
    fn blob_aliases() {
        assert_alias!(TinyBlob = Confined<Vec<u8>, 0, 255>);
        assert_alias!(SmallBlob = Confined<Vec<u8>, 0, 65_535>);
        assert_alias!(MediumBlob = Confined<Vec<u8>, 0, U24MAX>);
        assert_alias!(LargeBlob = Confined<Vec<u8>, 0, U32MAX>);
        assert_alias!(ConfinedBlob = Confined<Vec<u8>, 0, U64MAX>);
        assert_alias!(ConfinedBlob<2> = Confined<Vec<u8>, 2, U64MAX>);
        assert_alias!(ConfinedBlob<2, 8> = Confined<Vec<u8>, 2, 8>);
        assert_alias!(NonEmptyBlob = Confined<Vec<u8>, 1, U64MAX>);
        assert_alias!(NonEmptyBlob<8> = Confined<Vec<u8>, 1, 8>);
    }

    #[test]
    fn blob_aliases_with_an_allocator() {
        assert_alias!(TinyBlob<System> = Confined<Vec<u8, System>, 0, 255>);
        assert_alias!(SmallBlob<System> = Confined<Vec<u8, System>, 0, 65_535>);
        assert_alias!(MediumBlob<System> = Confined<Vec<u8, System>, 0, U24MAX>);
        assert_alias!(LargeBlob<System> = Confined<Vec<u8, System>, 0, U32MAX>);
        assert_alias!(ConfinedBlob<2, 8, System> = Confined<Vec<u8, System>, 2, 8>);
        assert_alias!(NonEmptyBlob<8, System> = Confined<Vec<u8, System>, 1, 8>);
    }

    #[test]
    fn vec_aliases() {
        assert_alias!(TinyVec<u16> = Confined<Vec<u16>, 0, 255>);
        assert_alias!(SmallVec<u16> = Confined<Vec<u16>, 0, 65_535>);
        assert_alias!(MediumVec<u16> = Confined<Vec<u16>, 0, U24MAX>);
        assert_alias!(LargeVec<u16> = Confined<Vec<u16>, 0, U32MAX>);
        assert_alias!(ConfinedVec<u16> = Confined<Vec<u16>, 0, U64MAX>);
        assert_alias!(ConfinedVec<u16, 2> = Confined<Vec<u16>, 2, U64MAX>);
        assert_alias!(ConfinedVec<u16, 2, 8> = Confined<Vec<u16>, 2, 8>);
        assert_alias!(NonEmptyVec<u16> = Confined<Vec<u16>, 1, U64MAX>);
        assert_alias!(NonEmptyVec<u16, 8> = Confined<Vec<u16>, 1, 8>);
    }

    #[test]
    fn vec_aliases_with_an_allocator() {
        assert_alias!(TinyVec<u16, System> = Confined<Vec<u16, System>, 0, 255>);
        assert_alias!(SmallVec<u16, System> = Confined<Vec<u16, System>, 0, 65_535>);
        assert_alias!(MediumVec<u16, System> = Confined<Vec<u16, System>, 0, U24MAX>);
        assert_alias!(LargeVec<u16, System> = Confined<Vec<u16, System>, 0, U32MAX>);
        assert_alias!(ConfinedVec<u16, 2, 8, System> = Confined<Vec<u16, System>, 2, 8>);
        assert_alias!(NonEmptyVec<u16, 8, System> = Confined<Vec<u16, System>, 1, 8>);
    }

    #[test]
    fn deque_aliases() {
        assert_alias!(TinyDeque<u16> = Confined<VecDeque<u16>, 0, 255>);
        assert_alias!(SmallDeque<u16> = Confined<VecDeque<u16>, 0, 65_535>);
        assert_alias!(MediumDeque<u16> = Confined<VecDeque<u16>, 0, U24MAX>);
        assert_alias!(LargeDeque<u16> = Confined<VecDeque<u16>, 0, U32MAX>);
        assert_alias!(ConfinedDeque<u16> = Confined<VecDeque<u16>, 0, U64MAX>);
        assert_alias!(ConfinedDeque<u16, 2> = Confined<VecDeque<u16>, 2, U64MAX>);
        assert_alias!(ConfinedDeque<u16, 2, 8> = Confined<VecDeque<u16>, 2, 8>);
        assert_alias!(NonEmptyDeque<u16> = Confined<VecDeque<u16>, 1, U64MAX>);
        assert_alias!(NonEmptyDeque<u16, 8> = Confined<VecDeque<u16>, 1, 8>);
    }

    #[test]
    fn list_aliases() {
        assert_alias!(TinyList<u16> = Confined<LinkedList<u16>, 0, 255>);
        assert_alias!(SmallList<u16> = Confined<LinkedList<u16>, 0, 65_535>);
        assert_alias!(MediumList<u16> = Confined<LinkedList<u16>, 0, U24MAX>);
        assert_alias!(LargeList<u16> = Confined<LinkedList<u16>, 0, U32MAX>);
        assert_alias!(ConfinedList<u16> = Confined<LinkedList<u16>, 0, U64MAX>);
        assert_alias!(ConfinedList<u16, 2> = Confined<LinkedList<u16>, 2, U64MAX>);
        assert_alias!(ConfinedList<u16, 2, 8> = Confined<LinkedList<u16>, 2, 8>);
        assert_alias!(NonEmptyList<u16> = Confined<LinkedList<u16>, 1, U64MAX>);
        assert_alias!(NonEmptyList<u16, 8> = Confined<LinkedList<u16>, 1, 8>);
    }

    #[test]
    fn heap_aliases() {
        assert_alias!(TinyHeap<u16> = Confined<BinaryHeap<u16>, 0, 255>);
        assert_alias!(SmallHeap<u16> = Confined<BinaryHeap<u16>, 0, 65_535>);
        assert_alias!(MediumHeap<u16> = Confined<BinaryHeap<u16>, 0, U24MAX>);
        assert_alias!(LargeHeap<u16> = Confined<BinaryHeap<u16>, 0, U32MAX>);
        assert_alias!(ConfinedHeap<u16> = Confined<BinaryHeap<u16>, 0, U64MAX>);
        assert_alias!(ConfinedHeap<u16, 2> = Confined<BinaryHeap<u16>, 2, U64MAX>);
        assert_alias!(ConfinedHeap<u16, 2, 8> = Confined<BinaryHeap<u16>, 2, 8>);
        assert_alias!(NonEmptyHeap<u16> = Confined<BinaryHeap<u16>, 1, U64MAX>);
        assert_alias!(NonEmptyHeap<u16, 8> = Confined<BinaryHeap<u16>, 1, 8>);
    }

    #[test]
    fn ordered_set_aliases() {
        assert_alias!(TinyOrdSet<u16> = Confined<BTreeSet<u16>, 0, 255>);
        assert_alias!(SmallOrdSet<u16> = Confined<BTreeSet<u16>, 0, 65_535>);
        assert_alias!(MediumOrdSet<u16> = Confined<BTreeSet<u16>, 0, U24MAX>);
        assert_alias!(LargeOrdSet<u16> = Confined<BTreeSet<u16>, 0, U32MAX>);
        assert_alias!(ConfinedOrdSet<u16> = Confined<BTreeSet<u16>, 0, U64MAX>);
        assert_alias!(ConfinedOrdSet<u16, 2> = Confined<BTreeSet<u16>, 2, U64MAX>);
        assert_alias!(ConfinedOrdSet<u16, 2, 8> = Confined<BTreeSet<u16>, 2, 8>);
        assert_alias!(NonEmptyOrdSet<u16> = Confined<BTreeSet<u16>, 1, U64MAX>);
        assert_alias!(NonEmptyOrdSet<u16, 8> = Confined<BTreeSet<u16>, 1, 8>);
    }

    #[test]
    fn ordered_map_aliases() {
        assert_alias!(TinyOrdMap<u16, char> = Confined<BTreeMap<u16, char>, 0, 255>);
        assert_alias!(SmallOrdMap<u16, char> = Confined<BTreeMap<u16, char>, 0, 65_535>);
        assert_alias!(MediumOrdMap<u16, char> = Confined<BTreeMap<u16, char>, 0, U24MAX>);
        assert_alias!(LargeOrdMap<u16, char> = Confined<BTreeMap<u16, char>, 0, U32MAX>);
        assert_alias!(ConfinedOrdMap<u16, char> = Confined<BTreeMap<u16, char>, 0, U64MAX>);
        assert_alias!(ConfinedOrdMap<u16, char, 2> = Confined<BTreeMap<u16, char>, 2, U64MAX>);
        assert_alias!(ConfinedOrdMap<u16, char, 2, 8> = Confined<BTreeMap<u16, char>, 2, 8>);
        assert_alias!(NonEmptyOrdMap<u16, char> = Confined<BTreeMap<u16, char>, 1, U64MAX>);
        assert_alias!(NonEmptyOrdMap<u16, char, 8> = Confined<BTreeMap<u16, char>, 1, 8>);
    }

    #[cfg(feature = "ascii")]
    #[test]
    fn ascii_aliases() {
        use confined::ascii::AsciiString;
        use confined::{
            ConfinedAscii, LargeAscii, MediumAscii, NonEmptyAscii, SmallAscii, TinyAscii,
        };

        assert_alias!(TinyAscii = Confined<AsciiString, 0, 255>);
        assert_alias!(SmallAscii = Confined<AsciiString, 0, 65_535>);
        assert_alias!(MediumAscii = Confined<AsciiString, 0, U24MAX>);
        assert_alias!(LargeAscii = Confined<AsciiString, 0, U32MAX>);
        assert_alias!(ConfinedAscii = Confined<AsciiString, 0, U64MAX>);
        assert_alias!(ConfinedAscii<2> = Confined<AsciiString, 2, U64MAX>);
        assert_alias!(ConfinedAscii<2, 8> = Confined<AsciiString, 2, 8>);
        assert_alias!(NonEmptyAscii = Confined<AsciiString, 1, U64MAX>);
        assert_alias!(NonEmptyAscii<8> = Confined<AsciiString, 1, 8>);
    }

    #[cfg(feature = "std")]
    #[test]
    fn hash_set_aliases() {
        use std::collections::HashSet;

        use confined::{
            ConfinedHashSet, LargeHashSet, MediumHashSet, NonEmptyHashSet, SmallHashSet,
            TinyHashSet,
        };

        assert_alias!(TinyHashSet<u16> = Confined<HashSet<u16>, 0, 255>);
        assert_alias!(SmallHashSet<u16> = Confined<HashSet<u16>, 0, 65_535>);
        assert_alias!(MediumHashSet<u16> = Confined<HashSet<u16>, 0, U24MAX>);
        assert_alias!(LargeHashSet<u16> = Confined<HashSet<u16>, 0, U32MAX>);
        assert_alias!(ConfinedHashSet<u16> = Confined<HashSet<u16>, 0, U64MAX>);
        assert_alias!(ConfinedHashSet<u16, 2> = Confined<HashSet<u16>, 2, U64MAX>);
        assert_alias!(ConfinedHashSet<u16, 2, 8> = Confined<HashSet<u16>, 2, 8>);
        assert_alias!(NonEmptyHashSet<u16> = Confined<HashSet<u16>, 1, U64MAX>);
        assert_alias!(NonEmptyHashSet<u16, 8> = Confined<HashSet<u16>, 1, 8>);
    }

    #[cfg(feature = "std")]
    #[test]
    fn hash_map_aliases() {
        use std::collections::HashMap;

        use confined::{
            ConfinedHashMap, LargeHashMap, MediumHashMap, NonEmptyHashMap, SmallHashMap,
            TinyHashMap,
        };

        assert_alias!(TinyHashMap<u16, char> = Confined<HashMap<u16, char>, 0, 255>);
        assert_alias!(SmallHashMap<u16, char> = Confined<HashMap<u16, char>, 0, 65_535>);
        assert_alias!(MediumHashMap<u16, char> = Confined<HashMap<u16, char>, 0, U24MAX>);
        assert_alias!(LargeHashMap<u16, char> = Confined<HashMap<u16, char>, 0, U32MAX>);
        assert_alias!(ConfinedHashMap<u16, char> = Confined<HashMap<u16, char>, 0, U64MAX>);
        assert_alias!(ConfinedHashMap<u16, char, 2> = Confined<HashMap<u16, char>, 2, U64MAX>);
        assert_alias!(ConfinedHashMap<u16, char, 2, 8> = Confined<HashMap<u16, char>, 2, 8>);
        assert_alias!(NonEmptyHashMap<u16, char> = Confined<HashMap<u16, char>, 1, U64MAX>);
        assert_alias!(NonEmptyHashMap<u16, char, 8> = Confined<HashMap<u16, char>, 1, 8>);
    }

    #[cfg(all(feature = "indexmap", feature = "std"))]
    #[test]
    fn index_set_aliases() {
        use confined::{
            ConfinedIndexSet, LargeIndexSet, MediumIndexSet, NonEmptyIndexSet, SmallIndexSet,
            TinyIndexSet,
        };
        use indexmap::IndexSet;

        assert_alias!(TinyIndexSet<u16> = Confined<IndexSet<u16>, 0, 255>);
        assert_alias!(SmallIndexSet<u16> = Confined<IndexSet<u16>, 0, 65_535>);
        assert_alias!(MediumIndexSet<u16> = Confined<IndexSet<u16>, 0, U24MAX>);
        assert_alias!(LargeIndexSet<u16> = Confined<IndexSet<u16>, 0, U32MAX>);
        assert_alias!(ConfinedIndexSet<u16> = Confined<IndexSet<u16>, 0, U64MAX>);
        assert_alias!(ConfinedIndexSet<u16, 2> = Confined<IndexSet<u16>, 2, U64MAX>);
        assert_alias!(ConfinedIndexSet<u16, 2, 8> = Confined<IndexSet<u16>, 2, 8>);
        assert_alias!(NonEmptyIndexSet<u16> = Confined<IndexSet<u16>, 1, U64MAX>);
        assert_alias!(NonEmptyIndexSet<u16, 8> = Confined<IndexSet<u16>, 1, 8>);
    }

    #[cfg(all(feature = "indexmap", feature = "std"))]
    #[test]
    fn index_map_aliases() {
        use confined::{
            ConfinedIndexMap, LargeIndexMap, MediumIndexMap, NonEmptyIndexMap, SmallIndexMap,
            TinyIndexMap,
        };
        use indexmap::IndexMap;

        assert_alias!(TinyIndexMap<u16, char> = Confined<IndexMap<u16, char>, 0, 255>);
        assert_alias!(SmallIndexMap<u16, char> = Confined<IndexMap<u16, char>, 0, 65_535>);
        assert_alias!(MediumIndexMap<u16, char> = Confined<IndexMap<u16, char>, 0, U24MAX>);
        assert_alias!(LargeIndexMap<u16, char> = Confined<IndexMap<u16, char>, 0, U32MAX>);
        assert_alias!(ConfinedIndexMap<u16, char> = Confined<IndexMap<u16, char>, 0, U64MAX>);
        assert_alias!(ConfinedIndexMap<u16, char, 2> = Confined<IndexMap<u16, char>, 2, U64MAX>);
        assert_alias!(ConfinedIndexMap<u16, char, 2, 8> = Confined<IndexMap<u16, char>, 2, 8>);
        assert_alias!(NonEmptyIndexMap<u16, char> = Confined<IndexMap<u16, char>, 1, U64MAX>);
        assert_alias!(NonEmptyIndexMap<u16, char, 8> = Confined<IndexMap<u16, char>, 1, 8>);
    }

    #[cfg(feature = "hashbrown")]
    #[test]
    fn hashbrown_set_aliases() {
        use std::hash::RandomState;

        use confined::hashbrown::HashSet;
        use confined::{ConfinedHashbrownSet, SmallHashbrownSet, TinyHashbrownSet};

        assert_alias!(TinyHashbrownSet<u16> = Confined<HashSet<u16>, 0, 255>);
        assert_alias!(SmallHashbrownSet<u16> = Confined<HashSet<u16>, 0, 65_535>);
        assert_alias!(ConfinedHashbrownSet<u16> = Confined<HashSet<u16>, 0, U64MAX>);
        assert_alias!(ConfinedHashbrownSet<u16, 2> = Confined<HashSet<u16>, 2, U64MAX>);
        assert_alias!(ConfinedHashbrownSet<u16, 2, 8> = Confined<HashSet<u16>, 2, 8>);

        // The hasher and the allocator are the last parameters.
        type Custom = HashSet<u16, RandomState, System>;
        assert_alias!(TinyHashbrownSet<u16, RandomState, System> = Confined<Custom, 0, 255>);
        assert_alias!(SmallHashbrownSet<u16, RandomState, System> = Confined<Custom, 0, 65_535>);
        assert_alias!(ConfinedHashbrownSet<u16, 2, 8, RandomState, System> = Confined<Custom, 2, 8>);
    }

    #[cfg(feature = "hashbrown")]
    #[test]
    fn hashbrown_map_aliases() {
        use std::hash::RandomState;

        use confined::hashbrown::HashMap;
        use confined::{ConfinedHashbrownMap, SmallHashbrownMap, TinyHashbrownMap};

        assert_alias!(TinyHashbrownMap<u16, char> = Confined<HashMap<u16, char>, 0, 255>);
        assert_alias!(SmallHashbrownMap<u16, char> = Confined<HashMap<u16, char>, 0, 65_535>);
        assert_alias!(ConfinedHashbrownMap<u16, char> = Confined<HashMap<u16, char>, 0, U64MAX>);
        assert_alias!(ConfinedHashbrownMap<u16, char, 2> = Confined<HashMap<u16, char>, 2, U64MAX>);
        assert_alias!(ConfinedHashbrownMap<u16, char, 2, 8> = Confined<HashMap<u16, char>, 2, 8>);

        // The hasher and the allocator are the last parameters.
        type Custom = HashMap<u16, char, RandomState, System>;
        assert_alias!(TinyHashbrownMap<u16, char, RandomState, System> = Confined<Custom, 0, 255>);
        assert_alias!(
            SmallHashbrownMap<u16, char, RandomState, System> = Confined<Custom, 0, 65_535>
        );
        assert_alias!(
            ConfinedHashbrownMap<u16, char, 2, 8, RandomState, System> = Confined<Custom, 2, 8>
        );
    }
}

mod upper_bounds {
    use amplify_num::u24;
    use confined::{ConfinedVec, LargeVec, MediumVec, SmallVec, TinyVec, U24MAX, U32MAX, U64MAX};

    use super::*;

    /// A vector of `N` zero-sized items, which is built without a loop and
    /// takes no memory.
    fn units<const N: usize>() -> Vec<()> {
        Vec::from([(); N])
    }

    #[test]
    fn tiny_collection_holds_up_to_255_items() {
        let mut tiny = TinyVec::<()>::try_from(units::<255>()).unwrap();
        assert_eq!(tiny.len_u8(), u8::MAX);
        assert_eq!(
            tiny.push(()),
            Err(Error::Oversize {
                len: 255,
                max_len: 255
            })
        );
        assert_eq!(
            TinyVec::<()>::try_from(units::<256>()).err(),
            Some(Error::Oversize {
                len: 256,
                max_len: 255
            })
        );
    }

    #[test]
    fn small_collection_holds_up_to_65_535_items() {
        let mut small = SmallVec::<()>::try_from(units::<65_535>()).unwrap();
        assert_eq!(small.len_u16(), u16::MAX);
        assert_eq!(
            small.push(()),
            Err(Error::Oversize {
                len: 65_535,
                max_len: 65_535
            })
        );
        assert_eq!(
            SmallVec::<()>::try_from(units::<65_536>()).err(),
            Some(Error::Oversize {
                len: 65_536,
                max_len: 65_535
            })
        );
    }

    #[test]
    fn medium_collection_holds_up_to_16_777_215_items() {
        let mut medium = MediumVec::<()>::try_from(units::<U24MAX>()).unwrap();
        assert_eq!(medium.len_u24(), u24::MAX);
        assert_eq!(
            medium.push(()),
            Err(Error::Oversize {
                len: 16_777_215,
                max_len: 16_777_215
            })
        );
        assert_eq!(
            MediumVec::<()>::try_from(units::<16_777_216>()).err(),
            Some(Error::Oversize {
                len: 16_777_216,
                max_len: 16_777_215
            })
        );
    }

    #[test]
    fn large_collection_holds_up_to_the_largest_32_bit_number_of_items() {
        let mut large = LargeVec::<()>::try_from(units::<U32MAX>()).unwrap();
        assert_eq!(usize::try_from(large.len_u32()), Ok(U32MAX));
        assert_eq!(
            large.push(()),
            Err(Error::Oversize {
                len: U32MAX,
                max_len: U32MAX
            })
        );
    }

    #[cfg(target_pointer_width = "64")]
    #[test]
    fn large_collection_refuses_one_item_more() {
        assert_eq!(
            LargeVec::<()>::try_from(units::<4_294_967_296>()).err(),
            Some(Error::Oversize {
                len: 4_294_967_296,
                max_len: 4_294_967_295
            })
        );
    }

    #[test]
    fn collection_without_explicit_bounds_holds_any_number_of_items() {
        let mut any = ConfinedVec::<()>::try_from(units::<U64MAX>()).unwrap();
        assert_eq!(any.len(), usize::MAX);
        assert_eq!(
            any.push(()),
            Err(Error::Oversize {
                len: usize::MAX,
                max_len: usize::MAX
            })
        );
        assert!(
            ConfinedVec::<()>::try_from(units::<0>())
                .unwrap()
                .is_empty()
        );
    }

    #[test]
    fn wider_prefix_accepts_what_a_narrower_one_refuses() {
        let items = units::<256>;
        assert!(TinyVec::<()>::try_from(items()).is_err());
        assert_eq!(SmallVec::<()>::try_from(items()).unwrap().len(), 256);
        assert_eq!(MediumVec::<()>::try_from(items()).unwrap().len(), 256);
        assert_eq!(LargeVec::<()>::try_from(items()).unwrap().len(), 256);
        assert_eq!(ConfinedVec::<()>::try_from(items()).unwrap().len(), 256);
    }

    #[test]
    fn narrower_collection_is_widened_to_each_wider_prefix() {
        let tiny = TinyVec::<()>::try_from(units::<255>()).unwrap();
        let small: SmallVec<()> = tiny.widen();
        let medium: MediumVec<()> = small.widen();
        let large: LargeVec<()> = medium.widen();
        let any: ConfinedVec<()> = large.widen();
        assert_eq!(any.len(), 255);
    }
}

mod lower_bounds {
    use std::collections::{BTreeMap, BTreeSet, BinaryHeap, LinkedList, VecDeque};

    use confined::{
        NonEmptyBlob, NonEmptyDeque, NonEmptyHeap, NonEmptyList, NonEmptyOrdMap, NonEmptyOrdSet,
        NonEmptyString, NonEmptyVec,
    };

    use super::*;

    const EMPTY: Error = Error::Undersize { len: 0, min_len: 1 };

    #[test]
    fn string_can_not_be_empty() {
        assert_eq!(NonEmptyString::<8>::try_from(String::new()), Err(EMPTY));
        assert_eq!("".parse::<NonEmptyString<8>>(), Err(EMPTY));
        assert_eq!("a".parse::<NonEmptyString<8>>().unwrap().len(), 1);
    }

    #[test]
    fn blob_can_not_be_empty() {
        assert_eq!(NonEmptyBlob::<8>::try_from(vec![]), Err(EMPTY));
        assert_eq!(NonEmptyBlob::<8>::try_from(vec![1]).unwrap().len(), 1);
    }

    #[test]
    fn vec_can_not_be_empty() {
        assert_eq!(NonEmptyVec::<u16, 8>::try_from(vec![]), Err(EMPTY));
        assert_eq!(NonEmptyVec::<u16, 8>::try_from_iter([]), Err(EMPTY));
        assert_eq!(*NonEmptyVec::<u16, 8>::with(7).first(), 7);
    }

    #[test]
    fn deque_can_not_be_empty() {
        assert_eq!(
            NonEmptyDeque::<u16, 8>::try_from(VecDeque::new()),
            Err(EMPTY)
        );
        assert_eq!(*NonEmptyDeque::<u16, 8>::with(7).last(), 7);
    }

    #[test]
    fn list_can_not_be_empty() {
        assert_eq!(
            NonEmptyList::<u16, 8>::try_from(LinkedList::new()),
            Err(EMPTY)
        );
        assert_eq!(NonEmptyList::<u16, 8>::with(7).front(), Some(&7));
    }

    #[test]
    fn heap_can_not_be_empty() {
        assert_eq!(
            NonEmptyHeap::<u16, 8>::try_from(BinaryHeap::new()).err(),
            Some(EMPTY)
        );
        assert_eq!(NonEmptyHeap::<u16, 8>::with(7).peek(), Some(&7));
    }

    #[test]
    fn ordered_set_can_not_be_empty() {
        assert_eq!(
            NonEmptyOrdSet::<u16, 8>::try_from(BTreeSet::new()),
            Err(EMPTY)
        );
        let mut set = NonEmptyOrdSet::<u16, 8>::with(7);
        assert_eq!(set.remove(&7), Err(Error::Undersize { len: 1, min_len: 1 }));
    }

    #[test]
    fn ordered_map_can_not_be_empty() {
        assert_eq!(
            NonEmptyOrdMap::<u16, char, 8>::try_from(BTreeMap::new()),
            Err(EMPTY)
        );
        let mut map = NonEmptyOrdMap::<u16, char, 8>::with_key_value(7, 'a');
        assert_eq!(map.remove(&7), Err(Error::Undersize { len: 1, min_len: 1 }));
    }

    #[cfg(feature = "ascii")]
    #[test]
    fn ascii_string_can_not_be_empty() {
        use confined::ascii::AsciiString;
        use confined::{AsciiError, NonEmptyAscii};

        assert_eq!(NonEmptyAscii::<8>::try_from(AsciiString::new()), Err(EMPTY));
        assert_eq!(
            "".parse::<NonEmptyAscii<8>>(),
            Err(AsciiError::Confinement(EMPTY))
        );
    }

    #[cfg(feature = "std")]
    #[test]
    fn hash_collections_can_not_be_empty() {
        use std::collections::{HashMap, HashSet};

        use confined::{NonEmptyHashMap, NonEmptyHashSet};

        assert_eq!(
            NonEmptyHashSet::<u16, 8>::try_from(HashSet::new()),
            Err(EMPTY)
        );
        assert_eq!(
            NonEmptyHashMap::<u16, char, 8>::try_from(HashMap::new()),
            Err(EMPTY)
        );
    }

    #[cfg(all(feature = "indexmap", feature = "std"))]
    #[test]
    fn index_collections_can_not_be_empty() {
        use confined::{NonEmptyIndexMap, NonEmptyIndexSet};
        use indexmap::{IndexMap, IndexSet};

        assert_eq!(
            NonEmptyIndexSet::<u16, 8>::try_from(IndexSet::new()),
            Err(EMPTY)
        );
        assert_eq!(
            NonEmptyIndexMap::<u16, char, 8>::try_from(IndexMap::new()),
            Err(EMPTY)
        );
    }
}
