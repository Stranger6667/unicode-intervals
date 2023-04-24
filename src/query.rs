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

    let full = intervals_for_set(version, categories);
    // Depending on the codepoint range, it could be less work to do
    let mut intervals = match (min_codepoint, max_codepoint) {
        // Full range, no need to filter
        (0, MAX_CODEPOINT) => full.to_vec(),
        // Only check for the left bound
        (0, _) => {
            let mut intervals = vec![];
            for (left, right) in full.iter().copied() {
                if left <= max_codepoint {
                    intervals.push((max(left, min_codepoint), min(right, max_codepoint)));
                }
            }
            intervals
        }
        // Only check for the right bound
        (_, MAX_CODEPOINT) => {
            let mut intervals = vec![];
            for (left, right) in full.iter().copied() {
                if right >= min_codepoint {
                    intervals.push((max(left, min_codepoint), min(right, max_codepoint)));
                }
            }
            intervals
        }
        // Check for both bounds
        _ => {
            let mut intervals = vec![];
            for (left, right) in full.iter().copied() {
                if left <= max_codepoint && right >= min_codepoint {
                    intervals.push((max(left, min_codepoint), min(right, max_codepoint)));
                }
            }
            intervals
        }
    };
    // Include intervals
    if intervals.is_empty() {
        intervals = include_intervals;
    } else if !include_intervals.is_empty() {
        intervals.extend_from_slice(&include_intervals);
        intervals::merge(&mut intervals);
    } else {
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
}
