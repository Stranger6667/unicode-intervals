use crate::Interval;

/// Create a set of intervals for the given string.
#[inline]
#[must_use]
pub fn from_str(string: &str) -> Vec<Interval> {
    if string.is_empty() {
        return vec![];
    }
    let mut intervals: Vec<_> = string.chars().map(|c| (c as u32, c as u32)).collect();
    merge(&mut intervals);
    intervals
}

/// Subtract `right` set of intervals from `left`.
#[inline]
// Practically all interval values are < u32::MAX
// Therefore there will be no panic (debug) / wrapping (release)
#[allow(clippy::integer_arithmetic)]
#[must_use]
pub fn subtract(mut left: Vec<Interval>, right: &[Interval]) -> Vec<Interval> {
    if right.is_empty() || left.is_empty() {
        left
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
#[allow(clippy::integer_arithmetic)]
pub fn merge(intervals: &mut Vec<Interval>) {
    #[allow(clippy::stable_sort_primitive)]
    intervals.sort_by_key(|a| a.0);
    let mut border = 0_usize;
    for index in 1..intervals.len() {
        let interval = intervals[index];
        if interval.0 <= intervals[border].1 + 1 {
            // Intervals overlap
            if interval.1 > intervals[border].1 {
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

    #[test_case("", &[])]
    #[test_case("\u{10A07}", &[(68103, 68103)])]
    #[test_case("a", &[(97, 97)])]
    #[test_case("aa", &[(97, 97)])]
    #[test_case("abcdef0123456789", &[(48, 57), (97, 102)])]
    #[test_case("01234fedcba98765", &[(48, 57), (97, 102)])]
    fn test_from_str(value: &str, expected: &[Interval]) {
        assert_eq!(from_str(value), expected);
    }
}
