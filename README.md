unicode-intervals
=================

[<img alt="github" src="https://img.shields.io/badge/github-8da0cb?style=flat-square&labelColor=555555&logo=github" height="20">](https://github.com/Stranger6667/unicode-intervals)
[<img alt="crates.io" src="https://img.shields.io/crates/v/unicode-intervals.svg?style=flat-square&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/unicode-intervals)
[<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-unicode-intervals-66c2a5?style=flat-square&labelColor=555555&logo=docs.rs" height="20">](https://docs.rs/unicode-intervals)
[<img alt="build status" src="https://img.shields.io/github/actions/workflow/status/Stranger6667/unicode-intervals/ci.yml?branch=main&style=flat-square" height="20">](https://github.com/Stranger6667/unicode-intervals/actions?query=branch%main)

This library provides a way to search for Unicode code point intervals by categories, ranges, and custom character sets.

The main purpose of `unicode-intervals` is to simplify generating strings that matching specific criteria.

```toml
[dependencies]
unicode-intervals = "0.1"
```

<br>

## Example

The example below will produce code point intervals of uppercase & lowercase letters less than 128 and will include the `☃` character.

```rust
use unicode_intervals::{UnicodeVersion, UnicodeCategory};

let intervals = UnicodeVersion::V15_0_0.query(
    // include categories
    UnicodeCategory::UPPERCASE_LETTER | UnicodeCategory::LOWERCASE_LETTER,
    None,        // exclude categories
    None,        // minimum codepoint
    128,         // maximum codepoint
    "☃",         // include characters
    None         // exclude characters
).expect("Invalid query input");
assert_eq!(intervals, &[(65, 90), (97, 122), (9731, 9731)])
```

## Details

Include or exclude Unicode general categories and their combinations by their full name or abbreviation:

```rust
// All letters + open punctuation
let categories = UnicodeCategory::L | UnicodeCategory::Ps;
```

Restrict the output to code points within a certain range:

```rust
let intervals = UnicodeVersion::V15_0_0.query(
    None,
    None,
    65,
    128,
    None,
    None
).expect("Invalid query input");
assert_eq!(intervals, &[(65, 128)])
```

Include or exclude specific characters:

```rust
let intervals = UnicodeVersion::V15_0_0.query(
    UnicodeCategory::PARAGRAPH_SEPARATOR,
    None,
    None,
    None,
    "☃-123",
    None
).expect("Invalid query input");
assert_eq!(intervals, &[(45, 45), (49, 51), (8233, 8233), (9731, 9731)])
```

## Unicode version support

`unicode-intervals` supports Unicode 9.0.0 - 15.0.0.

#### License

<sup>
Licensed under either of <a href="LICENSE-APACHE">Apache License, Version
2.0</a> or <a href="LICENSE-MIT">MIT license</a> at your option.
</sup>

<br>

<sub>
Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this crate by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.
</sub>
