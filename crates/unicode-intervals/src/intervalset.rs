use crate::Interval;

/// A collection of non-overlapping Unicode codepoint intervals that enables interval-based
/// operations, such as iteration over all Unicode codepoints or finding the codepoint at a
/// specific position within the intervals.
#[derive(Debug, Clone)]
pub struct IntervalSet {
    intervals: Vec<Interval>,
    offsets: Vec<u32>,
    size: u32,
}

impl IntervalSet {
    #[must_use]
    pub(crate) fn new(intervals: Vec<Interval>) -> IntervalSet {
        let mut offsets = vec![0];
        offsets.reserve_exact(intervals.len());
        let mut size = 0;
        // INVARIANT: `right` is always `>= left`, hence no overflow
        #[allow(clippy::arithmetic_side_effects)]
        for (left, right) in &intervals {
            size += *right - *left + 1;
            offsets.push(size);
        }
        IntervalSet {
            intervals,
            offsets,
            size,
        }
    }

    /// Returns the number of Unicode codepoints in the interval set.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use unicode_intervals::UnicodeCategory;
    /// let interval_set = unicode_intervals::query()
    ///     .include_categories(UnicodeCategory::UPPERCASE_LETTER)
    ///     .interval_set()
    ///     .expect("Invalid query input");
    /// assert_eq!(interval_set.len(), 1886);
    /// ```
    #[inline]
    #[must_use]
    pub fn len(&self) -> usize {
        self.size as usize
    }

    /// Returns `true` if the interval set contains no elements.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use unicode_intervals::UnicodeCategory;
    /// let interval_set = unicode_intervals::query()
    ///     .include_categories(UnicodeCategory::UPPERCASE_LETTER)
    ///     // The first upper case letter has 65
    ///     .max_codepoint(50)
    ///     .interval_set()
    ///     .expect("Invalid query input");
    /// assert!(interval_set.is_empty());
    /// ```
    #[inline]
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.size == 0
    }

    /// Returns `true` if the interval set contains a codepoint with the given value.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use unicode_intervals::UnicodeCategory;
    /// let interval_set = unicode_intervals::query()
    ///     .include_categories(UnicodeCategory::UPPERCASE_LETTER)
    ///     .interval_set()
    ///     .expect("Invalid query input");
    /// assert!(interval_set.contains('C'));
    /// assert!(!interval_set.contains('a'));
    /// ```
    #[inline]
    #[must_use]
    pub fn contains(&self, codepoint: impl Into<u32>) -> bool {
        self.index_of(codepoint.into()).is_some()
    }

    /// Returns the codepoint at `index` in the `IntervalSet`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use unicode_intervals::UnicodeCategory;
    /// let interval_set = unicode_intervals::query()
    ///     .include_categories(UnicodeCategory::UPPERCASE_LETTER)
    ///     .interval_set()
    ///     .expect("Invalid query input");
    /// // Get 10th codepoint in this interval set
    ///assert_eq!(interval_set.codepoint_at(10), Some('K' as u32));
    /// ```
    #[inline]
    #[must_use]
    pub fn codepoint_at(&self, index: u32) -> Option<u32> {
        if index >= self.size {
            return None;
        }
        // Last interval whose start offset is `<= index`. `offsets[0]` is always 0 and `index`
        // is in range, so the partition point is at least 1 and the subtraction can't underflow.
        #[allow(clippy::arithmetic_side_effects)]
        let current = self.offsets.partition_point(|&offset| offset <= index) - 1;
        // INVARIANT: `index >= offsets[current]` and values are small enough to not overflow.
        #[allow(clippy::arithmetic_side_effects)]
        Some(self.intervals[current].0 + index - self.offsets[current])
    }

    /// Returns the index of a specific codepoint in the `IntervalSet`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use unicode_intervals::UnicodeCategory;
    /// let interval_set = unicode_intervals::query()
    ///     .include_categories(UnicodeCategory::UPPERCASE_LETTER)
    ///     .interval_set()
    ///     .expect("Invalid query input");
    /// assert_eq!(interval_set.index_of('A'), Some(0));
    /// assert_eq!(interval_set.index_of('c'), None);
    /// ```
    #[inline]
    #[must_use]
    pub fn index_of(&self, codepoint: impl Into<u32>) -> Option<u32> {
        let codepoint = codepoint.into();
        // Last interval whose left bound is `<= codepoint`.
        let idx = self
            .intervals
            .partition_point(|&(left, _)| left <= codepoint);
        if idx == 0 {
            return None;
        }
        // INVARIANT: `idx >= 1` per the check above.
        #[allow(clippy::arithmetic_side_effects)]
        let (left, right) = self.intervals[idx - 1];
        if codepoint <= right {
            // INVARIANT: `left <= codepoint` and offsets are small enough to not overflow.
            #[allow(clippy::arithmetic_side_effects)]
            Some(self.offsets[idx - 1] + (codepoint - left))
        } else {
            None
        }
    }

    /// Returns the index of a specific codepoint in the `IntervalSet` if it is present in the set,
    /// or the index of the closest codepoint that is greater than the given one.
    ///
    /// If the given codepoint is greater than the largest codepoint in the set, then the set's
    /// size is returned.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use unicode_intervals::UnicodeCategory;
    /// let interval_set = unicode_intervals::query()
    ///     .include_categories(UnicodeCategory::UPPERCASE_LETTER)
    ///     .interval_set()
    ///     .expect("Invalid query input");
    /// assert_eq!(interval_set.index_above('Z'), 25);
    /// ```
    #[inline]
    #[must_use]
    pub fn index_above(&self, codepoint: impl Into<u32>) -> u32 {
        let codepoint = codepoint.into();
        // Last interval whose left bound is `<= codepoint`.
        let idx = self
            .intervals
            .partition_point(|&(left, _)| left <= codepoint);
        if idx > 0 {
            // INVARIANT: `idx >= 1` per the check above.
            #[allow(clippy::arithmetic_side_effects)]
            let (left, right) = self.intervals[idx - 1];
            if codepoint <= right {
                // INVARIANT: `left <= codepoint` and offsets are small enough to not overflow.
                #[allow(clippy::arithmetic_side_effects)]
                return self.offsets[idx - 1] + (codepoint - left);
            }
        }
        // `codepoint` falls in a gap (or after the last interval): the next codepoint is the
        // start of interval `idx`, whose index is `offsets[idx]` (== `size` when `idx == len`).
        self.offsets[idx]
    }

    /// Returns an iterator over all codepoints in all contained intervals.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use unicode_intervals::UnicodeCategory;
    /// let interval_set = unicode_intervals::query()
    ///     .include_categories(UnicodeCategory::UPPERCASE_LETTER)
    ///     .max_codepoint(67)
    ///     .interval_set()
    ///     .expect("Invalid query input");
    /// let mut iterator = interval_set.iter();
    /// assert_eq!(iterator.next(), Some('A' as u32));
    /// assert_eq!(iterator.next(), Some('B' as u32));
    /// assert_eq!(iterator.next(), Some('C' as u32));
    /// assert_eq!(iterator.next(), None);
    /// ```
    pub fn iter(&self) -> Codepoints<'_> {
        fn expand((left, right): Interval) -> core::ops::RangeInclusive<u32> {
            left..=right
        }
        let expand: Expand = expand;
        Codepoints(self.intervals.iter().copied().flat_map(expand))
    }
}

type Expand = fn(Interval) -> core::ops::RangeInclusive<u32>;

/// Iterator over the codepoints of an [`IntervalSet`].
#[derive(Debug, Clone)]
pub struct Codepoints<'a>(
    core::iter::FlatMap<
        core::iter::Copied<core::slice::Iter<'a, Interval>>,
        core::ops::RangeInclusive<u32>,
        Expand,
    >,
);

impl Iterator for Codepoints<'_> {
    type Item = u32;

    #[inline]
    fn next(&mut self) -> Option<u32> {
        self.0.next()
    }
}

impl DoubleEndedIterator for Codepoints<'_> {
    #[inline]
    fn next_back(&mut self) -> Option<u32> {
        self.0.next_back()
    }
}

impl<'a> IntoIterator for &'a IntervalSet {
    type Item = u32;
    type IntoIter = Codepoints<'a>;

    #[inline]
    fn into_iter(self) -> Codepoints<'a> {
        self.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{UnicodeCategory, UnicodeVersion};
    use test_case::test_case;

    // Pinned to a fixed Unicode version so the expected indices/counts below stay
    // deterministic across Unicode upgrades; these tests exercise `IntervalSet`
    // mechanics, not the latest Unicode data.
    fn uppercase_letters() -> IntervalSet {
        UnicodeVersion::V15_0_0
            .query()
            .include_categories(UnicodeCategory::UPPERCASE_LETTER)
            .interval_set()
            .expect("Invalid query input")
    }

    #[test_case(vec![(1, 1)])]
    #[test_case(vec![])]
    fn test_index_not_present(intervals: Vec<Interval>) {
        assert!(IntervalSet::new(intervals).index_of(0_u32).is_none());
    }

    #[test_case(vec![], 1, None)]
    #[test_case(vec![(1, 10)], 11, None)]
    fn test_get(intervals: Vec<Interval>, index: u32, expected: Option<u32>) {
        assert_eq!(IntervalSet::new(intervals).codepoint_at(index), expected);
    }

    #[test_case(vec![(1, 10)], 1, 0)]
    #[test_case(vec![(1, 10)], 2, 1)]
    #[test_case(vec![(1, 10)], 100, 10)]
    fn test_index_above(intervals: Vec<Interval>, index: u32, expected: u32) {
        assert_eq!(IntervalSet::new(intervals).index_above(index), expected);
    }

    #[test_case('Z' as u32, 25; "In the set")]
    #[test_case('b' as u32, 26; "Not in the set")]
    #[test_case(125218, 1831; "Greater than all")]
    fn test_index_above_with_uppercase_letters(codepoint: u32, expected: u32) {
        let interval_set = uppercase_letters();
        assert_eq!(interval_set.index_above(codepoint), expected);
    }

    #[test_case('C', true)]
    #[test_case('a', false)]
    fn test_contains(codepoint: char, expected: bool) {
        let interval_set = uppercase_letters();
        assert_eq!(interval_set.contains(codepoint), expected);
    }

    #[test_case(10, Some('K' as u32); "Look from left")]
    #[test_case(27, Some('Á' as u32); "Look from right")]
    #[test_case(1830, Some(125217); "Max codepoint in the set")]
    #[test_case(10000, None)]
    #[test_case(u32::MAX, None)]
    fn test_codepoint_at(index: u32, expected: Option<u32>) {
        let interval_set = uppercase_letters();
        assert_eq!(interval_set.codepoint_at(index), expected);
    }

    #[test]
    fn test_codepoint_at_empty_set() {
        let interval_set = IntervalSet::new(vec![]);
        assert!(interval_set.codepoint_at(0).is_none());
    }

    // Oracle: compare every lookup method against a flattened reference over a set
    // with single-element intervals, multi-element intervals, and gaps between them.
    #[test]
    fn test_lookups_against_oracle() {
        let intervals = vec![(1, 3), (10, 12), (20, 20), (100, 200)];
        let set = IntervalSet::new(intervals.clone());
        // Reference: every codepoint paired with its index, in order.
        let flat: Vec<u32> = intervals
            .iter()
            .flat_map(|(left, right)| *left..=*right)
            .collect();
        let total = u32::try_from(flat.len()).expect("fits in u32");

        // codepoint_at over every valid index, plus past the end.
        for (index, expected) in (0u32..).zip(flat.iter()) {
            assert_eq!(
                set.codepoint_at(index),
                Some(*expected),
                "codepoint_at({index})"
            );
        }
        assert_eq!(set.codepoint_at(total), None);

        // index_of / contains / index_above over the full codepoint span and beyond.
        for codepoint in 0..=210_u32 {
            let expected_index = (0u32..)
                .zip(flat.iter())
                .find(|(_, &c)| c == codepoint)
                .map(|(index, _)| index);
            assert_eq!(
                set.index_of(codepoint),
                expected_index,
                "index_of({codepoint})"
            );
            assert_eq!(
                set.contains(codepoint),
                expected_index.is_some(),
                "contains({codepoint})"
            );
            // Index of the first codepoint `>= codepoint`, or `total` if none.
            let expected_above = (0u32..)
                .zip(flat.iter())
                .find(|(_, &c)| c >= codepoint)
                .map_or(total, |(index, _)| index);
            assert_eq!(
                set.index_above(codepoint),
                expected_above,
                "index_above({codepoint})"
            );
        }
    }

    #[test_case('K' as u32, Some(10); "Look from left")]
    #[test_case('Á' as u32, Some(27); "Look from right")]
    #[test_case(125184, Some(1797))]
    #[test_case(5, None)]
    fn test_index_of(codepoint: u32, expected: Option<u32>) {
        let interval_set = uppercase_letters();
        assert_eq!(interval_set.index_of(codepoint), expected);
    }

    #[test]
    fn test_iter() {
        let intervals = crate::query()
            .include_categories(UnicodeCategory::LOWERCASE_LETTER)
            .intervals()
            .expect("Invalid query input");
        let interval_set = IntervalSet::new(intervals);
        let codepoints: Vec<_> = interval_set.iter().collect();
        let mut expected = Vec::with_capacity(interval_set.len());
        for (left, right) in
            UnicodeVersion::latest().intervals_for(UnicodeCategory::LOWERCASE_LETTER)
        {
            for codepoint in *left..=*right {
                expected.push(codepoint);
            }
        }
        assert_eq!(codepoints, expected);
        assert_eq!(interval_set.len(), codepoints.len());
        assert!(!interval_set.is_empty());
    }

    #[test]
    fn test_iter_rev() {
        let interval_set = uppercase_letters();
        let mut iter = interval_set.iter().rev();
        assert_eq!(iter.next(), Some(125217));
    }

    #[test]
    fn test_into_iterator_for_ref() {
        let interval_set = IntervalSet::new(vec![(65, 67), (70, 70)]);
        let collected: Vec<u32> = (&interval_set).into_iter().collect();
        assert_eq!(collected, vec![65, 66, 67, 70]);
        let mut via_for_loop = Vec::new();
        for codepoint in &interval_set {
            via_for_loop.push(codepoint);
        }
        assert_eq!(via_for_loop, vec![65, 66, 67, 70]);
    }

    #[test]
    #[allow(clippy::redundant_clone)]
    fn test_interval_set_traits() {
        let interval_set = IntervalSet::new(vec![(0, 1)]);
        let _ = interval_set.clone();
        assert_eq!(
            format!("{interval_set:?}"),
            "IntervalSet { intervals: [(0, 1)], offsets: [0, 2], size: 2 }"
        );
    }
}
