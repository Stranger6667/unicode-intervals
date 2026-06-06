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
    // Small ranges with ASCII-only custom characters fit in a 256-bit mask.
    if max_codepoint < ASCII_BOUND
        && is_ascii_range(include_characters)
        && is_ascii_range(exclude_characters)
    {
        return ascii_query(
            version,
            include_categories,
            exclude_categories,
            include_characters,
            exclude_characters,
            min_codepoint,
            max_codepoint,
        );
    }
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

/// Below this length, scanning the prefix is cheaper than a `log2(n)` search for the start.
const BINARY_SEARCH_THRESHOLD: usize = 32;

/// Append intervals overlapping `[min_codepoint, max_codepoint]`, clamped to it.
/// `table` is sorted and non-overlapping, so binary search past the leading intervals below
/// the range, then stop on the first one past it. Only search when it can pay off.
fn extend_clamped(
    out: &mut Vec<Interval>,
    table: &[Interval],
    min_codepoint: u32,
    max_codepoint: u32,
) {
    let start = match table.first() {
        Some(&(_, right)) if right < min_codepoint && table.len() >= BINARY_SEARCH_THRESHOLD => {
            table.partition_point(|&(_, right)| right < min_codepoint)
        }
        _ => 0,
    };
    for &(left, right) in &table[start..] {
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

/// Codepoints below this bound fit in a 256-bit mask (`[u64; 4]`).
const ASCII_BOUND: u32 = 256;

type AsciiMask = [u64; 4];

/// Per-category membership masks for codepoints `0..256`, built at compile time from `table`.
#[allow(clippy::arithmetic_side_effects)]
const fn build_ascii_masks(table: &[&[Interval]]) -> [AsciiMask; 30] {
    let mut masks = [[0u64; 4]; 30];
    let mut category = 0;
    while category < 30 {
        let slice = table[category];
        let mut i = 0;
        while i < slice.len() {
            let (left, right) = slice[i];
            if left >= ASCII_BOUND {
                break; // sorted by left bound: nothing else reaches into 0..256
            }
            let hi = if right >= ASCII_BOUND {
                ASCII_BOUND - 1
            } else {
                right
            };
            let mut cp = left;
            while cp <= hi {
                masks[category][(cp / 64) as usize] |= 1u64 << (cp % 64);
                cp += 1;
            }
            i += 1;
        }
        category += 1;
    }
    masks
}

/// ASCII masks for every bundled version, indexed by `UnicodeVersion as usize`.
const ASCII_MASKS: [[AsciiMask; 30]; 11] = [
    build_ascii_masks(crate::tables::v9_0_0::BY_NAME),
    build_ascii_masks(crate::tables::v10_0_0::BY_NAME),
    build_ascii_masks(crate::tables::v11_0_0::BY_NAME),
    build_ascii_masks(crate::tables::v12_0_0::BY_NAME),
    build_ascii_masks(crate::tables::v12_1_0::BY_NAME),
    build_ascii_masks(crate::tables::v13_0_0::BY_NAME),
    build_ascii_masks(crate::tables::v14_0_0::BY_NAME),
    build_ascii_masks(crate::tables::v15_0_0::BY_NAME),
    build_ascii_masks(crate::tables::v15_1_0::BY_NAME),
    build_ascii_masks(crate::tables::v16_0_0::BY_NAME),
    build_ascii_masks(crate::tables::v17_0_0::BY_NAME),
];

/// `true` when every character in `s` is below `ASCII_BOUND`.
fn is_ascii_range(s: &str) -> bool {
    s.chars().all(|c| (c as u32) < ASCII_BOUND)
}

/// Bitset fast path for `max_codepoint < ASCII_BOUND` with ASCII-only custom characters.
#[allow(clippy::arithmetic_side_effects)]
fn ascii_query(
    version: UnicodeVersion,
    include_categories: Option<UnicodeCategorySet>,
    exclude_categories: UnicodeCategorySet,
    include_characters: &str,
    exclude_characters: &str,
    min_codepoint: u32,
    max_codepoint: u32,
) -> Vec<Interval> {
    let categories = categories::merge(include_categories, exclude_categories);
    let masks = &ASCII_MASKS[version as usize];
    let mut bits = match categories.into_value() {
        0 => [0u64; 4],
        ALL_CATEGORIES => [u64::MAX; 4],
        _ => {
            let mut bits = [0u64; 4];
            for category in categories.iter() {
                let mask = masks[category as usize];
                bits[0] |= mask[0];
                bits[1] |= mask[1];
                bits[2] |= mask[2];
                bits[3] |= mask[3];
            }
            bits
        }
    };
    // Clamp category membership to the range; custom characters apply regardless of it.
    let range = range_mask(min_codepoint, max_codepoint);
    bits[0] &= range[0];
    bits[1] &= range[1];
    bits[2] &= range[2];
    bits[3] &= range[3];
    for character in include_characters.chars() {
        let cp = character as u32;
        bits[(cp / 64) as usize] |= 1u64 << (cp % 64);
    }
    for character in exclude_characters.chars() {
        let cp = character as u32;
        bits[(cp / 64) as usize] &= !(1u64 << (cp % 64));
    }
    extract_intervals(&bits)
}

/// 256-bit mask with bits `[min_codepoint, max_codepoint]` set (both `< ASCII_BOUND`).
#[allow(clippy::arithmetic_side_effects)]
fn range_mask(min_codepoint: u32, max_codepoint: u32) -> AsciiMask {
    let mut mask = [0u64; 4];
    let mut word: u32 = 0;
    while word < 4 {
        let base = word * 64;
        let top = base + 63;
        if max_codepoint >= base && min_codepoint <= top {
            let lo = min_codepoint.saturating_sub(base);
            let hi = if max_codepoint < top {
                max_codepoint - base
            } else {
                63
            };
            let width = hi - lo + 1;
            mask[word as usize] = if width == 64 {
                u64::MAX
            } else {
                ((1u64 << width) - 1) << lo
            };
        }
        word += 1;
    }
    mask
}

/// Extract sorted, coalesced intervals from a 256-bit membership mask.
#[allow(clippy::arithmetic_side_effects)]
fn extract_intervals(bits: &AsciiMask) -> Vec<Interval> {
    let mut result = Vec::new();
    let mut current: Option<Interval> = None;
    let mut word: u32 = 0;
    while word < 4 {
        let mut remaining = bits[word as usize];
        let base = word * 64;
        while remaining != 0 {
            let cp = base + remaining.trailing_zeros();
            match current {
                Some((_, ref mut hi)) if *hi + 1 == cp => *hi = cp,
                Some(interval) => {
                    result.push(interval);
                    current = Some((cp, cp));
                }
                None => current = Some((cp, cp)),
            }
            remaining &= remaining - 1; // clear lowest set bit
        }
        word += 1;
    }
    if let Some(interval) = current {
        result.push(interval);
    }
    result
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
    // high min_codepoint: skips leading intervals in `extend_clamped`
    #[test_case(Some(UnicodeCategory::Lo.into()), UnicodeCategorySet::new(), "", "", 0x1_0000, 0x1_2000; "single category high range")]
    #[test_case(Some(UnicodeCategory::Lu | UnicodeCategory::Ll), UnicodeCategorySet::new(), "", "", 0xFF00, 0x1_0400; "multi category high range")]
    // min_codepoint inside an interval (boundary for the skip predicate)
    #[test_case(Some(UnicodeCategory::Ll.into()), UnicodeCategorySet::new(), "", "", 98, 200; "min inside interval")]
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

    // The bitset fast path must agree with the per-codepoint oracle across every small-range
    // shape: category combinations, ranges within 0..256, and ASCII include/exclude chars.
    #[test]
    fn test_ascii_query_against_oracle() {
        let version = UnicodeVersion::V15_0_0;
        let includes = [
            None,
            Some(UnicodeCategorySet::new()),
            Some(UnicodeCategorySet::all()),
            Some(UnicodeCategory::Lu.into()),
            Some(UnicodeCategory::Lu | UnicodeCategory::Ll),
            Some(UnicodeCategory::Nd | UnicodeCategory::Po | UnicodeCategory::Sm),
        ];
        let excludes = [
            UnicodeCategorySet::new(),
            UnicodeCategory::Lu.into(),
            UnicodeCategory::Po.into(),
        ];
        let ranges = [
            (0, 127),
            (0, 255),
            (65, 90),
            (0, 0),
            (255, 255),
            (32, 200),
            (128, 255),
        ];
        let char_sets = ["", "ABC", "0az", "~\u{00ff}", "@"];
        for include in includes {
            for exclude in excludes {
                for (min_codepoint, max_codepoint) in ranges {
                    for include_characters in char_sets {
                        for exclude_characters in char_sets {
                            let got = ascii_query(
                                version,
                                include,
                                exclude,
                                include_characters,
                                exclude_characters,
                                min_codepoint,
                                max_codepoint,
                            );
                            let expected = oracle(
                                version,
                                include,
                                exclude,
                                include_characters,
                                exclude_characters,
                                min_codepoint,
                                max_codepoint,
                            );
                            assert_eq!(
                                got, expected,
                                "include={include:?} exclude={exclude:?} range=({min_codepoint},{max_codepoint}) inc={include_characters:?} exc={exclude_characters:?}"
                            );
                        }
                    }
                }
            }
        }
    }

    // `build_ascii_masks` runs at compile time for `ASCII_MASKS`, so exercise it at runtime
    // here against an independent per-codepoint reference. Also checks the version indexing.
    #[test]
    fn test_build_ascii_masks() {
        for version in UnicodeVersion::ALL {
            let table = version.table();
            let masks = build_ascii_masks(table);
            for (category, slice) in table.iter().enumerate() {
                for cp in 0..ASCII_BOUND {
                    let expected = slice.iter().any(|&(left, right)| left <= cp && cp <= right);
                    let actual = masks[category][(cp / 64) as usize] & (1u64 << (cp % 64)) != 0;
                    assert_eq!(
                        actual, expected,
                        "version={version} category={category} cp={cp}"
                    );
                }
            }
            assert_eq!(masks, ASCII_MASKS[version as usize], "version={version}");
        }
    }

    // No real category interval straddles 256, so cover the cap branch with a synthetic table:
    // an interval crossing the bound must contribute only its `0..256` part.
    #[test]
    fn test_build_ascii_masks_caps_at_bound() {
        let crossing: &[Interval] = &[(250, 300)];
        let mut table: [&[Interval]; 30] = [&[]; 30];
        table[0] = crossing;
        let masks = build_ascii_masks(&table);
        for cp in 0..ASCII_BOUND {
            let set = masks[0][(cp / 64) as usize] & (1u64 << (cp % 64)) != 0;
            assert_eq!(set, cp >= 250, "cp={cp}");
        }
    }
}
