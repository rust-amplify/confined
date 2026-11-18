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

//! Panic safety of the confined collections.
//!
//! Code supplied by the caller (a predicate, a closure, an iterator, a
//! comparison or a destructor) may panic in the middle of an operation. When
//! such a panic is caught, the collection must still be within its bounds and
//! internally consistent: its length, its iteration and its lookups have to
//! describe the same items.
//!
//! The tests are grouped by the code which panics:
//!
//! - `retain`: a predicate of `retain` / `try_retain`, or a destructor of an
//!   entry which they remove, for every supported map;
//! - `btree_append`: a destructor of a value or of a key replaced by
//!   `BTreeMap::append`;
//! - `closures`: closures given to `entry()`, `extend()` and the constructors;
//! - `comparisons`: `Ord` implementations called by the checked operations.
//!
//! Most retention tests of the `heapless` maps are not run by Miri with its
//! default aliasing model, Stacked Borrows. In `heapless` 0.9.3 the guard of
//! `Vec::retain_mut` which moves the remaining items when a panic unwinds
//! breaks the rules of that model by itself, before the confinement gets the
//! control back. The tests pass under Miri with Tree Borrows, which is how
//! they are meant to be run:
//!
//! ```console
//! MIRIFLAGS=-Zmiri-tree-borrows cargo +nightly miri test --all-features \
//!     --test panic_safety -- --ignored
//! ```
//!
//! The guard has nothing to move when the panic comes from the destructor of
//! the last visited entry, so the `..._of_the_last_destructor` tests run with
//! any aliasing model, for every map.

#![cfg(any(feature = "alloc", feature = "heapless"))]

use std::cell::Cell;
use std::panic::{self, AssertUnwindSafe};
use std::rc::Rc;

use collections::CollectionMap;
use confined::{Confined, ItemLen};

/// The number of entries in the maps built by the tests, with keys `1..=KEYS`.
const KEYS: u32 = 6;

/// A value which can tell the key it was stored under: `key * 100`.
trait Tagged {
    fn tag(&self) -> u32;
}

impl Tagged for u32 {
    fn tag(&self) -> u32 {
        *self
    }
}

/// A value which destructor panics once after the shared trigger is armed.
#[derive(Debug)]
struct Bomb {
    tag: u32,
    armed: Rc<Cell<bool>>,
}

impl Tagged for Bomb {
    fn tag(&self) -> u32 {
        self.tag
    }
}

impl Drop for Bomb {
    fn drop(&mut self) {
        // The trigger disarms itself, so the unwinding never panics twice.
        assert!(!self.armed.replace(false), "destructor panic");
    }
}

/// Checks that the length, the iteration and the lookups of the map agree,
/// and that the map holds exactly the entries with the `expected` keys.
fn assert_consistent<C, const MIN: usize, const MAX: usize>(
    map: &Confined<C, MIN, MAX>,
    expected: &[u32],
) where
    C: CollectionMap<Key = u32> + ItemLen,
    C::Value: Tagged,
{
    let inner = map.as_unconfined();
    let mut iterated = inner
        .iter()
        .map(|(key, value)| {
            assert_eq!(
                value.tag(),
                key * 100,
                "a value is stored under a wrong key"
            );
            *key
        })
        .collect::<Vec<_>>();
    iterated.sort_unstable();
    let mut expected = expected.to_vec();
    expected.sort_unstable();
    assert_eq!(
        iterated, expected,
        "the iteration yields unexpected entries"
    );
    assert_eq!(
        map.len(),
        expected.len(),
        "the length disagrees with the entries"
    );
    assert!((MIN..=MAX).contains(&map.len()), "the bounds are violated");
    for key in 1..=KEYS + 1 {
        let found = inner.get(&key).map(Tagged::tag);
        if expected.contains(&key) {
            assert_eq!(found, Some(key * 100), "an entry is not found by its key");
            assert!(inner.contains_key(&key));
        } else {
            assert_eq!(found, None, "a removed entry is still found by its key");
            assert!(!inner.contains_key(&key));
        }
    }
}

fn remaining(removed: &[u32]) -> Vec<u32> {
    (1..=KEYS).filter(|key| !removed.contains(key)).collect()
}

mod retain {
    use super::*;

    fn plain<C, const MIN: usize>() -> Confined<C, MIN, 8>
    where
        C: CollectionMap<Key = u32, Value = u32> + ItemLen + Default,
    {
        Confined::try_from_iter((1..=KEYS).map(|key| (key, key * 100))).unwrap()
    }

    fn explosive<C, const MIN: usize>(armed: &Rc<Cell<bool>>) -> Confined<C, MIN, 8>
    where
        C: CollectionMap<Key = u32, Value = Bomb> + ItemLen + Default,
    {
        Confined::try_from_iter((1..=KEYS).map(|key| {
            let value = Bomb {
                tag: key * 100,
                armed: armed.clone(),
            };
            (key, value)
        }))
        .unwrap()
    }

    /// A predicate which rejects three entries and panics on the fourth one.
    fn rejecting(rejected: &mut Vec<u32>) -> impl FnMut(&u32, &mut u32) -> bool + '_ {
        move |key, _| {
            assert!(rejected.len() < 3, "predicate panic");
            rejected.push(*key);
            false
        }
    }

    /// The map is consistent after a panic of the predicate, and it can be
    /// changed and read further.
    fn check_after_predicate_panic<C, const MIN: usize>(
        mut map: Confined<C, MIN, 8>,
        retain: impl FnOnce(&mut Confined<C, MIN, 8>, &mut Vec<u32>),
    ) where
        C: CollectionMap<Key = u32, Value = u32> + ItemLen,
    {
        let mut rejected = vec![];
        let result = panic::catch_unwind(AssertUnwindSafe(|| retain(&mut map, &mut rejected)));
        assert!(result.is_err(), "the predicate has to panic");
        assert_eq!(rejected.len(), 3);

        let mut expected = remaining(&rejected);
        assert_consistent(&map, &expected);

        assert_eq!(map.insert(KEYS + 1, (KEYS + 1) * 100), Ok(None));
        expected.push(KEYS + 1);
        assert_consistent(&map, &expected);
    }

    /// The map is consistent after a panic of the destructor of the first
    /// entry removed by the retention.
    fn check_after_destructor_panic<C, const MIN: usize>(
        mut map: Confined<C, MIN, 8>,
        armed: &Rc<Cell<bool>>,
        retain: impl FnOnce(&mut Confined<C, MIN, 8>, &mut Vec<u32>),
    ) where
        C: CollectionMap<Key = u32, Value = Bomb> + ItemLen,
    {
        let mut visited = vec![];
        armed.set(true);
        let result = panic::catch_unwind(AssertUnwindSafe(|| retain(&mut map, &mut visited)));
        assert!(result.is_err(), "the destructor has to panic");
        assert!(!armed.get(), "the destructor has to disarm the trigger");

        assert_consistent(&map, &remaining(&visited[..1]));
    }

    /// A predicate which rejects only the last entry it visits.
    fn rejecting_the_last(visited: &mut Vec<u32>) -> impl FnMut(&u32, &mut Bomb) -> bool + '_ {
        move |key, _| {
            visited.push(*key);
            visited.len() < KEYS as usize
        }
    }

    /// The map is consistent after a panic of the destructor of the last
    /// entry visited by the retention, when no entry is left to visit.
    fn check_after_last_destructor_panic<C, const MIN: usize>(
        mut map: Confined<C, MIN, 8>,
        armed: &Rc<Cell<bool>>,
        retain: impl FnOnce(&mut Confined<C, MIN, 8>, &mut Vec<u32>),
    ) where
        C: CollectionMap<Key = u32, Value = Bomb> + ItemLen,
    {
        let mut visited = vec![];
        armed.set(true);
        let result = panic::catch_unwind(AssertUnwindSafe(|| retain(&mut map, &mut visited)));
        assert!(result.is_err(), "the destructor has to panic");
        assert!(!armed.get(), "the destructor has to disarm the trigger");
        assert_eq!(visited.len(), KEYS as usize);

        assert_consistent(&map, &remaining(&visited[visited.len() - 1..]));
    }

    macro_rules! retain_panic_safety {
        ($(#[$attr:meta])* $name:ident, $plain:ty, $explosive:ty) => {
            mod $name {
                use super::*;

                #[test]
                $(#[$attr])*
                fn try_retain_survives_a_panic_of_the_predicate() {
                    check_after_predicate_panic(plain::<$plain, 2>(), |map, rejected| {
                        map.try_retain(rejecting(rejected)).unwrap()
                    });
                }

                #[test]
                $(#[$attr])*
                fn retain_survives_a_panic_of_the_predicate() {
                    check_after_predicate_panic(plain::<$plain, 0>(), |map, rejected| {
                        map.retain(rejecting(rejected))
                    });
                }

                #[test]
                $(#[$attr])*
                fn try_retain_survives_a_panic_of_a_destructor() {
                    let armed = Rc::new(Cell::new(false));
                    let map = explosive::<$explosive, 2>(&armed);
                    check_after_destructor_panic(map, &armed, |map, visited| {
                        map.try_retain(|key, _| {
                            visited.push(*key);
                            false
                        })
                        .unwrap()
                    });
                }

                #[test]
                $(#[$attr])*
                fn retain_survives_a_panic_of_a_destructor() {
                    let armed = Rc::new(Cell::new(false));
                    let map = explosive::<$explosive, 0>(&armed);
                    check_after_destructor_panic(map, &armed, |map, visited| {
                        map.retain(|key, _| {
                            visited.push(*key);
                            false
                        })
                    });
                }

                #[test]
                fn try_retain_survives_a_panic_of_the_last_destructor() {
                    let armed = Rc::new(Cell::new(false));
                    let map = explosive::<$explosive, 2>(&armed);
                    check_after_last_destructor_panic(map, &armed, |map, visited| {
                        map.try_retain(rejecting_the_last(visited)).unwrap()
                    });
                }

                #[test]
                fn retain_survives_a_panic_of_the_last_destructor() {
                    let armed = Rc::new(Cell::new(false));
                    let map = explosive::<$explosive, 0>(&armed);
                    check_after_last_destructor_panic(map, &armed, |map, visited| {
                        map.retain(rejecting_the_last(visited))
                    });
                }
            }
        };
    }

    #[cfg(feature = "alloc")]
    retain_panic_safety!(
        btree_map,
        std::collections::BTreeMap<u32, u32>,
        std::collections::BTreeMap<u32, Bomb>
    );

    #[cfg(feature = "std")]
    retain_panic_safety!(
        hash_map,
        std::collections::HashMap<u32, u32>,
        std::collections::HashMap<u32, Bomb>
    );

    #[cfg(feature = "hashbrown")]
    retain_panic_safety!(
        hashbrown_map,
        confined::hashbrown::HashMap<u32, u32, std::hash::RandomState>,
        confined::hashbrown::HashMap<u32, Bomb, std::hash::RandomState>
    );

    #[cfg(feature = "indexmap")]
    retain_panic_safety!(
        index_map,
        indexmap::IndexMap<u32, u32, std::hash::RandomState>,
        indexmap::IndexMap<u32, Bomb, std::hash::RandomState>
    );

    #[cfg(feature = "heapless")]
    retain_panic_safety!(
        #[cfg_attr(miri, ignore = "`heapless` breaks Stacked Borrows; run with Tree Borrows")]
        heapless_index_map,
        confined::heapless::IndexMap<u32, u32, std::hash::RandomState, 8>,
        confined::heapless::IndexMap<u32, Bomb, std::hash::RandomState, 8>
    );

    #[cfg(feature = "heapless")]
    retain_panic_safety!(
        #[cfg_attr(miri, ignore = "`heapless` breaks Stacked Borrows; run with Tree Borrows")]
        heapless_linear_map,
        confined::heapless::LinearMap<u32, u32, 8>,
        confined::heapless::LinearMap<u32, Bomb, 8>
    );
}

#[cfg(feature = "alloc")]
mod btree_append {
    use std::cmp::Ordering;
    use std::collections::BTreeMap;

    use super::*;

    fn bombs<const MIN: usize>(
        keys: impl IntoIterator<Item = u32>,
        armed: &Rc<Cell<bool>>,
    ) -> Confined<BTreeMap<u32, Bomb>, MIN, 8> {
        Confined::try_from_iter(keys.into_iter().map(|key| {
            let value = Bomb {
                tag: key * 100,
                armed: armed.clone(),
            };
            (key, value)
        }))
        .unwrap()
    }

    #[test]
    fn destructor_of_the_only_replaced_value_keeps_the_minimum() {
        let armed = Rc::new(Cell::new(false));
        let mut map = bombs::<1>([1], &armed);
        let mut other = bombs::<0>([1], &armed);

        armed.set(true);
        let result = panic::catch_unwind(AssertUnwindSafe(|| map.append(&mut other)));
        assert!(result.is_err(), "the destructor has to panic");

        assert_consistent(&map, &[1]);
    }

    #[test]
    fn destructor_of_a_replaced_value_keeps_the_minimum_of_a_larger_map() {
        let armed = Rc::new(Cell::new(false));
        let mut map = bombs::<3>([1, 2, 3], &armed);
        let mut other = bombs::<0>([1, 2, 3], &armed);

        armed.set(true);
        let result = panic::catch_unwind(AssertUnwindSafe(|| map.append(&mut other)));
        assert!(result.is_err(), "the destructor has to panic");

        assert_consistent(&map, &[1, 2, 3]);
    }

    #[test]
    fn destructor_panic_leaves_the_unmoved_entries_in_the_source() {
        let armed = Rc::new(Cell::new(false));
        let mut map = bombs::<1>([1, 2], &armed);
        let mut other = bombs::<0>([1, 3, 4], &armed);

        armed.set(true);
        let result = panic::catch_unwind(AssertUnwindSafe(|| map.append(&mut other)));
        assert!(result.is_err(), "the destructor has to panic");

        // The entry with the key 1 was moved before its previous value was
        // dropped; the entries after it were not touched.
        assert_consistent(&map, &[1, 2]);
        assert_consistent(&other, &[3, 4]);
    }

    /// A key which destructor panics once after the shared trigger is armed.
    #[derive(Debug)]
    struct BombKey(u32, Rc<Cell<bool>>);

    impl Drop for BombKey {
        fn drop(&mut self) {
            assert!(!self.1.replace(false), "destructor panic");
        }
    }

    impl PartialEq for BombKey {
        fn eq(&self, other: &Self) -> bool {
            self.0 == other.0
        }
    }

    impl Eq for BombKey {}

    impl PartialOrd for BombKey {
        fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
            Some(self.cmp(other))
        }
    }

    impl Ord for BombKey {
        fn cmp(&self, other: &Self) -> Ordering {
            self.0.cmp(&other.0)
        }
    }

    #[test]
    fn destructor_of_a_duplicate_key_keeps_the_minimum() {
        let armed = Rc::new(Cell::new(false));
        let key = |key: u32| BombKey(key, armed.clone());
        let mut map =
            Confined::<BTreeMap<BombKey, u32>, 2, 8>::try_from_iter([(key(1), 100), (key(2), 200)])
                .unwrap();
        let mut other =
            Confined::<BTreeMap<BombKey, u32>, 0, 8>::try_from_iter([(key(1), 101), (key(2), 201)])
                .unwrap();

        armed.set(true);
        let result = panic::catch_unwind(AssertUnwindSafe(|| map.append(&mut other)));
        assert!(result.is_err(), "the destructor has to panic");

        assert_eq!(map.len(), 2);
        assert_eq!(map.iter().count(), 2);
        assert_eq!(map.keys().map(|key| key.0).collect::<Vec<_>>(), [1, 2]);
    }
}

#[cfg(feature = "alloc")]
mod closures {
    use std::collections::BTreeMap;

    use super::*;

    type Map = Confined<BTreeMap<u32, u32>, 1, 3>;

    fn map() -> Map {
        Map::try_from_iter([(1, 100), (2, 200)]).unwrap()
    }

    #[test]
    fn panic_of_a_vacant_entry_default_adds_nothing() {
        let mut map = map();
        let result = panic::catch_unwind(AssertUnwindSafe(|| {
            map.entry(3)
                .unwrap()
                .or_insert_with(|| panic!("default panic"));
        }));
        assert!(result.is_err());
        assert_consistent(&map, &[1, 2]);
    }

    #[test]
    fn panic_of_an_entry_modification_keeps_the_entry() {
        let mut map = map();
        let result = panic::catch_unwind(AssertUnwindSafe(|| {
            let _ = map
                .entry(2)
                .unwrap()
                .and_modify(|_| panic!("modification panic"));
        }));
        assert!(result.is_err());
        assert_consistent(&map, &[1, 2]);
    }

    #[test]
    fn panic_of_an_extending_iterator_keeps_the_items_added_before_it() {
        let mut map = map();
        let source = [(3, 300), (4, 400)]
            .into_iter()
            .inspect(|entry| assert_ne!(entry.0, 4, "iterator panic"));
        let result = panic::catch_unwind(AssertUnwindSafe(|| map.extend(source)));
        assert!(result.is_err());
        assert_consistent(&map, &[1, 2, 3]);
    }

    #[test]
    fn panic_of_a_constructing_iterator_is_propagated() {
        let source = (1..=3u32).map(|key| {
            assert_ne!(key, 3, "iterator panic");
            (key, key * 100)
        });
        let result = panic::catch_unwind(AssertUnwindSafe(|| Map::try_from_iter(source)));
        assert!(result.is_err());
    }

    #[test]
    fn panic_of_a_revalidated_mutation_keeps_the_bounds() {
        let mut map = map();
        let result = panic::catch_unwind(AssertUnwindSafe(|| {
            map.as_mut_checked(|map| {
                map.insert(3, 300).unwrap();
                panic!("mutation panic");
            });
        }));
        assert!(result.is_err());
        assert_consistent(&map, &[1, 2, 3]);
    }
}

#[cfg(feature = "alloc")]
mod comparisons {
    use std::cmp::Ordering;
    use std::collections::{BTreeMap, BTreeSet, BinaryHeap};

    use super::*;

    /// An item which comparison panics once after the shared trigger is armed.
    #[derive(Clone, Debug)]
    struct Fragile(u32, Rc<Cell<bool>>);

    impl PartialEq for Fragile {
        fn eq(&self, other: &Self) -> bool {
            self.0 == other.0
        }
    }

    impl Eq for Fragile {}

    impl PartialOrd for Fragile {
        fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
            Some(self.cmp(other))
        }
    }

    impl Ord for Fragile {
        fn cmp(&self, other: &Self) -> Ordering {
            assert!(!self.1.replace(false), "comparison panic");
            self.0.cmp(&other.0)
        }
    }

    fn values(items: impl IntoIterator<Item = Fragile>) -> Vec<u32> {
        let mut values = items.into_iter().map(|item| item.0).collect::<Vec<_>>();
        values.sort_unstable();
        values
    }

    #[test]
    fn panic_of_a_heap_push_keeps_the_bounds() {
        let armed = Rc::new(Cell::new(false));
        let item = |value: u32| Fragile(value, armed.clone());
        let mut heap =
            Confined::<BinaryHeap<Fragile>, 1, 3>::try_from_iter([item(1), item(2)]).unwrap();

        armed.set(true);
        let result = panic::catch_unwind(AssertUnwindSafe(|| heap.push(item(3))));
        assert!(result.is_err(), "the comparison has to panic");

        assert!((1..=3).contains(&heap.len()));
        assert_eq!(heap.len(), heap.iter().count());
        assert_eq!(values(heap.release()), [1, 2, 3]);
    }

    #[test]
    fn panic_of_a_set_push_adds_nothing() {
        let armed = Rc::new(Cell::new(false));
        let item = |value: u32| Fragile(value, armed.clone());
        let mut set =
            Confined::<BTreeSet<Fragile>, 1, 3>::try_from_iter([item(1), item(2)]).unwrap();

        armed.set(true);
        let result = panic::catch_unwind(AssertUnwindSafe(|| set.push(item(3))));
        assert!(result.is_err(), "the comparison has to panic");

        assert_eq!(set.len(), 2);
        assert_eq!(values(set.release()), [1, 2]);
    }

    #[test]
    fn panic_of_a_set_removal_removes_nothing() {
        let armed = Rc::new(Cell::new(false));
        let item = |value: u32| Fragile(value, armed.clone());
        let mut set =
            Confined::<BTreeSet<Fragile>, 1, 3>::try_from_iter([item(1), item(2)]).unwrap();

        armed.set(true);
        let result = panic::catch_unwind(AssertUnwindSafe(|| set.remove(&item(2))));
        assert!(result.is_err(), "the comparison has to panic");

        assert_eq!(set.len(), 2);
        assert_eq!(values(set.release()), [1, 2]);
    }

    #[test]
    fn panic_of_a_map_insertion_adds_nothing() {
        let armed = Rc::new(Cell::new(false));
        let item = |value: u32| Fragile(value, armed.clone());
        let mut map =
            Confined::<BTreeMap<Fragile, u32>, 1, 3>::try_from_iter([(item(1), 1), (item(2), 2)])
                .unwrap();

        armed.set(true);
        let result = panic::catch_unwind(AssertUnwindSafe(|| map.insert(item(3), 3)));
        assert!(result.is_err(), "the comparison has to panic");

        assert_eq!(map.len(), 2);
        assert_eq!(values(map.release().into_keys()), [1, 2]);
    }

    #[test]
    fn panic_of_a_map_removal_removes_nothing() {
        let armed = Rc::new(Cell::new(false));
        let item = |value: u32| Fragile(value, armed.clone());
        let mut map =
            Confined::<BTreeMap<Fragile, u32>, 1, 3>::try_from_iter([(item(1), 1), (item(2), 2)])
                .unwrap();

        armed.set(true);
        let result = panic::catch_unwind(AssertUnwindSafe(|| map.remove(&item(2))));
        assert!(result.is_err(), "the comparison has to panic");

        assert_eq!(map.len(), 2);
        assert_eq!(values(map.release().into_keys()), [1, 2]);
    }
}
