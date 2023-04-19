use crate::{
    categories,
    categories::UnicodeCategorySet,
    constants::{ALL_CATEGORIES, MAX_CODEPOINT},
    intervals, Interval, UnicodeVersion,
};
use core::cmp::{max, min};

/// Non-generic query implementation to reduce the amount of generated code.
#[must_use]
pub fn query<'a>(
    version: UnicodeVersion,
    include_categories: Option<UnicodeCategorySet>,
    exclude_categories: UnicodeCategorySet,
    min_codepoint: u32,
    max_codepoint: u32,
    include_characters: Option<&'a str>,
    exclude_characters: Option<&'a str>,
) -> Vec<Interval> {
    let categories = categories::merge(include_categories, exclude_categories);
    let include_characters = include_characters.unwrap_or("");
    let exclude_characters = exclude_characters.unwrap_or("");

    let include_intervals = intervals::from_str(include_characters);
    let exclude_intervals = intervals::from_str(exclude_characters);

    let full = intervals_for_set(version, categories);
    // Depending on the codepoint range, it could be less work to do
    let mut intervals = match (min_codepoint, max_codepoint) {
        // Full range, no need to filter
        (0, MAX_CODEPOINT) => full,
        // Only check for the left bound
        (0, _) => {
            let mut intervals = vec![];
            for (left, right) in full {
                if left > max_codepoint {
                    // Intervals are sorted - all subsequent ones are greater than `max_codepoint`
                    break;
                }
                intervals.push((max(left, min_codepoint), min(right, max_codepoint)));
            }
            intervals
        }
        // Only check for the right bound
        (_, MAX_CODEPOINT) => {
            let mut intervals = vec![];
            for (left, right) in full {
                if right >= min_codepoint {
                    intervals.push((max(left, min_codepoint), min(right, max_codepoint)));
                }
            }
            intervals
        }
        // Check for both bounds
        _ => {
            let mut intervals = vec![];
            for (left, right) in full {
                if left > max_codepoint {
                    // Intervals are sorted - all subsequent ones are greater than `max_codepoint`
                    break;
                } else if right >= min_codepoint {
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
        intervals.extend(include_intervals);
        intervals::merge(&mut intervals);
    }
    // Exclude intervals
    intervals::subtract(intervals, exclude_intervals.as_slice())
}

/// Get intervals for the given `CategorySet`.
/// The final intervals are merged and sorted.
#[inline]
#[must_use]
pub fn intervals_for_set(version: UnicodeVersion, categories: UnicodeCategorySet) -> Vec<Interval> {
    match categories.into_value() {
        0 => vec![],
        ALL_CATEGORIES => vec![(0, MAX_CODEPOINT)],
        value => {
            if categories.len() == 1 {
                // If there is only one category - just transform the corresponding slice to a vector
                // the intervals there are sorted and do not intersect
                let category_idx = value.trailing_zeros() as usize;
                version.table()[category_idx].to_vec()
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
                intervals::merge(&mut intervals);
                intervals
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
