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

use std::collections::BTreeMap;
#[cfg(feature = "std")]
use std::collections::HashMap;

use confined::*;

#[test]
fn fits_max() {
    let mut s = TinyString::new();
    assert!(s.is_empty());
    for _ in 1..=255 {
        s.push('a').unwrap();
    }
    assert_eq!(s.len_u8(), u8::MAX);
    assert_eq!(s.len_u8(), s.len() as u8);
    assert!(!s.is_empty());

    let mut vec = TinyVec::new();
    let mut deque = TinyDeque::new();
    #[cfg(feature = "std")]
    let mut set = TinyHashSet::new();
    let mut bset = TinyOrdSet::new();
    #[cfg(feature = "std")]
    let mut map = TinyHashMap::new();
    let mut bmap = TinyOrdMap::new();
    #[cfg(feature = "indexmap")]
    let mut imap = TinyIndexMap::new();
    #[cfg(feature = "indexmap")]
    let mut iset = TinyIndexSet::new();
    assert!(vec.is_empty());
    assert!(deque.is_empty());
    #[cfg(feature = "std")]
    assert!(set.is_empty());
    assert!(bset.is_empty());
    #[cfg(feature = "std")]
    assert!(map.is_empty());
    assert!(bmap.is_empty());
    #[cfg(feature = "indexmap")]
    assert!(imap.is_empty());
    #[cfg(feature = "indexmap")]
    assert!(iset.is_empty());
    for index in 1..=255 {
        vec.push(5u8).unwrap();
        deque.push(5u8).unwrap();
        #[cfg(feature = "std")]
        set.push(index).unwrap();
        bset.push(5u8).unwrap();
        #[cfg(feature = "std")]
        map.insert(5u8, 'a').unwrap();
        bmap.insert(index, 'a').unwrap();
        #[cfg(feature = "indexmap")]
        imap.insert(index, 'a').unwrap();
        #[cfg(feature = "indexmap")]
        iset.push(index).unwrap();
    }
    assert_eq!(vec.len_u8(), u8::MAX);
    assert_eq!(deque.len_u8(), u8::MAX);
    #[cfg(feature = "std")]
    assert_eq!(set.len_u8(), u8::MAX);
    assert_eq!(bset.len_u8(), 1);
    #[cfg(feature = "std")]
    assert_eq!(map.len_u8(), 1);
    assert_eq!(bmap.len_u8(), u8::MAX);
    #[cfg(feature = "indexmap")]
    assert_eq!(imap.len_u8(), u8::MAX);
    #[cfg(feature = "indexmap")]
    assert_eq!(iset.len_u8(), u8::MAX);

    vec.clear();
    assert!(vec.is_empty());
}

#[test]
fn cant_go_above_max() {
    let mut s = TinyString::new();
    for _ in 1..=255 {
        s.push('a').unwrap();
    }
    assert!(matches!(
        s.push('a'),
        Err(Error::Oversize {
            len: 255,
            max_len: 255
        })
    ));

    let mut v = TinyVec::<u8>::new();
    for _ in 1..=255 {
        v.push(1).unwrap();
    }
    assert!(matches!(
        v.push(1),
        Err(Error::Oversize {
            len: 255,
            max_len: 255
        })
    ));

    let mut set = TinyOrdSet::<u8>::new();
    for i in 1..=255 {
        set.push(i).unwrap();
    }
    assert!(matches!(
        set.push(255),
        Err(Error::Oversize {
            len: 255,
            max_len: 255
        })
    ));
    assert!(matches!(
        set.push(0),
        Err(Error::Oversize {
            len: 255,
            max_len: 255
        })
    ));

    let mut map = TinyOrdMap::<u8, u8>::new();
    for i in 1..=255 {
        map.insert(i, i).unwrap();
    }
    // Replacing an existing key should succeed
    assert_eq!(map.insert(255, 0), Ok(Some(255)));
    assert!(matches!(
        map.insert(0, 0),
        Err(Error::Oversize {
            len: 255,
            max_len: 255
        })
    ));
}

#[test]
fn boundary() {
    let mut s = TinyString::new();
    s.push('a').unwrap();
    assert!(matches!(
        s.remove(1),
        Err(Error::OutOfBoundary { index: 1, len: 1 })
    ));

    let mut v = TinyVec::<u8>::new();
    v.push(1).unwrap();
    assert!(matches!(
        v.remove(1),
        Err(Error::OutOfBoundary { index: 1, len: 1 })
    ));

    let mut d = TinyDeque::<u8>::new();
    d.push_back(1).unwrap();
    assert!(matches!(
        d.remove(1),
        Err(Error::OutOfBoundary { index: 1, len: 1 })
    ));
}

#[test]
fn cant_go_below_min() {
    let mut s = TinyString::new();
    assert!(matches!(
        s.remove(0),
        Err(Error::OutOfBoundary { index: 0, len: 0 })
    ));

    let mut s = TinyVec::<u8>::new();
    assert!(matches!(
        s.remove(0),
        Err(Error::OutOfBoundary { index: 0, len: 0 })
    ));

    let mut s = TinyOrdSet::<u8>::new();
    assert!(matches!(s.remove(&0), Ok(false)));

    let mut s = TinyOrdMap::<u8, u8>::new();
    assert!(matches!(s.remove(&0), Ok(None)));

    let mut s = NonEmptyString::<U8MAX>::with('a');
    assert!(matches!(
        s.remove(0),
        Err(Error::Undersize { len: 1, min_len: 1 })
    ));
    assert!(matches!(
        s.remove(1),
        Err(Error::OutOfBoundary { index: 1, len: 1 })
    ));

    let mut v = NonEmptyVec::<u8>::with(1);
    assert!(matches!(
        v.remove(0),
        Err(Error::Undersize { len: 1, min_len: 1 })
    ));
    assert!(matches!(
        v.remove(1),
        Err(Error::OutOfBoundary { index: 1, len: 1 })
    ));

    let mut set = NonEmptyOrdSet::<u8>::with(1);
    assert!(matches!(
        set.remove(&1),
        Err(Error::Undersize { len: 1, min_len: 1 })
    ));

    let mut map = NonEmptyOrdMap::<u8, u8>::with_key_value(1, 1);
    assert!(matches!(
        map.remove(&1),
        Err(Error::Undersize { len: 1, min_len: 1 })
    ));
}

#[test]
fn entry() {
    let mut col = tiny_bmap!(0u16 => 'a');
    // An occupied entry keeps its value
    assert_eq!(col.entry(0).unwrap().key(), &0);
    assert_eq!(*col.entry(0).unwrap().or_insert('z'), 'a');
    // A vacant entry adds nothing to the map until a value is inserted
    assert_eq!(col.entry(1).unwrap().key(), &1);
    assert_eq!(col.len(), 1);
    for idx in 1..u8::MAX {
        col.insert(idx as u16, 'b').unwrap();
    }
    assert_eq!(col.len_u8(), u8::MAX);
    assert_eq!(*col.entry(2).unwrap().or_insert('z'), 'b');
    assert_eq!(col.entry(256).unwrap_err(), Error::Oversize {
        len: 255,
        max_len: 255
    });
    assert_eq!(col.len_u8(), u8::MAX);
}

#[test]
fn macros() {
    tiny_vec!() as TinyVec<&str>;
    tiny_vec!("a", "b", "c");
    small_vec!("a", "b", "c");

    #[cfg(feature = "std")]
    {
        tiny_set!() as TinyHashSet<&str>;
        tiny_set!("a", "b", "c");
    }
    tiny_bset!() as TinyOrdSet<&str>;
    tiny_bset!("a", "b", "c");
    #[cfg(feature = "std")]
    {
        tiny_map!() as TinyHashMap<&str, u8>;
        tiny_map!("a" => 1, "b" => 2, "c" => 3);
    }
    tiny_bmap!() as TinyOrdMap<&str, u8>;
    tiny_bmap!("a" => 1, "b" => 2, "c" => 3);

    #[cfg(feature = "std")]
    {
        small_set!("a", "b", "c");
        small_map!("a" => 1, "b" => 2, "c" => 3);
    }
    small_bset!("a", "b", "c");
    small_bmap!("a" => 1, "b" => 2, "c" => 3);

    #[cfg(feature = "indexmap")]
    {
        tiny_imap!() as TinyIndexMap<&str, u8>;
        tiny_imap!("a" => 1, "b" => 2, "c" => 3);
        small_imap!("a" => 1, "b" => 2, "c" => 3);

        tiny_iset!() as TinyIndexSet<&str>;
        tiny_iset!("a", "b", "c");
        small_iset!("a", "b", "c");
    }
}

#[test]
fn iter_mut_btree() {
    let mut coll = tiny_bmap!(1 => "one");
    for (_index, item) in &mut coll {
        *item = "two";
    }
    assert_eq!(coll.get(&1), Some(&"two"));
    for (_index, item) in coll.keyed_values_mut() {
        *item = "three";
    }
    assert_eq!(coll.get(&1), Some(&"three"));
    for item in coll.values_mut() {
        *item = "four";
    }
    assert_eq!(coll.get(&1), Some(&"four"));
    *coll.get_mut(&1).unwrap() = "five";
    assert_eq!(coll.get(&1), Some(&"five"));
}

#[test]
#[cfg(feature = "indexmap")]
fn iter_mut_indexmap() {
    let mut coll = tiny_imap!(1 => "one");
    for (_index, item) in &mut coll {
        *item = "two";
    }
    assert_eq!(coll.get(&1), Some(&"two"));
    for (_index, item) in coll.keyed_values_mut() {
        *item = "three";
    }
    assert_eq!(coll.get(&1), Some(&"three"));
    for item in coll.values_mut() {
        *item = "four";
    }
    assert_eq!(coll.get(&1), Some(&"four"));
    *coll.get_mut(&1).unwrap() = "five";
    assert_eq!(coll.get(&1), Some(&"five"));
}

#[test]
fn first_last_non_empty_vec() {
    let coll = NonEmptyVec::<u8, U8MAX>::try_from(vec![1, 2, 3]).unwrap();
    assert_eq!(*coll.first(), 1);
    assert_eq!(*coll.last(), 3);
}

#[test]
fn first_last_non_empty_deque() {
    let coll = NonEmptyDeque::<u8, U8MAX>::try_from_iter([1, 2, 3]).unwrap();
    assert_eq!(*coll.first(), 1);
    assert_eq!(*coll.last(), 3);
}

#[test]
fn test_index() {
    let mut v = TinyVec::<u8>::new();
    v.push(1u8).unwrap();
    v.push(2u8).unwrap();
    assert_eq!(v[0], 1);
    assert_eq!(v[1], 2);
    v[0] = 3;
    assert_eq!(v[0], 3);
    assert_eq!(&v[0..1], &[3]);

    let mut m = TinyOrdMap::<u8, u8>::new();
    m.insert(1u8, 10u8).unwrap();
    m.insert(2u8, 20u8).unwrap();
    assert_eq!(m[&1], 10);
    assert_eq!(m[&2], 20);
    // m[&1] = 11; // BTreeMap doesn't support IndexMut
}

#[test]
fn btree_pop_first() {
    let mut m = Confined::<BTreeMap<u8, u8>, 1, 3>::try_from_iter([(1, 10), (2, 20)]).unwrap();
    assert_eq!(m.pop_first().unwrap(), Some((1, 10)));
    assert_eq!(m.len(), 1);
    assert!(m.pop_first().is_err()); // Underflow 1 -> 0
}

#[test]
fn btree_pop_last() {
    let mut m = Confined::<BTreeMap<u8, u8>, 1, 3>::try_from_iter([(1, 10), (2, 20)]).unwrap();
    assert_eq!(m.pop_last().unwrap(), Some((2, 20)));
    assert_eq!(m.len(), 1);
    assert!(m.pop_last().is_err()); // Underflow 1 -> 0
}

#[test]
fn btree_remove_entry() {
    let mut m = Confined::<BTreeMap<u8, u8>, 1, 3>::try_from_iter([(1, 10), (2, 20)]).unwrap();
    assert_eq!(m.remove_entry(&2).unwrap(), Some((2, 20)));
    assert_eq!(m.len(), 1);
    assert!(m.remove_entry(&1).is_err()); // Underflow 1 -> 0
    assert_eq!(m.remove_entry(&3).unwrap(), None);
}

#[test]
fn btree_append() {
    let mut m1 = Confined::<BTreeMap<u8, u8>, 0, 3>::try_from_iter([(1, 10)]).unwrap();
    let mut m2 = Confined::<BTreeMap<u8, u8>, 0, 3>::try_from_iter([(2, 20), (3, 30)]).unwrap();
    m1.append(&mut m2).unwrap();
    assert_eq!(m1.len(), 3);
    assert_eq!(m2.len(), 0);

    let mut m3 = Confined::<BTreeMap<u8, u8>, 0, 3>::try_from_iter([(4, 40)]).unwrap();
    assert!(m1.append(&mut m3).is_err()); // Overflow 3 + 1 -> 4
}

#[test]
fn btree_range_mut() {
    let mut m =
        Confined::<BTreeMap<u8, u8>, 0, 3>::try_from_iter([(1, 10), (2, 20), (3, 30)]).unwrap();
    for (_, v) in m.range_mut(2..) {
        *v += 1;
    }
    assert_eq!(m[&2], 21);
    assert_eq!(m[&3], 31);
}

#[test]
fn btree_split_off() {
    let mut m =
        Confined::<BTreeMap<u8, u8>, 1, 5>::try_from_iter([(1, 10), (2, 20), (3, 30), (4, 40)])
            .unwrap();
    let m2: Confined<BTreeMap<u8, u8>, 1, 5> = m.split_off(&3).unwrap();
    assert_eq!(m.len(), 2);
    assert_eq!(m2.len(), 2);
    assert!(m.contains_key(&1));
    assert!(m.contains_key(&2));
    assert!(m2.contains_key(&3));
    assert!(m2.contains_key(&4));

    let mut m =
        Confined::<BTreeMap<u8, u8>, 1, 5>::try_from_iter([(1, 10), (2, 20), (3, 30), (4, 40)])
            .unwrap();
    // Underflow m: 4 -> 0 (min 1 is NOT OK)
    assert!(m.split_off::<1, 5>(&1).is_err());
    // Underflow m2: 4 -> 0 (min 1 is NOT OK)
    assert!(m.split_off::<1, 5>(&5).is_err());
}

#[test]
fn btree_try_retain() {
    let mut m =
        Confined::<BTreeMap<u8, u8>, 1, 5>::try_from_iter([(1, 10), (2, 20), (3, 30)]).unwrap();
    m.try_retain(|k, _v| *k < 3).unwrap();
    assert_eq!(m.len(), 2);
    assert!(m.try_retain(|k, _v| *k < 1).is_err()); // Underflow 2 -> 0
}

#[test]
fn btree_retain() {
    let mut m =
        Confined::<BTreeMap<u8, u8>, 0, 5>::try_from_iter([(1, 10), (2, 20), (3, 30)]).unwrap();
    m.retain(|k, _v| *k < 2);
    assert_eq!(m.len(), 1);
}

#[test]
#[cfg(feature = "std")]
fn hash_remove_entry() {
    let mut m = Confined::<HashMap<u8, u8>, 1, 3>::try_from_iter([(1, 10), (2, 20)]).unwrap();
    assert_eq!(m.remove_entry(&2).unwrap(), Some((2, 20)));
    assert_eq!(m.len(), 1);
    assert!(m.remove_entry(&1).is_err()); // Underflow 1 -> 0
    assert_eq!(m.remove_entry(&3).unwrap(), None);
}

#[test]
#[cfg(feature = "std")]
fn hash_drain() {
    let mut m = Confined::<HashMap<u8, u8>, 0, 3>::try_from_iter([(1, 10), (2, 20)]).unwrap();
    let drained: Vec<_> = m.drain().collect();
    assert_eq!(drained.len(), 2);
    assert_eq!(m.len(), 0);
}

#[test]
#[cfg(feature = "std")]
fn hash_reserve() {
    let mut m = Confined::<HashMap<u8, u8>, 0, 3>::try_from_iter([(1, 10)]).unwrap();
    m.reserve(2);
    assert!(m.capacity() >= 2);
    m.reserve(3);
    let capacity = m.capacity();
    m.reserve(4);
    assert_eq!(m.capacity(), capacity);
}

#[test]
#[cfg(feature = "std")]
fn hash_shrink_to_fit() {
    let mut m = Confined::<HashMap<u8, u8>, 0, 10>::try_from_iter([(1, 10)]).unwrap();
    m.reserve(5);
    m.shrink_to_fit();
    assert!(m.capacity() >= 1);
}

#[test]
#[cfg(feature = "std")]
fn hash_shrink_to() {
    let mut m = Confined::<HashMap<u8, u8>, 0, 10>::try_from_iter([(1, 10)]).unwrap();
    m.reserve(5);
    m.shrink_to(2);
    assert!(m.capacity() >= 2);
}

/// Each macro family is expanded in a module which imports nothing from the
/// crate, so the macros can't rely on anything in the scope of their caller.
mod standalone {
    #[test]
    #[should_panic(expected = "inline tiny_vec literal contains invalid number of items")]
    fn macro_panics_if_the_literal_does_not_fit() {
        let _ = confined::tiny_vec![0u8; 256];
    }

    #[test]
    fn macros_need_no_imports() {
        assert_eq!(confined::tiny_s!().len(), 0);
        assert_eq!(confined::tiny_s!("abc").len_u8(), 3);
        assert_eq!(confined::small_s!("abc").len_u16(), 3);
        assert_eq!(confined::medium_s!("abc").len(), 3);

        assert_eq!(confined::tiny_blob!().len(), 0);
        assert_eq!(confined::tiny_blob!(0xFF; 4).len_u8(), 4);
        assert_eq!(confined::small_blob!(1, 2, 3).len_u16(), 3);
        assert_eq!(confined::medium_blob!(1, 2, 3).len(), 3);

        assert_eq!(confined::tiny_vec!("a"; 2).len_u8(), 2);
        assert_eq!(confined::small_vec!("a", "b").len_u16(), 2);
        assert_eq!(confined::medium_vec!("a", "b").len(), 2);

        assert_eq!(confined::tiny_bset!("a", "b", "a").len_u8(), 2);
        assert_eq!(confined::small_bset!("a", "b").len_u16(), 2);
        assert_eq!(confined::medium_bset!("a", "b").len(), 2);

        assert_eq!(confined::tiny_bmap!("a" => 1, "b" => 2).len_u8(), 2);
        assert_eq!(confined::small_bmap!("a" => 1, "b" => 2).len_u16(), 2);
        assert_eq!(confined::medium_bmap!("a" => 1, "b" => 2).len(), 2);

        #[cfg(feature = "std")]
        {
            assert_eq!(confined::tiny_set!("a", "b", "a").len_u8(), 2);
            assert_eq!(confined::small_set!("a", "b").len_u16(), 2);
            assert_eq!(confined::medium_set!("a", "b").len(), 2);

            assert_eq!(confined::tiny_map!("a" => 1, "b" => 2).len_u8(), 2);
            assert_eq!(confined::small_map!("a" => 1, "b" => 2).len_u16(), 2);
            assert_eq!(confined::medium_map!("a" => 1, "b" => 2).len(), 2);
        }

        #[cfg(feature = "indexmap")]
        {
            assert_eq!(confined::tiny_iset!("a", "b", "a").len_u8(), 2);
            assert_eq!(confined::small_iset!("a", "b").len_u16(), 2);
            assert_eq!(confined::medium_iset!("a", "b").len(), 2);

            assert_eq!(confined::tiny_imap!("a" => 1, "b" => 2).len_u8(), 2);
            assert_eq!(confined::small_imap!("a" => 1, "b" => 2).len_u16(), 2);
            assert_eq!(confined::medium_imap!("a" => 1, "b" => 2).len(), 2);
        }
    }
}
