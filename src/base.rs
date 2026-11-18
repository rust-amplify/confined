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

use core::borrow::{Borrow, BorrowMut};
use core::fmt::{self, Display, Formatter};
use core::ops::{Deref, Index, IndexMut};

#[cfg(feature = "alloc")]
use collections::CollectionSet;
use collections::{CollectionAlloc, CollectionBase, CollectionEntry, CollectionMap, MapEntry};

use super::Error;

/// The confinement for the collection.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Confined<C, const MIN_LEN: usize, const MAX_LEN: usize>(pub(crate) C);

#[cfg(feature = "serde")]
impl<'de, C, const MIN_LEN: usize, const MAX_LEN: usize> serde::Deserialize<'de>
    for Confined<C, MIN_LEN, MAX_LEN>
where
    C: ItemLen + serde::Deserialize<'de>,
{
    /// Deserializes the collection and checks that it fits the confinement.
    /// The data which violate the confinement bounds are rejected with a
    /// custom error of the deserializer.
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        // The data are read in the same form as the derived `Serialize`
        // writes them: as a newtype struct named `Confined`.
        #[derive(serde::Deserialize)]
        #[serde(rename = "Confined")]
        struct Unchecked<C>(C);

        let Unchecked(col) = <Unchecked<C> as serde::Deserialize>::deserialize(deserializer)?;
        Self::try_from(col).map_err(serde::de::Error::custom)
    }
}

/// Defines the units used by confinement for a collection's length.
///
/// By default, confinement uses the collection's `item_count()` and each item
/// adds one. `String` uses its byte length instead.
///
/// [`Confined::push`], [`Confined::extend`], [`Confined::with`] and, for maps,
/// [`Confined::insert`], [`Confined::entry`] and [`Confined::try_retain`] rely
/// on this trait to keep a collection within its bounds. An implementation
/// must never report less than an item really adds, otherwise these methods
/// are not able to uphold the confinement.
///
/// The length which an item adds must not depend on anything which can be
/// changed while the item is in the collection: through a mutable reference
/// to the item or to the value of a map entry, or through a shared reference
/// to an item with interior mutability. Mutable references are handed out
/// without a check by [`Confined::iter_mut`], [`Confined::get_mut`],
/// [`Confined::values_mut`], the entries and the mutable indexing, because a
/// change of an item in place is assumed to keep the length.
///
/// A custom collection can override the default item growth when needed:
///
/// ```
/// # use confined::ItemLen;
/// # #[derive(Default)]
/// # struct Stack(Vec<u8>);
/// # impl FromIterator<u8> for Stack {
/// #     fn from_iter<I: IntoIterator<Item = u8>>(iter: I) -> Self {
/// #         Stack(iter.into_iter().collect())
/// #     }
/// # }
/// # impl Extend<u8> for Stack {
/// #     fn extend<I: IntoIterator<Item = u8>>(&mut self, iter: I) {
/// #         self.0.extend(iter)
/// #     }
/// # }
/// # impl collections::CollectionBase for Stack {
/// #     type Item = u8;
/// #     fn item_count(&self) -> usize { self.0.len() }
/// #     fn push(&mut self, elem: u8) { self.0.push(elem) }
/// #     fn clear(&mut self) { self.0.clear() }
/// # }
/// # impl collections::Collection for Stack {
/// #     fn with_capacity(capacity: usize) -> Self {
/// #         Stack(Vec::with_capacity(capacity))
/// #     }
/// # }
/// impl ItemLen for Stack {}
///
/// let mut stack = confined::Confined::<Stack, 0, 1>::try_from(Stack(vec![])).unwrap();
/// assert!(stack.push(1).is_ok());
/// assert!(stack.push(2).is_err());
/// ```
pub trait ItemLen: CollectionBase + Sized {
    /// The largest value which [`ItemLen::item_len`] can return.
    ///
    /// [`Confined::with`] requires an upper bound of at least this much.
    /// [`Confined::entry`] gives out a vacant entry only when this much room
    /// is left, and [`Confined::try_retain`] counts each removal as this much
    /// when the confinement has a lower bound, since neither of them knows
    /// the item.
    const MAX_ITEM_LEN: usize = 1;

    /// Returns the length used by confinement.
    fn confined_len(&self) -> usize {
        self.item_count()
    }

    /// Returns the largest number by which the length of the collection can
    /// grow when the `item` is added to it.
    fn item_len(_item: &Self::Item) -> usize {
        1
    }

    /// Returns how much the confined length grows when `item` is inserted.
    fn item_growth(&self, item: &Self::Item) -> usize {
        Self::item_len(item)
    }

    /// Pushes an item, returning it if the collection cannot hold it.
    fn try_push(&mut self, item: Self::Item) -> Result<(), Self::Item> {
        CollectionBase::push(self, item);
        Ok(())
    }

    /// Prepares storage for a vacant map entry. Fixed-capacity maps must
    /// return an error if another entry cannot fit.
    fn prepare_entry(&mut self) -> Result<(), Error> {
        Ok(())
    }
}

/// Runs the native `retain` of a map with a caller-supplied predicate, keeping
/// the map consistent if the predicate or a destructor of a removed entry
/// panics.
///
/// The maps backed by an index table (`indexmap::IndexMap` and
/// `heapless::IndexMap`) rebuild the table only at the end of a retention
/// pass. A panic in the middle of the pass leaves the table pointing to the
/// entries which were moved or removed: lookups then return wrong entries or
/// panic, and in `heapless` they read the storage out of bounds without a
/// check. A second pass which keeps every entry makes such a map rebuild its
/// table. The pass calls no code of the caller, so it doesn't panic by itself,
/// and it runs only when the first one is unwinding.
pub(crate) fn retain_unwind_safe<C: CollectionMap>(
    map: &mut C,
    f: impl FnMut(&C::Key, &mut C::Value) -> bool,
) {
    struct Repair<'a, C: CollectionMap> {
        map: &'a mut C,
        unwinding: bool,
    }

    impl<C: CollectionMap> Drop for Repair<'_, C> {
        fn drop(&mut self) {
            if self.unwinding {
                self.map.retain(|_, _| true);
            }
        }
    }

    let mut repair = Repair {
        map,
        unwinding: true,
    };
    repair.map.retain(f);
    repair.unwinding = false;
}

/// A view into a single entry of a confined map, which may either be vacant or
/// occupied. It is constructed with [`Confined::entry`].
///
/// Unlike the entries of the wrapped maps, it has no methods removing the
/// entry: a removal could take the map below the lower bound of the
/// confinement. Use the `remove` method of the confined map, which checks the
/// bound.
///
/// ```
/// # #[cfg(feature = "alloc")]
/// # {
/// # use std::collections::BTreeMap;
/// # use confined::{Confined, Error};
/// let mut map = Confined::<BTreeMap<u8, u8>, 1, 3>::with_key_value(1, 10);
/// *map.entry(1).unwrap().or_insert(0) += 1;
/// assert_eq!(map.remove(&1), Err(Error::Undersize { len: 1, min_len: 1 }));
/// # }
/// ```
///
/// ```compile_fail,E0599
/// # use std::collections::BTreeMap;
/// # use confined::{Confined, Error};
/// let mut map = Confined::<BTreeMap<u8, u8>, 1, 3>::with_key_value(1, 10);
/// map.entry(1).unwrap().remove();
/// assert_eq!(map.remove(&1), Err(Error::Undersize { len: 1, min_len: 1 }));
/// ```
///
/// The methods are provided for collections implementing [`CollectionEntry`].
pub struct Entry<'a, C: CollectionEntry + 'a>(pub(crate) C::Entry<'a>);

impl<'a, C: CollectionEntry + 'a> fmt::Debug for Entry<'a, C>
where
    C::Entry<'a>: fmt::Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Entry").field(&self.0).finish()
    }
}

impl<C, const MIN_LEN: usize, const MAX_LEN: usize> Deref for Confined<C, MIN_LEN, MAX_LEN> {
    type Target = C;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<C, const MIN_LEN: usize, const MAX_LEN: usize> AsRef<[C::Item]>
    for Confined<C, MIN_LEN, MAX_LEN>
where
    C: ItemLen + AsRef<[C::Item]>,
{
    fn as_ref(&self) -> &[C::Item] {
        self.0.as_ref()
    }
}

impl<C, const MIN_LEN: usize, const MAX_LEN: usize> AsMut<[C::Item]>
    for Confined<C, MIN_LEN, MAX_LEN>
where
    C: ItemLen + AsMut<[C::Item]>,
{
    fn as_mut(&mut self) -> &mut [C::Item] {
        self.0.as_mut()
    }
}

impl<C, const MIN_LEN: usize, const MAX_LEN: usize> Borrow<[C::Item]>
    for Confined<C, MIN_LEN, MAX_LEN>
where
    C: ItemLen + Borrow<[C::Item]>,
{
    fn borrow(&self) -> &[C::Item] {
        self.0.borrow()
    }
}

impl<C, const MIN_LEN: usize, const MAX_LEN: usize> BorrowMut<[C::Item]>
    for Confined<C, MIN_LEN, MAX_LEN>
where
    C: ItemLen + BorrowMut<[C::Item]>,
{
    fn borrow_mut(&mut self) -> &mut [C::Item] {
        self.0.borrow_mut()
    }
}

impl<C, const MIN_LEN: usize, const MAX_LEN: usize> IntoIterator for Confined<C, MIN_LEN, MAX_LEN>
where
    C: IntoIterator,
{
    type Item = <C as IntoIterator>::Item;
    type IntoIter = <C as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'c, C, const MIN_LEN: usize, const MAX_LEN: usize> IntoIterator
    for &'c Confined<C, MIN_LEN, MAX_LEN>
where
    &'c C: IntoIterator,
{
    type Item = <&'c C as IntoIterator>::Item;
    type IntoIter = <&'c C as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'c, C, const MIN_LEN: usize, const MAX_LEN: usize> IntoIterator
    for &'c mut Confined<C, MIN_LEN, MAX_LEN>
where
    &'c mut C: IntoIterator,
{
    type Item = <&'c mut C as IntoIterator>::Item;
    type IntoIter = <&'c mut C as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'c, C, const MIN_LEN: usize, const MAX_LEN: usize> Confined<C, MIN_LEN, MAX_LEN>
where
    C: ItemLen + 'c,
    &'c mut C: IntoIterator<Item = &'c mut C::Item>,
{
    /// Returns an iterator that allows modifying each value.
    ///
    /// The iterator yields all items from start to end.
    pub fn iter_mut(&'c mut self) -> <&'c mut C as IntoIterator>::IntoIter {
        let coll = &mut self.0;
        coll.into_iter()
    }
}

impl<C, const MIN_LEN: usize, const MAX_LEN: usize> Confined<C, MIN_LEN, MAX_LEN>
where
    C: CollectionMap,
{
    /// Returns an iterator that allows modifying each value for each key.
    pub fn values_mut(&mut self) -> impl Iterator<Item = &mut C::Value> {
        let coll = &mut self.0;
        coll.values_mut()
    }

    /// Returns an iterator that allows modifying each value for each key.
    pub fn keyed_values_mut(&mut self) -> impl Iterator<Item = (&C::Key, &mut C::Value)> {
        let coll = &mut self.0;
        coll.iter_mut()
    }
}

impl<C, I, const MIN_LEN: usize, const MAX_LEN: usize> Index<I> for Confined<C, MIN_LEN, MAX_LEN>
where
    C: Index<I>,
{
    type Output = C::Output;

    #[inline]
    fn index(&self, index: I) -> &Self::Output {
        self.0.index(index)
    }
}

impl<C, I, const MIN_LEN: usize, const MAX_LEN: usize> IndexMut<I> for Confined<C, MIN_LEN, MAX_LEN>
where
    C: IndexMut<I>,
{
    #[inline]
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        self.0.index_mut(index)
    }
}

impl<C, const MIN_LEN: usize, const MAX_LEN: usize> Display for Confined<C, MIN_LEN, MAX_LEN>
where
    C: Display,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl<C, const MIN_LEN: usize, const MAX_LEN: usize> Confined<C, MIN_LEN, MAX_LEN>
where
    C: ItemLen,
{
    pub(crate) fn does_fit_confinement(col: &C) -> Result<(), Error> {
        Self::check_len(col.confined_len())
    }

    pub(crate) fn check_len(len: usize) -> Result<(), Error> {
        const { assert!(MIN_LEN <= MAX_LEN, "invalid confinement bounds") };
        (len >= MIN_LEN).ok_or(Error::Undersize {
            len,
            min_len: MIN_LEN,
        })?;
        (len <= MAX_LEN).ok_or(Error::Oversize {
            len,
            max_len: MAX_LEN,
        })?;
        Ok(())
    }

    #[cfg(any(feature = "alloc", feature = "heapless"))]
    pub(crate) fn check_undersize(&self) -> Result<(), Error> {
        let len = self.len();
        (len > MIN_LEN).ok_or(Error::Undersize {
            len,
            min_len: MIN_LEN,
        })?;
        Ok(())
    }

    #[cfg(any(feature = "alloc", feature = "heapless"))]
    pub(crate) fn check_oversize(&self) -> Result<(), Error> {
        let len = self.len();
        (len < MAX_LEN).ok_or(Error::Oversize {
            len,
            max_len: MAX_LEN,
        })?;
        Ok(())
    }

    #[cfg(any(feature = "alloc", feature = "heapless"))]
    pub(crate) fn check_boundary(&self, index: usize) -> Result<(), Error> {
        let len = self.len();
        (index < len).ok_or(Error::OutOfBoundary { index, len })?;
        Ok(())
    }

    /// Returns the confined length: bytes for strings, items otherwise.
    pub fn len(&self) -> usize {
        self.0.confined_len()
    }

    /// Returns whether the collection contains no items.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Constructs confinement over collection which was already size-checked.
    ///
    /// # Panics
    ///
    /// Panics if the collection size doesn't fit confinement type requirements.
    pub fn from_checked(col: C) -> Self {
        Self::try_from(col).expect("collection size mismatch, use try_from instead")
    }

    /// Tries to construct a confinement over a collection. Fails if the number
    /// of items in the collection exceeds one of the confinement bounds.
    ///
    /// A confinement which lower bound exceeds the upper bound can't hold any
    /// collection, so the use of such a type fails to build. The failure is
    /// reported by `cargo build` and `cargo test`, but not by `cargo check`.
    ///
    /// ```
    /// # #[cfg(feature = "alloc")]
    /// # {
    /// # use confined::Confined;
    /// let vec = Confined::<Vec<u8>, 2, 5>::try_from(vec![1, 2, 3]);
    /// assert!(vec.is_ok());
    /// # }
    /// ```
    ///
    /// ```compile_fail,E0080
    /// # use confined::Confined;
    /// let vec = Confined::<Vec<u8>, 5, 2>::try_from(vec![1, 2, 3]);
    /// assert!(vec.is_ok());
    /// ```
    // We can't use `impl TryFrom` due to the conflict with core library blanked
    // implementation
    pub fn try_from(col: C) -> Result<Self, Error> {
        const {
            assert!(
                MIN_LEN <= MAX_LEN,
                "the lower bound of the confinement exceeds its upper bound"
            )
        };
        Self::does_fit_confinement(&col)?;
        Ok(Self(col))
    }

    /// Converts the confinement into a confinement with wider bounds, which
    /// holds the same collection. The conversion can't fail, since any
    /// collection which fits the current bounds fits the wider ones too.
    ///
    /// The new bounds must include the current ones: `MIN_LEN2 <= MIN_LEN` and
    /// `MAX_LEN <= MAX_LEN2`. A conversion which doesn't meet this requirement
    /// fails to build. The failure is reported by `cargo build` and
    /// `cargo test`, but not by `cargo check`.
    ///
    /// ```
    /// # #[cfg(feature = "alloc")]
    /// # {
    /// # use confined::Confined;
    /// let narrow = Confined::<Vec<u8>, 1, 3>::try_from(vec![1, 2]).unwrap();
    /// let wide: Confined<Vec<u8>, 0, 255> = narrow.widen();
    /// assert_eq!(wide.len_u8(), 2);
    /// # }
    /// ```
    ///
    /// ```compile_fail,E0080
    /// # use confined::Confined;
    /// let narrow = Confined::<Vec<u8>, 1, 3>::try_from(vec![1, 2]).unwrap();
    /// let wide: Confined<Vec<u8>, 2, 255> = narrow.widen();
    /// assert_eq!(wide.len_u8(), 2);
    /// ```
    pub fn widen<const MIN_LEN2: usize, const MAX_LEN2: usize>(
        self,
    ) -> Confined<C, MIN_LEN2, MAX_LEN2> {
        const {
            assert!(
                MIN_LEN2 <= MIN_LEN && MAX_LEN <= MAX_LEN2,
                "the new bounds of the confinement do not include the current bounds"
            )
        };
        Confined(self.0)
    }

    /// Returns reference to the inner collection type.
    pub fn as_unconfined(&self) -> &C {
        &self.0
    }

    /// Clones inner collection and returns an unconfined version of it.
    pub fn to_unconfined(&self) -> C
    where
        C: Clone,
    {
        self.0.clone()
    }

    /// Attempts to add a single element to the confined collection. Fails if
    /// the number of elements in the collection already maximal.
    ///
    /// The length of a `String` is measured in bytes,
    /// so a `char` is refused when its UTF-8 encoding doesn't fit the space
    /// left, which may happen before the string reaches the maximal length.
    /// [`Error::Oversize`] reports the length of the collection before the
    /// call.
    pub fn push(&mut self, elem: C::Item) -> Result<(), Error> {
        let len = self.len();
        let growth = self.0.item_growth(&elem);
        if len > MAX_LEN || growth > MAX_LEN - len {
            return Err(Error::Oversize {
                len,
                max_len: MAX_LEN,
            });
        }
        self.0
            .try_push(elem)
            .map_err(|_| Error::CapacityExceeded { len })
    }

    /// Fallibly reserves room for additional native collection units.
    ///
    /// For strings the units are bytes; for other collections they are items.
    pub fn try_reserve(&mut self, additional: usize) -> Result<(), Error>
    where
        C: collections::CollectionCapacity,
    {
        let len = self.len();
        if len > MAX_LEN || additional > MAX_LEN - len {
            return Err(Error::Oversize {
                len,
                max_len: MAX_LEN,
            });
        }
        collections::CollectionCapacity::try_reserve(&mut self.0, additional)
            .map_err(|_| Error::CapacityExceeded { len })
    }

    /// Attempts to add all elements from an iterator to the confined
    /// collection. Fails if the number of elements in the collection
    /// already maximal.
    ///
    /// The elements are added one by one with [`Self::push`]. The method stops
    /// at the first element which doesn't fit; the elements before it remain
    /// in the collection.
    pub fn extend<T: IntoIterator<Item = C::Item>>(&mut self, iter: T) -> Result<(), Error> {
        for elem in iter {
            self.push(elem)?;
        }
        Ok(())
    }

    /// Releases underlying collection from the confinement.
    pub fn release(self) -> C {
        self.0
    }

    /// Allows mutation of the confinement, checking that the mutation does not
    /// break confinement requirements.
    ///
    /// The closure gets the confinement itself, so it can use the checked
    /// methods only, and none of them takes a collection out of its bounds.
    /// The bounds are checked once more after the closure returns.
    ///
    /// # Errors
    ///
    /// Errors if the collection is outside of the bounds after the mutation.
    pub fn with_mut(mut self, f: impl FnOnce(&mut Self)) -> Result<Self, Error> {
        f(&mut self);
        Self::does_fit_confinement(&self)?;
        Ok(self)
    }

    /// Allows mutation of the confinement, checking that the mutation does not
    /// break confinement requirements.
    ///
    /// # Panics
    ///
    /// Panics if the collection is outside of the bounds after the mutation.
    /// The closure can use the checked methods only, none of which takes a
    /// collection out of its bounds, so the panic means a defect of the
    /// wrapped collection or of its [`ItemLen`] implementation.
    pub fn as_mut_checked(&mut self, f: impl FnOnce(&mut Self)) {
        f(self);
        Self::does_fit_confinement(self).expect("confinement broken");
    }

    fn collect_checked(mut col: C, iter: impl IntoIterator<Item = C::Item>) -> Result<Self, Error> {
        for item in iter {
            let len = col.confined_len();
            let growth = col.item_growth(&item);
            if len > MAX_LEN || growth > MAX_LEN - len {
                return Err(Error::Oversize {
                    len: len.saturating_add(growth),
                    max_len: MAX_LEN,
                });
            }
            col.try_push(item)
                .map_err(|_| Error::CapacityExceeded { len })?;
        }
        Self::try_from(col)
    }
}

#[cfg(feature = "alloc")]
impl<C, const MIN_LEN: usize, const MAX_LEN: usize> Confined<C, MIN_LEN, MAX_LEN>
where
    C: CollectionSet + ItemLen,
{
    pub(crate) fn remove_from_set(
        &mut self,
        item: &C::Item,
        remove: impl FnOnce(&mut C, &C::Item) -> bool,
    ) -> Result<bool, Error> {
        if !self.0.contains(item) {
            return Ok(false);
        }
        self.check_undersize()?;
        Ok(remove(&mut self.0, item))
    }

    pub(crate) fn take_from_set(
        &mut self,
        item: &C::Item,
        take: impl FnOnce(&mut C, &C::Item) -> Option<C::Item>,
    ) -> Result<Option<C::Item>, Error> {
        if !self.0.contains(item) {
            return Ok(None);
        }
        self.check_undersize()?;
        Ok(take(&mut self.0, item))
    }
}

#[cfg(any(feature = "alloc", feature = "heapless"))]
impl<C, const MIN_LEN: usize, const MAX_LEN: usize> Confined<C, MIN_LEN, MAX_LEN>
where
    C: CollectionMap + ItemLen,
{
    pub(crate) fn remove_from_map<T>(
        &mut self,
        key: &C::Key,
        remove: impl FnOnce(&mut C, &C::Key) -> Option<T>,
    ) -> Result<Option<T>, Error> {
        if !self.0.contains_key(key) {
            return Ok(None);
        }
        self.check_undersize()?;
        Ok(remove(&mut self.0, key))
    }
}

impl<C, const MIN_LEN: usize, const MAX_LEN: usize> Confined<C, MIN_LEN, MAX_LEN>
where
    C: Default + ItemLen,
{
    /// Tries to construct a confinement with a collection of elements taken
    /// from an iterator. Fails if the number of items in the collection
    /// exceeds one of the confinement bounds.
    ///
    /// The iterator is not read past the item with which the collection
    /// exceeds the upper bound, and [`Error::Oversize`] reports the length of
    /// the collection at that moment.
    pub fn try_from_iter<I: IntoIterator<Item = C::Item>>(iter: I) -> Result<Self, Error> {
        Self::collect_checked(C::default(), iter)
    }

    /// Constructs a confinement with a collection of elements taken from an
    /// iterator.
    ///
    /// # Panics
    ///
    /// Panics if the number of items in the collection exceeds one of the
    /// confinement bounds.
    pub fn from_iter_checked<I: IntoIterator<Item = C::Item>>(iter: I) -> Self {
        Self::try_from_iter(iter).expect("collection size mismatch, use try_from_iter instead")
    }
}

impl<C, const MIN_LEN: usize, const MAX_LEN: usize> Confined<C, MIN_LEN, MAX_LEN>
where
    C: CollectionAlloc + ItemLen,
{
    /// Tries to construct a confinement with a collection of elements taken
    /// from an iterator, using the supplied allocator.
    ///
    /// Like [`Self::try_from_iter`], this stops reading as soon as the
    /// collection exceeds the upper bound. The allocator need not implement
    /// [`Default`].
    pub fn try_from_iter_in<I: IntoIterator<Item = C::Item>>(
        iter: I,
        allocator: C::Allocator,
    ) -> Result<Self, Error> {
        Self::collect_checked(C::new_in(allocator), iter)
    }

    /// Constructs a confinement from an iterator, using the supplied
    /// allocator.
    ///
    /// # Panics
    ///
    /// Panics if the number of items in the collection exceeds one of the
    /// confinement bounds.
    pub fn from_iter_checked_in<I: IntoIterator<Item = C::Item>>(
        iter: I,
        allocator: C::Allocator,
    ) -> Self {
        Self::try_from_iter_in(iter, allocator)
            .expect("collection size mismatch, use try_from_iter_in instead")
    }
}

impl<C, const MIN_LEN: usize, const MAX_LEN: usize> Confined<C, MIN_LEN, MAX_LEN>
where
    C: CollectionMap + ItemLen,
{
    /// Gets mutable reference to an element of the collection.
    pub fn get_mut(&mut self, key: &C::Key) -> Option<&mut C::Value> {
        self.0.get_mut(key)
    }

    /// Inserts a new value into the confined collection under a given key.
    ///
    /// If the key is already present, the value is overwritten and the previous
    /// value is returned. This succeeds even when the collection is at
    /// capacity. The value is replaced in place: the stored key is kept, and
    /// the wrapped map is not asked for room, so no storage is allocated.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Oversize`] when the entry adds to the length more than
    /// the room left below `MAX_LEN`: for most maps, when the key is not
    /// already present and the collection has reached `MAX_LEN`. Returns
    /// [`Error::CapacityExceeded`] when the wrapped map has no storage for a
    /// new entry.
    pub fn insert(&mut self, key: C::Key, value: C::Value) -> Result<Option<C::Value>, Error> {
        let item = (key, value);
        let len = self.len();
        let growth = self.0.item_growth(&item);
        if len > MAX_LEN || growth > MAX_LEN - len {
            return Err(Error::Oversize {
                len,
                max_len: MAX_LEN,
            });
        }
        let (key, value) = item;
        // The native `insert` of a hash map reserves room for a new entry
        // before it looks the key up, which may reallocate the table, or abort
        // on a failed allocation, without adding anything.
        if let Some(slot) = self.0.get_mut(&key) {
            return Ok(Some(core::mem::replace(slot, value)));
        }
        self.0
            .try_push((key, value))
            .map_err(|_| Error::CapacityExceeded { len })?;
        Ok(None)
    }

    /// Retains only the elements specified by the predicate.
    ///
    /// In other words, remove all pairs `(k, v)` for which `f(&k, &mut v)`
    /// returns `false`. The elements are visited in unsorted (and
    /// unspecified) order.
    ///
    /// The predicate is called once for each element.
    ///
    /// # Errors
    ///
    /// Errors if the predicate rejects more elements than the lower bound of
    /// the confinement allows to remove. The map is never left below the
    /// bound: the rejected elements are removed in the order of the iteration
    /// while the map stays above it, and the rest of them are kept.
    /// [`Error::Undersize`] reports the length which the map would have had
    /// with all the rejected elements removed.
    ///
    /// The length which a removed element takes away is not known to the
    /// method, so with a lower bound each removal is counted as
    /// [`ItemLen::MAX_ITEM_LEN`]. It is exact for the maps where each element
    /// adds one to the length, and keeps more elements than necessary
    /// otherwise.
    ///
    /// # Panics
    ///
    /// A panic of the predicate, or of a destructor of a removed element, is
    /// propagated. The elements removed before it stay removed, the map stays
    /// within the confinement and remains usable.
    pub fn try_retain(
        &mut self,
        mut f: impl FnMut(&C::Key, &mut C::Value) -> bool,
    ) -> Result<(), Error> {
        let mut removable = self.len().saturating_sub(MIN_LEN);
        let mut kept_rejected = 0usize;
        retain_unwind_safe(&mut self.0, |key, value| {
            if f(key, value) {
                return true;
            }
            if MIN_LEN > 0 {
                if removable < C::MAX_ITEM_LEN {
                    kept_rejected += 1;
                    return true;
                }
                removable -= C::MAX_ITEM_LEN;
            }
            false
        });
        if kept_rejected > 0 {
            let rejected_len = kept_rejected.saturating_mul(C::MAX_ITEM_LEN);
            return Err(Error::Undersize {
                len: self.len().saturating_sub(rejected_len),
                min_len: MIN_LEN,
            });
        }
        Ok(())
    }
}

impl<C, const MIN_LEN: usize, const MAX_LEN: usize> Confined<C, MIN_LEN, MAX_LEN>
where
    C: CollectionEntry + ItemLen,
{
    /// Gets the given key's corresponding entry for in-place manipulation.
    /// The upper bound is checked when the entry is vacant.
    ///
    /// The value of a vacant entry is not known yet, so the entry is given out
    /// only when the room left below `MAX_LEN` fits the largest item of the
    /// collection, [`ItemLen::MAX_ITEM_LEN`]: one for most maps.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Oversize`] when the key is not present and there is
    /// no room for a new entry, and [`Error::CapacityExceeded`] when the
    /// wrapped map has no storage for it.
    pub fn entry(&mut self, key: C::Key) -> Result<Entry<'_, C>, Error> {
        if !self.0.contains_key(&key) {
            let len = self.len();
            if len > MAX_LEN || C::MAX_ITEM_LEN > MAX_LEN - len {
                return Err(Error::Oversize {
                    len,
                    max_len: MAX_LEN,
                });
            }
            self.0.prepare_entry()?;
        }
        Ok(Entry(self.0.entry(key)))
    }
}

impl<'a, C> Entry<'a, C>
where
    C: CollectionEntry + 'a,
{
    /// Returns a reference to this entry's key.
    pub fn key(&self) -> &C::Key {
        self.0.key()
    }

    /// Inserts the provided value if the entry is vacant, then returns it.
    pub fn or_insert(self, default: C::Value) -> &'a mut C::Value {
        self.0.or_insert(default)
    }

    /// Inserts a value produced by `default` if the entry is vacant.
    pub fn or_insert_with<F: FnOnce() -> C::Value>(self, default: F) -> &'a mut C::Value {
        self.0.or_insert_with(default)
    }

    /// Inserts the default value if the entry is vacant.
    pub fn or_default(self) -> &'a mut C::Value
    where
        C::Value: Default,
    {
        self.0.or_default()
    }

    /// Modifies the value in an occupied entry.
    pub fn and_modify<F: FnOnce(&mut C::Value)>(self, f: F) -> Self {
        Entry(self.0.and_modify(f))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A stack of up to four bytes, which needs no allocator.
    #[derive(Default, PartialEq, Debug)]
    struct Slots {
        items: [u8; 4],
        len: usize,
    }

    impl Slots {
        fn of(items: &[u8]) -> Self {
            let mut slots = Slots::default();
            for item in items {
                CollectionBase::push(&mut slots, *item);
            }
            slots
        }

        fn as_slice(&self) -> &[u8] {
            &self.items[..self.len]
        }
    }

    impl CollectionBase for Slots {
        type Item = u8;

        fn item_count(&self) -> usize {
            self.len
        }

        fn push(&mut self, elem: u8) {
            self.items[self.len] = elem;
            self.len += 1;
        }

        fn clear(&mut self) {
            self.len = 0;
        }
    }

    impl ItemLen for Slots {
        fn try_push(&mut self, item: u8) -> Result<(), u8> {
            if self.len == self.items.len() {
                return Err(item);
            }
            CollectionBase::push(self, item);
            Ok(())
        }
    }

    /// A collection which only reports a length. It lets the tests build a
    /// confinement which is already broken, to check the defensive branches
    /// unreachable through the public API.
    #[derive(Debug)]
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

    mod check_len {
        use super::*;

        type Bounds = Confined<Slots, 2, 3>;

        #[test]
        fn length_below_the_minimum_is_undersize() {
            assert_eq!(
                Bounds::check_len(1),
                Err(Error::Undersize { len: 1, min_len: 2 })
            );
        }

        #[test]
        fn length_at_the_minimum_fits() {
            assert_eq!(Bounds::check_len(2), Ok(()));
        }

        #[test]
        fn length_at_the_maximum_fits() {
            assert_eq!(Bounds::check_len(3), Ok(()));
        }

        #[test]
        fn length_above_the_maximum_is_oversize() {
            assert_eq!(
                Bounds::check_len(4),
                Err(Error::Oversize { len: 4, max_len: 3 })
            );
        }

        #[test]
        fn equal_bounds_accept_a_single_length() {
            type Exact = Confined<Slots, 2, 2>;
            assert!(Exact::check_len(1).is_err());
            assert_eq!(Exact::check_len(2), Ok(()));
            assert!(Exact::check_len(3).is_err());
        }

        #[test]
        fn zero_bounds_accept_only_an_empty_collection() {
            type Empty = Confined<Slots, 0, 0>;
            assert_eq!(Empty::check_len(0), Ok(()));
            assert_eq!(
                Empty::check_len(1),
                Err(Error::Oversize { len: 1, max_len: 0 })
            );
        }

        #[test]
        fn the_whole_range_accepts_any_length() {
            type Any = Confined<Slots, 0, { usize::MAX }>;
            assert_eq!(Any::check_len(0), Ok(()));
            assert_eq!(Any::check_len(usize::MAX), Ok(()));
        }
    }

    mod does_fit_confinement {
        use super::*;

        #[test]
        fn confined_length_of_the_collection_is_checked() {
            type Bounds = Confined<Slots, 1, 2>;
            assert!(Bounds::does_fit_confinement(&Slots::of(&[])).is_err());
            assert_eq!(Bounds::does_fit_confinement(&Slots::of(&[1])), Ok(()));
            assert_eq!(Bounds::does_fit_confinement(&Slots::of(&[1, 2])), Ok(()));
            assert!(Bounds::does_fit_confinement(&Slots::of(&[1, 2, 3])).is_err());
        }
    }

    #[cfg(any(feature = "alloc", feature = "heapless"))]
    mod check_oversize {
        use super::*;

        #[test]
        fn collection_below_the_maximum_has_room() {
            let col = Confined::<Slots, 0, 2>(Slots::of(&[1]));
            assert_eq!(col.check_oversize(), Ok(()));
        }

        #[test]
        fn collection_at_the_maximum_has_no_room() {
            let col = Confined::<Slots, 0, 2>(Slots::of(&[1, 2]));
            assert_eq!(
                col.check_oversize(),
                Err(Error::Oversize { len: 2, max_len: 2 })
            );
        }
    }

    #[cfg(any(feature = "alloc", feature = "heapless"))]
    mod check_undersize {
        use super::*;

        #[test]
        fn collection_above_the_minimum_can_lose_an_item() {
            let col = Confined::<Slots, 1, 3>(Slots::of(&[1, 2]));
            assert_eq!(col.check_undersize(), Ok(()));
        }

        #[test]
        fn collection_at_the_minimum_can_not_lose_an_item() {
            let col = Confined::<Slots, 2, 3>(Slots::of(&[1, 2]));
            assert_eq!(
                col.check_undersize(),
                Err(Error::Undersize { len: 2, min_len: 2 })
            );
        }

        #[test]
        fn empty_collection_can_not_lose_an_item() {
            let col = Confined::<Slots, 0, 3>(Slots::of(&[]));
            assert_eq!(
                col.check_undersize(),
                Err(Error::Undersize { len: 0, min_len: 0 })
            );
        }
    }

    #[cfg(any(feature = "alloc", feature = "heapless"))]
    mod check_boundary {
        use super::*;

        #[test]
        fn index_below_the_length_is_inside() {
            let col = Confined::<Slots, 0, 3>(Slots::of(&[1, 2]));
            assert_eq!(col.check_boundary(0), Ok(()));
            assert_eq!(col.check_boundary(1), Ok(()));
        }

        #[test]
        fn index_equal_to_the_length_is_outside() {
            let col = Confined::<Slots, 0, 3>(Slots::of(&[1, 2]));
            assert_eq!(
                col.check_boundary(2),
                Err(Error::OutOfBoundary { index: 2, len: 2 })
            );
        }

        #[test]
        fn largest_index_is_outside() {
            let col = Confined::<Slots, 0, 3>(Slots::of(&[1, 2]));
            assert_eq!(
                col.check_boundary(usize::MAX),
                Err(Error::OutOfBoundary {
                    index: usize::MAX,
                    len: 2
                })
            );
        }

        #[test]
        fn empty_collection_has_no_index_inside() {
            let col = Confined::<Slots, 0, 3>(Slots::of(&[]));
            assert_eq!(
                col.check_boundary(0),
                Err(Error::OutOfBoundary { index: 0, len: 0 })
            );
        }
    }

    mod collect_checked {
        use core::cell::Cell;

        use super::*;

        #[test]
        fn items_within_the_bounds_are_collected() {
            let col = Confined::<Slots, 1, 3>::collect_checked(Slots::default(), [1, 2]).unwrap();
            assert_eq!(col.0.as_slice(), [1, 2]);
        }

        #[test]
        fn item_above_the_maximum_is_reported_with_the_length_it_makes() {
            let result = Confined::<Slots, 0, 2>::collect_checked(Slots::default(), [1, 2, 3]);
            assert_eq!(result.unwrap_err(), Error::Oversize { len: 3, max_len: 2 });
        }

        #[test]
        fn iterator_is_not_read_past_the_rejected_item() {
            let reads = Cell::new(0);
            let source = (1..=4).inspect(|_| reads.set(reads.get() + 1));
            let result = Confined::<Slots, 0, 2>::collect_checked(Slots::default(), source);
            assert!(result.is_err());
            assert_eq!(reads.get(), 3);
        }

        #[test]
        fn refused_item_is_reported_with_the_length_before_it() {
            // The confinement allows five items, the storage holds four.
            let result =
                Confined::<Slots, 0, 5>::collect_checked(Slots::default(), [1, 2, 3, 4, 5]);
            assert_eq!(result.unwrap_err(), Error::CapacityExceeded { len: 4 });
        }

        #[test]
        fn too_few_items_are_reported_when_the_iterator_ends() {
            let result = Confined::<Slots, 3, 4>::collect_checked(Slots::default(), [1, 2]);
            assert_eq!(result.unwrap_err(), Error::Undersize { len: 2, min_len: 3 });
        }

        #[test]
        fn items_are_added_to_the_given_collection() {
            let col = Confined::<Slots, 0, 3>::collect_checked(Slots::of(&[1]), [2, 3]).unwrap();
            assert_eq!(col.0.as_slice(), [1, 2, 3]);
        }

        #[test]
        fn collection_above_the_maximum_accepts_nothing() {
            let result = Confined::<Reported, 0, 2>::collect_checked(Reported(3), [()]);
            assert_eq!(result.unwrap_err(), Error::Oversize { len: 4, max_len: 2 });
        }
    }

    mod broken_confinement {
        use super::*;

        #[test]
        fn push_adds_nothing_above_the_maximum() {
            let mut col = Confined::<Reported, 0, 2>(Reported(3));
            assert_eq!(col.push(()), Err(Error::Oversize { len: 3, max_len: 2 }));
            assert_eq!(col.len(), 3);
        }

        #[test]
        fn extend_adds_nothing_above_the_maximum() {
            let mut col = Confined::<Reported, 0, 2>(Reported(3));
            assert_eq!(
                col.extend([(), ()]),
                Err(Error::Oversize { len: 3, max_len: 2 })
            );
            assert_eq!(col.len(), 3);
        }

        #[test]
        fn revalidation_reports_the_length_above_the_maximum() {
            let col = Confined::<Reported, 0, 2>(Reported(3));
            assert_eq!(col.with_mut(|_| ()).unwrap_err(), Error::Oversize {
                len: 3,
                max_len: 2
            });
        }

        #[test]
        fn revalidation_reports_the_length_below_the_minimum() {
            let col = Confined::<Reported, 2, 4>(Reported(1));
            assert_eq!(col.with_mut(|_| ()).unwrap_err(), Error::Undersize {
                len: 1,
                min_len: 2
            });
        }

        #[test]
        #[should_panic(expected = "confinement broken")]
        fn checked_mutation_panics_outside_of_the_bounds() {
            let mut col = Confined::<Reported, 0, 2>(Reported(3));
            col.as_mut_checked(|_| ());
        }
    }

    mod retain_unwind_safe {
        use super::*;

        /// A map of up to four entries which counts the passes of `retain`.
        #[derive(Default)]
        struct Ledger {
            entries: [Option<(u8, u8)>; 4],
            passes: usize,
        }

        impl Ledger {
            fn of(keys: &[u8]) -> Self {
                let mut ledger = Ledger::default();
                for key in keys {
                    CollectionBase::push(&mut ledger, (*key, 0));
                }
                ledger
            }

            fn keys(&self) -> impl Iterator<Item = u8> {
                self.entries.iter().flatten().map(|(key, _)| *key)
            }
        }

        impl CollectionBase for Ledger {
            type Item = (u8, u8);

            fn item_count(&self) -> usize {
                self.entries.iter().flatten().count()
            }

            fn push(&mut self, (key, value): (u8, u8)) {
                CollectionMap::insert(self, key, value);
            }

            fn clear(&mut self) {
                self.entries = [None; 4];
            }
        }

        impl CollectionMap for Ledger {
            type Key = u8;
            type Value = u8;

            fn contains_key(&self, key: &u8) -> bool {
                self.get(key).is_some()
            }

            fn get(&self, key: &u8) -> Option<&u8> {
                self.iter().find(|(k, _)| *k == key).map(|(_, value)| value)
            }

            fn get_mut(&mut self, key: &u8) -> Option<&mut u8> {
                self.iter_mut()
                    .find(|(k, _)| *k == key)
                    .map(|(_, value)| value)
            }

            fn iter(&self) -> impl Iterator<Item = (&u8, &u8)> {
                self.entries
                    .iter()
                    .flatten()
                    .map(|(key, value)| (key, value))
            }

            fn iter_mut(&mut self) -> impl Iterator<Item = (&u8, &mut u8)> {
                self.entries
                    .iter_mut()
                    .flatten()
                    .map(|(key, value)| (&*key, value))
            }

            fn values_mut(&mut self) -> impl Iterator<Item = &mut u8> {
                self.iter_mut().map(|(_, value)| value)
            }

            fn insert(&mut self, key: u8, value: u8) -> Option<u8> {
                if let Some(slot) = self.get_mut(&key) {
                    return Some(core::mem::replace(slot, value));
                }
                let vacant = self.entries.iter_mut().find(|slot| slot.is_none());
                *vacant.expect("the ledger is full") = Some((key, value));
                None
            }

            fn remove(&mut self, key: &u8) -> Option<u8> {
                let slot = self
                    .entries
                    .iter_mut()
                    .find(|slot| matches!(slot, Some((k, _)) if k == key))?;
                slot.take().map(|(_, value)| value)
            }

            fn retain(&mut self, mut f: impl FnMut(&u8, &mut u8) -> bool) {
                self.passes += 1;
                for slot in &mut self.entries {
                    let keep = match slot {
                        Some((key, value)) => f(key, value),
                        None => true,
                    };
                    if !keep {
                        *slot = None;
                    }
                }
            }
        }

        #[test]
        fn rejected_entries_are_removed_in_a_single_pass() {
            let mut ledger = Ledger::of(&[1, 2, 3]);
            retain_unwind_safe(&mut ledger, |key, _| *key != 2);
            assert!(ledger.keys().eq([1, 3]));
            assert_eq!(ledger.passes, 1, "no repair pass is made without a panic");
        }

        #[test]
        fn predicate_is_called_once_for_each_entry() {
            let mut ledger = Ledger::of(&[1, 2, 3]);
            let mut calls = 0;
            retain_unwind_safe(&mut ledger, |_, _| {
                calls += 1;
                true
            });
            assert_eq!(calls, 3);
        }

        #[cfg(feature = "std")]
        #[test]
        fn panic_of_the_predicate_is_followed_by_a_repair_pass_keeping_the_entries() {
            use std::panic::{self, AssertUnwindSafe};

            let mut ledger = Ledger::of(&[1, 2, 3]);
            let mut calls = 0;
            let result = panic::catch_unwind(AssertUnwindSafe(|| {
                retain_unwind_safe(&mut ledger, |key, _| {
                    calls += 1;
                    assert_ne!(*key, 2, "predicate panic");
                    false
                })
            }));
            assert!(result.is_err(), "the panic is propagated");
            assert_eq!(calls, 2, "the repair pass doesn't call the predicate");
            assert_eq!(ledger.passes, 2);
            assert!(ledger.keys().eq([2, 3]));
        }
    }

    #[cfg(feature = "alloc")]
    mod removal {
        use alloc::collections::{BTreeMap, BTreeSet};
        use core::cell::Cell;

        use super::*;

        fn set<const MIN: usize>() -> Confined<BTreeSet<u8>, MIN, 3> {
            Confined(BTreeSet::from([1, 2]))
        }

        fn map<const MIN: usize>() -> Confined<BTreeMap<u8, char>, MIN, 3> {
            Confined(BTreeMap::from([(1, 'a'), (2, 'b')]))
        }

        #[test]
        fn absent_item_is_not_passed_to_the_removal_from_a_set() {
            let calls = Cell::new(0);
            let result = set::<2>().remove_from_set(&3, |_, _| {
                calls.set(calls.get() + 1);
                true
            });
            assert_eq!(result, Ok(false));
            assert_eq!(calls.get(), 0);
        }

        #[test]
        fn item_at_the_minimum_is_not_passed_to_the_removal_from_a_set() {
            let calls = Cell::new(0);
            let result = set::<2>().remove_from_set(&2, |_, _| {
                calls.set(calls.get() + 1);
                true
            });
            assert_eq!(result, Err(Error::Undersize { len: 2, min_len: 2 }));
            assert_eq!(calls.get(), 0);
        }

        #[test]
        fn present_item_is_removed_from_a_set() {
            let mut set = set::<1>();
            assert_eq!(
                set.remove_from_set(&2, |set, item| set.remove(item)),
                Ok(true)
            );
            assert!(set.0.iter().eq([&1]));
        }

        #[test]
        fn absent_item_is_not_taken_from_a_set() {
            let result = set::<2>().take_from_set(&3, |_, _| unreachable!());
            assert_eq!(result, Ok(None));
        }

        #[test]
        fn item_at_the_minimum_is_not_taken_from_a_set() {
            let result = set::<2>().take_from_set(&2, |_, _| unreachable!());
            assert_eq!(result, Err(Error::Undersize { len: 2, min_len: 2 }));
        }

        #[test]
        fn present_item_is_taken_from_a_set() {
            let mut set = set::<1>();
            assert_eq!(
                set.take_from_set(&2, |set, item| set.take(item)),
                Ok(Some(2))
            );
            assert!(set.0.iter().eq([&1]));
        }

        #[test]
        fn absent_key_is_not_removed_from_a_map() {
            let result = map::<2>().remove_from_map(&3, |_, _| -> Option<char> { unreachable!() });
            assert_eq!(result, Ok(None));
        }

        #[test]
        fn key_at_the_minimum_is_not_removed_from_a_map() {
            let result = map::<2>().remove_from_map(&2, |_, _| -> Option<char> { unreachable!() });
            assert_eq!(result, Err(Error::Undersize { len: 2, min_len: 2 }));
        }

        #[test]
        fn present_key_is_removed_from_a_map() {
            let mut map = map::<1>();
            assert_eq!(
                map.remove_from_map(&2, |map, key| map.remove(key)),
                Ok(Some('b'))
            );
            assert!(map.0.keys().eq([&1]));
        }
    }
}
