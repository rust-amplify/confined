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
use core::str::FromStr;

use collections::{Collection, KeyedCollection};

use super::Error;

/// The confinement for the collection.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(crate = "serde")
)]
pub struct Confined<C: Collection, const MIN_LEN: usize, const MAX_LEN: usize>(pub(crate) C);

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

impl<C: Collection, const MIN_LEN: usize, const MAX_LEN: usize> FromStr
    for Confined<C, MIN_LEN, MAX_LEN>
where
    C: FromStr,
{
    type Err = C::Err;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        C::from_str(s).map(Self)
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

    #[deprecated(since = "4.7.0", note = "use `from_checked`")]
    pub fn from_collection_unsafe(col: C) -> Self {
        Self::from_checked(col)
    }

    /// Tries to construct a confinement over a collection. Fails if the number
    /// of items in the collection exceeds one of the confinement bounds.
    // We can't use `impl TryFrom` due to the conflict with core library blanked
    // implementation
    pub fn try_from(col: C) -> Result<Self, Error> {
        Self::does_fit_confinement(&col)?;
        Ok(Self(col))
    }

    /// Tries to construct a confinement with a collection of elements taken
    /// from an iterator. Fails if the number of items in the collection
    /// exceeds one of the confinement bounds.
    pub fn try_from_iter<I: IntoIterator<Item = C::Item>>(iter: I) -> Result<Self, Error> {
        let mut col = C::with_capacity(MIN_LEN);
        for item in iter {
            col.push(item);
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
        Self::from_checked(iter.into_iter().collect())
    }

    #[deprecated(since = "4.7.0", note = "use `from_iter_checked`")]
    pub fn from_iter_unsafe<I: IntoIterator<Item = C::Item>>(iter: I) -> Self {
        Self::from_iter_checked(iter)
    }

    /// Returns inner collection type
    #[deprecated(since = "4.7.0", note = "use as_unconfined method")]
    pub fn as_inner(&self) -> &C {
        &self.0
    }

    /// Returns reference to the inner collection type.
    pub fn as_unconfined(&self) -> &C {
        &self.0
    }

    /// Clones inner collection type and returns it
    #[deprecated(since = "4.7.0", note = "use to_unconfined method")]
    pub fn to_inner(&self) -> C
    where
        C: Clone,
    {
        self.0.clone()
    }

    /// Clones inner collection and returns an unconfined version of it.
    pub fn to_unconfined(&self) -> C
    where
        C: Clone,
    {
        self.0.clone()
    }

    /// Decomposes into the inner collection type
    #[deprecated(since = "4.7.0", note = "use release method")]
    pub fn into_inner(self) -> C {
        self.0
    }

    /// Attempts to add a single element to the confined collection. Fails if
    /// the number of elements in the collection already maximal.
    pub fn push(&mut self, elem: C::Item) -> Result<(), Error> {
        self.check_oversize()?;
        self.0.push(elem);
        Ok(())
    }

    /// Attempts to add all elements from an iterator to the confined
    /// collection. Fails if the number of elements in the collection
    /// already maximal.
    pub fn extend<T: IntoIterator<Item = C::Item>>(&mut self, iter: T) -> Result<(), Error> {
        for elem in iter {
            self.push(elem)?;
        }
        Ok(())
    }

    /// Removes confinement and returns the underlying collection.
    #[deprecated(since = "4.7.0", note = "use release method")]
    pub fn unbox(self) -> C {
        self.0
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

    // TODO: This is strange; it doesn't need to check the bound and error!
    /// Gets the given key's corresponding entry in the map for in-place
    /// manipulation. Errors if the collection entry is vacant and the
    /// collection has already reached maximal size of its confinement.
    pub fn entry(&mut self, key: C::Key) -> Result<C::Entry<'_>, Error> {
        if !self.0.contains_key(&key) {
            self.check_oversize()?;
        }
        Ok(self.0.entry(key))
    }

    /// Retains only the elements specified by the predicate.
    ///
    /// In other words, remove all pairs `(k, v)` for which `f(&k, &mut v)`
    /// returns `false`. The elements are visited in unsorted (and
    /// unspecified) order.
    ///
    /// # Errors
    ///
    /// Errors if the minimum confinement is not met after the retain operation.
    pub fn try_retain(
        &mut self,
        f: impl FnMut(&C::Key, &mut C::Value) -> bool,
    ) -> Result<(), Error> {
        self.0.retain(f);
        let len = self.len();
        if len < MIN_LEN {
            return Err(Error::Undersize {
                len,
                min_len: MIN_LEN,
            });
        }
        Ok(())
    }
}
