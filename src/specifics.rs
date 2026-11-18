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

use core::ops::{Index, IndexMut};

use amplify_num::u24;
use collections::{Collection, CollectionAlloc, CollectionBase, CollectionCapacity, CollectionMap};

use super::base::retain_unwind_safe;
use super::{Confined, Error, ItemLen, U8MAX, U16MAX, U24MAX, U32MAX, ZERO};

impl<C, const MAX_LEN: usize> Confined<C, ZERO, MAX_LEN>
where
    C: ItemLen,
{
    /// Removes all elements from the confined collection.
    ///
    /// ```
    /// # #[cfg(feature = "alloc")]
    /// # {
    /// # use confined::Confined;
    /// let mut vec = Confined::<Vec<u8>, 0, 3>::try_from(vec![1, 2]).unwrap();
    /// vec.clear();
    /// assert!(vec.is_empty());
    /// # }
    /// ```
    ///
    /// A collection with a lower bound can't be emptied, so it has no such
    /// method and the call fails to compile:
    ///
    /// ```compile_fail,E0596
    /// # use confined::Confined;
    /// let mut vec = Confined::<Vec<u8>, 1, 3>::try_from(vec![1, 2]).unwrap();
    /// vec.clear();
    /// assert!(vec.is_empty());
    /// ```
    pub fn clear(&mut self) {
        CollectionBase::clear(&mut self.0)
    }
}

impl<C, const MAX_LEN: usize> Confined<C, ZERO, MAX_LEN>
where
    C: ItemLen + Default,
{
    /// Constructs a new confinement containing no elements.
    ///
    /// ```
    /// # #[cfg(feature = "alloc")]
    /// # {
    /// # use confined::Confined;
    /// let vec = Confined::<Vec<u8>, 0, 3>::new();
    /// assert!(vec.is_empty());
    /// # }
    /// ```
    ///
    /// An empty collection doesn't fit a confinement with a lower bound, so
    /// such a type has no such function and the call fails to compile:
    ///
    /// ```compile_fail,E0599
    /// # use confined::Confined;
    /// let vec = Confined::<Vec<u8>, 1, 3>::new();
    /// assert!(vec.is_empty());
    /// ```
    pub fn new() -> Self {
        Self::default()
    }
}

impl<C, const MAX_LEN: usize> Confined<C, ZERO, MAX_LEN>
where
    C: Collection + ItemLen,
{
    /// Constructs a new confinement containing no elements, but with a
    /// pre-allocated storage for the `capacity` of elements.
    ///
    /// The storage is never allocated for more than `MAX_LEN` elements,
    /// whatever `capacity` is asked for.
    ///
    /// # Panics
    ///
    /// Like the wrapped collection, panics or aborts the process if the
    /// storage can't be allocated. Use [`Self::try_with_capacity`] to get an
    /// error instead.
    pub fn with_capacity(capacity: usize) -> Self {
        Self::from_checked(C::with_capacity(capacity.min(MAX_LEN)))
    }

    /// Fallibly constructs an empty confinement with reserved capacity.
    pub fn try_with_capacity(capacity: usize) -> Result<Self, Error>
    where
        C: CollectionCapacity,
    {
        let mut collection = C::default();
        let capacity = capacity.min(MAX_LEN);
        CollectionCapacity::try_reserve(&mut collection, capacity)
            .map_err(|_| Error::CapacityExceeded { len: 0 })?;
        Self::try_from(collection)
    }
}

impl<C, const MAX_LEN: usize> Confined<C, ZERO, MAX_LEN>
where
    C: CollectionAlloc + ItemLen,
{
    /// Constructs a new confinement containing no elements, using the
    /// supplied allocator.
    pub fn new_in(allocator: C::Allocator) -> Self {
        Self::from_checked(C::new_in(allocator))
    }

    /// Constructs a new confinement containing no elements, with the supplied
    /// allocator and pre-allocated storage for `min(capacity, MAX_LEN)`
    /// elements.
    pub fn with_capacity_in(capacity: usize, allocator: C::Allocator) -> Self {
        Self::from_checked(C::with_capacity_in(capacity.min(MAX_LEN), allocator))
    }

    /// Fallibly constructs an empty confinement with reserved capacity and the
    /// supplied allocator.
    pub fn try_with_capacity_in(capacity: usize, allocator: C::Allocator) -> Result<Self, Error>
    where
        C: CollectionCapacity,
    {
        let mut collection = C::new_in(allocator);
        let capacity = capacity.min(MAX_LEN);
        CollectionCapacity::try_reserve(&mut collection, capacity)
            .map_err(|_| Error::CapacityExceeded { len: 0 })?;
        Self::try_from(collection)
    }
}

impl<C, const MAX_LEN: usize> Default for Confined<C, ZERO, MAX_LEN>
where
    C: ItemLen + Default,
{
    fn default() -> Self {
        Self::from_checked(C::default())
    }
}

impl<C, const MIN_LEN: usize, const MAX_LEN: usize> Confined<C, MIN_LEN, MAX_LEN>
where
    C: Default + ItemLen,
{
    /// Constructs a confinement containing a single element.
    ///
    /// The bounds must allow exactly one item: `MIN_LEN <= 1 <= MAX_LEN`.
    /// The upper bound must also leave room for an item of any size. For most
    /// collections it means `MAX_LEN >= 1`. The length of a
    /// `String` is measured in bytes, and a single
    /// `char` may take up to four of them, so a string needs `MAX_LEN >= 4`.
    ///
    /// A type which doesn't meet this requirement fails to build. The failure
    /// is reported by `cargo build` and `cargo test`, but not by
    /// `cargo check`.
    ///
    /// ```
    /// # #[cfg(feature = "alloc")]
    /// # {
    /// # use confined::Confined;
    /// let vec = Confined::<Vec<u8>, 1, 1>::with(7);
    /// assert_eq!(vec.as_slice(), &[7]);
    /// # }
    /// ```
    ///
    /// ```compile_fail,E0080
    /// # use confined::Confined;
    /// let vec = Confined::<Vec<u8>, 1, 0>::with(7);
    /// assert_eq!(vec.as_slice(), &[7]);
    /// ```
    ///
    /// ```
    /// # #[cfg(feature = "alloc")]
    /// # {
    /// # use confined::Confined;
    /// let vec = Confined::<Vec<u8>, 0, 1>::with(7);
    /// assert_eq!(vec.as_slice(), &[7]);
    /// # }
    /// ```
    ///
    /// ```compile_fail,E0080
    /// # use confined::Confined;
    /// let vec = Confined::<Vec<u8>, 2, 3>::with(7);
    /// assert_eq!(vec.as_slice(), &[7]);
    /// ```
    ///
    /// ```
    /// # #[cfg(feature = "alloc")]
    /// # {
    /// # use confined::Confined;
    /// let s = Confined::<String, 1, 4>::with('a');
    /// assert_eq!(s.as_str(), "a");
    /// # }
    /// ```
    ///
    /// ```compile_fail,E0080
    /// # use confined::Confined;
    /// let s = Confined::<String, 1, 2>::with('a');
    /// assert_eq!(s.as_str(), "a");
    /// ```
    ///
    /// # Panics
    ///
    /// Panics if the wrapped collection refuses the item: a fixed-capacity
    /// collection which has no storage, or a collection which fails to
    /// allocate it. The collections without a fallible allocation, such as
    /// trees and linked lists, abort the process in the latter case.
    pub fn with(elem: C::Item) -> Self {
        const {
            assert!(
                MIN_LEN <= 1 && 1 <= MAX_LEN,
                "the number of items is outside of the confinement bounds"
            )
        };
        const {
            assert!(
                C::MAX_ITEM_LEN <= MAX_LEN,
                "the upper bound of the confinement is less than the largest size of a single item"
            )
        };
        let mut c = C::default();
        assert!(
            ItemLen::try_push(&mut c, elem).is_ok(),
            "collection has no capacity"
        );
        Self::from_checked(c)
    }
}

impl<C, const MIN_LEN: usize, const MAX_LEN: usize> Confined<C, MIN_LEN, MAX_LEN>
where
    C: CollectionAlloc + ItemLen,
{
    /// Constructs a confinement containing a single element with the supplied
    /// allocator. Like [`Self::with`], the bounds must allow exactly one item
    /// and leave room for the largest item of the collection.
    ///
    /// ```
    /// # #[cfg(feature = "alloc")]
    /// # {
    /// # use confined::Confined;
    /// # use std::alloc::System;
    /// let values = Confined::<Vec<u8, System>, 1, 3>::with_in(7, System);
    /// assert_eq!(values.as_slice(), &[7]);
    /// # }
    /// ```
    ///
    /// ```compile_fail,E0080
    /// # use confined::Confined;
    /// # use std::alloc::System;
    /// let _ = Confined::<Vec<u8, System>, 2, 3>::with_in(7, System);
    /// ```
    ///
    /// ```compile_fail,E0080
    /// # use confined::Confined;
    /// # use std::alloc::System;
    /// let _ = Confined::<Vec<u8, System>, 0, 0>::with_in(7, System);
    /// ```
    pub fn with_in(elem: C::Item, allocator: <C as CollectionAlloc>::Allocator) -> Self {
        const {
            assert!(
                MIN_LEN <= 1 && 1 <= MAX_LEN,
                "the number of items is outside of the confinement bounds"
            )
        };
        const {
            assert!(
                C::MAX_ITEM_LEN <= MAX_LEN,
                "the upper bound of the confinement is less than the largest size of a single item"
            )
        };
        let mut c = C::new_in(allocator);
        assert!(
            ItemLen::try_push(&mut c, elem).is_ok(),
            "collection has no capacity"
        );
        Self::from_checked(c)
    }
}

impl<C, const MIN_LEN: usize, const MAX_LEN: usize> Confined<C, MIN_LEN, MAX_LEN>
where
    C: ItemLen + Index<usize, Output = C::Item>,
{
    /// Returns the first element.
    ///
    /// The confinement must guarantee at least one item (`MIN_LEN > 0`). This
    /// is checked by `cargo build` and `cargo test`, but not by `cargo check`.
    /// This method shadows the wrapped collection's `Option`-returning
    /// `first()` where it has one; use `as_slice().first()` or
    /// `as_unconfined().first()` when the optional result is needed.
    ///
    /// ```
    /// # #[cfg(feature = "alloc")]
    /// # {
    /// # use confined::Confined;
    /// let values = Confined::<Vec<u8>, 2, 4>::try_from(vec![1, 2]).unwrap();
    /// assert_eq!(*values.first(), 1);
    /// # }
    /// ```
    ///
    /// ```compile_fail,E0080
    /// # use confined::Confined;
    /// let values = Confined::<Vec<u8>, 0, 4>::new();
    /// let _ = values.first();
    /// ```
    pub fn first(&self) -> &C::Item {
        const { assert!(MIN_LEN > 0, "the confinement may be empty") };
        self.0.index(0)
    }

    /// Returns the last element.
    ///
    /// The confinement must guarantee at least one item (`MIN_LEN > 0`). This
    /// is checked by `cargo build` and `cargo test`, but not by `cargo check`.
    /// This method shadows the wrapped collection's `Option`-returning
    /// `last()` where it has one; use `as_slice().last()` or
    /// `as_unconfined().last()` when the optional result is needed.
    ///
    /// ```
    /// # #[cfg(feature = "alloc")]
    /// # {
    /// # use confined::Confined;
    /// let values = Confined::<Vec<u8>, 2, 4>::try_from(vec![1, 2]).unwrap();
    /// assert_eq!(*values.last(), 2);
    /// # }
    /// ```
    ///
    /// ```compile_fail,E0080
    /// # use confined::Confined;
    /// let values = Confined::<Vec<u8>, 0, 4>::new();
    /// let _ = values.last();
    /// ```
    pub fn last(&self) -> &C::Item {
        const { assert!(MIN_LEN > 0, "the confinement may be empty") };
        // The position is counted in items, which differs from the confined
        // length when an item adds more than one to it.
        self.0.index(self.0.item_count() - 1)
    }
}

impl<C, const MIN_LEN: usize, const MAX_LEN: usize> Confined<C, MIN_LEN, MAX_LEN>
where
    C: ItemLen + IndexMut<usize, Output = C::Item>,
{
    /// Returns a mutable reference to the first element.
    ///
    /// The confinement must guarantee at least one item (`MIN_LEN > 0`). This
    /// is checked by `cargo build` and `cargo test`, but not by `cargo check`.
    /// Where the wrapped collection exposes an `Option`-returning
    /// `first_mut()` through `AsMut`, use `as_mut().first_mut()` to get the
    /// optional result instead.
    ///
    /// ```
    /// # #[cfg(feature = "alloc")]
    /// # {
    /// # use confined::Confined;
    /// let mut values = Confined::<Vec<u8>, 2, 4>::try_from(vec![1, 2]).unwrap();
    /// *values.first_mut() = 3;
    /// assert_eq!(values.as_slice(), &[3, 2]);
    /// # }
    /// ```
    ///
    /// ```compile_fail,E0080
    /// # use confined::Confined;
    /// let mut values = Confined::<Vec<u8>, 0, 4>::new();
    /// let _ = values.first_mut();
    /// ```
    pub fn first_mut(&mut self) -> &mut C::Item {
        const { assert!(MIN_LEN > 0, "the confinement may be empty") };
        self.0.index_mut(0)
    }

    /// Returns a mutable reference to the last element.
    ///
    /// The confinement must guarantee at least one item (`MIN_LEN > 0`). This
    /// is checked by `cargo build` and `cargo test`, but not by `cargo check`.
    /// Where the wrapped collection exposes an `Option`-returning
    /// `last_mut()` through `AsMut`, use `as_mut().last_mut()` to get the
    /// optional result instead.
    ///
    /// ```
    /// # #[cfg(feature = "alloc")]
    /// # {
    /// # use confined::Confined;
    /// let mut values = Confined::<Vec<u8>, 2, 4>::try_from(vec![1, 2]).unwrap();
    /// *values.last_mut() = 3;
    /// assert_eq!(values.as_slice(), &[1, 3]);
    /// # }
    /// ```
    ///
    /// ```compile_fail,E0080
    /// # use confined::Confined;
    /// let mut values = Confined::<Vec<u8>, 0, 4>::new();
    /// let _ = values.last_mut();
    /// ```
    pub fn last_mut(&mut self) -> &mut C::Item {
        const { assert!(MIN_LEN > 0, "the confinement may be empty") };
        let last = self.0.item_count() - 1;
        self.0.index_mut(last)
    }
}

impl<C: ItemLen, const MIN_LEN: usize, const MAX_LEN: usize> Confined<C, MIN_LEN, MAX_LEN> {
    /// Returns the number of elements in the confined collection as `u8`. The
    /// maximum length must not exceed `u8::MAX`. This is checked by `cargo
    /// build` and `cargo test`, but not by `cargo check`.
    ///
    /// ```
    /// # #[cfg(feature = "alloc")]
    /// # {
    /// # use confined::Confined;
    /// let values = Confined::<Vec<u8>, 0, 10>::try_from(vec![1, 2]).unwrap();
    /// assert_eq!(values.len_u8(), 2);
    /// # }
    /// ```
    ///
    /// ```compile_fail,E0080
    /// # use confined::Confined;
    /// let values = Confined::<Vec<u8>, 0, 256>::new();
    /// let _ = values.len_u8();
    /// ```
    pub fn len_u8(&self) -> u8 {
        const {
            assert!(
                MAX_LEN <= U8MAX,
                "the upper bound of the confinement exceeds the integer limit"
            )
        };
        self.len().try_into().expect("confinement broken")
    }

    /// Returns the number of elements in the confined collection as `u16`. The
    /// maximum length must not exceed `u16::MAX`. This is checked by `cargo
    /// build` and `cargo test`, but not by `cargo check`.
    ///
    /// ```compile_fail,E0080
    /// # use confined::Confined;
    /// let values = Confined::<Vec<u8>, 0, 65_536>::new();
    /// let _ = values.len_u16();
    /// ```
    ///
    /// ```
    /// # #[cfg(feature = "alloc")]
    /// # {
    /// # use confined::Confined;
    /// let values = Confined::<Vec<u8>, 0, 10>::try_from(vec![1, 2]).unwrap();
    /// assert_eq!(values.len_u16(), 2);
    /// # }
    /// ```
    pub fn len_u16(&self) -> u16 {
        // On 16-bit targets, `U16MAX` equals `usize::MAX`.
        #[allow(clippy::absurd_extreme_comparisons)]
        const {
            assert!(
                MAX_LEN <= U16MAX,
                "the upper bound of the confinement exceeds the integer limit"
            )
        };
        self.len().try_into().expect("confinement broken")
    }

    /// Returns the number of elements in the confined collection as `u24`. The
    /// maximum length must not exceed `u24::MAX`. This is checked by `cargo
    /// build` and `cargo test`, but not by `cargo check`.
    ///
    /// ```compile_fail,E0080
    /// # use confined::Confined;
    /// let values = Confined::<Vec<u8>, 0, 16_777_216>::new();
    /// let _ = values.len_u24();
    /// ```
    ///
    /// ```
    /// # #[cfg(feature = "alloc")]
    /// # {
    /// # use confined::Confined;
    /// # use amplify_num::u24;
    /// let values = Confined::<Vec<u8>, 0, 10>::try_from(vec![1, 2]).unwrap();
    /// assert_eq!(values.len_u24(), u24::try_from(2u32).unwrap());
    /// # }
    /// ```
    pub fn len_u24(&self) -> u24 {
        // On 16-bit targets, `U24MAX` is capped at `usize::MAX`.
        #[allow(clippy::absurd_extreme_comparisons)]
        const {
            assert!(
                MAX_LEN <= U24MAX,
                "the upper bound of the confinement exceeds the integer limit"
            )
        };
        let len = u32::try_from(self.len()).expect("confinement broken");
        u24::try_from(len).expect("confinement broken")
    }

    /// Returns the number of elements in the confined collection as `u32`. The
    /// maximum length must not exceed `u32::MAX`. This is checked by `cargo
    /// build` and `cargo test`, but not by `cargo check`.
    ///
    /// ```compile_fail,E0080
    /// # use confined::Confined;
    /// let values = Confined::<Vec<u8>, 0, 4_294_967_296>::new();
    /// let _ = values.len_u32();
    /// ```
    ///
    /// ```
    /// # #[cfg(feature = "alloc")]
    /// # {
    /// # use confined::Confined;
    /// let values = Confined::<Vec<u8>, 0, 10>::try_from(vec![1, 2]).unwrap();
    /// assert_eq!(values.len_u32(), 2);
    /// # }
    /// ```
    pub fn len_u32(&self) -> u32 {
        // On 16-bit and 32-bit targets, `U32MAX` is capped at `usize::MAX`.
        #[allow(clippy::absurd_extreme_comparisons)]
        const {
            assert!(
                MAX_LEN <= U32MAX,
                "the upper bound of the confinement exceeds the integer limit"
            )
        };
        self.len().try_into().expect("confinement broken")
    }
}

impl<C: CollectionMap, const MAX_LEN: usize> Confined<C, ZERO, MAX_LEN> {
    /// Retains only the elements specified by the predicate.
    ///
    /// In other words, remove all pairs `(k, v)` for which `f(&k, &mut v)`
    /// returns `false`. The elements are visited in unsorted (and
    /// unspecified) order.
    ///
    /// ```
    /// # #[cfg(feature = "alloc")]
    /// # {
    /// # use std::collections::BTreeMap;
    /// # use confined::Confined;
    /// let mut map = Confined::<BTreeMap<u8, u8>, 0, 3>::try_from_iter([(1, 1), (2, 2)]).unwrap();
    /// map.retain(|key, _| *key > 1);
    /// assert_eq!(map.len(), 1);
    /// # }
    /// ```
    ///
    /// A map with a lower bound has no such method, so the call fails to
    /// compile. Such a map is filtered with [`Confined::try_retain`], which
    /// checks the bound:
    ///
    /// ```compile_fail,E0596
    /// # use std::collections::BTreeMap;
    /// # use confined::Confined;
    /// let mut map = Confined::<BTreeMap<u8, u8>, 1, 3>::try_from_iter([(1, 1), (2, 2)]).unwrap();
    /// map.retain(|key, _| *key > 1);
    /// assert_eq!(map.len(), 1);
    /// ```
    ///
    /// # Panics
    ///
    /// A panic of the predicate, or of a destructor of a removed element, is
    /// propagated. The elements removed before it stay removed, and the map
    /// remains usable.
    pub fn retain(&mut self, f: impl FnMut(&C::Key, &mut C::Value) -> bool) {
        retain_unwind_safe(&mut self.0, f)
    }
}

impl<C, const MIN_LEN: usize, const MAX_LEN: usize> Confined<C, MIN_LEN, MAX_LEN>
where
    C: CollectionMap + Default + ItemLen,
{
    /// Constructs a confinement with a collection made of a single required
    /// key-value pair.
    ///
    /// The bounds must allow exactly one pair: `MIN_LEN <= 1 <= MAX_LEN`. A
    /// type which doesn't meet this requirement fails to build. The failure
    /// is reported by `cargo build` and `cargo test`, but not by `cargo check`.
    ///
    /// ```
    /// # #[cfg(feature = "alloc")]
    /// # {
    /// # use std::collections::BTreeMap;
    /// # use confined::Confined;
    /// let map = Confined::<BTreeMap<u8, char>, 0, 1>::with_key_value(7, 'a');
    /// assert_eq!(map.len(), 1);
    /// # }
    /// ```
    ///
    /// ```compile_fail,E0080
    /// # use std::collections::BTreeMap;
    /// # use confined::Confined;
    /// let map = Confined::<BTreeMap<u8, char>, 2, 3>::with_key_value(7, 'a');
    /// assert_eq!(map.len(), 1);
    /// ```
    ///
    /// ```
    /// # #[cfg(feature = "alloc")]
    /// # {
    /// # use std::collections::BTreeMap;
    /// # use confined::Confined;
    /// let map = Confined::<BTreeMap<u8, char>, 1, 1>::with_key_value(7, 'a');
    /// assert_eq!(map.len(), 1);
    /// # }
    /// ```
    ///
    /// ```compile_fail,E0080
    /// # use std::collections::BTreeMap;
    /// # use confined::Confined;
    /// let map = Confined::<BTreeMap<u8, char>, 1, 0>::with_key_value(7, 'a');
    /// assert_eq!(map.len(), 1);
    /// ```
    pub fn with_key_value(key: C::Key, value: C::Value) -> Self {
        Self::with((key, value))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A collection which only reports a length. It reaches the limits of
    /// the integer types without allocating gigabytes, and lets the tests
    /// build a confinement which is already broken.
    #[derive(Default, Debug)]
    struct Reported(usize);

    impl CollectionBase for Reported {
        type Item = ();

        fn item_count(&self) -> usize {
            self.0
        }

        fn push(&mut self, (): ()) {
            self.0 += 1;
        }

        fn clear(&mut self) {
            self.0 = 0;
        }
    }

    impl ItemLen for Reported {}

    /// A collection which refuses every item.
    #[derive(Default, Debug)]
    struct Full;

    impl CollectionBase for Full {
        type Item = u8;

        fn item_count(&self) -> usize {
            0
        }

        fn push(&mut self, _: u8) {
            unreachable!("the collection is full")
        }

        fn clear(&mut self) {}
    }

    impl ItemLen for Full {
        fn try_push(&mut self, item: u8) -> Result<(), u8> {
            Err(item)
        }
    }

    mod len_u8 {
        use super::*;

        #[test]
        fn largest_length_is_returned() {
            let col = Confined::<Reported, 0, 255>(Reported(255));
            assert_eq!(col.len_u8(), u8::MAX);
        }

        #[test]
        fn zero_length_is_returned() {
            let col = Confined::<Reported, 0, 255>(Reported(0));
            assert_eq!(col.len_u8(), 0);
        }

        #[test]
        #[should_panic(expected = "confinement broken")]
        fn length_above_the_integer_is_never_truncated() {
            let col = Confined::<Reported, 0, 255>(Reported(256));
            col.len_u8();
        }
    }

    mod len_u16 {
        use super::*;

        #[test]
        fn largest_length_is_returned() {
            let col = Confined::<Reported, 0, 65_535>(Reported(65_535));
            assert_eq!(col.len_u16(), u16::MAX);
        }

        #[test]
        #[should_panic(expected = "confinement broken")]
        fn length_above_the_integer_is_never_truncated() {
            let col = Confined::<Reported, 0, 65_535>(Reported(65_536));
            col.len_u16();
        }
    }

    mod len_u24 {
        use super::*;

        #[test]
        fn largest_length_is_returned() {
            let col = Confined::<Reported, 0, U24MAX>(Reported(U24MAX));
            assert_eq!(col.len_u24(), u24::MAX);
        }

        #[test]
        #[should_panic(expected = "confinement broken")]
        fn length_above_the_integer_is_never_truncated() {
            let col = Confined::<Reported, 0, U24MAX>(Reported(U24MAX + 1));
            col.len_u24();
        }

        #[cfg(target_pointer_width = "64")]
        #[test]
        #[should_panic(expected = "confinement broken")]
        fn length_above_four_bytes_is_never_truncated() {
            // The low three bytes of the length are zero.
            let col = Confined::<Reported, 0, U24MAX>(Reported(1 << 32));
            col.len_u24();
        }
    }

    mod len_u32 {
        use super::*;

        #[test]
        fn largest_length_is_returned() {
            let col = Confined::<Reported, 0, U32MAX>(Reported(U32MAX));
            assert_eq!(usize::try_from(col.len_u32()), Ok(U32MAX));
        }

        #[cfg(target_pointer_width = "64")]
        #[test]
        #[should_panic(expected = "confinement broken")]
        fn length_above_the_integer_is_never_truncated() {
            let col = Confined::<Reported, 0, U32MAX>(Reported(U32MAX + 1));
            col.len_u32();
        }
    }

    mod with {
        use super::*;

        #[test]
        fn single_item_is_stored() {
            let col = Confined::<Reported, 1, 1>::with(());
            assert_eq!(col.len(), 1);
        }

        #[test]
        #[should_panic(expected = "collection has no capacity")]
        fn refused_item_is_a_panic() {
            let _ = Confined::<Full, 0, 1>::with(7);
        }
    }

    mod constructors {
        use super::*;

        #[test]
        fn new_is_empty() {
            assert!(Confined::<Reported, 0, 1>::new().is_empty());
        }

        #[test]
        fn default_is_empty() {
            assert!(Confined::<Reported, 0, 1>::default().is_empty());
        }

        #[test]
        fn clear_removes_everything() {
            let mut col = Confined::<Reported, 0, 3>(Reported(3));
            col.clear();
            assert!(col.is_empty());
        }
    }
}
