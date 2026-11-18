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

//! Verifies that exported constructor macros expand hygienically in a separate
//! consumer crate without relying on the standard prelude or caller imports.

#![cfg(feature = "alloc")]
#![cfg_attr(not(feature = "std"), no_std)]
#![no_implicit_prelude]

#[test]
fn alloc_macros_are_hygienic_without_prelude() {
    let empty_s = ::confined::tiny_s!();
    let s1 = ::confined::tiny_s!("alpha");
    let s2 = ::confined::small_s!("beta");
    let s3 = ::confined::medium_s!("gamma");
    ::core::assert_eq!(empty_s.len_u8(), 0);
    ::core::assert_eq!(s1.len_u8(), 5);
    ::core::assert_eq!(s2.len_u16(), 4);
    ::core::assert_eq!(s3.len(), 5);

    let b0 = ::confined::tiny_blob!();
    let b1 = ::confined::tiny_blob![1u8, 2u8];
    let b2 = ::confined::small_blob![3u8; 4];
    let b3 = ::confined::medium_blob![5u8, 6u8,];
    ::core::assert_eq!(b0.len_u8(), 0);
    ::core::assert_eq!(b1.len_u8(), 2);
    ::core::assert_eq!(b2.len_u16(), 4);
    ::core::assert_eq!(b3.len(), 2);

    let v0: ::confined::TinyVec<u8> = ::confined::tiny_vec![];
    let v1 = ::confined::tiny_vec![10u8; 2];
    let v2 = ::confined::small_vec![10u8, 20u8];
    let v3 = ::confined::medium_vec![30u8; 3];
    ::core::assert_eq!(v0.len_u8(), 0);
    ::core::assert_eq!(v1.len_u8(), 2);
    ::core::assert_eq!(v2.len_u16(), 2);
    ::core::assert_eq!(v3.len(), 3);

    let d0: ::confined::TinyDeque<u8> = ::confined::tiny_deque![];
    let d1 = ::confined::tiny_deque![1u8, 2u8];
    let d2 = ::confined::small_deque![3u8; 2];
    let d3 = ::confined::medium_deque![4u8];
    ::core::assert_eq!(d0.len_u8(), 0);
    ::core::assert_eq!(d1.len_u8(), 2);
    ::core::assert_eq!(d2.len_u16(), 2);
    ::core::assert_eq!(d3.len(), 1);

    let l0: ::confined::TinyList<u8> = ::confined::tiny_list![];
    let l1 = ::confined::tiny_list![1u8, 2u8];
    let l2 = ::confined::small_list![3u8];
    let l3 = ::confined::medium_list![4u8, 5u8];
    ::core::assert_eq!(l0.len_u8(), 0);
    ::core::assert_eq!(l1.len_u8(), 2);
    ::core::assert_eq!(l2.len_u16(), 1);
    ::core::assert_eq!(l3.len(), 2);

    let h0: ::confined::TinyHeap<u8> = ::confined::tiny_heap![];
    let h1 = ::confined::tiny_heap![1u8, 3u8, 2u8];
    let h2 = ::confined::small_heap![4u8];
    let h3 = ::confined::medium_heap![5u8, 6u8];
    ::core::assert_eq!(h0.len_u8(), 0);
    ::core::assert_eq!(h1.len_u8(), 3);
    ::core::assert_eq!(h2.len_u16(), 1);
    ::core::assert_eq!(h3.len(), 2);

    let bs0: ::confined::TinyOrdSet<u8> = ::confined::tiny_bset![];
    let bs1 = ::confined::tiny_bset![1u8, 2u8, 1u8];
    let bs2 = ::confined::small_bset![3u8];
    let bs3 = ::confined::medium_bset![4u8, 5u8];
    ::core::assert_eq!(bs0.len_u8(), 0);
    ::core::assert_eq!(bs1.len_u8(), 2);
    ::core::assert_eq!(bs2.len_u16(), 1);
    ::core::assert_eq!(bs3.len(), 2);

    let bm0: ::confined::TinyOrdMap<u8, u8> = ::confined::tiny_bmap! {};
    let bm1 = ::confined::tiny_bmap! { 1u8 => 10u8, 2u8 => 20u8 };
    let bm2 = ::confined::small_bmap! { 3u8 => 30u8 };
    let bm3 = ::confined::medium_bmap! { 4u8 => 40u8 };
    ::core::assert_eq!(bm0.len_u8(), 0);
    ::core::assert_eq!(bm1.len_u8(), 2);
    ::core::assert_eq!(bm2.len_u16(), 1);
    ::core::assert_eq!(bm3.len(), 1);
}

#[cfg(feature = "std")]
#[test]
fn std_hash_and_index_macros_are_hygienic_without_prelude() {
    let hs0: ::confined::TinyHashSet<u8> = ::confined::tiny_set![];
    let hs1 = ::confined::tiny_set![1u8, 2u8, 1u8];
    let hs2 = ::confined::small_set![3u8];
    let hs3 = ::confined::medium_set![4u8, 5u8];
    ::core::assert_eq!(hs0.len_u8(), 0);
    ::core::assert_eq!(hs1.len_u8(), 2);
    ::core::assert_eq!(hs2.len_u16(), 1);
    ::core::assert_eq!(hs3.len(), 2);

    let hm0: ::confined::TinyHashMap<u8, u8> = ::confined::tiny_map! {};
    let hm1 = ::confined::tiny_map! { 1u8 => 10u8, 2u8 => 20u8 };
    let hm2 = ::confined::small_map! { 3u8 => 30u8 };
    let hm3 = ::confined::medium_map! { 4u8 => 40u8 };
    ::core::assert_eq!(hm0.len_u8(), 0);
    ::core::assert_eq!(hm1.len_u8(), 2);
    ::core::assert_eq!(hm2.len_u16(), 1);
    ::core::assert_eq!(hm3.len(), 1);

    #[cfg(feature = "indexmap")]
    {
        let is0: ::confined::TinyIndexSet<u8> = ::confined::tiny_iset![];
        let is1 = ::confined::tiny_iset![1u8, 2u8, 1u8];
        let is2 = ::confined::small_iset![3u8];
        let is3 = ::confined::medium_iset![4u8, 5u8];
        ::core::assert_eq!(is0.len_u8(), 0);
        ::core::assert_eq!(is1.len_u8(), 2);
        ::core::assert_eq!(is2.len_u16(), 1);
        ::core::assert_eq!(is3.len(), 2);

        let im0: ::confined::TinyIndexMap<u8, u8> = ::confined::tiny_imap! {};
        let im1 = ::confined::tiny_imap! { 1u8 => 10u8, 2u8 => 20u8 };
        let im2 = ::confined::small_imap! { 3u8 => 30u8 };
        let im3 = ::confined::medium_imap! { 4u8 => 40u8 };
        ::core::assert_eq!(im0.len_u8(), 0);
        ::core::assert_eq!(im1.len_u8(), 2);
        ::core::assert_eq!(im2.len_u16(), 1);
        ::core::assert_eq!(im3.len(), 1);
    }
}
