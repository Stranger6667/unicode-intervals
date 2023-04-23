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
        #[allow(clippy::integer_arithmetic)]
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
    /// # use unicode_intervals::{UnicodeVersion, UnicodeCategory};
    /// let interval_set = UnicodeVersion::V15_0_0.query()
    ///     .include_categories(UnicodeCategory::UPPERCASE_LETTER)
    ///     .interval_set()
    ///     .expect("Invalid query input");
    /// assert_eq!(interval_set.len(), 1831);
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
    /// # use unicode_intervals::{UnicodeVersion, UnicodeCategory};
    /// let interval_set = UnicodeVersion::V15_0_0.query()
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
    /// # use unicode_intervals::{UnicodeVersion, UnicodeCategory};
    /// let interval_set = UnicodeVersion::V15_0_0.query()
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
    /// # use unicode_intervals::{UnicodeVersion, UnicodeCategory};
    /// let interval_set = UnicodeVersion::V15_0_0.query()
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
        // INVARIANT: There is a positive number of intervals at this point per the check above
        #[allow(clippy::integer_arithmetic)]
        let mut current = self.intervals.len() - 1;
        if self.offsets[current] > index {
            let (mut high, mut low) = (current, 0_usize);
            // INVARIANTS:
            //   - `low + 1` never overflows as all possible values are far below `u32::MAX`
            //   - `low + high` never overflows because two maximum values for these variables
            //     are far below `u32::MAX`
            #[allow(clippy::integer_arithmetic)]
            while low + 1 < high {
                let mid = (low + high) / 2;
                if self.offsets[mid] <= index {
                    low = mid;
                } else {
                    high = mid;
                }
            }
            current = low;
        }
        // INVARIANT: `index` & offsets are small enough and won't cause overflow
        #[allow(clippy::integer_arithmetic)]
        Some(self.intervals[current].0 + index - self.offsets[current])
    }

    /// Returns the index of a specific codepoint in the `IntervalSet`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use unicode_intervals::{UnicodeVersion, UnicodeCategory};
    /// let interval_set = UnicodeVersion::V15_0_0.query()
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
        for (offset, (left, right)) in self.offsets.iter().zip(self.intervals.iter()) {
            if *left == codepoint {
                return Some(*offset);
            } else if *left > codepoint {
                return None;
            } else if codepoint <= *right {
                // INVARIANT: `left` is smaller than `codepoint` and `offset` is small enough,
                // so there is no overflow
                #[allow(clippy::integer_arithmetic)]
                return Some(*offset + (codepoint - left));
            }
        }
        None
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
    /// # use unicode_intervals::{UnicodeVersion, UnicodeCategory};
    /// let interval_set = UnicodeVersion::V15_0_0.query()
    ///     .include_categories(UnicodeCategory::UPPERCASE_LETTER)
    ///     .interval_set()
    ///     .expect("Invalid query input");
    /// assert_eq!(interval_set.index_above('Z'), 25);
    /// ```
    #[inline]
    #[must_use]
    pub fn index_above(&self, codepoint: impl Into<u32>) -> u32 {
        let codepoint = codepoint.into();
        for (offset, (left, right)) in self.offsets.iter().zip(self.intervals.iter()) {
            if *left >= codepoint {
                return *offset;
            } else if codepoint <= *right {
                // INVARIANT: `left` is smaller than `codepoint` and `offset` is small enough,
                // so there is no overflow
                #[allow(clippy::integer_arithmetic)]
                return *offset + (codepoint - left);
            }
        }
        self.size
    }

    /// Returns an iterator over all codepoints in all contained intervals.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use unicode_intervals::{UnicodeVersion, UnicodeCategory};
    /// let interval_set = UnicodeVersion::V15_0_0.query()
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
    pub fn iter(&self) -> impl Iterator<Item = u32> + '_ {
        self.intervals
            .iter()
            .flat_map(|(left, right)| *left..=*right)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{UnicodeCategory, UnicodeVersion};
    use test_case::test_case;

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

    #[test_case('K' as u32, Some(10); "Look from left")]
    #[test_case('Á' as u32, Some(27); "Look from right")]
    #[test_case(125184, Some(1797))]
    #[test_case(5, None)]
    fn test_index_of(codepoint: u32, expected: Option<u32>) {
        let interval_set = uppercase_letters();
        assert_eq!(interval_set.index_of(codepoint), expected);
    }

    #[test]
    fn test_intervals_iter() {
        let intervals = UnicodeVersion::V15_0_0
            .query()
            .include_categories(UnicodeCategory::LOWERCASE_LETTER)
            .intervals()
            .expect("Invalid query input");
        let interval_set = IntervalSet::new(intervals);
        let codepoints: Vec<_> = interval_set.iter().collect();
        let mut expected = Vec::with_capacity(interval_set.len());
        for (left, right) in
            UnicodeVersion::V15_0_0.intervals_for(UnicodeCategory::LOWERCASE_LETTER)
        {
            for codepoint in *left..=*right {
                expected.push(codepoint);
            }
        }
        assert_eq!(codepoints, expected);
        assert_eq!(interval_set.len(), codepoints.len());
        assert!(!interval_set.is_empty());
    }
}
