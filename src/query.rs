use crate::{
    categories,
    categories::UnicodeCategorySet,
    constants::{ALL_CATEGORIES, MAX_CODEPOINT},
    intervals, Interval, UnicodeVersion,
};
use core::cmp::{max, min};
use std::borrow::Cow;

/// Non-generic query implementation to reduce the amount of generated code.
#[must_use]
pub fn query(
    version: UnicodeVersion,
    include_categories: Option<UnicodeCategorySet>,
    exclude_categories: UnicodeCategorySet,
    include_characters: &str,
    exclude_characters: &str,
    min_codepoint: u32,
    max_codepoint: u32,
) -> Vec<Interval> {
    let categories = categories::merge(include_categories, exclude_categories);

    let include_intervals = intervals::from_str(include_characters);
    let exclude_intervals = intervals::from_str(exclude_characters);

    // Concatenated category slices need a re-merge; a single category (or the whole
    // range) is already sorted and non-overlapping.
    let needs_merge = categories.len() > 1 && categories.into_value() != ALL_CATEGORIES;
    // Collect category intervals already clamped to the codepoint range. For a restricted
    // range this avoids materializing and scanning the entire set.
    let mut intervals = if min_codepoint == 0 && max_codepoint == MAX_CODEPOINT {
        intervals_for_set(version, categories).into_owned()
    } else {
        collect_in_range(version, categories, min_codepoint, max_codepoint)
    };
    // Include intervals
    if intervals.is_empty() {
        intervals = include_intervals;
    } else if !include_intervals.is_empty() {
        intervals.extend_from_slice(&include_intervals);
        intervals::merge(&mut intervals);
    } else if needs_merge {
        intervals::merge(&mut intervals);
    }
    // Exclude intervals
    intervals::subtract(intervals, exclude_intervals.as_slice())
}

/// Get intervals for the given `CategorySet`.
/// The final intervals are merged and sorted.
#[inline]
#[must_use]
pub fn intervals_for_set(
    version: UnicodeVersion,
    categories: UnicodeCategorySet,
) -> Cow<'static, [Interval]> {
    match categories.into_value() {
        0 => Cow::Borrowed(&[]),
        ALL_CATEGORIES => Cow::Borrowed(&[(0, MAX_CODEPOINT)]),
        value => {
            if categories.len() == 1 {
                let category_idx = value.trailing_zeros() as usize;
                Cow::Borrowed(version.table()[category_idx])
            } else {
                // Pre-allocate space for intervals from all categories
                let size: usize = categories
                    .iter()
                    .map(|c| version.table()[c as usize].len())
                    .sum();
                let mut intervals = Vec::with_capacity(size);
                for category in categories.iter() {
                    intervals.extend_from_slice(version.table()[category as usize]);
                }
                Cow::Owned(intervals)
            }
        }
    }
}

/// Collect category intervals clamped to `[min_codepoint, max_codepoint]`.
fn collect_in_range(
    version: UnicodeVersion,
    categories: UnicodeCategorySet,
    min_codepoint: u32,
    max_codepoint: u32,
) -> Vec<Interval> {
    match categories.into_value() {
        0 => Vec::new(),
        ALL_CATEGORIES => vec![(min_codepoint, max_codepoint)],
        value => {
            let table = version.table();
            let mut intervals = Vec::new();
            if categories.len() == 1 {
                let category_idx = value.trailing_zeros() as usize;
                extend_clamped(
                    &mut intervals,
                    table[category_idx],
                    min_codepoint,
                    max_codepoint,
                );
            } else {
                for category in categories.iter() {
                    extend_clamped(
                        &mut intervals,
                        table[category as usize],
                        min_codepoint,
                        max_codepoint,
                    );
                }
            }
            intervals
        }
    }
}

/// Append intervals overlapping `[min_codepoint, max_codepoint]`, clamped to it.
/// Relies on `table` being sorted by the left bound to stop early.
fn extend_clamped(
    out: &mut Vec<Interval>,
    table: &[Interval],
    min_codepoint: u32,
    max_codepoint: u32,
) {
    for &(left, right) in table {
        if left > max_codepoint {
            break;
        }
        let lo = max(left, min_codepoint);
        let hi = min(right, max_codepoint);
        if lo <= hi {
            out.push((lo, hi));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::UnicodeCategory;
    use test_case::test_case;

    #[test_case(UnicodeCategorySet::new(), &[])]
    #[test_case(UnicodeCategorySet::all(), &[(0, MAX_CODEPOINT)])]
    #[test_case(UnicodeCategory::Zl.into(), &[(8232, 8232)])]
    #[test_case(UnicodeCategory::Zl | UnicodeCategory::Cs, &[(8232, 8232), (55296, 57343)])]
    fn test_intervals_for_set(categories: UnicodeCategorySet, expected: &[Interval]) {
        let intervals = intervals_for_set(UnicodeVersion::V15_0_0, categories);
        assert_eq!(intervals, expected);
    }

    // Independent reference: decide membership per codepoint from the raw category
    // tables, then coalesce into intervals. Knows nothing about `query`'s algorithm.
    fn oracle(
        version: UnicodeVersion,
        include_categories: Option<UnicodeCategorySet>,
        exclude_categories: UnicodeCategorySet,
        include_characters: &str,
        exclude_characters: &str,
        min_codepoint: u32,
        max_codepoint: u32,
    ) -> Vec<Interval> {
        let mask = match include_categories {
            Some(inc) if inc.into_value() == 0 => 0,
            Some(inc) => (ALL_CATEGORIES ^ exclude_categories.into_value()) & inc.into_value(),
            None => ALL_CATEGORIES ^ exclude_categories.into_value(),
        };
        let is_all = mask == ALL_CATEGORIES;
        let table = version.table();
        let inc: std::collections::HashSet<u32> =
            include_characters.chars().map(|c| c as u32).collect();
        let exc: std::collections::HashSet<u32> =
            exclude_characters.chars().map(|c| c as u32).collect();
        let bound = inc
            .iter()
            .chain(exc.iter())
            .copied()
            .fold(max_codepoint, u32::max);
        let mut covered = vec![false; (bound as usize).saturating_add(1)];
        if is_all {
            for cp in min_codepoint..=max_codepoint.min(bound) {
                covered[cp as usize] = true;
            }
        } else {
            for (i, slice) in table.iter().enumerate() {
                if (mask >> i) & 1 == 0 {
                    continue;
                }
                for &(l, r) in *slice {
                    let lo = l.max(min_codepoint);
                    let hi = r.min(max_codepoint).min(bound);
                    if lo <= hi {
                        for cp in lo..=hi {
                            covered[cp as usize] = true;
                        }
                    }
                }
            }
        }
        let mut out: Vec<Interval> = Vec::new();
        for cp in 0..=bound {
            let member = (covered[cp as usize] || inc.contains(&cp)) && !exc.contains(&cp);
            if member {
                match out.last_mut() {
                    Some(last) if last.1.saturating_add(1) == cp => last.1 = cp,
                    _ => out.push((cp, cp)),
                }
            }
        }
        out
    }

    #[allow(clippy::too_many_arguments)]
    fn check(
        include_categories: Option<UnicodeCategorySet>,
        exclude_categories: UnicodeCategorySet,
        include_characters: &str,
        exclude_characters: &str,
        min_codepoint: u32,
        max_codepoint: u32,
    ) {
        let version = UnicodeVersion::V15_0_0;
        let actual = query(
            version,
            include_categories,
            exclude_categories,
            include_characters,
            exclude_characters,
            min_codepoint,
            max_codepoint,
        );
        let expected = oracle(
            version,
            include_categories,
            exclude_categories,
            include_characters,
            exclude_characters,
            min_codepoint,
            max_codepoint,
        );
        assert_eq!(actual, expected);
    }

    // multi-category exclude + bounded range (mirrors the hot benchmarks)
    #[test_case(None, UnicodeCategory::Lu.into(), "", "", 0, 128; "exclude one, both bounds")]
    #[test_case(None, UnicodeCategory::Lu.into(), "", "A@\u{0442}", 0, 128; "exclude chars")]
    #[test_case(None, UnicodeCategory::Lu.into(), "0123456789", "QWERTYUIOP", 0, 128; "include and exclude chars")]
    // empty category set in a ranged query (`collect_in_range` zero branch)
    #[test_case(Some(UnicodeCategorySet::new()), UnicodeCategorySet::new(), "", "", 0, 128; "empty set ranged")]
    #[test_case(Some(UnicodeCategorySet::new()), UnicodeCategorySet::new(), "abc", "", 0, 128; "empty set ranged with chars")]
    // include-driven paths
    #[test_case(Some(UnicodeCategory::Ll.into()), UnicodeCategorySet::new(), "ABC", "", 0, 50; "include one + chars")]
    #[test_case(Some(UnicodeCategory::Lu | UnicodeCategory::Ll), UnicodeCategorySet::new(), "\u{2603}", "", 0, 128; "include char beyond max")]
    // single category, larger bounded range (no early-exit today)
    #[test_case(Some(UnicodeCategory::Lo.into()), UnicodeCategorySet::new(), "", "", 0, 0x1_0000; "single category right bound")]
    // both-bounds slice in the middle (cross-check with test_intervals oracle case)
    #[test_case(None, UnicodeCategorySet::new(), "", "", 65076, 65102; "both bounds mid range")]
    // all categories, clamped
    #[test_case(Some(UnicodeCategorySet::all()), UnicodeCategorySet::new(), "", "", 0, 200; "all categories clamped")]
    // left-bound-only arm (max == MAX_CODEPOINT), small set keeps the scan cheap
    #[test_case(Some(UnicodeCategory::Zl.into()), UnicodeCategorySet::new(), "", "", 8000, MAX_CODEPOINT; "left bound only")]
    // full range arm
    #[test_case(Some(UnicodeCategory::Zl.into()), UnicodeCategorySet::new(), "", "", 0, MAX_CODEPOINT; "full range")]
    fn test_query_against_oracle(
        include_categories: Option<UnicodeCategorySet>,
        exclude_categories: UnicodeCategorySet,
        include_characters: &str,
        exclude_characters: &str,
        min_codepoint: u32,
        max_codepoint: u32,
    ) {
        check(
            include_categories,
            exclude_categories,
            include_characters,
            exclude_characters,
            min_codepoint,
            max_codepoint,
        );
    }
}
