use crate::{constants::ALL_CATEGORIES, error};
use core::{
    fmt,
    ops::{BitOr, BitOrAssign},
    str::FromStr,
};
use UnicodeCategory::*;

/// Unicode category abbreviation.
#[derive(Debug, Eq, PartialEq, Copy, Clone, Hash)]
pub enum UnicodeCategory {
    /// Close Punctuation.
    Pe,
    /// Connector Punctuation.
    Pc,
    /// Control.
    Cc,
    /// Currency Symbol.
    Sc,
    /// Dash Punctuation.
    Pd,
    /// Decimal Number.
    Nd,
    /// Enclosing Mark.
    Me,
    /// Final Punctuation.
    Pf,
    /// Format.
    Cf,
    /// Initial Punctuation.
    Pi,
    /// Letter Number.
    Nl,
    /// Line Separator.
    Zl,
    /// Lowercase Letter.
    Ll,
    /// Math Symbol.
    Sm,
    /// Modifier Letter.
    Lm,
    /// Modifier Symbol.
    Sk,
    /// Nonspacing Mark.
    Mn,
    /// Open Punctuation.
    Ps,
    /// Other Letter.
    Lo,
    /// Other Number.
    No,
    /// Other Punctuation.
    Po,
    /// Other Symbol.
    So,
    /// Paragraph Separator.
    Zp,
    /// Private Use.
    Co,
    /// Space Separator.
    Zs,
    /// Spacing Mark.
    Mc,
    /// Surrogate.
    Cs,
    /// Titlecase Letter.
    Lt,
    /// Unassigned.
    Cn,
    /// Uppercase Letter.
    Lu,
}

impl FromStr for UnicodeCategory {
    type Err = error::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "Pe" => Pe,
            "Pc" => Pc,
            "Cc" => Cc,
            "Sc" => Sc,
            "Pd" => Pd,
            "Nd" => Nd,
            "Me" => Me,
            "Pf" => Pf,
            "Cf" => Cf,
            "Pi" => Pi,
            "Nl" => Nl,
            "Zl" => Zl,
            "Ll" => Ll,
            "Sm" => Sm,
            "Lm" => Lm,
            "Sk" => Sk,
            "Mn" => Mn,
            "Ps" => Ps,
            "Lo" => Lo,
            "No" => No,
            "Po" => Po,
            "So" => So,
            "Zp" => Zp,
            "Co" => Co,
            "Zs" => Zs,
            "Mc" => Mc,
            "Cs" => Cs,
            "Lt" => Lt,
            "Cn" => Cn,
            "Lu" => Lu,
            _ => return Err(Self::Err::InvalidCategory(s.to_owned().into_boxed_str())),
        })
    }
}

impl UnicodeCategory {
    /// Letters.
    pub const L: UnicodeCategorySet = UnicodeCategorySet(
        1 << Ll as u32 | 1 << Lm as u32 | 1 << Lo as u32 | 1 << Lt as u32 | 1 << Lu as u32,
    );
    /// Marks.
    pub const M: UnicodeCategorySet =
        UnicodeCategorySet(1 << Mc as u32 | 1 << Me as u32 | 1 << Mn as u32);
    /// Numbers.
    pub const N: UnicodeCategorySet =
        UnicodeCategorySet(1 << Nd as u32 | 1 << Nl as u32 | 1 << No as u32);
    /// Punctuation.
    pub const P: UnicodeCategorySet = UnicodeCategorySet(
        1 << Pc as u32
            | 1 << Pd as u32
            | 1 << Pe as u32
            | 1 << Pf as u32
            | 1 << Pi as u32
            | 1 << Po as u32
            | 1 << Ps as u32,
    );
    /// Symbols.
    pub const S: UnicodeCategorySet =
        UnicodeCategorySet(1 << Sc as u32 | 1 << Sk as u32 | 1 << Sm as u32 | 1 << So as u32);
    /// Separators.
    pub const Z: UnicodeCategorySet =
        UnicodeCategorySet(1 << Zp as u32 | 1 << Zs as u32 | 1 << Zl as u32);
    /// Control, format, private, unassigned and surrogates.
    pub const C: UnicodeCategorySet = UnicodeCategorySet(
        1 << Cc as u32 | 1 << Cf as u32 | 1 << Cn as u32 | 1 << Co as u32 | 1 << Cs as u32,
    );
    // Full category names
    /// Close Punctuation (alias).
    pub const CLOSE_PUNCTUATION: UnicodeCategory = Pe;
    /// Connector Punctuation (alias).
    pub const CONNECTOR_PUNCTUATION: UnicodeCategory = Pc;
    /// Control (alias).
    pub const CONTROL: UnicodeCategory = Cc;
    /// Currency Symbol (alias).
    pub const CURRENCY_SYMBOL: UnicodeCategory = Sc;
    /// Dash Punctuation (alias).
    pub const DASH_PUNCTUATION: UnicodeCategory = Pd;
    /// Decimal Number (alias).
    pub const DECIMAL_NUMBER: UnicodeCategory = Nd;
    /// Enclosing Mark (alias).
    pub const ENCLOSING_MARK: UnicodeCategory = Me;
    /// Final Punctuation (alias).
    pub const FINAL_PUNCTUATION: UnicodeCategory = Pf;
    /// Format (alias).
    pub const FORMAT: UnicodeCategory = Cf;
    /// Initial Punctuation (alias).
    pub const INITIAL_PUNCTUATION: UnicodeCategory = Pi;
    /// Letter Number (alias).
    pub const LETTER_NUMBER: UnicodeCategory = Nl;
    /// Line Separator (alias).
    pub const LINE_SEPARATOR: UnicodeCategory = Zl;
    /// Lowercase Letter (alias).
    pub const LOWERCASE_LETTER: UnicodeCategory = Ll;
    /// Math Symbol (alias).
    pub const MATH_SYMBOL: UnicodeCategory = Sm;
    /// Modifier Letter (alias).
    pub const MODIFIER_LETTER: UnicodeCategory = Lm;
    /// Modifier Symbol (alias).
    pub const MODIFIER_SYMBOL: UnicodeCategory = Sk;
    /// Nonspacing Mark (alias).
    pub const NONSPACING_MARK: UnicodeCategory = Mn;
    /// Open Punctuation (alias).
    pub const OPEN_PUNCTUATION: UnicodeCategory = Ps;
    /// Other Letter (alias).
    pub const OTHER_LETTER: UnicodeCategory = Lo;
    /// Other Number (alias).
    pub const OTHER_NUMBER: UnicodeCategory = No;
    /// Other Punctuation (alias).
    pub const OTHER_PUNCTUATION: UnicodeCategory = Po;
    /// Other Symbol (alias).
    pub const OTHER_SYMBOL: UnicodeCategory = So;
    /// Paragraph Separator (alias).
    pub const PARAGRAPH_SEPARATOR: UnicodeCategory = Zp;
    /// Private Use (alias).
    pub const PRIVATE_USE: UnicodeCategory = Co;
    /// Space Separator (alias).
    pub const SPACE_SEPARATOR: UnicodeCategory = Zs;
    /// Spacing Mark (alias).
    pub const SPACING_MARK: UnicodeCategory = Mc;
    /// Surrogate (alias).
    pub const SURROGATE: UnicodeCategory = Cs;
    /// Titlecase Letter (alias).
    pub const TITLECASE_LETTER: UnicodeCategory = Lt;
    /// Unassigned (alias).
    pub const UNASSIGNED: UnicodeCategory = Cn;
    /// Uppercase Letter (alias).
    pub const UPPERCASE_LETTER: UnicodeCategory = Lu;

    /// Abbreviation as a string.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Pe => "Pe",
            Pc => "Pc",
            Cc => "Cc",
            Sc => "Sc",
            Pd => "Pd",
            Nd => "Nd",
            Me => "Me",
            Pf => "Pf",
            Cf => "Cf",
            Pi => "Pi",
            Nl => "Nl",
            Zl => "Zl",
            Ll => "Ll",
            Sm => "Sm",
            Lm => "Lm",
            Sk => "Sk",
            Mn => "Mn",
            Ps => "Ps",
            Lo => "Lo",
            No => "No",
            Po => "Po",
            So => "So",
            Zp => "Zp",
            Co => "Co",
            Zs => "Zs",
            Mc => "Mc",
            Cs => "Cs",
            Lt => "Lt",
            Cn => "Cn",
            Lu => "Lu",
        }
    }
}

impl fmt::Display for UnicodeCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Set of Unicode categories.
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct UnicodeCategorySet(u32);

impl UnicodeCategorySet {
    /// Empty set of Unicode categories.
    #[inline]
    #[must_use]
    pub const fn new() -> Self {
        Self(0)
    }
    /// All Unicode categories.
    #[inline]
    #[must_use]
    pub const fn all() -> Self {
        Self(ALL_CATEGORIES)
    }
    /// Create a category set, but do not check whether the input value is valid.
    #[inline]
    #[must_use]
    pub(crate) const fn from_value_unchecked(value: u32) -> Self {
        Self(value)
    }
    /// Add a new Unicode category to the set.
    #[inline]
    pub fn add_category(&mut self, category: UnicodeCategory) {
        self.set(category as u8);
    }
    /// Whether the set contains `category`.
    #[inline]
    #[must_use]
    pub const fn has_category(self, category: UnicodeCategory) -> bool {
        self.is_set(category as u8)
    }
    /// The size of the set.
    #[inline]
    #[must_use]
    pub const fn len(self) -> usize {
        self.0.count_ones() as usize
    }
    /// Whether the set is empty.
    #[inline]
    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }
    /// Extract the inner storage value.
    #[inline]
    #[must_use]
    pub const fn into_value(self) -> u32 {
        self.0
    }
    /// Iterate over included Unicode categories.
    #[inline]
    #[must_use]
    pub const fn iter(self) -> Iter {
        Iter {
            index: 0,
            data: self,
        }
    }
    // `index` is always < 30 and can't overflow
    #[inline]
    #[allow(clippy::integer_arithmetic)]
    pub(crate) fn set(&mut self, index: u8) {
        self.0 |= 1 << index;
    }
    // `index`` is always < 30 and can't overflow
    #[inline]
    #[allow(clippy::integer_arithmetic)]
    const fn is_set(self, index: u8) -> bool {
        self.0 & (1 << index) != 0
    }
}

impl Default for UnicodeCategorySet {
    #[inline]
    fn default() -> Self {
        UnicodeCategorySet::new()
    }
}

impl fmt::Display for UnicodeCategorySet {
    // `idx` can't overflow as the maximum possible size of `iter` is 30 < usize::MAX
    #[allow(clippy::integer_arithmetic)]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (idx, category) in self.iter().enumerate() {
            f.write_str(category.as_str())?;
            if idx + 1 != self.len() {
                f.write_str(", ")?;
            }
        }
        Ok(())
    }
}

impl BitOr for UnicodeCategory {
    type Output = UnicodeCategorySet;

    // `self` and `rhs` are both < 30; Therefore shift won't overflow
    #[inline]
    #[allow(clippy::integer_arithmetic)]
    fn bitor(self, rhs: Self) -> Self::Output {
        UnicodeCategorySet(1 << self as u8 | 1 << rhs as u8)
    }
}
impl BitOr<UnicodeCategorySet> for UnicodeCategory {
    type Output = UnicodeCategorySet;

    #[inline]
    fn bitor(self, rhs: UnicodeCategorySet) -> Self::Output {
        // Reusing existing `BitOr<UnicodeCategory> for UnicodeCategorySet`
        rhs | self
    }
}

impl BitOr<UnicodeCategory> for UnicodeCategorySet {
    type Output = Self;

    // `rhs as u8` can't overflow as it has only 30 elements
    #[inline]
    #[allow(clippy::integer_arithmetic)]
    fn bitor(self, rhs: UnicodeCategory) -> Self::Output {
        Self(self.into_value() | 1 << rhs as u8)
    }
}

impl BitOr<UnicodeCategorySet> for UnicodeCategorySet {
    type Output = Self;

    #[inline]
    fn bitor(self, rhs: UnicodeCategorySet) -> Self::Output {
        Self(self.into_value() | rhs.into_value())
    }
}

impl BitOrAssign<UnicodeCategorySet> for UnicodeCategorySet {
    #[inline]
    fn bitor_assign(&mut self, rhs: UnicodeCategorySet) {
        self.0 |= rhs.into_value();
    }
}

impl BitOrAssign<UnicodeCategory> for UnicodeCategorySet {
    #[inline]
    fn bitor_assign(&mut self, rhs: UnicodeCategory) {
        self.add_category(rhs);
    }
}

#[derive(Debug)]
pub struct Iter {
    index: u8,
    data: UnicodeCategorySet,
}

impl Iterator for Iter {
    type Item = UnicodeCategory;

    // `self.index` can't be greater than 30 as checked in the beginning of the function
    #[allow(clippy::integer_arithmetic)]
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.index >= 30 {
                return None;
            }
            if self.data.is_set(self.index) {
                let category = match self.index {
                    0 => Pe,
                    1 => Pc,
                    2 => Cc,
                    3 => Sc,
                    4 => Pd,
                    5 => Nd,
                    6 => Me,
                    7 => Pf,
                    8 => Cf,
                    9 => Pi,
                    10 => Nl,
                    11 => Zl,
                    12 => Ll,
                    13 => Sm,
                    14 => Lm,
                    15 => Sk,
                    16 => Mn,
                    17 => Ps,
                    18 => Lo,
                    19 => No,
                    20 => Po,
                    21 => So,
                    22 => Zp,
                    23 => Co,
                    24 => Zs,
                    25 => Mc,
                    26 => Cs,
                    27 => Lt,
                    28 => Cn,
                    29 => Lu,
                    _ => unreachable!("The index can't be >= 30 as checked above"),
                };
                self.index += 1;
                return Some(category);
            };
            self.index += 1;
        }
    }
}

impl ExactSizeIterator for Iter {
    #[inline]
    fn len(&self) -> usize {
        self.data.len()
    }
}

impl From<UnicodeCategory> for UnicodeCategorySet {
    // `category as u8` can't overflow as it has only 30 elements
    #[inline]
    #[allow(clippy::integer_arithmetic)]
    fn from(category: UnicodeCategory) -> Self {
        Self::from_value_unchecked(1 << category as u8)
    }
}

impl From<UnicodeCategory> for Option<UnicodeCategorySet> {
    #[inline]
    fn from(category: UnicodeCategory) -> Self {
        Some(category.into())
    }
}

/// Return all Unicode categories that are in `include`, but not in `exclude`.
#[inline]
#[must_use]
pub const fn merge(
    include: Option<UnicodeCategorySet>,
    exclude: UnicodeCategorySet,
) -> UnicodeCategorySet {
    if let Some(include) = include {
        if include.is_empty() {
            // include no categories
            include
        } else {
            UnicodeCategorySet::from_value_unchecked(
                (ALL_CATEGORIES ^ exclude.into_value()) & include.into_value(),
            )
        }
    } else {
        UnicodeCategorySet::from_value_unchecked(ALL_CATEGORIES ^ exclude.into_value())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    fn hash(_: impl core::hash::Hash) {}

    #[test]
    fn test_category_from_str_error() {
        assert_eq!(
            UnicodeCategory::from_str("wrong")
                .expect_err("Should fail")
                .to_string(),
            "'wrong' is not a valid Unicode category"
        );
    }

    #[test]
    fn test_category_is_hashable() {
        hash(Ll);
    }

    #[test]
    fn test_single_letter_categories() {
        assert_eq!(UnicodeCategory::L, Ll | Lm | Lo | Lt | Lu);
    }

    #[test]
    fn test_set_display() {
        assert_eq!(UnicodeCategory::L.to_string(), "Ll, Lm, Lo, Lt, Lu");
    }

    #[test]
    fn test_set_add_category() {
        let mut set = UnicodeCategorySet::new();
        assert!(set.is_empty());
        set.add_category(Ll);
        assert!(set.has_category(Ll));
        assert_eq!(set.len(), 1);
    }

    #[test]
    fn test_bit_or() {
        assert_eq!(Ll | UnicodeCategorySet::new(), Ll.into());
        assert_eq!(
            UnicodeCategory::L | UnicodeCategory::C,
            Ll | Lm | Lo | Lt | Lu | Cs | Cc | Cf | Cn | Co
        );
        let mut set = UnicodeCategorySet::new();
        set |= Ll;
        set |= UnicodeCategory::C;
        assert_eq!(set, Ll | Cs | Cc | Cf | Cn | Co);
    }

    #[test]
    fn test_set_iter() {
        let all_categories = UnicodeCategorySet::all();
        assert_eq!(all_categories.iter().len(), all_categories.len());
        let mut set = UnicodeCategorySet::new();
        for category in all_categories.iter() {
            let name = format!("{category}");
            assert_eq!(
                UnicodeCategory::from_str(&name).expect("Invalid category"),
                category
            );
            set.add_category(category);
        }
        assert_eq!(all_categories, set);
    }

    #[test]
    fn test_set_default() {
        assert_eq!(UnicodeCategorySet::default(), UnicodeCategorySet::new());
    }

    #[test]
    fn test_set_is_hashable() {
        hash(UnicodeCategory::L);
    }

    #[test]
    fn test_set_option_from_category() {
        let set: Option<UnicodeCategorySet> = Ll.into();
        assert!(set.is_some());
        assert_eq!(set.expect("Unexpected `None`"), Ll.into());
    }

    #[test_case(Some(Lu | Me | Cs | So), So.into(), Lu | Me | Cs)]
    #[test_case(None, UnicodeCategory::L | UnicodeCategory::M | UnicodeCategory::N | UnicodeCategory::P | UnicodeCategory::S, UnicodeCategory::Z | UnicodeCategory::C)]
    #[test_case(
        Some(UnicodeCategorySet::new()),
        UnicodeCategorySet::new(),
        UnicodeCategorySet::new()
    )]
    fn test_category_merge(
        include: Option<UnicodeCategorySet>,
        exclude: UnicodeCategorySet,
        expected: UnicodeCategorySet,
    ) {
        assert_eq!(merge(include, exclude), expected);
    }
}
