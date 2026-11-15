use crate::{
    query::{extract_intervals_with_capacity, AsciiMask},
    Interval,
};

/// Create a set of intervals for the given string.
#[inline]
#[must_use]
#[allow(clippy::arithmetic_side_effects)]
pub fn from_str(string: &str) -> Vec<Interval> {
    if string.is_empty() {
        return vec![];
    }
    // UTF-8 lead bytes 0xC2 and 0xC3 encode U+0080..=U+00FF, so no byte above them means
    // every character fits the 256-bit mask.
    if string.bytes().fold(0, u8::max) < 0xC4 {
        let mut bits: AsciiMask = [0; 4];
        for character in string.chars() {
            let codepoint = character as u32;
            bits[(codepoint / 64) as usize] |= 1u64 << (codepoint % 64);
        }
        return extract_intervals_with_capacity(&bits, string.len().min(128));
    }
    let mut intervals: Vec<_> = string.chars().map(|c| (c as u32, c as u32)).collect();
    merge(&mut intervals);
    intervals
}

/// Union of two sorted sets of non-overlapping, non-adjacent intervals.
/// Bulk-copies the runs of `left` between the intervals of `right`, which is usually tiny.
#[must_use]
// Practically all interval values are < u32::MAX
#[allow(clippy::arithmetic_side_effects)]
pub fn union(left: &[Interval], right: &[Interval]) -> Vec<Interval> {
    // Only the leading intervals of `slice` can touch what is already in `result`.
    fn extend(result: &mut Vec<Interval>, slice: &[Interval]) {
        let mut absorbed = 0;
        if let Some(last) = result.last_mut() {
            while absorbed < slice.len() && slice[absorbed].0 <= last.1 + 1 {
                last.1 = last.1.max(slice[absorbed].1);
                absorbed += 1;
            }
        }
        result.extend_from_slice(&slice[absorbed..]);
    }
    let mut result = Vec::with_capacity(left.len() + right.len());
    let mut rest = left;
    for interval in right {
        // Intervals of `left` starting at or before this one go first.
        let (before, after) =
            rest.split_at(rest.partition_point(|&(start, _)| start <= interval.0));
        extend(&mut result, before);
        extend(&mut result, core::slice::from_ref(interval));
        rest = after;
    }
    extend(&mut result, rest);
    result
}

/// `subtract` copies `left` in bulk when it is at least this many times longer than `right`.
const SPARSE_RATIO: usize = 8;

/// Number of leading intervals ending before `bound`.
/// Doubling steps, then a binary search: cheap for short runs, logarithmic for long ones.
#[allow(clippy::arithmetic_side_effects)]
fn run_before(intervals: &[Interval], bound: u32) -> usize {
    let mut step = 1;
    while step <= intervals.len() && intervals[step - 1].1 < bound {
        step *= 2;
    }
    // The first `step / 2` intervals end before `bound`, the one at `step - 1` does not
    // (or is past the end).
    let low = step / 2;
    let high = (step - 1).min(intervals.len());
    low + intervals[low..high].partition_point(|&(_, end)| end < bound)
}

/// `subtract` for a `right` much shorter than `left`: bulk-copy the runs of `left` between
/// the intervals of `right`.
#[allow(clippy::arithmetic_side_effects)]
fn subtract_sparse(mut left: Vec<Interval>, right: &[Interval]) -> Vec<Interval> {
    let mut result = Vec::with_capacity(left.len() + right.len());
    let mut i = 0;
    for &(rl, rr) in right {
        let run = run_before(&left[i..], rl);
        result.extend_from_slice(&left[i..i + run]);
        i += run;
        // Intervals overlapping `(rl, rr)`: keep what sticks out on either side.
        while i < left.len() && left[i].0 <= rr {
            let (ll, lr) = left[i];
            if ll < rl {
                result.push((ll, rl - 1));
            }
            if lr > rr {
                left[i].0 = rr + 1;
                break;
            }
            i += 1;
        }
    }
    result.extend_from_slice(&left[i..]);
    result
}

/// Subtract `right` set of intervals from `left`.
#[inline]
// Practically all interval values are < u32::MAX
// Therefore there will be no panic (debug) / wrapping (release)
#[allow(clippy::arithmetic_side_effects)]
#[must_use]
pub fn subtract(mut left: Vec<Interval>, right: &[Interval]) -> Vec<Interval> {
    if right.is_empty() || left.is_empty() {
        left
    } else if right.len().saturating_mul(SPARSE_RATIO) < left.len() {
        subtract_sparse(left, right)
    } else {
        let (mut i, mut j) = (0, 0);
        let mut result = Vec::with_capacity(left.len());
        while i < left.len() && j < right.len() {
            let (ll, lr) = left[i];
            let (rl, rr) = right[j];
            if rr < ll {
                j += 1;
            } else if rl > lr {
                result.push((ll, lr));
                i += 1;
            } else if rl <= ll {
                if rr >= lr {
                    i += 1;
                } else {
                    left[i].0 = rr + 1;
                    j += 1;
                }
            } else {
                result.push((ll, rl - 1));
                if rr < lr {
                    left[i].0 = rr + 1;
                    j += 1;
                } else {
                    i += 1;
                }
            }
        }
        result.extend_from_slice(&left[i..]);
        result
    }
}

/// Merge intersecting intervals in-place.
// Note, `#[inline]` leads to worse performance
// Practically all interval values are < u32::MAX
// Therefore there will be no panic (debug) / wrapping (release)
#[allow(clippy::arithmetic_side_effects)]
pub fn merge(intervals: &mut Vec<Interval>) {
    #[allow(clippy::stable_sort_primitive)]
    intervals.sort_by_key(|a| a.0);
    let mut border = 0_usize;
    for index in 1..intervals.len() {
        let interval = intervals[index];
        let right = intervals[border].1;
        if interval.0 <= right + 1 {
            // Intervals overlap
            if interval.1 > right {
                // Extend the one behind the border only if the current candidate right border
                // is greater
                intervals[border].1 = interval.1;
            }
        } else {
            // No overlap, this interval should be next behind the border
            border += 1;
            intervals[border] = interval;
        }
    }
    intervals.truncate(border + 1);
}

#[cfg(test)]
mod tests {
    use super::*;

    use test_case::test_case;

    #[test_case(vec![], &[], &[]; "empty both")]
    #[test_case(vec![], &[(1, 2)], &[(1, 2)]; "empty left")]
    #[test_case(vec![(1, 2)], &[], &[(1, 2)]; "empty right")]
    #[test_case(vec![(2, 3)], &[(1, 2), (4, 5)], &[(1, 5)]; "totally overlapped gap")]
    #[test_case(vec![(3, 3)], &[(1, 2), (5, 5)], &[(1, 3), (5, 5)]; "partially overlapped gap")]
    fn union_intervals_empty(mut left: Vec<Interval>, right: &[Interval], expected: &[Interval]) {
        left.extend_from_slice(right);
        merge(&mut left);
        assert_eq!(left, expected);
    }

    #[test_case(&[], &[], &[]; "empty both")]
    #[test_case(&[], &[(1, 2)], &[(1, 2)]; "empty left")]
    #[test_case(&[(1, 2)], &[], &[(1, 2)]; "empty right")]
    #[test_case(&[(2, 3)], &[(1, 2), (4, 5)], &[(1, 5)]; "bridge both sides")]
    #[test_case(&[(3, 3)], &[(1, 2), (5, 5)], &[(1, 3), (5, 5)]; "adjacent on one side")]
    #[test_case(&[(0, 5), (10, 15), (20, 25)], &[(7, 8)], &[(0, 5), (7, 8), (10, 15), (20, 25)]; "in a gap")]
    #[test_case(&[(0, 5), (10, 15), (20, 25), (30, 35)], &[(4, 22)], &[(0, 25), (30, 35)]; "swallow several")]
    #[test_case(&[(0, 5), (10, 15)], &[(12, 13)], &[(0, 5), (10, 15)]; "contained")]
    #[test_case(&[(10, 15)], &[(0, 1), (3, 4), (20, 21), (30, 30)], &[(0, 1), (3, 4), (10, 15), (20, 21), (30, 30)]; "around")]
    #[test_case(&[(0, 5), (10, 15)], &[(6, 9)], &[(0, 15)]; "fill the gap exactly")]
    fn test_union(left: &[Interval], right: &[Interval], expected: &[Interval]) {
        assert_eq!(union(left, right), expected);
        assert_eq!(union(right, left), expected);
    }

    // Reference: concatenate, then sort + merge.
    #[test]
    fn test_union_against_merge() {
        fn next(state: &mut u64, bound: u32) -> u32 {
            *state = state
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            (*state >> 33) as u32 % bound
        }
        // Sorted, non-overlapping, non-adjacent intervals, as `union` expects.
        fn random(state: &mut u64, max_count: u32) -> Vec<Interval> {
            let mut intervals: Vec<Interval> = (0..next(state, max_count))
                .map(|_| {
                    let left = next(state, 400);
                    (left, left + next(state, 6))
                })
                .collect();
            merge(&mut intervals);
            intervals
        }
        let mut state = 0x1234_5678_9abc_def0u64;
        for _ in 0..20_000 {
            let left = random(&mut state, 40);
            let right = random(&mut state, 8);
            let mut expected = left.clone();
            expected.extend_from_slice(&right);
            merge(&mut expected);
            assert_eq!(union(&left, &right), expected, "{left:?} | {right:?}");
        }
    }

    #[test_case(vec![(0, 1)], &[], &[(0, 1)])]
    #[test_case(vec![(0, 1), (3, 3)], &[(0, 3)], &[])]
    #[test_case(vec![(0, 1), (3, 3)], &[(1, 3)], &[(0, 0)])]
    #[test_case(vec![(0, 10)], &[(2, 3), (9, 15)], &[(0, 1), (4, 8)])]
    #[test_case(vec![(0, 10)], &[(11, 15)], &[(0, 10)])]
    #[test_case(vec![(0, 10)], &[(8, 9)], &[(0, 7), (10, 10)])]
    #[test_case(vec![(5, 10)], &[(4, 7)], &[(8, 10)])]
    #[test_case(vec![(5, 10)], &[(1, 3)], &[(5, 10)])]
    fn test_subtract(left: Vec<Interval>, right: &[Interval], expected: &[Interval]) {
        assert_eq!(subtract(left, right), expected);
    }

    // Independent reference: flip per-codepoint membership, then coalesce.
    fn subtract_oracle(left: &[Interval], right: &[Interval]) -> Vec<Interval> {
        let bound = left
            .iter()
            .chain(right.iter())
            .map(|&(_, r)| r)
            .max()
            .unwrap_or(0);
        let mut covered = vec![false; (bound as usize).saturating_add(1)];
        for &(l, r) in left {
            for cp in l..=r {
                covered[cp as usize] = true;
            }
        }
        for &(l, r) in right {
            for cp in l..=r {
                covered[cp as usize] = false;
            }
        }
        let mut out: Vec<Interval> = Vec::new();
        for cp in 0..=bound {
            if covered[cp as usize] {
                match out.last_mut() {
                    Some(last) if last.1.saturating_add(1) == cp => last.1 = cp,
                    _ => out.push((cp, cp)),
                }
            }
        }
        out
    }

    // Sparse and dense `right` against long `left`, so runs of every length are skipped.
    #[test]
    fn test_subtract_random_against_oracle() {
        fn next(state: &mut u64, bound: u32) -> u32 {
            *state = state
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            (*state >> 33) as u32 % bound
        }
        fn random(state: &mut u64, max_count: u32) -> Vec<Interval> {
            let mut intervals: Vec<Interval> = (0..next(state, max_count))
                .map(|_| {
                    let left = next(state, 600);
                    (left, left + next(state, 4))
                })
                .collect();
            merge(&mut intervals);
            intervals
        }
        let mut state = 0x1234_5678_9abc_def0u64;
        for round in 0..5_000 {
            let left = random(&mut state, 120);
            let right = random(&mut state, if round % 2 == 0 { 6 } else { 120 });
            let expected = subtract_oracle(&left, &right);
            assert_eq!(
                subtract(left.clone(), &right),
                expected,
                "{left:?} - {right:?}"
            );
        }
    }

    // Multi-interval `left` so bulk runs between/around `right` are exercised.
    #[test_case(vec![(0, 5), (10, 15), (20, 25), (30, 35)], &[(12, 13)]; "split with runs around")]
    #[test_case(vec![(0, 5), (10, 15), (20, 25), (30, 35)], &[(0, 0), (33, 33)]; "trim first and last")]
    #[test_case(vec![(0, 5), (10, 15), (20, 25)], &[(50, 60)]; "right after all left")]
    #[test_case(vec![(0, 5), (10, 15), (20, 25)], &[(0, 100)]; "right covers everything")]
    #[test_case(vec![(0, 5), (10, 15), (20, 25)], &[(10, 15)]; "drop whole middle interval")]
    #[test_case(vec![(0, 5), (10, 15), (20, 25), (30, 35), (40, 45)], &[(22, 23), (42, 42)]; "splits with bulk runs between")]
    #[test_case(vec![(0, 5), (10, 15), (20, 25)], &[(7, 8)]; "right in gap between intervals")]
    fn test_subtract_against_oracle(left: Vec<Interval>, right: &[Interval]) {
        let expected = subtract_oracle(&left, right);
        assert_eq!(subtract(left, right), expected);
    }

    #[test_case("", &[])]
    #[test_case("\u{10A07}", &[(68103, 68103)])]
    #[test_case("a", &[(97, 97)])]
    #[test_case("aa", &[(97, 97)])]
    #[test_case("abcdef0123456789", &[(48, 57), (97, 102)])]
    #[test_case("01234fedcba98765", &[(48, 57), (97, 102)])]
    #[test_case("\u{ff}\u{e9}\u{e8}a\u{80}", &[(97, 97), (128, 128), (232, 233), (255, 255)]; "latin1")]
    #[test_case("\u{0}\u{3f}\u{40}\u{7f}\u{80}\u{bf}\u{c0}", &[(0, 0), (63, 64), (127, 128), (191, 192)]; "mask word edges")]
    #[test_case("\u{100}\u{ff}a", &[(97, 97), (255, 256)]; "first char past the mask")]
    #[test_case("a\u{442}@", &[(64, 64), (97, 97), (1090, 1090)]; "mixed")]
    #[test_case("\u{442}\u{442}\u{443}", &[(1090, 1091)]; "duplicates past the mask")]
    fn test_from_str(value: &str, expected: &[Interval]) {
        assert_eq!(from_str(value), expected);
    }
}
