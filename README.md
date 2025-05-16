KString2
===========

> Key String: Optimized for map keys.

[![github](https://img.shields.io/badge/github-katyo/kstring-8da0cb.svg?style=for-the-badge&logo=github)](https://github.com/katyo/kstring)
[![crate](https://img.shields.io/crates/v/kstring2.svg?style=for-the-badge&color=fc8d62&logo=rust)](https://crates.io/crates/kstring2)
[![docs](https://img.shields.io/badge/docs.rs-kstring2-66c2a5?style=for-the-badge&logo=docs.rs)](https://docs.rs/kstring2)
[![MIT](https://img.shields.io/badge/License-MIT-brightgreen.svg?style=for-the-badge&logo=opensourceinitiative&logoColor=white)](https://opensource.org/licenses/MIT)
[![Apache 2.0](https://img.shields.io/badge/License-Apache--2.0-brightgreen.svg?style=for-the-badge&logo=opensourceinitiative&logoColor=white)](https://opensource.org/licenses/apache-2-0)
[![CI](https://img.shields.io/github/actions/workflow/status/katyo/kstring/ci.yml?branch=fork&style=for-the-badge&logo=github-actions&logoColor=white)](https://github.com/katyo/kstring/actions?query=workflow%3ACI)

This crate is extended and more feature rich version of [kstring](https://crates.io/crates/kstring) with some significant changes.
But internal representation of string types still is same so you can convert corresponding types between both crates (see `kstring` feature).

## Background

Considerations:
- Large maps
- Most keys live and drop without being used in any other way
- Most keys are relatively small (single to double digit bytes)
- Keys are immutable
- Allow zero-cost abstractions between structs and maps (e.g. no allocating
  when dealing with struct field names)

Ramifications:
- Inline small strings rather than going to the heap.
- Preserve `&'static str` across strings (`KString`),
  references (`KStringRef`), and lifetime abstractions (`KStringCow`) to avoid
  allocating for struct field names.
- Use `Box<str>` rather than `String` to use less memory.

Significant changes:
- Because `From<&'static str>` is unsound it changed to `From<&str>`. To instantiate from static str use `KString::from_static` instead of `From::from`.
- Added default generic to types `KStringBase` and `KStringCowBase` which renamed to `KString` and `KStringCow`. Corresponding type aliases is removed. To instantiate types with default backend wrap it with angle brackets (`KString::from_ref("abc")` => `<KString>::from_ref("abc")`).

Features:
- `max_inline`: Instead of aligning the inline-string for performance (15 bytes + length on 64-bit), use the full width (22 bytes on 64-bit)
- `arc`: Instead of using `Box<str>`, use `Arc<str>`.  Note: allocations are fast enough that this can actually slow things down for small enough strings.
- `serde`: Enablse [serde](https://serde.rs/) support.
- `diesel`: Enables [diesel](https://diesel.rs/) support.
- `kstring`: Enables conversion from/into corresponding [kstring](https://crates.io/crates/kstring) types.

Alternatives, see [rust-string-comparison](https://github.com/Wybxc/rust-string-comparison) and [string-benchmarks-rs](https://github.com/epage/string-benchmarks-rs).

## License

Licensed under either of

* Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in the work by you, as defined in the Apache-2.0
license, shall be dual licensed as above, without any additional terms or
conditions.
