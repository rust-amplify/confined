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

#![cfg(feature = "alloc")]

extern crate alloc;

use alloc::alloc::{AllocError, Allocator, Global};
use alloc::rc::Rc;
use alloc::vec::Vec;
use core::alloc::Layout;
use core::cell::Cell;
use core::ptr::NonNull;

use confined::{Confined, Error, TinyBlob, TinyVec};

#[derive(Clone, Debug)]
struct CountingAllocator {
    allocations: Rc<Cell<usize>>,
    deallocations: Rc<Cell<usize>>,
}

impl CountingAllocator {
    fn new() -> Self {
        Self {
            allocations: Rc::new(Cell::new(0)),
            deallocations: Rc::new(Cell::new(0)),
        }
    }

    fn assert_balanced(&self) {
        assert_eq!(self.allocations.get(), self.deallocations.get());
    }
}

// SAFETY: Allocations and deallocations use Global with unchanged layouts and
// pointers.
unsafe impl Allocator for CountingAllocator {
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        let allocation = Global.allocate(layout)?;
        self.allocations.set(self.allocations.get() + 1);
        Ok(allocation)
    }

    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout) {
        // SAFETY: The caller provides a live allocation and its matching
        // layout.
        unsafe { Global.deallocate(ptr, layout) };
        self.deallocations.set(self.deallocations.get() + 1);
    }
}

fn assert_allocator_identity(vec: &Vec<u8, CountingAllocator>, allocator: &CountingAllocator) {
    assert!(Rc::ptr_eq(
        &vec.allocator().allocations,
        &allocator.allocations,
    ));
    assert!(Rc::ptr_eq(
        &vec.allocator().deallocations,
        &allocator.deallocations,
    ));
}

#[test]
fn new_in_accepts_a_non_default_allocator() {
    let allocator = CountingAllocator::new();
    let mut vec = Confined::<Vec<u8, CountingAllocator>, 0, 4>::new_in(allocator.clone());
    assert!(vec.is_empty());
    assert_eq!(vec.len(), 0);
    assert_eq!(vec.as_slice(), &[] as &[u8]);

    vec.push(1).unwrap();
    assert_eq!(vec.as_slice(), &[1]);
    assert!(allocator.allocations.get() > 0);
    drop(vec);
    allocator.assert_balanced();
}

#[test]
fn with_capacity_in_reserves_without_changing_length() {
    let allocator = CountingAllocator::new();
    for capacity in [0, 4] {
        let vec = Confined::<Vec<u8, CountingAllocator>, 0, 4>::with_capacity_in(
            capacity,
            allocator.clone(),
        );
        assert!(vec.is_empty());
        assert_eq!(vec.len(), 0);
        let inner = vec.into_vec();
        assert!(inner.capacity() >= capacity);
        assert_allocator_identity(&inner, &allocator);
        drop(inner);
        allocator.assert_balanced();
    }
    assert!(allocator.allocations.get() > 0);
}

#[test]
fn with_in_initializes_one_item_with_a_non_default_allocator() {
    let allocator = CountingAllocator::new();
    let vec = Confined::<Vec<u8, CountingAllocator>, 1, 4>::with_in(7, allocator.clone());
    assert_eq!(vec.len(), 1);
    assert_eq!(vec.as_slice(), &[7]);
    drop(vec);

    let fixed = Confined::<Vec<u8, CountingAllocator>, 1, 1>::with_in(9, allocator.clone());
    assert_eq!(fixed.as_slice(), &[9]);
    drop(fixed);
    allocator.assert_balanced();
}

#[test]
fn try_from_iter_in_checks_both_bounds() {
    let allocator = CountingAllocator::new();
    for len in 2u8..=4 {
        let vec = Confined::<Vec<u8, CountingAllocator>, 2, 4>::try_from_iter_in(
            (0..len).filter(|_| true),
            allocator.clone(),
        )
        .unwrap();
        assert_eq!(vec.len(), usize::from(len));
        assert!(vec.as_slice().iter().copied().eq(0..len));
        drop(vec);
        allocator.assert_balanced();
    }

    for len in [0u8, 1, 5] {
        assert!(
            Confined::<Vec<u8, CountingAllocator>, 2, 4>::try_from_iter_in(
                (0..len).filter(|_| true),
                allocator.clone(),
            )
            .is_err()
        );
        allocator.assert_balanced();
    }
}

#[test]
fn try_from_iter_in_stops_reading_at_overflow() {
    let allocator = CountingAllocator::new();
    let reads = Cell::new(0);
    let iter = (0u8..).inspect(|_| reads.set(reads.get() + 1));
    let result =
        Confined::<Vec<u8, CountingAllocator>, 0, 2>::try_from_iter_in(iter, allocator.clone());
    assert_eq!(result.unwrap_err(), Error::Oversize { len: 3, max_len: 2 });
    assert_eq!(reads.get(), 3);
    allocator.assert_balanced();
}

#[test]
fn from_iter_checked_in_accepts_exact_size_iterators_at_both_bounds() {
    let allocator = CountingAllocator::new();
    for len in [2u8, 4] {
        let vec = Confined::<Vec<u8, CountingAllocator>, 2, 4>::from_iter_checked_in(
            0..len,
            allocator.clone(),
        );
        assert_eq!(vec.len(), usize::from(len));
        assert!(vec.as_slice().iter().copied().eq(0..len));
        drop(vec);
        allocator.assert_balanced();
    }
}

#[test]
#[should_panic]
fn from_iter_checked_in_panics_below_the_minimum() {
    let _ = Confined::<Vec<u8, CountingAllocator>, 2, 4>::from_iter_checked_in(
        0u8..1,
        CountingAllocator::new(),
    );
}

#[test]
#[should_panic]
fn from_iter_checked_in_panics_above_the_maximum() {
    let _ = Confined::<Vec<u8, CountingAllocator>, 2, 4>::from_iter_checked_in(
        0u8..5,
        CountingAllocator::new(),
    );
}

#[test]
fn zero_upper_bound_only_allows_empty_values() {
    type Empty = Confined<Vec<u8, CountingAllocator>, 0, 0>;

    let allocator = CountingAllocator::new();
    let mut vec = Empty::new_in(allocator.clone());
    assert!(vec.push(1).is_err());
    assert!(vec.extend([1]).is_err());
    vec.extend(core::iter::empty::<u8>()).unwrap();
    assert!(vec.remove(0).is_err());
    assert_eq!(vec.pop(), None);
    vec.clear();
    assert!(vec.is_empty());
    drop(vec);

    let empty = Empty::try_from_iter_in(core::iter::empty::<u8>(), allocator.clone()).unwrap();
    assert!(empty.is_empty());
    drop(empty);
    assert!(Empty::try_from_iter_in([1], allocator.clone()).is_err());

    let checked = Empty::from_iter_checked_in(core::iter::empty::<u8>(), allocator.clone());
    assert!(checked.is_empty());
    drop(checked);
    allocator.assert_balanced();
}

#[test]
fn push_stops_at_the_upper_bound() {
    let allocator = CountingAllocator::new();
    let mut vec =
        Confined::<Vec<u8, CountingAllocator>, 2, 4>::try_from_iter_in([1, 2], allocator.clone())
            .unwrap();
    vec.push(3).unwrap();
    vec.push(4).unwrap();
    assert!(vec.push(5).is_err());
    assert_eq!(vec.len(), 4);
    assert_eq!(vec.as_slice(), &[1, 2, 3, 4]);
    drop(vec);
    allocator.assert_balanced();
}

#[test]
fn extend_respects_the_upper_bound_even_on_failure() {
    let allocator = CountingAllocator::new();
    let mut vec =
        Confined::<Vec<u8, CountingAllocator>, 2, 4>::try_from_iter_in([1, 2], allocator.clone())
            .unwrap();
    vec.extend([3]).unwrap();
    assert_eq!(vec.as_slice(), &[1, 2, 3]);
    assert!(vec.extend([4, 5]).is_err());
    assert!((3..=4).contains(&vec.len()));
    assert!(vec.as_slice().starts_with(&[1, 2, 3]));
    vec.extend(core::iter::empty::<u8>()).unwrap();
    drop(vec);
    allocator.assert_balanced();
}

#[test]
fn remove_checks_the_index_and_lower_bound() {
    let allocator = CountingAllocator::new();
    let mut vec = Confined::<Vec<u8, CountingAllocator>, 2, 4>::try_from_iter_in(
        [1, 2, 3],
        allocator.clone(),
    )
    .unwrap();
    assert!(vec.remove(3).is_err());
    assert_eq!(vec.as_slice(), &[1, 2, 3]);
    assert_eq!(vec.remove(1).unwrap(), 2);
    assert_eq!(vec.as_slice(), &[1, 3]);
    assert!(vec.remove(0).is_err());
    assert_eq!(vec.as_slice(), &[1, 3]);
    drop(vec);
    allocator.assert_balanced();
}

#[test]
fn pop_and_clear_allow_reusing_an_empty_vector() {
    let allocator = CountingAllocator::new();
    let mut vec = Confined::<Vec<u8, CountingAllocator>, 0, 4>::try_from_iter_in(
        [1, 2, 3],
        allocator.clone(),
    )
    .unwrap();
    assert_eq!(vec.pop(), Some(3));
    assert_eq!(vec.as_slice(), &[1, 2]);
    vec.clear();
    assert!(vec.is_empty());
    assert_eq!(vec.pop(), None);
    assert!(vec.remove(0).is_err());
    vec.push(4).unwrap();
    assert_eq!(vec.pop(), Some(4));
    assert_eq!(vec.pop(), None);
    drop(vec);
    allocator.assert_balanced();
}

#[test]
fn unwrapping_preserves_the_allocator_and_balances_allocations_after_drop() {
    type Bounded = Confined<Vec<u8, CountingAllocator>, 2, 4>;

    let allocator = CountingAllocator::new();
    let vec = Bounded::try_from_iter_in([1, 2], allocator.clone())
        .unwrap()
        .into_vec();
    assert_eq!(vec.as_slice(), &[1, 2]);
    assert_allocator_identity(&vec, &allocator);
    assert!(allocator.allocations.get() > allocator.deallocations.get());
    drop(vec);
    allocator.assert_balanced();

    let inner = Bounded::try_from_iter_in([3, 4], allocator.clone())
        .unwrap()
        .to_unconfined();
    assert_eq!(inner.as_slice(), &[3, 4]);
    assert_allocator_identity(&inner, &allocator);
    drop(inner);
    allocator.assert_balanced();

    let released = Bounded::try_from_iter_in([5, 6], allocator.clone())
        .unwrap()
        .release();
    assert_eq!(released.as_slice(), &[5, 6]);
    assert_allocator_identity(&released, &allocator);
    drop(released);
    allocator.assert_balanced();
}

#[test]
fn tiny_aliases_accept_custom_and_default_allocators() {
    let allocator = CountingAllocator::new();
    let mut vec: TinyVec<u16, CountingAllocator> = TinyVec::new_in(allocator.clone());
    vec.extend([1, 2, 3]).unwrap();
    assert_eq!(vec.as_slice(), &[1, 2, 3]);
    drop(vec);

    let mut blob: TinyBlob<CountingAllocator> = TinyBlob::new_in(allocator.clone());
    blob.extend([4, 5, 6]).unwrap();
    assert_eq!(blob.as_slice(), &[4, 5, 6]);
    drop(blob);
    allocator.assert_balanced();

    let mut default_vec: TinyVec<u16> = TinyVec::new_in(Global);
    default_vec.push(7).unwrap();
    assert_eq!(default_vec.as_slice(), &[7]);
    let mut default_blob: TinyBlob = TinyBlob::new_in(Global);
    default_blob.push(8).unwrap();
    assert_eq!(default_blob.as_slice(), &[8]);
}

#[cfg(feature = "hex")]
#[test]
fn hex_supports_allocator_aware_bytes() {
    use amplify_num::hex::FromHex;

    let allocator = CountingAllocator::new();
    let bytes = Confined::<Vec<u8, CountingAllocator>, 1, 2>::try_from_iter_in(
        [0xab, 0xcd],
        allocator.clone(),
    )
    .unwrap();
    assert_eq!(format!("{bytes:x}"), "abcd");
    assert_eq!(format!("{bytes:X}"), "ABCD");
    drop(bytes);
    allocator.assert_balanced();

    let bytes = Confined::<Vec<u8, std::alloc::System>, 1, 2>::from_hex("abcd").unwrap();
    assert_eq!(bytes.as_slice(), &[0xab, 0xcd]);
    assert!(Confined::<Vec<u8, std::alloc::System>, 1, 2>::from_hex("").is_err());
    assert!(Confined::<Vec<u8, std::alloc::System>, 1, 2>::from_hex("abcdef").is_err());
}

#[test]
fn errors_implement_core_error() {
    fn assert_error<T: core::error::Error>() {}

    assert_error::<Error>();
}

#[cfg(feature = "std")]
#[test]
fn system_allocator_supports_construction_and_mutation() {
    use std::alloc::System;

    type Bounded = Confined<Vec<u8, System>, 2, 4>;

    let mut vec = Confined::<Vec<u8, System>, 0, 4>::new_in(System);
    vec.push(1).unwrap();
    vec.extend([2, 3, 4]).unwrap();
    assert!(vec.push(5).is_err());
    assert_eq!(vec.remove(1).unwrap(), 2);
    assert_eq!(vec.pop(), Some(4));
    vec.clear();
    assert!(vec.is_empty());
    assert_eq!(vec.pop(), None);
    let inner: Vec<u8, System> = vec.into_vec();
    assert!(inner.is_empty());

    let reserved = Confined::<Vec<u8, System>, 0, 4>::with_capacity_in(4, System);
    assert!(reserved.is_empty());
    assert!(reserved.into_vec().capacity() >= 4);
    assert_eq!(
        Confined::<Vec<u8, System>, 1, 4>::with_in(7, System).as_slice(),
        &[7]
    );
    assert_eq!(
        Bounded::try_from_iter_in([1, 2], System)
            .unwrap()
            .as_slice(),
        &[1, 2],
    );
    assert!(Bounded::try_from_iter_in([1], System).is_err());
    assert!(Bounded::try_from_iter_in([1, 2, 3, 4, 5], System).is_err());
    assert_eq!(Bounded::from_iter_checked_in(0u8..4, System).as_slice(), &[
        0, 1, 2, 3
    ],);
}

#[cfg(feature = "std")]
fn assert_write_bounds<A: Allocator>(allocator: A) {
    use std::io::Write;

    let mut blob = Confined::<Vec<u8, A>, 1, 4>::with_in(1, allocator);
    assert_eq!(blob.write(&[2]).unwrap(), 1);
    blob.write_all(&[3, 4]).unwrap();
    assert_eq!(blob.as_slice(), &[1, 2, 3, 4]);
    assert_eq!(blob.write(&[]).unwrap(), 0);
    assert!(blob.write_all(&[5]).is_err());
    assert_eq!(blob.as_slice(), &[1, 2, 3, 4]);

    let mut blob = blob.widen::<0, 4>();
    blob.clear();
    assert!(blob.write_all(&[1, 2, 3, 4, 5]).is_err());
    assert!(blob.len() <= 4);
}

#[cfg(feature = "std")]
#[test]
fn write_supports_the_system_allocator_and_enforces_bounds() {
    assert_write_bounds(std::alloc::System);
}

#[cfg(feature = "std")]
#[test]
fn write_supports_a_non_default_allocator_and_balances_allocations() {
    let allocator = CountingAllocator::new();
    assert_write_bounds(allocator.clone());
    assert!(allocator.allocations.get() > 0);
    allocator.assert_balanced();
}

#[derive(Clone, Debug, Default)]
struct RefusingAllocator {
    attempts: Rc<Cell<usize>>,
}

// SAFETY: This allocator never returns an allocation, so it never supplies a
// pointer which could require deallocation.
unsafe impl Allocator for RefusingAllocator {
    fn allocate(&self, _: Layout) -> Result<NonNull<[u8]>, AllocError> {
        self.attempts.set(self.attempts.get() + 1);
        Err(AllocError)
    }

    unsafe fn deallocate(&self, _: NonNull<u8>, _: Layout) {
        unreachable!("refusing allocator never allocated memory");
    }
}

#[test]
fn allocation_failure_is_a_capacity_error_for_push_and_extend() {
    let allocator = RefusingAllocator::default();
    let mut vec = Confined::<Vec<u8, RefusingAllocator>, 0, 4>::new_in(allocator.clone());
    assert_eq!(allocator.attempts.get(), 0);
    assert!(matches!(vec.push(1), Err(Error::CapacityExceeded { .. })));
    assert!(allocator.attempts.get() > 0);
    assert!(vec.is_empty());
    let attempts = allocator.attempts.get();
    assert!(matches!(
        vec.extend([1, 2]),
        Err(Error::CapacityExceeded { .. })
    ));
    assert!(allocator.attempts.get() > attempts);
    assert!(vec.is_empty());
    assert_eq!(vec.extend([]), Ok(()));
}

#[test]
fn fallible_constructors_report_refused_allocations() {
    let allocator = RefusingAllocator::default();
    type Bounded = Confined<Vec<u8, RefusingAllocator>, 0, 4>;

    assert!(matches!(
        Bounded::try_from_iter_in([1, 2], allocator.clone()),
        Err(Error::CapacityExceeded { .. })
    ));
    assert!(allocator.attempts.get() > 0);
    let attempts = allocator.attempts.get();
    assert!(matches!(
        Bounded::try_with_capacity_in(1, allocator.clone()),
        Err(Error::CapacityExceeded { .. })
    ));
    assert!(allocator.attempts.get() > attempts);
    let attempts = allocator.attempts.get();
    assert!(
        Bounded::try_with_capacity_in(0, allocator.clone())
            .unwrap()
            .is_empty()
    );
    assert!(
        Bounded::try_from_iter_in([], allocator.clone())
            .unwrap()
            .is_empty()
    );
    assert_eq!(allocator.attempts.get(), attempts);
}

#[test]
fn huge_minimum_with_an_empty_iterator_makes_no_allocation_attempt() {
    let allocator = RefusingAllocator::default();
    assert!(matches!(
        Confined::<Vec<u8, RefusingAllocator>, { usize::MAX }, { usize::MAX }>::try_from_iter_in(
            [],
            allocator.clone(),
        ),
        Err(Error::Undersize {
            len: 0,
            min_len: usize::MAX
        })
    ));
    assert_eq!(allocator.attempts.get(), 0);
}

#[test]
fn fallible_capacity_constructors_cap_the_request_to_maximum() {
    let vec = Confined::<Vec<u8>, 0, 4>::try_with_capacity(usize::MAX).unwrap();
    assert!(vec.is_empty());
    assert!(vec.as_unconfined().capacity() >= 4);
    assert!(vec.as_unconfined().capacity() <= 16);

    let allocator = CountingAllocator::new();
    let vec = Confined::<Vec<u8, CountingAllocator>, 0, 4>::try_with_capacity_in(
        usize::MAX,
        allocator.clone(),
    )
    .unwrap();
    assert!(vec.is_empty());
    assert!(vec.as_unconfined().capacity() >= 4);
    assert!(vec.as_unconfined().capacity() <= 16);
    assert_allocator_identity(vec.as_unconfined(), &allocator);
    drop(vec);
    allocator.assert_balanced();

    let allocator = RefusingAllocator::default();
    let vec = Confined::<Vec<u8, RefusingAllocator>, 0, 0>::try_with_capacity_in(
        usize::MAX,
        allocator.clone(),
    )
    .unwrap();
    assert!(vec.is_empty());
    assert_eq!(vec.as_unconfined().capacity(), 0);
    assert_eq!(allocator.attempts.get(), 0);
}

#[test]
fn fallible_reservation_checks_confinement_before_the_allocator() {
    let allocator = RefusingAllocator::default();
    let mut vec = Confined::<Vec<u8, RefusingAllocator>, 0, 4>::new_in(allocator.clone());
    assert_eq!(vec.try_reserve(0), Ok(()));
    assert!(matches!(vec.try_reserve(5), Err(Error::Oversize { .. })));
    assert!(matches!(
        vec.try_reserve(usize::MAX),
        Err(Error::Oversize { .. })
    ));
    assert_eq!(allocator.attempts.get(), 0);
    assert!(matches!(
        vec.try_reserve(1),
        Err(Error::CapacityExceeded { .. })
    ));
    assert!(allocator.attempts.get() > 0);
    assert!(vec.is_empty());

    let mut vec = Confined::<Vec<u8>, 0, 4>::try_from_iter([1]).unwrap();
    assert_eq!(vec.try_reserve(3), Ok(()));
    assert!(matches!(vec.try_reserve(4), Err(Error::Oversize { .. })));
    assert!(matches!(
        vec.try_reserve(usize::MAX),
        Err(Error::Oversize { .. })
    ));
    assert_eq!(vec.as_slice(), &[1]);
}

#[cfg(feature = "std")]
#[test]
fn io_write_reports_refused_allocations_without_aborting() {
    use std::io::Write;

    let allocator = RefusingAllocator::default();
    let mut bytes = Confined::<Vec<u8, RefusingAllocator>, 0, 4>::new_in(allocator.clone());
    assert_eq!(bytes.write(&[]).unwrap(), 0);
    assert_eq!(allocator.attempts.get(), 0);
    assert!(bytes.write(&[1]).is_err());
    assert!(allocator.attempts.get() > 0);
    assert!(bytes.is_empty());
    assert!(bytes.write_all(&[1, 2]).is_err());
    assert!(bytes.is_empty());
    let attempts = allocator.attempts.get();
    assert!(bytes.write_all(&[1, 2, 3, 4, 5]).is_err());
    assert_eq!(allocator.attempts.get(), attempts);
    assert!(bytes.is_empty());
}
