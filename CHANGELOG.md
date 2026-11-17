Change Log
==========

0.5.0 (unreleased)
------------------

This version repairs confinement bypasses and restores the type aliases,
macros and tests left behind when the crate was split out of `amplify`. It
migrates to `collections-traits`, adds explicit allocation features,
allocator-aware and fixed-capacity collections, and requires Rust 1.100.0
with edition 2024. The changes below are not an audit or validation report;
resource limitations are documented in [SECURITY.md](./SECURITY.md).

### Breaking changes

| Change | Migration |
|--------|-----------|
| The MSRV is Rust 1.100.0 and the crate uses edition 2024. | Update the Rust toolchain; dependent crates need not change their edition. |
| `VecDeque`: `pop_front`, `pop_back`, `truncate` and `drain` exist only when `MIN_LEN` is 0, like `pop` of `Vec`. | Use `remove(index)`, which checks the lower bound. |
| `entry()` returns `confined::Entry` instead of the entry of the wrapped map. The new type can't remove the entry. | Use `key`, `or_insert`, `or_insert_with`, `or_default` and `and_modify`; `or_insert_with_key` is available only for built-in maps. Remove with `remove(&key)` of the confined map. |
| The blanket `FromStr` implementation is replaced with implementations for `String` and `AsciiString`. `FromStr::Err` is `Error` for `String` (it was `Infallible`) and `AsciiError` for `AsciiString` (it was `AsAsciiStrError`). | Handle the error. |
| `"...".parse()`, `FromHex` and `Deserialize` reject the data which do not fit the limits. `FromHex` reports `hex::Error::InvalidLength` with the violated limit and the number of bytes. | None. The data which were accepted in violation of the limits are an error now. |
| `Confined` has no struct-level collection bound; size-aware methods require `ItemLen: collections::CollectionBase`. `Item` is inherited; `Access`, `ViaCollection` and `ViaCollectionAlloc` are removed. | Ordinary custom collections can use `impl ItemLen for MyCollection {}`. Implement `CollectionBase` for `Item`, `item_count`, `push` and `clear`; override length/growth/capacity hooks where needed. |
| `Collection` now adds only `with_capacity` and the `Default` / `Extend` / `FromIterator` construction requirements to `CollectionBase`. | Use `CollectionBase` for basic collection operations instead of unnecessarily requiring construction traits. |
| The map API uses `CollectionMap`; `entry()` separately requires `CollectionEntry` and `MapEntry`. Set operations use `CollectionSet` where available. | Entry-less maps can use generic `insert`, `get_mut`, `values_mut` and retention methods. Add `CollectionEntry` only when implementing an entry API. |
| Allocation is an explicit `alloc` feature. Default features remain `std` + `hex`; `std`, `ascii`, `indexmap` and `hashbrown` imply `alloc`, but `heapless`, `serde` and `hex` do not. | With defaults disabled, enable `alloc` for allocating collections. Use `heapless` for fixed-capacity storage without an allocator. |
| `serde` no longer enables `ascii?/serde`. ASCII serialization now needs `ascii-serde`, which enables `ascii`, `serde`, `std` and `ascii/serde`; `all` includes it. | Replace `ascii,serde` with `ascii-serde` when serializing ASCII collections. Bare-metal `ascii` + `serde` builds without ASCII serialization; ASCII 1.1's serde code unconditionally imports `std`. |
| `Error` adds `CapacityExceeded { len }` for native capacity/allocation failure. | Update exhaustive matches and distinguish native capacity failure from the confinement `Oversize` error. |
| `with()`, `with_in()` and `with_key_value()` fail to build unless `MIN_LEN <= 1 <= MAX_LEN`. `with()` and `with_in()` also require `MAX_LEN >= MAX_ITEM_LEN` (`char::MAX_LEN_UTF8` for `String`). | Construct a value with a valid length using `try_from` or `parse`. |
| `try_from` and the constructors built on it fail to build for a type with `MIN_LEN > MAX_LEN`. | Fix the limits. Such a type could never hold a value. |
| `BTreeMap::append` takes a source map with `MIN_LEN = 0` and has one const parameter instead of two. | Release a source which has a lower bound, and confine it again with `MIN_LEN = 0`. |
| `BTreeMap::split_off` requires `K: Clone, V: Clone` and returns cloned entries. | This avoids Rust 1.100's panic-unsafe native split. Bounds are checked before copying or mutation; comparison/clone panics leave the source unchanged. See `SECURITY.md` for the memory tradeoff. |
| The deprecated methods are removed. | `from_collection_unsafe` → `from_checked`, `from_iter_unsafe` → `from_iter_checked`, `from_slice_unsafe` → `from_slice_checked`, `as_inner` → `as_unconfined`, `to_inner` → `to_unconfined`, `into_inner` and `unbox` → `release`, `push_from` → `push_front`. |
| The `indexmap` feature no longer enables `std`; default-hasher index aliases and macros require both features. | Enable `std` explicitly for those aliases/macros. In `no_std` + `alloc`, wrap a custom-hasher collection directly in `Confined` with `S: BuildHasher`; `Default` is only required by constructors that need it. |
| `first()` and `last()` on indexable collections, and `first_mut()` and `last_mut()` where mutable indexing is supported, now resolve to the confined accessors for all `MIN_LEN` values; calling them requires `MIN_LEN > 0` at build time. | For an optional result, use `as_slice().first()` / `as_slice().last()` or call the wrapped collection through `as_unconfined()`. Mutable optional accessors are available through `as_mut()` when the collection implements `AsMut`. |

The inline const bound failures listed above are reported by `cargo build`
and `cargo test`, but not by `cargo check`.

### Fixed

- `"...".parse()`, `FromHex` and serde's `Deserialize` returned values outside
  the limits of their type.
- `String`: `push` and `remove` counted a character as a single item, while
  the length of a string is measured in bytes. A multi-byte character could
  take the string above `MAX_LEN` or below `MIN_LEN`.
- `VecDeque`: `pop_front`, `pop_back`, `truncate` and `drain` ignored
  `MIN_LEN`.
- `with()` and `with_key_value()` ignored `MAX_LEN`.
- `entry()` returned the entry of the wrapped map, which could remove itself
  and take a `BTreeMap`, `HashMap` or `IndexMap` below `MIN_LEN`.
- `try_retain` returned `Undersize` after the elements were already removed,
  leaving the map below `MIN_LEN`.
- `BTreeMap::append` emptied the source map whatever its `MIN_LEN` was, and
  refused a merge which fits when the maps shared keys.
- `HashMap::reserve` passed the intended total length to the wrapped map as
  the number of additional elements. It reserved too much, and in a debug
  build `reserve(usize::MAX)` panicked on an arithmetic overflow.
- `from_slice_checked` panicked for a slice of exactly `MIN_LEN` items.
- Iterator construction and checked hex decoding could consume or reserve
  from untrusted iterator size hints. Construction is now lazy, does not
  reserve `MIN_LEN`, and stops at rejected growth or native capacity failure.
- Borrowed slices, UTF-8 strings and ASCII strings could be copied before
  confinement validation; bounds are now checked first.
- Duplicate set/map `push` and `extend` incorrectly failed at `MAX_LEN`.
  Duplicate items and existing keys now have zero growth.
- B-tree split error recovery could violate the lower bound after a comparison
  panic. Native multi-level splitting also exposed an upstream panic-safety
  defect; clone-before-mutation splitting avoids both paths.
- Byte-vector `io::Write` now reports allocation failure instead of invoking
  an infallible allocation path.
- Checked fixed-capacity growth could panic. Heapless `push`, `extend`,
  `insert` and `entry` return `CapacityExceeded` for exhausted native storage.
- Allocating serde integrations now follow the explicit `alloc` feature,
  while `serde` alone remains allocator-free.
- No-default builds and bare-metal `ascii` + `serde` builds no longer inherit
  unwanted allocation or the upstream ASCII serde dependency on `std`.
  ASCII itself still requires pointer atomics because of an upstream `Arc`
  import; Cortex-M0 supports the non-ASCII configurations only.
- `U24MAX`, `U32MAX`, `U40MAX`, `U48MAX`, `U56MAX` and `U64MAX` are capped at
  `usize::MAX` on narrower targets; `amplify_num` conversions do not use
  truncating intermediate casts.
- Unresolved documentation links to `u24::MAX` and `mem::forget`.

### Added

- Allocation-free lower/uppercase hex formatting. Alternate formatting (`#`)
  abbreviates blobs longer than 16 bytes to their first and last eight bytes.

- Type aliases for the common limits, exported from the crate root: `Tiny*`,
  `Small*`, `Medium*`, `Large*`, `Confined*` and `NonEmpty*` for `String`,
  `AsciiString` (`*Ascii`), `Vec<u8>` (`*Blob`), `Vec`, `VecDeque` (`*Deque`),
  `LinkedList` (`*List`), `BinaryHeap` (`*Heap`), `HashSet`, `BTreeSet`
  (`*OrdSet`), `HashMap`, `BTreeMap` (`*OrdMap`), `IndexSet` and `IndexMap`.
  `ConfinedHashMap` wraps `HashMap<K, V>`; the unported version wrapped
  `HashSet<K, V>`.
- Constructor macros `tiny_*!`, `small_*!` and `medium_*!` for `s`, `blob`,
  `vec`, `deque`, `list`, `heap`, `set`, `bset`, `map`, `bmap`, `iset` and
  `imap`. They do not need any other macro or import in the scope of the
  caller.
- `LinkedList` and `BinaryHeap` confinement. Lists support `push_front`,
  `push_back` and checked `remove(index)`; `pop_front` / `pop_back` on lists
  and `pop` on heaps require `MIN_LEN = 0`.
- Allocator-aware `Vec<T, A: Allocator>` support through the same `Confined`
  type. `new_in`, `with_capacity_in`, `try_with_capacity_in`, `with_in`,
  `try_from_iter_in` and `from_iter_checked_in` accept an allocator explicitly
  without requiring `A: Default`, including single-item `with_in` construction.
- `hashbrown` support and the `confined::hashbrown` re-export, including
  `HashMap<K, V, S, A>` / `HashSet<T, S, A>` with `S: BuildHasher` and
  `A: Allocator`. Native operations do not require default hashers/allocators;
  constructors require `Default` only where needed.
- Allocator-free `heapless` support and the `confined::heapless` re-export:
  `Vec<T, N, L>`, `String<N, L>`, `Deque`, `BinaryHeap`, `IndexSet`, `IndexMap`
  and `LinearMap`. `IndexSet` has a direct checked `remove(&T)` returning
  `Result<bool, Error>`, but no `take` because upstream lacks `CollectionSet`.
- Fixed-capacity aliases `HeaplessVec`, `HeaplessString`, `HeaplessDeque`,
  `HeaplessHeap`, `HeaplessIndexSet`, `HeaplessIndexMap` and
  `HeaplessLinearMap`, with separate native `CAP` and confinement `MIN` /
  `MAX` parameters; vectors and strings also accept `L = usize`.
- `try_reserve(additional)` rejects requests beyond the remaining confinement
  allowance before fallible native reservation. `try_with_capacity(cap)` and
  `try_with_capacity_in(cap, allocator)` cap their reservation request at
  `MAX_LEN`; native failure is `Error::CapacityExceeded { len }`.
- An allocator parameter defaulting to `Global` on all `*Vec` and `*Blob`
  aliases, for example `TinyVec<T, A = Global>` and `TinyBlob<A = Global>`.
- `Entry`, the type returned by `entry()` of a confined map, with the methods
  `key`, `or_insert`, `or_insert_with`, `or_default` and `and_modify` for any
  `CollectionEntry`, including custom maps using `MapEntry`. The additional
  `or_insert_with_key` method remains specific to `BTreeMap`, `HashMap` and
  `IndexMap`.
- `ItemLen` hooks: `confined_len` defaults to `item_count`; `item_len` and
  `MAX_ITEM_LEN` default to one. `item_growth` defaults to `item_len`, with
  zero growth for duplicate sets/maps. `try_push` defaults to infallible
  `CollectionBase::push`; capacity-backed types override it fallibly.
  `prepare_entry` ensures native room before exposing an infallible entry
  insertion. UTF-8 strings use `CollectionBytes` for O(1) byte confinement,
  rather than upstream's character-counting `item_count`.
- `FromStr` for confined `String` and `AsciiString`.
- `AsciiError` is exported from the crate root under the `ascii` feature.
- `first_mut()` and `last_mut()` for the collections which contain at least
  one item and can be indexed mutably by position.
- `from_array()` for `Vec`. The length of the array is checked against the
  limits when the program is built.
- `widen()`, which converts a confinement into a confinement with wider
  limits. The limits are compared when the program is built.
- Tests of the public API in `tests/confinement.rs` and regression tests of
  the size guarantee in `tests/guarantee.rs`, plus allocator coverage in
  `tests/allocator.rs` and collection integration coverage in
  `tests/collections.rs`. The examples of `README.md`, including allocator,
  list and heap usage, are run as documentation tests when all features are
  enabled.
- Crate-level documentation.
- CI matrices for no-allocation feature combinations and both Cortex-M
  `thumbv6m-none-eabi` and `thumbv7em-none-eabi`. Embedded checks cover
  heapless/no-alloc and `alloc,hex,indexmap,hashbrown,ascii,serde`; the
  `cortex_m` example provides a no-allocator startup/link smoke check, not
  hardware execution. MSRV, stable, beta and nightly coverage is retained.
- Resource-limit and custom-collection guidance in `SECURITY.md`; README
  doctests are feature-gated for all-features and no-default-feature builds.

### Changed

- The `collections` import aliases the released `collections-traits` 0.1.0
  package from crates.io. `amplify_num` 0.6.0 remains pinned directly to
  develop commit `e359d07`. Downstream manifests no longer need a `[patch]`
  section.
- `HashMap`, `HashSet`, `IndexMap` and `IndexSet` implementations accept
  `S: BuildHasher`; `Default` is needed only by constructors that use it.
  Existing aliases keep their default hasher. Custom-hasher index collections
  work in `no_std` + `alloc`; default-hasher index aliases/macros require `std`.
- `BTreeMap` keys require `Ord` but no longer require `Hash`.
- Compile-time bound checks use inline const assertions instead of helper
  structs. String item-size bounds use `char::MAX_LEN_UTF8`, and boolean
  confinement checks use `bool::ok_or`.
- `Error` and, with `ascii`, `AsciiError` implement `core::error::Error`
  even without `std`.
- `with()` and `with_key_value()` are available for any bounds that can hold
  exactly one item. `with()` also checks the maximum item size for the
  collection.
- `first()`, `last()`, `first_mut()` and `last_mut()` are available for any
  positive `MIN_LEN`, rather than only `MIN_LEN = 1`.
- `len_u8()`, `len_u16()`, `len_u24()` and `len_u32()` are available whenever
  `MAX_LEN` does not exceed the maximum representable by their return type.
- `std::io::Write` for confined `Vec<u8, A>` is available for every `MIN_LEN`
  and any `A: Allocator`; writes append and preserve the lower bound. Hex
  formatting also supports any allocator; `FromHex` requires `A: Default`.
- `try_retain` never takes the map below `MIN_LEN`: it stops removing at the
  lower bound, keeps the remaining rejected elements and returns `Undersize`.
- `try_from_iter` and `from_iter_checked` stop at the first rejected growth.
  `Error::Oversize` does not report the iterator's total length. Duplicates
  can continue indefinitely without growing a set or map.
- `HashMap::reserve` reserves space for `min(additional, MAX_LEN - len)`
  elements.
- `alloc` enables allocating dependency integrations when those dependencies
  are enabled; `std` implies `alloc` and enables their standard integrations.
  `ascii`, `indexmap` and `hashbrown` imply `alloc`; `hex`, `serde` and
  `heapless` alone do not.
- Hex formatting streams without an intermediate allocation.

### Removed

- The `Access` / `ViaCollection` / `ViaCollectionAlloc` dispatch layer.
- `wasm-bindgen` integration dependencies.
- `src/confinement.rs`, the single-file version of the crate which was not a
  part of the module tree. The deprecated `confined_*!` macros and the
  `smallvec` aliases and macros it contained are not ported.

### Known limitations

- Set `insert` and `retain` methods remain excluded because they overlap
  with generic map inherent methods. `heapless::IndexSet` supports `remove`
  but not `take`.
- Trees, linked lists and ASCII strings lack fallible upstream reservation
  and may abort on OOM. `Clone`, `From`, `Default` and explicit infallible
  constructors follow underlying behavior; fixed-capacity convenience
  constructors such as `with()` can panic.
- Generic serde deserialization validates only after constructing the inner
  collection; configure resource limits in the deserializer.
- Collection bounds do not bound iterator execution time, nested item sizes
  or hashing cost. Hashbrown's default hasher is not collision-DoS resistant;
  use a keyed/randomized hasher for hostile keys.
- Custom collection length, growth and mutation contracts must be correct.
- ASCII serialization requires the std-only `ascii-serde` feature, even
  though `ascii` + `serde` builds without `std`.
