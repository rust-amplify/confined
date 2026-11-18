# Security

`confined` enforces collection-length bounds, not a complete memory or
execution-time budget. Applications handling untrusted input must also limit
the resources used by elements, iterators, hashers and deserializers.

The library uses `#![forbid(unsafe_code)]`. This does not audit dependencies,
exclude unsafe code inside them, or guarantee absence of logic errors.
`Cargo.lock` records the tested Git dependency revision for this repository,
while downstream Git consumers resolve the `develop` branch unless locked in
their own `Cargo.lock`; neither is an audit result or a statement about
dependency advisories.

## Length and capacity guarantees

- Confinement checks `MIN_LEN..=MAX_LEN`. UTF-8 strings are confined by bytes,
  not characters, using O(1) byte length. Other supported collections count
  items; an item's nested allocations are not bounded by the outer length.
- Native capacity and confinement are separate limits. Checked growth of
  fixed-capacity heapless storage returns `Error::CapacityExceeded { len }`
  rather than panicking when the native storage is full. Checked `push`,
  `extend`, `insert` and `entry` respect both limits.
- On supported reservable collections, `try_reserve(additional)` rejects a
  request exceeding the remaining confinement allowance before fallible
  native reservation. `try_with_capacity(cap)` and
  `try_with_capacity_in(cap, allocator)` cap the initial reservation request
  at `MAX_LEN`. Native allocators may round capacity upward, so a length
  bound is not an exact allocation-byte limit.
- Borrowed slices, UTF-8 strings and ASCII strings are length-checked before
  copying. Iterator construction is lazy: it does not reserve from untrusted
  size hints or preallocate `MIN_LEN`. Checked hex decoding similarly bounds
  growth without trusting allocation hints. Hex formatting streams without
  an intermediate allocation, although the output consumer may allocate.
- `extend` is not transactional: successfully inserted items remain if a
  later item fails. Handle errors without assuming the entire operation was
  rolled back.

## Remaining resource risks

**Allocation can still abort.** Trees, linked lists and ASCII strings lack
fallible upstream reservation. Their growth may abort on OOM even through a
checked confinement method. `Clone`, `From`, `Default` and explicit infallible
constructors retain the behavior of the inner collection. Basic convenience
constructors such as `new()` and `with()` are not general fallible resource
APIs; `with()` can panic when fixed capacity is insufficient. Prefer checked
construction and fallible reservation where available.

`FromHex` also retains `Vec::push` allocation semantics: the upstream
`hex::Error` type cannot represent allocation failure. Decoding validates
actual length and ignores iterator size hints, but allocation can still abort.

**B-tree splitting avoids an upstream panic-safety defect.** On Rust 1.100,
a panicking comparator during a multi-level native `BTreeMap::split_off`
can corrupt the tree ([rust-lang/rust#158165](https://github.com/rust-lang/rust/issues/158165)).
The confined implementation never calls it: it checks both resulting lengths,
builds the right-hand map by cloning, then removes the original suffix without
comparisons. Consequently `split_off` requires `K: Clone, V: Clone` and uses
additional memory. Comparison or clone panics leave the original unchanged;
a destructor panic can leave partial removal, always above the lower bound.
Regression tests inject a panic at every comparison in both small and
multi-level trees. This protection does not apply to a map after `release()`.

**Deserialization constructs first.** Generic serde `Deserialize` validates
confinement only after constructing the inner collection `C`. Rejecting an
oversized result does not undo the memory or CPU already spent constructing
it. Configure input-byte, nesting, collection-length and other resource limits
in the deserializer or an application-specific bounded parser.

**Iteration is not time-bounded.** Duplicate set items and existing map keys
have zero growth, so checked `push` / `extend` accept them at `MAX_LEN`.
An infinite iterator of duplicates, or another valid non-growing iterator,
need not terminate. Iterator production itself can also be arbitrarily
expensive. Impose item-processing budgets, timeouts or cancellation separately
when input is untrusted.

**Hashing can be adversarial.** Hashbrown's default hasher is not resistant to
hash-collision denial of service. For hostile keys, explicitly choose a
keyed/randomized hasher designed for that threat model, with appropriate key
generation. The same consideration applies to custom hashers used by
`indexmap` or other collections. A maximum entry count does not bound hashing
or key-comparison work.

**Custom collection contracts matter.** `ItemLen` inherits `Item` from
`collections::CollectionBase`. Its length, item-size, growth and mutation hooks
must accurately describe the collection. The default `try_push` calls
infallible `CollectionBase::push`; capacity-limited implementations need a
fallible override. `prepare_entry` must provide native room before an
infallible entry insertion is exposed. Incorrect custom length/growth,
mutation or entry behavior can invalidate confinement guarantees.

## Feature and platform boundaries

Default features are `std` + `hex`, and `std` implies `alloc`. With defaults
disabled, the core wrapper and the `heapless`, `serde` and `hex` features do
not require allocation. `ascii`, `indexmap` and `hashbrown` imply `alloc`.

`ascii` + `serde` compiles without `std`, but does not enable ASCII
serialization. The separate `ascii-serde` feature enables `ascii/serde` and
requires `std` because ASCII 1.1's serde code unconditionally imports it.
The `all` feature includes this std-only integration.

ASCII 1.1 also imports `alloc::sync::Arc` without a target-atomic guard.
Consequently `ascii` is unavailable on Cortex-M0 (`thumbv6m-none-eabi`), even
without serde. The core, heapless and non-ASCII allocating integrations do not
have that restriction. ASCII is cross-checked on Cortex-M4 instead.

The Cortex-M CI checks and allocator-free example link smoke checks cover
`thumbv6m-none-eabi` and `thumbv7em-none-eabi`; they do not execute on hardware.
Checks with `alloc` enabled do not supply a device allocator or establish a
runtime memory budget. Neither CI configuration nor this document constitutes
an independent security audit or a claim that validation has completed.

## Reporting Security Issues

**Please do not report security vulnerabilities through public GitHub issues.**

If you believe you have found an issue meeting the
[definition of a security vulnerability][definition], please report it
privately as described below.

Instead, please report them to the repository maintainers by sending a **GPG
encrypted e-mail** to _all maintainers of a specific repo_ using their GPG keys.

A list of repository maintainers and their keys and e-mail addresses is
provided in [MAINTAINERS.md](./MAINTAINERS.md).

You should receive a response within 72 hours. If for some reason you do not,
please follow up via email to ensure we received your original message.

Please include the requested information listed below (as much as you can
provide) to help us better understand the nature and scope of the possible
issue:

* Type of issue (e.g. confinement bypass, unexpected panic or resource exhaustion)
* Full paths of source file(s) related to the manifestation of the issue
* The location of the affected source code (tag/branch/commit or direct URL)
* Rust toolchain, target, enabled features and dependency revisions
* Collection type, confinement bounds, allocator/hasher and native capacity
* Any other configuration required to reproduce the issue
* Step-by-step instructions to reproduce the issue
* Proof-of-concept or exploit code (if possible)
* Impact of the issue, including how an attacker might exploit the issue

This information will help us triage your report more quickly.

## Preferred Languages

We prefer all communications to be in English.

## Policy

We follow the principle of [Coordinated Vulnerability Disclosure][disclosure].

[definition]: https://aka.ms/opensource/security/definition
[disclosure]: https://aka.ms/opensource/security/cvd
