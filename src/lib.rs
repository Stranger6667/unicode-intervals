//! [![github]](https://github.com/Stranger6667/unicode-intervals)&ensp;[![crates-io]](https://crates.io/crates/unicode-intervals)&ensp;[![docs-rs]](https://docs.rs/unicode-intervals)
//!
//! [github]: https://img.shields.io/badge/github-8da0cb?style=flat-square&labelColor=555555&logo=github
//! [crates-io]: https://img.shields.io/badge/crates.io-fc8d62?style=flat-square&labelColor=555555&logo=rust
//! [docs-rs]: https://img.shields.io/badge/docs.rs-66c2a5?style=flat-square&labelColor=555555&logo=docs.rs
//!
//! <br>
//!
//! This library provides a way to search for Unicode code point intervals by categories, ranges,
//! and custom character sets.
//!
//! The main purpose of `unicode-intervals` is to simplify generating strings that matching
//! specific criteria.
//!
//! # Example
//!
//! ```rust
//! use unicode_intervals::{UnicodeVersion, UnicodeCategory};
//!
//! let intervals = UnicodeVersion::V15_0_0.query(
//!     // include categories
//!     UnicodeCategory::UPPERCASE_LETTER | UnicodeCategory::LOWERCASE_LETTER,
//!     None,        // exclude categories
//!     None,        // minimum codepoint
//!     128,         // maximum codepoint
//!     "☃",         // include characters
//!     None         // exclude characters
//! ).expect("Invalid query input");
//! assert_eq!(intervals, &[(65, 90), (97, 122), (9731, 9731)])
//! ```
//!
//! # Details
//!
//! Include or exclude Unicode general categories and their combinations by their full name or
//! abbreviation:
//!
//! ```rust
//! # use unicode_intervals::{UnicodeVersion, UnicodeCategory};
//! // All letters + open punctuation
//! let categories = UnicodeCategory::L | UnicodeCategory::Ps;
//! ```
//!
//! Restrict the output to code points within a certain range:
//!
//! ```rust
//! # use unicode_intervals::{UnicodeVersion, UnicodeCategory};
//! let intervals = UnicodeVersion::V15_0_0.query(
//!     None,
//!     None,
//!     65,
//!     128,
//!     None,
//!     None
//! ).expect("Invalid query input");
//! assert_eq!(intervals, &[(65, 128)])
//! ```
//!
//! Include or exclude specific characters:
//!
//! ```rust
//! # use unicode_intervals::{UnicodeVersion, UnicodeCategory};
//! let intervals = UnicodeVersion::V15_0_0.query(
//!     UnicodeCategory::PARAGRAPH_SEPARATOR,
//!     None,
//!     None,
//!     None,
//!     "☃-123",
//!     None
//! ).expect("Invalid query input");
//! assert_eq!(intervals, &[(45, 45), (49, 51), (8233, 8233), (9731, 9731)])
//! ```
//!
//! # Unicode version support
//!
//! `unicode-intervals` supports Unicode 9.0.0 - 15.0.0.
#![warn(
    clippy::cast_possible_truncation,
    clippy::doc_markdown,
    clippy::explicit_iter_loop,
    clippy::map_unwrap_or,
    clippy::match_same_arms,
    clippy::needless_borrow,
    clippy::needless_pass_by_value,
    clippy::print_stdout,
    clippy::redundant_closure,
    clippy::trivially_copy_pass_by_ref,
    missing_debug_implementations,
    missing_docs,
    trivial_casts,
    trivial_numeric_casts,
    unused_extern_crates,
    unused_import_braces,
    variant_size_differences,
    clippy::integer_arithmetic,
    clippy::unwrap_used,
    clippy::semicolon_if_nothing_returned,
    clippy::cargo
)]
#![allow(clippy::redundant_static_lifetimes)]
use crate::constants::MAX_CODEPOINT;
use core::fmt;
use std::str::FromStr;

mod categories;
mod constants;
mod error;
mod intervals;
mod query;
mod tables;
pub use crate::{
    categories::{UnicodeCategory, UnicodeCategorySet},
    error::Error,
};

#[cfg(feature = "__benchmark_internals")]
/// Internals used for benchmarking.
pub mod internals {
    /// Unicode categories.
    pub mod categories {
        pub use crate::categories::merge;
    }

    /// Intervals manipulation.
    pub mod intervals {
        pub use crate::intervals::{from_str, merge, subtract};
    }

    /// Querying Unicode intervals.
    pub mod query {
        pub use crate::query::{intervals_for_set, query};
    }
}

/// Interval between two Unicode codepoints.
pub type Interval = (u32, u32);

/// Supported Unicode versions.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum UnicodeVersion {
    /// Unicode 9.0.0
    V9_0_0,
    /// Unicode 10.0.0
    V10_0_0,
    /// Unicode 11.0.0
    V11_0_0,
    /// Unicode 12.0.0
    V12_0_0,
    /// Unicode 12.1.0
    V12_1_0,
    /// Unicode 13.0.0
    V13_0_0,
    /// Unicode 14.0.0
    V14_0_0,
    /// Unicode 15.0.0
    V15_0_0,
}

impl fmt::Display for UnicodeVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for UnicodeVersion {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "9.0.0" => Ok(UnicodeVersion::V9_0_0),
            "10.0.0" => Ok(UnicodeVersion::V10_0_0),
            "11.0.0" => Ok(UnicodeVersion::V11_0_0),
            "12.0.0" => Ok(UnicodeVersion::V12_0_0),
            "12.1.0" => Ok(UnicodeVersion::V12_1_0),
            "13.0.0" => Ok(UnicodeVersion::V13_0_0),
            "14.0.0" => Ok(UnicodeVersion::V14_0_0),
            "15.0.0" => Ok(UnicodeVersion::V15_0_0),
            _ => Err(Error::InvalidVersion(s.to_string().into_boxed_str())),
        }
    }
}

impl UnicodeVersion {
    /// Unicode version as a string.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            UnicodeVersion::V9_0_0 => "9.0.0",
            UnicodeVersion::V10_0_0 => "10.0.0",
            UnicodeVersion::V11_0_0 => "11.0.0",
            UnicodeVersion::V12_0_0 => "12.0.0",
            UnicodeVersion::V12_1_0 => "12.1.0",
            UnicodeVersion::V13_0_0 => "13.0.0",
            UnicodeVersion::V14_0_0 => "14.0.0",
            UnicodeVersion::V15_0_0 => "15.0.0",
        }
    }

    /// A sorted slice of slices where each item is a slice of intervals for every Unicode category.
    /// They are sorted alphabetically by their full name.
    #[inline]
    #[must_use]
    pub const fn table(self) -> &'static [&'static [Interval]] {
        match self {
            UnicodeVersion::V9_0_0 => tables::v9_0_0::BY_NAME,
            UnicodeVersion::V10_0_0 => tables::v10_0_0::BY_NAME,
            UnicodeVersion::V11_0_0 => tables::v11_0_0::BY_NAME,
            UnicodeVersion::V12_0_0 => tables::v12_0_0::BY_NAME,
            UnicodeVersion::V12_1_0 => tables::v12_1_0::BY_NAME,
            UnicodeVersion::V13_0_0 => tables::v13_0_0::BY_NAME,
            UnicodeVersion::V14_0_0 => tables::v14_0_0::BY_NAME,
            UnicodeVersion::V15_0_0 => tables::v15_0_0::BY_NAME,
        }
    }

    /// Get a slice of intervals for the provided Unicode category.
    #[inline]
    #[must_use]
    pub const fn intervals_for(self, category: UnicodeCategory) -> &'static [Interval] {
        self.table()[category as usize]
    }

    /// Unicode categories sorted by the number of intervals inside.
    #[inline]
    #[must_use]
    pub const fn normalized_categories(self) -> [UnicodeCategory; 30] {
        // Collect all categories & their lengths
        let mut lengths: [(UnicodeCategory, usize); 30] = [(UnicodeCategory::Cc, 0); 30];
        let mut idx = 0;
        let table = self.table();
        let categories = [
            UnicodeCategory::Pe,
            UnicodeCategory::Pc,
            UnicodeCategory::Cc,
            UnicodeCategory::Sc,
            UnicodeCategory::Pd,
            UnicodeCategory::Nd,
            UnicodeCategory::Me,
            UnicodeCategory::Pf,
            UnicodeCategory::Cf,
            UnicodeCategory::Pi,
            UnicodeCategory::Nl,
            UnicodeCategory::Zl,
            UnicodeCategory::Ll,
            UnicodeCategory::Sm,
            UnicodeCategory::Lm,
            UnicodeCategory::Sk,
            UnicodeCategory::Mn,
            UnicodeCategory::Ps,
            UnicodeCategory::Lo,
            UnicodeCategory::No,
            UnicodeCategory::Po,
            UnicodeCategory::So,
            UnicodeCategory::Zp,
            UnicodeCategory::Co,
            UnicodeCategory::Zs,
            UnicodeCategory::Mc,
            UnicodeCategory::Cs,
            UnicodeCategory::Lt,
            UnicodeCategory::Cn,
            UnicodeCategory::Lu,
        ];
        // `idx` is always less than 30 and will not overflow
        #[allow(clippy::integer_arithmetic)]
        while idx < table.len() {
            lengths[idx] = (categories[idx], table[idx].len());
            idx += 1;
        }
        // Bubble sort by length.
        // The main reason to use bubble sort is that it works in the `const` context

        loop {
            let mut swapped = false;
            let mut idx = 1;
            // Arithmetic here will not overflow as it is always less than 30 and more than 1
            #[allow(clippy::integer_arithmetic)]
            while idx < lengths.len() {
                if lengths[idx - 1].1 > lengths[idx].1 {
                    let left = lengths[idx - 1];
                    let right = lengths[idx];
                    lengths[idx - 1] = right;
                    lengths[idx] = left;
                    swapped = true;
                }
                idx += 1;
            }
            if !swapped {
                break;
            }
        }

        // Fill only categories & skip Cc & Cs
        let mut output: [UnicodeCategory; 30] = [
            UnicodeCategory::Cc,
            UnicodeCategory::Cc,
            UnicodeCategory::Cc,
            UnicodeCategory::Cc,
            UnicodeCategory::Cc,
            UnicodeCategory::Cc,
            UnicodeCategory::Cc,
            UnicodeCategory::Cc,
            UnicodeCategory::Cc,
            UnicodeCategory::Cc,
            UnicodeCategory::Cc,
            UnicodeCategory::Cc,
            UnicodeCategory::Cc,
            UnicodeCategory::Cc,
            UnicodeCategory::Cc,
            UnicodeCategory::Cc,
            UnicodeCategory::Cc,
            UnicodeCategory::Cc,
            UnicodeCategory::Cc,
            UnicodeCategory::Cc,
            UnicodeCategory::Cc,
            UnicodeCategory::Cc,
            UnicodeCategory::Cc,
            UnicodeCategory::Cc,
            UnicodeCategory::Cc,
            UnicodeCategory::Cc,
            UnicodeCategory::Cc,
            UnicodeCategory::Cc,
            UnicodeCategory::Cc,
            UnicodeCategory::Cs,
        ];
        let mut idx = 0;
        let mut ptr = 0;

        while idx < lengths.len() {
            let (category, _) = lengths[idx];
            // `idx` & `ptr` are always less than 30 and will not overflow
            #[allow(clippy::integer_arithmetic)]
            if category as u8 == UnicodeCategory::Cc as u8
                || category as u8 == UnicodeCategory::Cs as u8
            {
                idx += 1;
            } else {
                output[ptr] = category;
                ptr += 1;
                idx += 1;
            }
        }
        output
    }

    /// Return a vector of intervals covering the codepoints for all codepoints
    /// that meet the input criteria.
    ///
    /// # Errors
    ///
    ///   - `min_codepoint > max_codepoint`
    ///   - `min_codepoint > 1114111` or `max_codepoint > 1114111`
    #[inline]
    pub fn query<'a>(
        self,
        include_categories: impl Into<Option<UnicodeCategorySet>>,
        exclude_categories: impl Into<Option<UnicodeCategorySet>>,
        min_codepoint: impl Into<Option<u32>>,
        max_codepoint: impl Into<Option<u32>>,
        include_characters: impl Into<Option<&'a str>>,
        exclude_characters: impl Into<Option<&'a str>>,
    ) -> Result<Vec<Interval>, Error> {
        let exclude_categories: UnicodeCategorySet = exclude_categories
            .into()
            .unwrap_or_else(UnicodeCategorySet::new);
        let min_codepoint = min_codepoint.into().unwrap_or(0);
        let max_codepoint = max_codepoint.into().unwrap_or(MAX_CODEPOINT);
        if min_codepoint > MAX_CODEPOINT || max_codepoint > MAX_CODEPOINT {
            return Err(Error::CodepointNotInRange(min_codepoint, max_codepoint));
        }
        if min_codepoint > max_codepoint {
            return Err(Error::InvalidCodepoints(min_codepoint, max_codepoint));
        }
        Ok(query::query(
            self,
            include_categories.into(),
            exclude_categories,
            min_codepoint,
            max_codepoint,
            include_characters.into(),
            exclude_characters.into(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    #[test_case(None, None, &[(95, 95), (8255, 8256), (8276, 8276), (65075, 65076), (65101, 65103), (65343, 65343)])]
    #[test_case(None, Some(128), &[(95, 95)])]
    #[test_case(Some(65077), None, &[(65101, 65103), (65343, 65343)])]
    #[test_case(Some(65076), Some(65102), &[(65076, 65076), (65101, 65102)])]
    fn test_query(min_codepoint: Option<u32>, max_codepoint: Option<u32>, expected: &[Interval]) {
        let intervals = UnicodeVersion::V15_0_0
            .query(
                UnicodeCategory::Pc,
                None,
                min_codepoint,
                max_codepoint,
                None,
                None,
            )
            .expect("Invalid query");
        assert_eq!(intervals, expected);
    }

    #[test]
    fn test_query_include_only_characters() {
        let intervals = UnicodeVersion::V15_0_0
            .query(UnicodeCategory::Pc, None, 0, 50, "abc", None)
            .expect("Invalid query");
        assert_eq!(intervals, &[(97, 99)]);
    }

    #[test]
    fn test_query_include_category_and_characters() {
        let intervals = UnicodeVersion::V15_0_0
            .query(UnicodeCategory::Pc, None, None, None, "abc", None)
            .expect("Invalid query");
        assert_eq!(
            intervals,
            &[
                (95, 95),
                (97, 99),
                (8255, 8256),
                (8276, 8276),
                (65075, 65076),
                (65101, 65103),
                (65343, 65343)
            ]
        );
    }

    #[test_case(
        1073741824,
        2147483648,
        "Codepoints should be in [0; 1114111] range. Got: [1073741824; 2147483648]"
    )]
    #[test_case(
        0,
        2147483648,
        "Codepoints should be in [0; 1114111] range. Got: [0; 2147483648]"
    )]
    #[test_case(
        5,
        1,
        "Minimum codepoint should be less or equal than maximum codepoint. Got 5 < 1"
    )]
    fn test_query_invalid_codepoints(min_codepoint: u32, max_codepoint: u32, expected: &str) {
        let error = UnicodeVersion::V15_0_0
            .query(None, None, min_codepoint, max_codepoint, None, None)
            .expect_err("Should error");
        assert_eq!(error.to_string(), expected);
    }

    #[test]
    fn test_intervals_for() {
        assert_eq!(
            UnicodeVersion::V15_0_0.intervals_for(UnicodeCategory::Pc),
            &[
                (95, 95),
                (8255, 8256),
                (8276, 8276),
                (65075, 65076),
                (65101, 65103),
                (65343, 65343),
            ]
        );
    }

    #[test]
    fn test_normalized_categories() {
        assert_eq!(
            UnicodeVersion::V15_0_0.normalized_categories(),
            [
                UnicodeCategory::Zl,
                UnicodeCategory::Zp,
                UnicodeCategory::Co,
                UnicodeCategory::Me,
                UnicodeCategory::Pc,
                UnicodeCategory::Zs,
                UnicodeCategory::Pf,
                UnicodeCategory::Lt,
                UnicodeCategory::Pi,
                UnicodeCategory::Nl,
                UnicodeCategory::Pd,
                UnicodeCategory::Sc,
                UnicodeCategory::Cf,
                UnicodeCategory::Sk,
                UnicodeCategory::Nd,
                UnicodeCategory::Sm,
                UnicodeCategory::Lm,
                UnicodeCategory::No,
                UnicodeCategory::Pe,
                UnicodeCategory::Ps,
                UnicodeCategory::Mc,
                UnicodeCategory::So,
                UnicodeCategory::Po,
                UnicodeCategory::Mn,
                UnicodeCategory::Lo,
                UnicodeCategory::Lu,
                UnicodeCategory::Ll,
                UnicodeCategory::Cn,
                UnicodeCategory::Cc,
                UnicodeCategory::Cs,
            ]
        );
    }

    #[test_case(UnicodeVersion::V9_0_0)]
    #[test_case(UnicodeVersion::V10_0_0)]
    #[test_case(UnicodeVersion::V11_0_0)]
    #[test_case(UnicodeVersion::V12_0_0)]
    #[test_case(UnicodeVersion::V12_1_0)]
    #[test_case(UnicodeVersion::V13_0_0)]
    #[test_case(UnicodeVersion::V14_0_0)]
    #[test_case(UnicodeVersion::V15_0_0)]
    fn test_successive_union(version: UnicodeVersion) {
        let mut x = vec![];
        for v in version.table() {
            x.extend_from_slice(v);
        }
        intervals::merge(&mut x);
        assert_eq!(x, vec![(0, MAX_CODEPOINT)]);
    }

    #[test_case(UnicodeVersion::V9_0_0, "9.0.0")]
    #[test_case(UnicodeVersion::V10_0_0, "10.0.0")]
    #[test_case(UnicodeVersion::V11_0_0, "11.0.0")]
    #[test_case(UnicodeVersion::V12_0_0, "12.0.0")]
    #[test_case(UnicodeVersion::V12_1_0, "12.1.0")]
    #[test_case(UnicodeVersion::V13_0_0, "13.0.0")]
    #[test_case(UnicodeVersion::V14_0_0, "14.0.0")]
    #[test_case(UnicodeVersion::V15_0_0, "15.0.0")]
    fn test_display(version: UnicodeVersion, expected: &str) {
        let string = version.to_string();
        assert_eq!(string, expected);
        assert_eq!(
            UnicodeVersion::from_str(&string).expect("Invalid version"),
            version
        );
    }

    #[test_case("9.0.0", UnicodeVersion::V9_0_0)]
    #[test_case("10.0.0", UnicodeVersion::V10_0_0)]
    #[test_case("11.0.0", UnicodeVersion::V11_0_0)]
    #[test_case("12.0.0", UnicodeVersion::V12_0_0)]
    #[test_case("12.1.0", UnicodeVersion::V12_1_0)]
    #[test_case("13.0.0", UnicodeVersion::V13_0_0)]
    #[test_case("14.0.0", UnicodeVersion::V14_0_0)]
    #[test_case("15.0.0", UnicodeVersion::V15_0_0)]
    fn test_version_from_str(version: &str, expected: UnicodeVersion) {
        assert_eq!(
            UnicodeVersion::from_str(version).expect("Invalid version"),
            expected
        );
    }

    #[test]
    fn test_version_from_str_error() {
        assert_eq!(
            UnicodeVersion::from_str("invalid")
                .expect_err("Should fail")
                .to_string(),
            "'invalid' is not a valid Unicode version"
        );
    }

    fn hash(_: impl core::hash::Hash) {}

    #[test]
    fn test_is_hashable() {
        hash(UnicodeVersion::V15_0_0);
    }
}
