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
use core::marker::PhantomData;
use core::ops::{Deref, Index, IndexMut};

use collections::{Collection, KeyedCollection};

use super::Error;

/// The confinement for the collection.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct Confined<C: Collection, const MIN_LEN: usize, const MAX_LEN: usize>(pub(crate) C);

#[cfg(feature = "serde")]
impl<'de, C, const MIN_LEN: usize, const MAX_LEN: usize> serde::Deserialize<'de>
    for Confined<C, MIN_LEN, MAX_LEN>
where
    C: Collection + serde::Deserialize<'de>,
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
        #[derive(Deserialize)]
        #[serde(rename = "Confined")]
        struct Unchecked<C>(C);

        let Unchecked(col) = <Unchecked<C> as serde::Deserialize>::deserialize(deserializer)?;
        Self::try_from(col).map_err(serde::de::Error::custom)
    }
}

/// Tells how much a single item can add to the length of a collection.
///
/// The length of a confined collection is the value returned by
/// [`Collection::len`]. An item added to most of the collections increases it
/// by one at most, which is the default of this trait.
/// [`String`](alloc::string::String) is the exception: its length is measured
/// in bytes, and a `char` takes from one to four of them.
///
/// [`Confined::push`], [`Confined::extend`] and [`Confined::with`] rely on this
/// trait to keep a collection within its upper bound. An implementation must
/// never report less than an item really adds, otherwise these methods are not
/// able to uphold the confinement.
///
/// A collection where each item counts as one needs an empty implementation:
///
/// ```
/// # use confined::ItemLen;
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
/// # impl collections::Collection for Stack {
/// #     type Item = u8;
/// #     fn with_capacity(capacity: usize) -> Self {
/// #         Stack(Vec::with_capacity(capacity))
/// #     }
/// #     fn len(&self) -> usize {
/// #         self.0.len()
/// #     }
/// #     fn push(&mut self, elem: u8) {
/// #         self.0.push(elem)
/// #     }
/// #     fn clear(&mut self) {
/// #         self.0.clear()
/// #     }
/// # }
/// impl ItemLen for Stack {}
///
/// let mut stack = confined::Confined::<Stack, 0, 1>::try_from(Stack(vec![])).unwrap();
/// assert!(stack.push(1).is_ok());
/// assert!(stack.push(2).is_err());
/// ```
pub trait ItemLen: Collection {
    /// The largest value which [`ItemLen::item_len`] can return.
    const MAX_ITEM_LEN: usize = 1;

    /// Returns the largest number by which the length of the collection can
    /// grow when the `item` is added to it.
    fn item_len(_item: &Self::Item) -> usize {
        1
    }
}

/// A view into a single entry of a confined map, which may either be vacant or
/// occupied. It is constructed with [`Confined::entry`].
///
/// Unlike the entries of the wrapped maps, it has no methods removing the
/// entry: a removal could take the map below the lower bound of the
/// confinement. Use the `remove` method of the confined map, which checks the
/// bound.
///
/// The methods are provided for the maps supported by the crate: `BTreeMap`,
/// `HashMap` and `IndexMap`.
pub struct Entry<'a, C: KeyedCollection + 'a>(pub(crate) C::Entry<'a>);

impl<'a, C: KeyedCollection + 'a> fmt::Debug for Entry<'a, C>
where
    C::Entry<'a>: fmt::Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Entry").field(&self.0).finish()
    }
}

/// Build-time check that the bounds of a confinement do not contradict each
/// other.
///
/// The constants of this and other build-time checks are evaluated when the
/// machine code is generated for the concrete values of the generic
/// parameters. Thus, a violation stops `cargo build` and `cargo test`, but
/// not `cargo check`.
pub(crate) struct Bounds<const MIN_LEN: usize, const MAX_LEN: usize>;

impl<const MIN_LEN: usize, const MAX_LEN: usize> Bounds<MIN_LEN, MAX_LEN> {
    pub(crate) const VALID: () = assert!(
        MIN_LEN <= MAX_LEN,
        "the lower bound of the confinement exceeds its upper bound"
    );
}

/// Build-time check that a collection of `LEN` items fits a confinement.
pub(crate) struct FitsLen<const MIN_LEN: usize, const MAX_LEN: usize, const LEN: usize>;

impl<const MIN_LEN: usize, const MAX_LEN: usize, const LEN: usize> FitsLen<MIN_LEN, MAX_LEN, LEN> {
    pub(crate) const VALID: () = assert!(
        MIN_LEN <= LEN && LEN <= MAX_LEN,
        "the number of items is outside of the confinement bounds"
    );
}

/// Build-time check that a confinement cannot be empty.
pub(crate) struct NeverEmpty<const MIN_LEN: usize>;

impl<const MIN_LEN: usize> NeverEmpty<MIN_LEN> {
    pub(crate) const VALID: () = assert!(MIN_LEN > 0, "the confinement may be empty");
}

/// Build-time check that the maximum length fits an integer limit.
pub(crate) struct LenWithin<const MAX_LEN: usize, const LIMIT: usize>;

impl<const MAX_LEN: usize, const LIMIT: usize> LenWithin<MAX_LEN, LIMIT> {
    pub(crate) const VALID: () = assert!(
        MAX_LEN <= LIMIT,
        "the upper bound of the confinement exceeds the integer limit"
    );
}

/// Build-time check that the bounds of the second confinement include the
/// bounds of the first one.
pub(crate) struct Widens<
    const MIN_LEN: usize,
    const MAX_LEN: usize,
    const MIN_LEN2: usize,
    const MAX_LEN2: usize,
>;

impl<const MIN_LEN: usize, const MAX_LEN: usize, const MIN_LEN2: usize, const MAX_LEN2: usize>
    Widens<MIN_LEN, MAX_LEN, MIN_LEN2, MAX_LEN2>
{
    pub(crate) const VALID: () = assert!(
        MIN_LEN2 <= MIN_LEN && MAX_LEN <= MAX_LEN2,
        "the new bounds of the confinement do not include the current bounds"
    );
}

/// Build-time check that a confinement is able to hold a single item of any
/// size.
pub(crate) struct HoldsItem<C: ItemLen, const MAX_LEN: usize>(PhantomData<C>);

impl<C: ItemLen, const MAX_LEN: usize> HoldsItem<C, MAX_LEN> {
    pub(crate) const VALID: () = assert!(
        C::MAX_ITEM_LEN <= MAX_LEN,
        "the upper bound of the confinement is less than the largest size of a single item"
    );
}

impl<C: Collection, const MIN_LEN: usize, const MAX_LEN: usize> Deref
    for Confined<C, MIN_LEN, MAX_LEN>
{
    type Target = C;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<C, const MIN_LEN: usize, const MAX_LEN: usize> AsRef<[C::Item]>
    for Confined<C, MIN_LEN, MAX_LEN>
where
    C: Collection + AsRef<[C::Item]>,
{
    fn as_ref(&self) -> &[C::Item] {
        self.0.as_ref()
    }
}

impl<C, const MIN_LEN: usize, const MAX_LEN: usize> AsMut<[C::Item]>
    for Confined<C, MIN_LEN, MAX_LEN>
where
    C: Collection + AsMut<[C::Item]>,
{
    fn as_mut(&mut self) -> &mut [C::Item] {
        self.0.as_mut()
    }
}

impl<C, const MIN_LEN: usize, const MAX_LEN: usize> Borrow<[C::Item]>
    for Confined<C, MIN_LEN, MAX_LEN>
where
    C: Collection + Borrow<[C::Item]>,
{
    fn borrow(&self) -> &[C::Item] {
        self.0.borrow()
    }
}

impl<C, const MIN_LEN: usize, const MAX_LEN: usize> BorrowMut<[C::Item]>
    for Confined<C, MIN_LEN, MAX_LEN>
where
    C: Collection + BorrowMut<[C::Item]>,
{
    fn borrow_mut(&mut self) -> &mut [C::Item] {
        self.0.borrow_mut()
    }
}

impl<C: Collection, const MIN_LEN: usize, const MAX_LEN: usize> IntoIterator
    for Confined<C, MIN_LEN, MAX_LEN>
where
    C: IntoIterator,
{
    type Item = <C as IntoIterator>::Item;
    type IntoIter = <C as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'c, C: Collection, const MIN_LEN: usize, const MAX_LEN: usize> IntoIterator
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

impl<'c, C: Collection, const MIN_LEN: usize, const MAX_LEN: usize> IntoIterator
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
    C: Collection + 'c,
    &'c mut C: IntoIterator<Item = &'c mut <C as Collection>::Item>,
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
    C: KeyedCollection,
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

impl<C: Collection, I, const MIN_LEN: usize, const MAX_LEN: usize> Index<I>
    for Confined<C, MIN_LEN, MAX_LEN>
where
    C: Index<I>,
{
    type Output = C::Output;

    #[inline]
    fn index(&self, index: I) -> &Self::Output {
        self.0.index(index)
    }
}

impl<C: Collection, I, const MIN_LEN: usize, const MAX_LEN: usize> IndexMut<I>
    for Confined<C, MIN_LEN, MAX_LEN>
where
    C: IndexMut<I>,
{
    #[inline]
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        self.0.index_mut(index)
    }
}

impl<C: Collection, const MIN_LEN: usize, const MAX_LEN: usize> Display
    for Confined<C, MIN_LEN, MAX_LEN>
where
    C: Display,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl<C: Collection, const MIN_LEN: usize, const MAX_LEN: usize> Confined<C, MIN_LEN, MAX_LEN> {
    pub(crate) fn does_fit_confinement(col: &C) -> Result<(), Error> {
        let len = col.len();
        if len < MIN_LEN {
            return Err(Error::Undersize {
                len,
                min_len: MIN_LEN,
            });
        }
        if len > MAX_LEN {
            return Err(Error::Oversize {
                len,
                max_len: MAX_LEN,
            });
        }
        Ok(())
    }

    pub(crate) fn check_undersize(&self) -> Result<(), Error> {
        let len = self.len();
        if len <= MIN_LEN {
            return Err(Error::Undersize {
                len,
                min_len: MIN_LEN,
            });
        }
        Ok(())
    }

    pub(crate) fn check_oversize(&self) -> Result<(), Error> {
        let len = self.len();
        if len >= MAX_LEN {
            return Err(Error::Oversize {
                len,
                max_len: MAX_LEN,
            });
        }
        Ok(())
    }

    pub(crate) fn check_boundary(&self, index: usize) -> Result<(), Error> {
        let len = self.len();
        if index >= len {
            return Err(Error::OutOfBoundary { index, len });
        }
        Ok(())
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
    /// # use confined::Confined;
    /// let vec = Confined::<Vec<u8>, 2, 5>::try_from(vec![1, 2, 3]);
    /// assert!(vec.is_ok());
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
        let () = Bounds::<MIN_LEN, MAX_LEN>::VALID;
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
    /// # use confined::Confined;
    /// let narrow = Confined::<Vec<u8>, 1, 3>::try_from(vec![1, 2]).unwrap();
    /// let wide: Confined<Vec<u8>, 0, 255> = narrow.widen();
    /// assert_eq!(wide.len_u8(), 2);
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
        let () = Widens::<MIN_LEN, MAX_LEN, MIN_LEN2, MAX_LEN2>::VALID;
        Confined(self.0)
    }

    /// Tries to construct a confinement with a collection of elements taken
    /// from an iterator. Fails if the number of items in the collection
    /// exceeds one of the confinement bounds.
    ///
    /// The iterator is not read past the item with which the collection
    /// exceeds the upper bound, and [`Error::Oversize`] reports the length of
    /// the collection at that moment.
    pub fn try_from_iter<I: IntoIterator<Item = C::Item>>(iter: I) -> Result<Self, Error> {
        let mut col = C::with_capacity(MIN_LEN);
        for item in iter {
            col.push(item);
            let len = col.len();
            if len > MAX_LEN {
                return Err(Error::Oversize {
                    len,
                    max_len: MAX_LEN,
                });
            }
        }
        Self::try_from(col)
    }

    /// Construct a confinement with a collection of elements taken from an
    /// iterator.
    ///
    /// # Panics
    ///
    /// Panics if the number of items in the collection exceeds one
    /// of the confinement bounds.
    pub fn from_iter_checked<I: IntoIterator<Item = C::Item>>(iter: I) -> Self {
        Self::try_from_iter(iter).expect("collection size mismatch, use try_from_iter instead")
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
    /// The length of a [`String`](alloc::string::String) is measured in bytes,
    /// so a `char` is refused when its UTF-8 encoding doesn't fit the space
    /// left, which may happen before the string reaches the maximal length.
    /// [`Error::Oversize`] reports the length of the collection before the
    /// call.
    pub fn push(&mut self, elem: C::Item) -> Result<(), Error>
    where
        C: ItemLen,
    {
        let len = self.len();
        if len.saturating_add(C::item_len(&elem)) > MAX_LEN {
            return Err(Error::Oversize {
                len,
                max_len: MAX_LEN,
            });
        }
        self.0.push(elem);
        Ok(())
    }

    /// Attempts to add all elements from an iterator to the confined
    /// collection. Fails if the number of elements in the collection
    /// already maximal.
    ///
    /// The elements are added one by one with [`Self::push`]. The method stops
    /// at the first element which doesn't fit; the elements before it remain
    /// in the collection.
    pub fn extend<T: IntoIterator<Item = C::Item>>(&mut self, iter: T) -> Result<(), Error>
    where
        C: ItemLen,
    {
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
    pub fn with_mut(mut self, f: impl FnOnce(&mut Self)) -> Result<Self, Error> {
        f(&mut self);
        Self::does_fit_confinement(&self)?;
        Ok(self)
    }

    /// Allows mutation of the confinement, checking that the mutation does not
    /// break confinement requirements.
    pub fn as_mut_checked(&mut self, f: impl FnOnce(&mut Self)) {
        f(self);
        Self::does_fit_confinement(self).expect("confinement broken");
    }
}

impl<C: KeyedCollection, const MIN_LEN: usize, const MAX_LEN: usize> Confined<C, MIN_LEN, MAX_LEN> {
    /// Gets mutable reference to an element of the collection.
    pub fn get_mut(&mut self, key: &C::Key) -> Option<&mut C::Value> {
        self.0.get_mut(key)
    }

    /// Inserts a new value into the confined collection under a given key.
    ///
    /// If the key is already present, the value is overwritten and the previous
    /// value is returned. This succeeds even when the collection is at
    /// capacity.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Oversize`] when the key is not already present and the
    /// collection has reached `MAX_LEN`.
    pub fn insert(&mut self, key: C::Key, value: C::Value) -> Result<Option<C::Value>, Error> {
        if !self.0.contains_key(&key) {
            self.check_oversize()?;
        }
        Ok(self.0.insert(key, value))
    }

    /// Gets the given key's corresponding entry in the map for in-place
    /// manipulation. Errors if the collection entry is vacant and the
    /// collection has already reached maximal size of its confinement.
    ///
    /// The upper bound is checked here, so the methods of the returned
    /// [`Entry`] which insert a value can't fail.
    ///
    /// ```
    /// # use std::collections::BTreeMap;
    /// # use confined::Confined;
    /// let mut map = Confined::<BTreeMap<u8, u8>, 1, 3>::with_key_value(1, 10);
    /// *map.entry(1).unwrap().or_insert(0) += 1;
    /// assert_eq!(map[&1], 11);
    /// ```
    ///
    /// The entry can't be removed, since the map might get below its lower
    /// bound:
    ///
    /// ```compile_fail,E0599
    /// # use std::collections::BTreeMap;
    /// # use confined::Confined;
    /// let mut map = Confined::<BTreeMap<u8, u8>, 1, 3>::with_key_value(1, 10);
    /// map.entry(1).unwrap().remove();
    /// ```
    ///
    /// Neither can it be used as an entry of the wrapped map, which is able to
    /// remove itself:
    ///
    /// ```compile_fail,E0308
    /// # use std::collections::{BTreeMap, btree_map};
    /// # use confined::Confined;
    /// let mut map = Confined::<BTreeMap<u8, u8>, 1, 3>::with_key_value(1, 10);
    /// if let btree_map::Entry::Occupied(entry) = map.entry(1).unwrap() {
    ///     entry.remove();
    /// }
    /// ```
    pub fn entry(&mut self, key: C::Key) -> Result<Entry<'_, C>, Error> {
        if !self.0.contains_key(&key) {
            self.check_oversize()?;
        }
        Ok(Entry(self.0.entry(key)))
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
    pub fn try_retain(
        &mut self,
        mut f: impl FnMut(&C::Key, &mut C::Value) -> bool,
    ) -> Result<(), Error> {
        let mut removable = self.len().saturating_sub(MIN_LEN);
        let mut kept_rejected = 0usize;
        self.0.retain(|key, value| {
            if f(key, value) {
                return true;
            }
            if removable == 0 {
                kept_rejected += 1;
                return true;
            }
            removable -= 1;
            false
        });
        if kept_rejected > 0 {
            return Err(Error::Undersize {
                len: self.len() - kept_rejected,
                min_len: MIN_LEN,
            });
        }
        Ok(())
    }
}
