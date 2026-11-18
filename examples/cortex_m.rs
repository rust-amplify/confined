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

//! Link-only bare-metal smoke test; no board-specific startup or heap required.

#![cfg_attr(target_os = "none", no_std)]
#![cfg_attr(target_os = "none", no_main)]

use confined::{HeaplessLinearMap, HeaplessString, HeaplessVec};

fn exercise() {
    let mut values = HeaplessVec::<u8, 4, 1, 3>::try_from_iter([1]).unwrap();
    values.extend([2, 3]).unwrap();
    assert!(values.push(4).is_err());
    assert_eq!(values.remove(1), Ok(2));

    let mut text = HeaplessString::<8, 0, 4>::new();
    text.push('🦀').unwrap();
    assert_eq!(text.len(), 4);
    assert!(text.push('!').is_err());

    let mut map = HeaplessLinearMap::<u8, u8, 2, 1, 2>::with_key_value(1, 10);
    map.insert(2, 20).unwrap();
    assert!(map.insert(3, 30).is_err());
    assert_eq!(map.remove(&2), Ok(Some(20)));
    core::hint::black_box((values, text, map));
}

#[cfg(not(target_os = "none"))]
fn main() {
    exercise();
}

// SAFETY: This standalone bare-metal link smoke binary defines `_start` once as
// the unique entry symbol selected by `-C link-arg=-e_start`.
#[cfg(target_os = "none")]
#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    exercise();
    loop {
        core::hint::spin_loop();
    }
}

#[cfg(target_os = "none")]
#[panic_handler]
fn panic(_: &core::panic::PanicInfo<'_>) -> ! {
    loop {
        core::hint::spin_loop();
    }
}
