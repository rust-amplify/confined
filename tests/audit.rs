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

use std::cell::Cell;
use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::panic::{self, AssertUnwindSafe};
use std::rc::Rc;

use collections::CollectionBase;
use confined::{Confined, Error, ItemLen};

struct Misreported<I> {
    inner: I,
    reported: usize,
}

impl<I: Iterator> Iterator for Misreported<I> {
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.reported, Some(self.reported))
    }
}

impl<I: Iterator> ExactSizeIterator for Misreported<I> {}

impl<I: DoubleEndedIterator> DoubleEndedIterator for Misreported<I> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.inner.next_back()
    }
}

#[test]
fn checked_iterator_constructors_do_not_trust_size_hints() {
    for reported in [0, 1, 2, usize::MAX] {
        let iter = Misreported {
            inner: [1u8, 2].into_iter(),
            reported,
        };
        let vec = Confined::<Vec<u8>, 2, 2>::from_iter_checked(iter);
        assert_eq!(vec.as_slice(), &[1, 2]);

        let iter = Misreported {
            inner: [1u8, 2].into_iter(),
            reported,
        };
        let vec = Confined::<Vec<u8, std::alloc::System>, 2, 2>::from_iter_checked_in(
            iter,
            std::alloc::System,
        );
        assert_eq!(vec.as_slice(), &[1, 2]);

        let iter = Misreported {
            inner: ['a', '€'].into_iter(),
            reported,
        };
        let string = Confined::<String, 4, 4>::from_iter_checked(iter);
        assert_eq!(string.as_str(), "a€");
        assert_eq!(string.len(), 4);
    }
}

#[test]
fn checked_iterator_constructors_validate_the_actual_lower_bound() {
    let iter = Misreported {
        inner: [1u8].into_iter(),
        reported: 2,
    };
    assert!(
        panic::catch_unwind(|| { Confined::<Vec<u8>, 2, 3>::from_iter_checked(iter) }).is_err()
    );

    let iter = Misreported {
        inner: ['a'].into_iter(),
        reported: 3,
    };
    assert!(panic::catch_unwind(|| { Confined::<String, 3, 4>::from_iter_checked(iter) }).is_err());
}

#[test]
fn checked_iterator_constructors_stop_even_when_the_length_is_underreported() {
    let reads = Cell::new(0);
    let iter = Misreported {
        inner: core::iter::repeat(7u8).inspect(|_| {
            reads.set(reads.get() + 1);
            assert!(reads.get() <= 16, "constructor did not stop at MAX + 1");
        }),
        reported: 0,
    };
    let result = panic::catch_unwind(AssertUnwindSafe(|| {
        Confined::<Vec<u8>, 0, 2>::from_iter_checked(iter)
    }));
    assert!(result.is_err());
    assert_eq!(reads.get(), 3);

    reads.set(0);
    let iter = Misreported {
        inner: core::iter::repeat('€').inspect(|_| {
            reads.set(reads.get() + 1);
            assert!(reads.get() <= 16, "constructor did not stop at byte bound");
        }),
        reported: 0,
    };
    let result = panic::catch_unwind(AssertUnwindSafe(|| {
        Confined::<String, 0, 4>::from_iter_checked(iter)
    }));
    assert!(result.is_err());
    assert_eq!(reads.get(), 2);
}

#[test]
fn iterator_constructors_do_not_allocate_from_the_minimum() {
    assert!(matches!(
        Confined::<Vec<u8>, { usize::MAX }, { usize::MAX }>::try_from_iter([]),
        Err(Error::Undersize {
            len: 0,
            min_len: usize::MAX
        })
    ));
    assert!(matches!(
        Confined::<String, { usize::MAX }, { usize::MAX }>::try_from_iter([]),
        Err(Error::Undersize {
            len: 0,
            min_len: usize::MAX
        })
    ));
}

#[derive(Debug)]
struct NearMaximum {
    len: usize,
    pushes: Rc<Cell<usize>>,
}

impl CollectionBase for NearMaximum {
    type Item = ();

    fn item_count(&self) -> usize {
        self.len
    }
    fn push(&mut self, _: Self::Item) {
        self.pushes.set(self.pushes.get() + 1);
        self.len = self.len.saturating_add(2);
    }
    fn clear(&mut self) {
        self.len = 0;
    }
}

impl ItemLen for NearMaximum {
    fn item_len(_: &Self::Item) -> usize {
        2
    }
}

#[test]
fn overflowing_growth_is_rejected_before_underlying_push() {
    let pushes = Rc::new(Cell::new(0));
    let inner = NearMaximum {
        len: usize::MAX - 1,
        pushes: pushes.clone(),
    };
    let mut col = Confined::<_, 0, { usize::MAX }>::try_from(inner).unwrap();
    assert!(matches!(col.push(()), Err(Error::Oversize { .. })));
    assert_eq!(col.len(), usize::MAX - 1);
    assert_eq!(pushes.get(), 0);
    assert!(matches!(col.extend([()]), Err(Error::Oversize { .. })));
    assert_eq!(col.len(), usize::MAX - 1);
    assert_eq!(pushes.get(), 0);
}

#[derive(Debug)]
struct CountClone<'a>(&'a Cell<usize>);

impl Clone for CountClone<'_> {
    fn clone(&self) -> Self {
        self.0.set(self.0.get() + 1);
        Self(self.0)
    }
}

#[test]
fn slice_bounds_are_checked_before_cloning() {
    let clones = Cell::new(0);
    let items = [CountClone(&clones), CountClone(&clones)];
    assert!(matches!(
        Confined::<Vec<CountClone<'_>>, 0, 1>::try_from_slice(&items),
        Err(Error::Oversize { len: 2, max_len: 1 })
    ));
    assert_eq!(clones.get(), 0);
    assert!(matches!(
        Confined::<Vec<CountClone<'_>>, 3, 4>::try_from_slice(&items),
        Err(Error::Undersize { len: 2, min_len: 3 })
    ));
    assert_eq!(clones.get(), 0);

    let vec = Confined::<Vec<CountClone<'_>>, 2, 2>::try_from_slice(&items).unwrap();
    assert_eq!(vec.len(), 2);
    assert_eq!(clones.get(), 2);
}

#[test]
fn string_constructors_use_bytes_instead_of_character_count() {
    assert_eq!(
        Confined::<String, 0, 3>::try_from_iter(['a', '€']),
        Err(Error::Oversize { len: 4, max_len: 3 })
    );
    assert_eq!(
        Confined::<String, 4, 8>::try_from_iter(['€']),
        Err(Error::Undersize { len: 3, min_len: 4 })
    );
    let string = Confined::<String, 4, 4>::try_from_iter(['a', '€']).unwrap();
    assert_eq!(string.as_str(), "a€");
    assert_eq!(string.len(), 4);
}

#[cfg(feature = "ascii")]
#[test]
fn ascii_iterator_constructors_validate_actual_bytes_without_eager_reservation() {
    use confined::ascii::{AsciiChar, AsciiString};

    for reported in [0, 2, usize::MAX] {
        let iter = Misreported {
            inner: [AsciiChar::a, AsciiChar::b].into_iter(),
            reported,
        };
        let string = Confined::<AsciiString, 2, 2>::from_iter_checked(iter);
        assert_eq!(string.as_str(), "ab");
        assert_eq!(string.len(), 2);
    }
    assert!(matches!(
        Confined::<AsciiString, { usize::MAX }, { usize::MAX }>::try_from_iter([]),
        Err(Error::Undersize {
            len: 0,
            min_len: usize::MAX
        })
    ));
    assert!(matches!(
        Confined::<AsciiString, 0, 1>::try_from_iter([AsciiChar::a, AsciiChar::b]),
        Err(Error::Oversize { len: 2, max_len: 1 })
    ));

    let reads = Cell::new(0);
    let iter = Misreported {
        inner: core::iter::repeat(AsciiChar::a).inspect(|_| {
            reads.set(reads.get() + 1);
            assert!(
                reads.get() <= 16,
                "ASCII constructor did not stop at MAX + 1"
            );
        }),
        reported: 0,
    };
    assert!(
        panic::catch_unwind(AssertUnwindSafe(|| {
            Confined::<AsciiString, 0, 2>::from_iter_checked(iter)
        }))
        .is_err()
    );
    assert_eq!(reads.get(), 3);
}

#[derive(Debug, Default)]
struct Comparisons {
    count: Cell<usize>,
    panic_at: Cell<Option<usize>>,
}

#[derive(Clone, Debug)]
struct PanicOrd {
    value: u8,
    comparisons: Rc<Comparisons>,
}

impl PartialEq for PanicOrd {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl Eq for PanicOrd {}

impl PartialOrd for PanicOrd {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PanicOrd {
    fn cmp(&self, other: &Self) -> Ordering {
        let count = self.comparisons.count.get() + 1;
        self.comparisons.count.set(count);
        assert_ne!(
            self.comparisons.panic_at.get(),
            Some(count),
            "comparison panic"
        );
        self.value.cmp(&other.value)
    }
}

#[expect(
    clippy::mutable_key_type,
    reason = "only diagnostic counters mutate; key ordering is immutable"
)]
fn panic_map<const MIN: usize>(
    keys: impl IntoIterator<Item = u8>,
    comparisons: &Rc<Comparisons>,
) -> Confined<BTreeMap<PanicOrd, u8>, MIN, 4> {
    let inner: BTreeMap<_, _> = keys
        .into_iter()
        .map(|value| {
            (
                PanicOrd {
                    value,
                    comparisons: comparisons.clone(),
                },
                value * 10,
            )
        })
        .collect();
    Confined::try_from(inner).unwrap()
}

#[test]
fn btree_append_preserves_the_minimum_when_any_comparison_panics() {
    let comparisons = Rc::new(Comparisons::default());
    let mut map = panic_map::<2>([1, 2], &comparisons);
    let mut other = panic_map::<0>([2, 3], &comparisons);
    comparisons.count.set(0);
    map.append(&mut other).unwrap();
    let total = comparisons.count.get();
    assert!(total > 0);
    assert_eq!(map.len(), 3);
    assert!(other.is_empty());

    for panic_at in 1..=total {
        let comparisons = Rc::new(Comparisons::default());
        let mut map = panic_map::<2>([1, 2], &comparisons);
        let mut other = panic_map::<0>([2, 3], &comparisons);
        comparisons.count.set(0);
        comparisons.panic_at.set(Some(panic_at));
        let result = panic::catch_unwind(AssertUnwindSafe(|| map.append(&mut other)));
        comparisons.panic_at.set(None);
        assert!(result.is_err(), "comparison {panic_at} did not panic");
        assert!(
            map.len() >= 2,
            "append violated MIN after comparison {panic_at}"
        );
        assert!(map.len() <= 4);
        assert!(other.len() <= 4);
    }
}

#[test]
fn btree_split_errors_preserve_every_key_and_value() {
    let entries = [(1u8, 10u8), (2, 20), (3, 30), (4, 40)];
    let mut map = Confined::<BTreeMap<u8, u8>, 2, 5>::try_from_iter(entries).unwrap();
    assert!(map.split_off::<0, 5>(&2).is_err());
    assert_eq!(
        map.iter().map(|(k, v)| (*k, *v)).collect::<Vec<_>>(),
        entries
    );
    assert!(map.split_off::<1, 5>(&5).is_err());
    assert_eq!(
        map.iter().map(|(k, v)| (*k, *v)).collect::<Vec<_>>(),
        entries
    );
    assert!(map.split_off::<0, 1>(&3).is_err());
    assert_eq!(
        map.iter().map(|(k, v)| (*k, *v)).collect::<Vec<_>>(),
        entries
    );

    let right = map.split_off::<2, 2>(&3).unwrap();
    assert_eq!(
        map.iter().map(|(k, v)| (*k, *v)).collect::<Vec<_>>(),
        &entries[..2]
    );
    assert_eq!(
        right.iter().map(|(k, v)| (*k, *v)).collect::<Vec<_>>(),
        &entries[2..]
    );
}

#[test]
fn btree_split_comparison_panics_do_not_require_a_destructive_rollback() {
    for pivot in [2, 3, 5] {
        let comparisons = Rc::new(Comparisons::default());
        let mut map = panic_map::<2>([1, 2, 3, 4], &comparisons);
        let key = PanicOrd {
            value: pivot,
            comparisons: comparisons.clone(),
        };
        comparisons.count.set(0);
        let result = map.split_off::<1, 3>(&key);
        assert_eq!(result.is_ok(), pivot == 3);
        let total = comparisons.count.get();
        assert!(total > 0);

        for panic_at in 1..=total {
            let comparisons = Rc::new(Comparisons::default());
            let mut map = panic_map::<2>([1, 2, 3, 4], &comparisons);
            let key = PanicOrd {
                value: pivot,
                comparisons: comparisons.clone(),
            };
            comparisons.count.set(0);
            comparisons.panic_at.set(Some(panic_at));
            let result = panic::catch_unwind(AssertUnwindSafe(|| map.split_off::<1, 3>(&key)));
            comparisons.panic_at.set(None);
            assert!(result.is_err(), "comparison {panic_at} did not panic");
            assert!(
                map.len() >= 2,
                "split violated MIN after comparison {panic_at}"
            );
            assert_eq!(
                map.iter().map(|(k, v)| (k.value, *v)).collect::<Vec<_>>(),
                [(1, 10), (2, 20), (3, 30), (4, 40)]
            );
        }
    }
}

fn large_panic_map<const MIN: usize>(
    keys: core::ops::Range<u8>,
    comparisons: &Rc<Comparisons>,
) -> Confined<BTreeMap<PanicOrd, u8>, MIN, 128> {
    Confined::try_from(
        keys.map(|value| {
            (
                PanicOrd {
                    value,
                    comparisons: comparisons.clone(),
                },
                value,
            )
        })
        .collect(),
    )
    .unwrap()
}

#[test]
fn multi_level_btree_split_preserves_all_entries_on_comparison_panic() {
    let comparisons = Rc::new(Comparisons::default());
    let mut map = large_panic_map::<16>(0..64, &comparisons);
    let key = PanicOrd {
        value: 32,
        comparisons: comparisons.clone(),
    };
    comparisons.count.set(0);
    map.split_off::<16, 128>(&key).unwrap();
    let total = comparisons.count.get();
    for panic_at in 1..=total {
        let comparisons = Rc::new(Comparisons::default());
        let mut map = large_panic_map::<16>(0..64, &comparisons);
        let key = PanicOrd {
            value: 32,
            comparisons: comparisons.clone(),
        };
        comparisons.count.set(0);
        comparisons.panic_at.set(Some(panic_at));
        assert!(panic::catch_unwind(AssertUnwindSafe(|| map.split_off::<16, 128>(&key))).is_err());
        comparisons.panic_at.set(None);
        assert_eq!(
            map.len(),
            64,
            "split lost entries after comparison {panic_at}"
        );
        assert_eq!(
            map.values().copied().collect::<Vec<_>>(),
            (0..64).collect::<Vec<_>>()
        );
    }
}

#[test]
fn multi_level_btree_append_preserves_minimum_on_comparison_panic() {
    let comparisons = Rc::new(Comparisons::default());
    let mut map = large_panic_map::<32>(0..32, &comparisons);
    let mut other = large_panic_map::<0>(32..64, &comparisons);
    comparisons.count.set(0);
    map.append(&mut other).unwrap();
    let total = comparisons.count.get();
    for panic_at in 1..=total {
        let comparisons = Rc::new(Comparisons::default());
        let mut map = large_panic_map::<32>(0..32, &comparisons);
        let mut other = large_panic_map::<0>(32..64, &comparisons);
        comparisons.count.set(0);
        comparisons.panic_at.set(Some(panic_at));
        assert!(panic::catch_unwind(AssertUnwindSafe(|| map.append(&mut other))).is_err());
        comparisons.panic_at.set(None);
        assert!(
            map.len() >= 32,
            "append violated MIN after comparison {panic_at}"
        );
        assert!(map.len() <= 128);
    }
}

#[test]
fn btree_split_clone_panic_leaves_source_unchanged() {
    #[derive(Debug)]
    struct Value(Rc<Cell<bool>>);

    impl Clone for Value {
        fn clone(&self) -> Self {
            assert!(!self.0.get(), "clone panic");
            Self(self.0.clone())
        }
    }

    let fail = Rc::new(Cell::new(false));
    let mut map = Confined::<BTreeMap<u8, Value>, 1, 4>::try_from_iter(
        (1..=3).map(|key| (key, Value(fail.clone()))),
    )
    .unwrap();
    fail.set(true);
    assert!(panic::catch_unwind(AssertUnwindSafe(|| map.split_off::<1, 4>(&2))).is_err());
    assert_eq!(map.keys().copied().collect::<Vec<_>>(), [1, 2, 3]);
}

#[test]
fn btree_split_drop_panic_preserves_confinement() {
    #[derive(Clone, Debug)]
    struct Value(Rc<Cell<bool>>);

    impl Drop for Value {
        fn drop(&mut self) {
            assert!(!self.0.replace(false), "drop panic");
        }
    }

    let fail = Rc::new(Cell::new(false));
    let mut map = Confined::<BTreeMap<u8, Value>, 1, 4>::try_from_iter(
        (1..=3).map(|key| (key, Value(fail.clone()))),
    )
    .unwrap();
    fail.set(true);
    assert!(panic::catch_unwind(AssertUnwindSafe(|| map.split_off::<1, 4>(&2))).is_err());
    assert_eq!(map.keys().copied().collect::<Vec<_>>(), [1, 2]);
    assert!((1..=4).contains(&map.len()));
}

#[cfg(feature = "hex")]
mod hex {
    use amplify_num::hex::{Error as HexError, FromHex};

    use super::*;

    #[test]
    fn decoding_does_not_allocate_from_untrusted_iterator_lengths() {
        for reported in [0, 1, 2, usize::MAX] {
            let iter = Misreported {
                inner: [Ok(0xab), Ok(0xcd)].into_iter(),
                reported,
            };
            let bytes = Confined::<Vec<u8>, 2, 2>::from_byte_iter(iter).unwrap();
            assert_eq!(bytes.as_slice(), &[0xab, 0xcd]);
        }
    }

    #[test]
    fn decoding_validates_actual_length_and_propagates_invalid_bytes() {
        let iter = Misreported {
            inner: [Ok(0xab)].into_iter(),
            reported: 2,
        };
        assert_eq!(
            Confined::<Vec<u8>, 2, 3>::from_byte_iter(iter),
            Err(HexError::InvalidLength(2, 1))
        );

        let iter = Misreported {
            inner: [Ok(0xab), Err(HexError::InvalidChar(b'z'))].into_iter(),
            reported: 1,
        };
        assert_eq!(
            Confined::<Vec<u8>, 0, 3>::from_byte_iter(iter),
            Err(HexError::InvalidChar(b'z'))
        );
    }

    #[test]
    fn decoding_stops_at_maximum_even_when_the_iterator_claims_to_be_empty() {
        let reads = Cell::new(0);
        let iter = Misreported {
            inner: core::iter::repeat(Ok(0xab)).inspect(|_| {
                reads.set(reads.get() + 1);
                assert!(reads.get() <= 16, "hex decoder did not stop at MAX + 1");
            }),
            reported: 0,
        };
        assert_eq!(
            Confined::<Vec<u8>, 0, 2>::from_byte_iter(iter),
            Err(HexError::InvalidLength(2, 3))
        );
        assert_eq!(reads.get(), 3);
    }

    #[test]
    fn formatting_preserves_hex_case_and_alternate_endpoints() {
        let bytes = Confined::<Vec<u8>, 0, 80>::try_from_iter([0x00, 0xab, 0xcd, 0xef]).unwrap();
        assert_eq!(format!("{bytes:x}"), "00abcdef");
        assert_eq!(format!("{bytes:X}"), "00ABCDEF");
        let lower = format!("{bytes:#x}");
        let upper = format!("{bytes:#X}");
        assert_eq!(lower, upper.to_ascii_lowercase());
        assert!(lower.contains("00ab"));
        assert!(lower.ends_with("cdef"));

        let mut inner = vec![0xcd; 64];
        inner[0] = 0xab;
        inner[63] = 0xef;
        let bytes = Confined::<Vec<u8>, 0, 80>::try_from(inner).unwrap();
        assert_eq!(format!("{bytes:x}"), format!("ab{}ef", "cd".repeat(62)));
        assert_eq!(format!("{bytes:X}"), format!("AB{}EF", "CD".repeat(62)));
        let lower = format!("{bytes:#x}");
        let upper = format!("{bytes:#X}");
        assert_eq!(lower, upper.to_ascii_lowercase());
        assert!(lower.strip_prefix("0x").unwrap_or(&lower).starts_with("ab"));
        assert!(lower.ends_with("ef"));
        assert!(
            lower.len() < 128,
            "alternate formatting must truncate long blobs"
        );

        let empty = Confined::<Vec<u8>, 0, 0>::new();
        assert_eq!(format!("{empty:x}"), "");
        assert_eq!(format!("{empty:X}"), "");
    }
}
