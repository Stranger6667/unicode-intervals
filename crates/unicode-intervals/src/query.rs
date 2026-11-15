#[cfg(target_arch = "x86_64")]
use crate::avx512;
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

    let intervals = if categories.len() > 1 && categories.into_value() != ALL_CATEGORIES {
        Cow::Owned(union_runs(
            RUNS[version as usize],
            categories.into_value(),
            min_codepoint,
            max_codepoint,
            Simd::best(),
        ))
    } else if min_codepoint == 0 && max_codepoint == MAX_CODEPOINT {
        intervals_for_set(version, categories)
    } else {
        Cow::Owned(collect_in_range(
            version,
            categories,
            min_codepoint,
            max_codepoint,
        ))
    };
    // Include intervals
    let intervals = if include_intervals.is_empty() {
        intervals.into_owned()
    } else {
        intervals::union(&intervals, &include_intervals)
    };
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

/// Collect the intervals of at most one category (or all of them) clamped to
/// `[min_codepoint, max_codepoint]`.
fn collect_in_range(
    version: UnicodeVersion,
    categories: UnicodeCategorySet,
    min_codepoint: u32,
    max_codepoint: u32,
) -> Vec<Interval> {
    match categories.into_value() {
        0 => Vec::new(),
        ALL_CATEGORIES => vec![(min_codepoint, max_codepoint)],
        value => clamped(
            version.table()[value.trailing_zeros() as usize],
            min_codepoint,
            max_codepoint,
        ),
    }
}

/// Below this length, scanning the prefix is cheaper than a `log2(n)` search for the start.
const BINARY_SEARCH_THRESHOLD: usize = 32;

/// Skip the leading intervals below `min_codepoint`. Only search when it can pay off.
fn skip_below(table: &[Interval], min_codepoint: u32) -> &[Interval] {
    match table.first() {
        Some(&(_, right)) if right < min_codepoint && table.len() >= BINARY_SEARCH_THRESHOLD => {
            &table[table.partition_point(|&(_, right)| right < min_codepoint)..]
        }
        _ => table,
    }
}

/// Intervals of `table` overlapping `[min_codepoint, max_codepoint]`, clamped to it.
/// `table` is sorted and non-overlapping, so a long overlap is one contiguous slice:
/// copy it in bulk and clamp its ends.
fn clamped(table: &[Interval], min_codepoint: u32, max_codepoint: u32) -> Vec<Interval> {
    let table = skip_below(table, min_codepoint);
    match table.get(BINARY_SEARCH_THRESHOLD.saturating_sub(1)) {
        Some(&(left, _)) if left <= max_codepoint => {
            let end = table.partition_point(|&(left, _)| left <= max_codepoint);
            let mut intervals = table[..end].to_vec();
            // The probe matched, so `end >= BINARY_SEARCH_THRESHOLD`.
            #[allow(clippy::arithmetic_side_effects)]
            let last = end - 1;
            intervals[0].0 = max(intervals[0].0, min_codepoint);
            intervals[last].1 = min(intervals[last].1, max_codepoint);
            intervals
        }
        _ => {
            let mut intervals = Vec::new();
            extend_clamped(&mut intervals, table, min_codepoint, max_codepoint);
            intervals
        }
    }
}

/// Append intervals overlapping `[min_codepoint, max_codepoint]`, clamped to it.
/// `table` is sorted and non-overlapping, so binary search past the leading intervals below
/// the range, then stop on the first one past it. Only search when it can pay off.
fn extend_clamped(
    out: &mut Vec<Interval>,
    table: &[Interval],
    min_codepoint: u32,
    max_codepoint: u32,
) {
    for &(left, right) in skip_below(table, min_codepoint) {
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

/// A run packs a category interval as `left << CATEGORY_BITS | category`.
pub(crate) const CATEGORY_BITS: u32 = 5;
pub(crate) const CATEGORY_MASK: u32 = (1 << CATEGORY_BITS) - 1;

const fn count_runs(table: &[&[Interval]]) -> usize {
    let mut count = 0;
    let mut category = 0;
    // Table sizes are far below `usize::MAX`.
    #[allow(clippy::arithmetic_side_effects)]
    while category < table.len() {
        count += table[category].len();
        category += 1;
    }
    count
}

/// All category intervals of `table` as sorted runs. The categories partition the codepoint
/// space, so each run ends right before the next one starts.
#[allow(clippy::arithmetic_side_effects, clippy::cast_possible_truncation)]
const fn build_runs<const N: usize>(table: &[&[Interval]]) -> [u32; N] {
    let mut runs = [0u32; N];
    let mut cursors = [0usize; 30];
    let mut next_left = 0;
    let mut idx = 0;
    while idx < N {
        // The category whose next interval starts where the previous run ended.
        let mut category = 0;
        loop {
            assert!(
                category < table.len(),
                "categories must partition the codepoint space"
            );
            let slice = table[category];
            if cursors[category] < slice.len() && slice[cursors[category]].0 == next_left {
                break;
            }
            category += 1;
        }
        let (left, right) = table[category][cursors[category]];
        runs[idx] = left << CATEGORY_BITS | category as u32;
        cursors[category] += 1;
        next_left = right + 1;
        idx += 1;
    }
    assert!(
        next_left == MAX_CODEPOINT + 1,
        "categories must partition the codepoint space"
    );
    runs
}

macro_rules! runs {
    ($($version:ident),+) => {
        [$({
            static RUNS: [u32; count_runs(crate::tables::$version::BY_NAME)] =
                build_runs(crate::tables::$version::BY_NAME);
            &RUNS
        }),+]
    };
}

/// Runs for every bundled version, indexed by `UnicodeVersion as usize`.
static RUNS: [&[u32]; 12] = runs!(
    v9_0_0, v10_0_0, v11_0_0, v12_0_0, v12_1_0, v13_0_0, v14_0_0, v15_0_0, v15_1_0, v16_0_0,
    v17_0_0, v18_0_0
);

/// Union of the categories in `mask`, clamped to `[min_codepoint, max_codepoint]`.
/// Membership bits of 64 runs at a time: a bit that differs from the previous one is where
/// an interval starts or ends.
#[allow(clippy::arithmetic_side_effects)]
fn union_runs(
    runs: &[u32],
    mask: u32,
    min_codepoint: u32,
    max_codepoint: u32,
    simd: Simd,
) -> Vec<Interval> {
    assert!(simd.is_supported());
    // The first run starts at 0, so there is always one starting at or below `min_codepoint`.
    let start = runs.partition_point(|&run| run >> CATEGORY_BITS <= min_codepoint) - 1;
    let end = runs.partition_point(|&run| run >> CATEGORY_BITS <= max_codepoint);
    let runs = &runs[start..end];
    let mut intervals = Vec::with_capacity(runs.len() / 2 + 1);
    let mut left = 0;
    let mut previous = 0;
    for chunk in runs.chunks(64) {
        let members = match <&[u32; 64]>::try_from(chunk) {
            // SAFETY: SIMD support is checked above.
            Ok(chunk) => unsafe { simd.members(chunk, mask) },
            Err(_) => members_scalar(chunk, mask),
        };
        let mut edges = (members ^ (members << 1 | previous)) & (u64::MAX >> (64 - chunk.len()));
        // Most chunks of a sparse union have no edges at all: skip the bit count for them.
        if simd != Simd::Scalar && edges != 0 && edges.count_ones() >= COMPACT_EDGES {
            if let Ok(chunk) = <&[u32; 64]>::try_from(chunk) {
                let mut boundaries = [0; 64];
                // SAFETY: SIMD support is checked above.
                let count =
                    unsafe { simd.boundaries(chunk, edges, min_codepoint, &mut boundaries) };
                // Edges alternate between interval starts and ends.
                let mut boundaries = &boundaries[..count];
                if previous == 1 {
                    intervals.push((left, boundaries[0] - 1));
                    boundaries = &boundaries[1..];
                }
                let (pairs, rest) = boundaries.as_chunks::<2>();
                if let [start] = rest {
                    left = *start;
                }
                intervals.extend(pairs.iter().map(|&[start, end]| (start, end - 1)));
                edges = 0;
            }
        }
        previous = members >> (chunk.len() - 1);
        while edges != 0 {
            let idx = edges.trailing_zeros();
            let boundary = max(chunk[idx as usize] >> CATEGORY_BITS, min_codepoint);
            if members >> idx & 1 == 1 {
                left = boundary;
            } else {
                intervals.push((left, boundary - 1));
            }
            // Clear the lowest set bit.
            edges &= edges - 1;
        }
    }
    if previous == 1 {
        intervals.push((left, max_codepoint));
    }
    intervals
}

/// Implementation of the per-chunk steps of `union_runs`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Simd {
    Scalar,
    #[cfg(target_arch = "x86_64")]
    Avx2,
    #[cfg(target_arch = "x86_64")]
    Avx512,
    #[cfg(target_arch = "aarch64")]
    Neon,
}

impl Simd {
    /// Slowest first.
    const ALL: &'static [Simd] = &[
        Simd::Scalar,
        #[cfg(target_arch = "x86_64")]
        Simd::Avx2,
        #[cfg(target_arch = "x86_64")]
        Simd::Avx512,
        #[cfg(target_arch = "aarch64")]
        Simd::Neon,
    ];

    fn is_supported(self) -> bool {
        match self {
            Simd::Scalar => true,
            #[cfg(target_arch = "x86_64")]
            Simd::Avx2 => {
                std::arch::is_x86_feature_detected!("avx2")
                    && std::arch::is_x86_feature_detected!("popcnt")
            }
            #[cfg(target_arch = "x86_64")]
            Simd::Avx512 => {
                std::arch::is_x86_feature_detected!("avx512f")
                    && std::arch::is_x86_feature_detected!("popcnt")
            }
            // NEON is part of the `AArch64` baseline.
            #[cfg(target_arch = "aarch64")]
            Simd::Neon => true,
        }
    }

    /// The fastest implementation this CPU supports.
    fn best() -> Simd {
        let supported = Simd::ALL.iter().rev().find(|simd| simd.is_supported());
        supported.copied().unwrap_or(Simd::Scalar)
    }

    /// Membership bits of 64 runs.
    ///
    /// # Safety
    ///
    /// `self` must be supported by the CPU.
    #[inline]
    unsafe fn members(self, chunk: &[u32; 64], mask: u32) -> u64 {
        match self {
            Simd::Scalar => members_scalar(chunk, mask),
            #[cfg(target_arch = "x86_64")]
            // SAFETY: the caller guarantees AVX2 support.
            Simd::Avx2 => unsafe { members_avx2(chunk, mask) },
            #[cfg(target_arch = "x86_64")]
            // SAFETY: the caller guarantees AVX-512 support.
            Simd::Avx512 => unsafe { avx512::members(chunk, mask) },
            #[cfg(target_arch = "aarch64")]
            // SAFETY: NEON is always available.
            Simd::Neon => unsafe { members_neon(chunk, mask) },
        }
    }

    /// Start codepoints of the runs selected by `edges`, clamped to `min_codepoint` and
    /// packed to the front of `boundaries`. Returns how many were written.
    ///
    /// # Safety
    ///
    /// `self` must be supported by the CPU.
    #[inline]
    unsafe fn boundaries(
        self,
        chunk: &[u32; 64],
        edges: u64,
        min_codepoint: u32,
        boundaries: &mut [u32; 64],
    ) -> usize {
        match self {
            Simd::Scalar => boundaries_scalar(chunk, edges, min_codepoint, boundaries),
            #[cfg(target_arch = "x86_64")]
            // SAFETY: the caller guarantees AVX2 support.
            Simd::Avx2 => unsafe { boundaries_avx2(chunk, edges, min_codepoint, boundaries) },
            #[cfg(target_arch = "x86_64")]
            // SAFETY: the caller guarantees AVX-512 support.
            Simd::Avx512 => unsafe { avx512::boundaries(chunk, edges, min_codepoint, boundaries) },
            #[cfg(target_arch = "aarch64")]
            // SAFETY: NEON is always available.
            Simd::Neon => unsafe { boundaries_neon(chunk, edges, min_codepoint, boundaries) },
        }
    }
}

fn members_scalar(chunk: &[u32], mask: u32) -> u64 {
    chunk.iter().rev().fold(0u64, |members, &run| {
        members << 1 | u64::from(mask >> (run & CATEGORY_MASK) & 1)
    })
}

#[allow(clippy::arithmetic_side_effects)]
fn boundaries_scalar(
    chunk: &[u32; 64],
    mut edges: u64,
    min_codepoint: u32,
    boundaries: &mut [u32; 64],
) -> usize {
    let mut count = 0;
    while edges != 0 {
        let run = chunk[edges.trailing_zeros() as usize];
        boundaries[count] = max(run >> CATEGORY_BITS, min_codepoint);
        count += 1;
        // Clear the lowest set bit.
        edges &= edges - 1;
    }
    count
}

/// Membership bits of 64 runs, 4 per step: shift each run's category bit of `mask` to the
/// position of its lane and add the lanes up.
#[cfg(target_arch = "aarch64")]
#[target_feature(enable = "neon")]
#[allow(
    clippy::arithmetic_side_effects,
    clippy::cast_sign_loss,
    clippy::cast_possible_wrap
)]
unsafe fn members_neon(chunk: &[u32; 64], mask: u32) -> u64 {
    use std::arch::aarch64::{
        vaddvq_u32, vandq_u32, vdupq_n_u32, vld1q_u32, vreinterpretq_s32_u32, vshlq_u32, vsubq_s32,
    };
    const LANES: [u32; 4] = [0, 1, 2, 3];
    const LANE_BITS: [u32; 4] = [1, 2, 4, 8];
    let mask = vdupq_n_u32(mask);
    let category_mask = vdupq_n_u32(CATEGORY_MASK);
    // SAFETY: both arrays hold 4 `u32`, exactly one 128-bit load each.
    let (lanes, lane_bits) = unsafe { (vld1q_u32(LANES.as_ptr()), vld1q_u32(LANE_BITS.as_ptr())) };
    let mut members = 0u64;
    for (idx, quad) in chunk.as_chunks::<4>().0.iter().enumerate() {
        // SAFETY: `quad` holds 4 `u32`, exactly one 128-bit load.
        let runs = unsafe { vld1q_u32(quad.as_ptr()) };
        let categories = vandq_u32(runs, category_mask);
        // A negative count shifts right: bit `category` of `mask` lands on bit `lane`.
        let shifts = vsubq_s32(
            vreinterpretq_s32_u32(lanes),
            vreinterpretq_s32_u32(categories),
        );
        let bits = vandq_u32(vshlq_u32(mask, shifts), lane_bits);
        members |= u64::from(vaddvq_u32(bits)) << (idx * 4);
    }
    members
}

/// For every 4-bit mask, the byte indices of its set 32-bit lanes in ascending order.
#[cfg(target_arch = "aarch64")]
#[allow(clippy::arithmetic_side_effects, clippy::cast_possible_truncation)]
const LEFT_PACK_BYTES: [[u8; 16]; 16] = {
    let mut table = [[0u8; 16]; 16];
    let mut mask = 0;
    while mask < 16 {
        let mut count = 0;
        let mut lane = 0;
        while lane < 4 {
            if mask >> lane & 1 == 1 {
                let mut byte = 0;
                while byte < 4 {
                    table[mask][count * 4 + byte] = (lane * 4 + byte) as u8;
                    byte += 1;
                }
                count += 1;
            }
            lane += 1;
        }
        mask += 1;
    }
    table
};

/// Start codepoints of the runs selected by `edges`, clamped to `min_codepoint` and packed
/// to the front of `boundaries`, 4 runs per step. Returns how many were written.
#[cfg(target_arch = "aarch64")]
#[target_feature(enable = "neon")]
#[allow(clippy::arithmetic_side_effects, clippy::cast_possible_truncation)]
unsafe fn boundaries_neon(
    chunk: &[u32; 64],
    edges: u64,
    min_codepoint: u32,
    boundaries: &mut [u32; 64],
) -> usize {
    use std::arch::aarch64::{
        vdupq_n_u32, vld1q_u32, vld1q_u8, vmaxq_u32, vqtbl1q_u8, vreinterpretq_u32_u8,
        vreinterpretq_u8_u32, vshrq_n_u32, vst1q_u32,
    };
    let min_codepoint = vdupq_n_u32(min_codepoint);
    let mut count = 0;
    for (idx, quad) in chunk.as_chunks::<4>().0.iter().enumerate() {
        let selected = (edges >> (idx * 4)) as usize & 0xF;
        // SAFETY: `quad` holds 4 `u32`, exactly one 128-bit load.
        let runs = unsafe { vld1q_u32(quad.as_ptr()) };
        let starts = vmaxq_u32(vshrq_n_u32::<5>(runs), min_codepoint);
        // SAFETY: every `LEFT_PACK_BYTES` entry is 16 bytes, the size of this load.
        let order = unsafe { vld1q_u8(LEFT_PACK_BYTES[selected].as_ptr()) };
        let packed = vreinterpretq_u32_u8(vqtbl1q_u8(vreinterpretq_u8_u32(starts), order));
        // SAFETY: at most `idx * 4` values are written so far, so these 4 end within the
        // 64 of `boundaries`.
        unsafe { vst1q_u32(boundaries.as_mut_ptr().add(count), packed) };
        count += selected.count_ones() as usize;
    }
    count
}

/// Membership bits of 64 runs, 8 per instruction: shift each run's category bit of `mask`
/// into the sign position and collect the signs.
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
#[allow(
    clippy::arithmetic_side_effects,
    clippy::cast_sign_loss,
    clippy::cast_possible_wrap
)]
unsafe fn members_avx2(chunk: &[u32; 64], mask: u32) -> u64 {
    use std::arch::x86_64::{
        _mm256_andnot_si256, _mm256_castsi256_ps, _mm256_loadu_si256, _mm256_movemask_ps,
        _mm256_set1_epi32, _mm256_sllv_epi32,
    };
    let mask = _mm256_set1_epi32(mask as i32);
    let category_mask = _mm256_set1_epi32(CATEGORY_MASK as i32);
    let mut members = 0u64;
    for (idx, lanes) in chunk.as_chunks::<8>().0.iter().enumerate() {
        // SAFETY: `lanes` holds 8 `u32`, exactly one unaligned 256-bit load.
        let runs = unsafe { _mm256_loadu_si256(lanes.as_ptr().cast()) };
        // `!run & 31` is `31 - category`.
        let signs = _mm256_sllv_epi32(mask, _mm256_andnot_si256(runs, category_mask));
        let bits = _mm256_movemask_ps(_mm256_castsi256_ps(signs)) as u32;
        members |= u64::from(bits) << (idx * 8);
    }
    members
}

/// Chunks with at least this many edges are compacted with SIMD instead of bit by bit.
const COMPACT_EDGES: u32 = 16;

/// For every 8-bit mask, the indices of its set bits in ascending order.
#[cfg(target_arch = "x86_64")]
#[allow(clippy::arithmetic_side_effects, clippy::cast_possible_truncation)]
const LEFT_PACK: [[u8; 8]; 256] = {
    let mut table = [[0u8; 8]; 256];
    let mut mask = 0;
    while mask < 256 {
        let mut count = 0;
        let mut bit = 0;
        while bit < 8 {
            if mask >> bit & 1 == 1 {
                table[mask][count] = bit as u8;
                count += 1;
            }
            bit += 1;
        }
        mask += 1;
    }
    table
};

/// Start codepoints of the runs selected by `edges`, clamped to `min_codepoint` and packed
/// to the front of `boundaries`, 8 runs per step. Returns how many were written.
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2,popcnt")]
#[allow(
    clippy::arithmetic_side_effects,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap
)]
unsafe fn boundaries_avx2(
    chunk: &[u32; 64],
    edges: u64,
    min_codepoint: u32,
    boundaries: &mut [u32; 64],
) -> usize {
    use std::arch::x86_64::{
        _mm256_cvtepu8_epi32, _mm256_loadu_si256, _mm256_max_epu32, _mm256_permutevar8x32_epi32,
        _mm256_set1_epi32, _mm256_srli_epi32, _mm256_storeu_si256, _mm_loadl_epi64,
    };
    let min_codepoint = _mm256_set1_epi32(min_codepoint as i32);
    let mut count = 0;
    for (idx, lanes) in chunk.as_chunks::<8>().0.iter().enumerate() {
        let selected = (edges >> (idx * 8)) as u8;
        // SAFETY: `lanes` holds 8 `u32`, exactly one unaligned 256-bit load.
        let runs = unsafe { _mm256_loadu_si256(lanes.as_ptr().cast()) };
        let starts = _mm256_max_epu32(_mm256_srli_epi32::<5>(runs), min_codepoint);
        // SAFETY: every `LEFT_PACK` entry is 8 bytes, the size of this load.
        let order = unsafe { _mm_loadl_epi64(LEFT_PACK[selected as usize].as_ptr().cast()) };
        let packed = _mm256_permutevar8x32_epi32(starts, _mm256_cvtepu8_epi32(order));
        // SAFETY: at most `idx * 8` values are written so far, so these 8 end within the
        // 64 of `boundaries`.
        unsafe { _mm256_storeu_si256(boundaries.as_mut_ptr().add(count).cast(), packed) };
        count += selected.count_ones() as usize;
    }
    count
}

/// Codepoints below this bound fit in a 256-bit mask (`[u64; 4]`).
const ASCII_BOUND: u32 = 256;

pub(crate) type AsciiMask = [u64; 4];

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
const ASCII_MASKS: [[AsciiMask; 30]; 12] = [
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
    build_ascii_masks(crate::tables::v18_0_0::BY_NAME),
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

pub(crate) fn extract_intervals(bits: &AsciiMask) -> Vec<Interval> {
    extract_intervals_with_capacity(bits, 0)
}

/// Extract sorted, coalesced intervals from a 256-bit membership mask, a whole run of set
/// bits at a time (`trailing_zeros` to the run start, `trailing_ones` for its length).
#[allow(clippy::arithmetic_side_effects)]
pub(crate) fn extract_intervals_with_capacity(bits: &AsciiMask, capacity: usize) -> Vec<Interval> {
    let mut result = Vec::with_capacity(capacity);
    let mut current: Option<Interval> = None;
    let mut word: u32 = 0;
    while word < 4 {
        let mut remaining = bits[word as usize];
        let base = word * 64;
        while remaining != 0 {
            let start = remaining.trailing_zeros();
            let run = (remaining >> start).trailing_ones();
            let lo = base + start;
            let hi = lo + run - 1;
            match current {
                Some((_, ref mut chi)) if *chi + 1 == lo => *chi = hi,
                Some(interval) => {
                    result.push(interval);
                    current = Some((lo, hi));
                }
                None => current = Some((lo, hi)),
            }
            // Clear bits `[0, start + run)`; everything below `start` is already zero.
            let cleared = start + run;
            remaining = if cleared == 64 {
                0
            } else {
                remaining & !((1u64 << cleared) - 1)
            };
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
    // empty set on the slow path (max >= 256): `collect_in_range` zero branch + `intervals = include_intervals`
    #[test_case(Some(UnicodeCategorySet::new()), UnicodeCategorySet::new(), "", "", 0, 0x1000; "empty set ranged slow")]
    #[test_case(Some(UnicodeCategorySet::new()), UnicodeCategorySet::new(), "abc", "", 0, 0x1000; "empty set ranged with chars slow")]
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

    // Tables on both sides of `BINARY_SEARCH_THRESHOLD`, every range around their bounds.
    #[test]
    fn test_clamped() {
        for len in [0_u32, 1, 2, 31, 32, 33, 70] {
            // Intervals `(10 * i + 3, 10 * i + 6)`: gaps on both sides of each one.
            let table: Vec<Interval> = (0..len).map(|i| (10 * i + 3, 10 * i + 6)).collect();
            let bound = 10 * len + 10;
            for min_codepoint in 0..bound {
                for max_codepoint in min_codepoint..bound {
                    let expected: Vec<Interval> = table
                        .iter()
                        .filter(|&&(left, right)| left <= max_codepoint && right >= min_codepoint)
                        .map(|&(left, right)| (left.max(min_codepoint), right.min(max_codepoint)))
                        .collect();
                    assert_eq!(
                        clamped(&table, min_codepoint, max_codepoint),
                        expected,
                        "len={len} range=({min_codepoint},{max_codepoint})"
                    );
                    let mut extended = Vec::new();
                    extend_clamped(&mut extended, &table, min_codepoint, max_codepoint);
                    assert_eq!(
                        extended, expected,
                        "len={len} range=({min_codepoint},{max_codepoint})"
                    );
                }
            }
        }
    }

    // Every run is exactly one category interval, and every category interval is a run.
    #[test]
    fn test_runs() {
        for version in UnicodeVersion::ALL {
            let table = version.table();
            let runs = RUNS[version as usize];
            assert_eq!(
                runs.len(),
                table.iter().map(|slice| slice.len()).sum::<usize>()
            );
            for (idx, &run) in runs.iter().enumerate() {
                let left = run >> CATEGORY_BITS;
                let right = runs
                    .get(idx + 1)
                    .map_or(MAX_CODEPOINT, |next| (next >> CATEGORY_BITS) - 1);
                let category = (run & CATEGORY_MASK) as usize;
                assert!(
                    table[category].binary_search(&(left, right)).is_ok(),
                    "version={version} run={idx} interval=({left},{right}) category={category}"
                );
            }
        }
    }

    // `build_runs` runs at compile time for `RUNS`, so exercise it at runtime too.
    #[test]
    fn test_build_runs() {
        const COUNT: usize = count_runs(crate::tables::v15_0_0::BY_NAME);
        let table = UnicodeVersion::V15_0_0.table();
        assert_eq!(count_runs(table), COUNT);
        assert_eq!(
            build_runs::<COUNT>(table),
            RUNS[UnicodeVersion::V15_0_0 as usize]
        );
    }

    #[test]
    #[should_panic(expected = "categories must partition the codepoint space")]
    fn test_build_runs_gap() {
        let _ = build_runs::<1>(&[&[(1, MAX_CODEPOINT)]]);
    }

    #[test]
    #[should_panic(expected = "categories must partition the codepoint space")]
    fn test_build_runs_short() {
        let _ = build_runs::<1>(&[&[(0, 5)]]);
    }

    fn supported() -> impl Iterator<Item = Simd> {
        Simd::ALL.iter().copied().filter(|simd| simd.is_supported())
    }

    #[test]
    fn test_simd_best_is_supported() {
        assert!(Simd::best().is_supported());
        assert!(Simd::Scalar.is_supported());
    }

    #[test]
    fn test_boundaries() {
        let mut state = 0x1234_5678_9abc_def0u64;
        let mut next = || {
            state = state
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            (state >> 33) as u32
        };
        let runs = RUNS[UnicodeVersion::V15_0_0 as usize];
        for round in 0..2_000 {
            let offset = next() as usize % (runs.len() - 64);
            let chunk: &[u32; 64] = runs[offset..offset + 64].try_into().expect("64 runs");
            let random = u64::from(next()) | u64::from(next()) << 31 | u64::from(next()) << 62;
            let edges = match round % 4 {
                0 => u64::MAX,
                1 => 0,
                2 => random & random.rotate_left(17),
                _ => random,
            };
            let min_codepoint = (round % 2) * (chunk[5] >> CATEGORY_BITS);
            let expected: Vec<u32> = (0..64)
                .filter(|idx| edges >> idx & 1 == 1)
                .map(|idx| (chunk[idx] >> CATEGORY_BITS).max(min_codepoint))
                .collect();
            for simd in supported() {
                let mut boundaries = [0; 64];
                // SAFETY: only supported implementations are iterated.
                let count =
                    unsafe { simd.boundaries(chunk, edges, min_codepoint, &mut boundaries) };
                assert_eq!(&boundaries[..count], expected, "{simd:?} edges={edges:#x}");
            }
        }
    }

    #[test]
    fn test_members() {
        let mut state = 0x9e37_79b9_7f4a_7c15u64;
        let mut next = || {
            state = state
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            (state >> 33) as u32
        };
        let runs = RUNS[UnicodeVersion::V15_0_0 as usize];
        for round in 0..2_000 {
            let offset = next() as usize % (runs.len() - 64);
            let chunk: &[u32; 64] = runs[offset..offset + 64].try_into().expect("64 runs");
            let mask = match round % 4 {
                0 => ALL_CATEGORIES,
                1 => 0,
                2 => 1 << (next() % 30),
                _ => next() & ALL_CATEGORIES,
            };
            let expected = chunk.iter().enumerate().fold(0u64, |members, (idx, &run)| {
                members | u64::from(mask >> (run & CATEGORY_MASK) & 1) << idx
            });
            for simd in supported() {
                // SAFETY: only supported implementations are iterated.
                let members = unsafe { simd.members(chunk, mask) };
                assert_eq!(members, expected, "{simd:?} mask={mask:#x} offset={offset}");
            }
        }
    }

    // Independent reference for `union_runs`: clamp every category on its own, then sort + merge.
    fn union_reference(
        version: UnicodeVersion,
        mask: u32,
        min_codepoint: u32,
        max_codepoint: u32,
    ) -> Vec<Interval> {
        let mut expected = Vec::new();
        for (category, slice) in version.table().iter().enumerate() {
            if mask >> category & 1 == 1 {
                extend_clamped(&mut expected, slice, min_codepoint, max_codepoint);
            }
        }
        if !expected.is_empty() {
            intervals::merge(&mut expected);
        }
        expected
    }

    #[test]
    fn test_union_runs() {
        let version = UnicodeVersion::V15_0_0;
        let runs = RUNS[version as usize];
        // Every implementation this CPU supports.
        let check = |mask: u32, min_codepoint: u32, max_codepoint: u32| {
            let expected = union_reference(version, mask, min_codepoint, max_codepoint);
            for simd in supported() {
                assert_eq!(
                    union_runs(runs, mask, min_codepoint, max_codepoint, simd),
                    expected,
                    "mask={mask:#x} range=({min_codepoint},{max_codepoint}) simd={simd:?}"
                );
            }
        };
        let mut state = 0x1234_5678_9abc_def0u64;
        let mut next = || {
            state = state
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            (state >> 33) as u32
        };
        let masks = [
            1,
            ALL_CATEGORIES,
            ALL_CATEGORIES ^ 1,
            ALL_CATEGORIES ^ (1 << 29),
            0x1555_5555,
            0x2aaa_aaaa,
        ];
        // Ranges covering an exact number of runs around the 64-run chunk size, starting on a
        // run boundary and in the middle of a run.
        for mask in masks {
            for first in [0, 1, 63, 64, 100, runs.len() - 130] {
                for count in [1, 2, 63, 64, 65, 127, 128, 129] {
                    let min_codepoint = runs[first] >> CATEGORY_BITS;
                    let max_codepoint = (runs[first + count] >> CATEGORY_BITS) - 1;
                    check(mask, min_codepoint, max_codepoint);
                    check(mask, min_codepoint + 1, max_codepoint + 1);
                    check(mask, min_codepoint, min_codepoint);
                }
            }
            check(mask, 0, MAX_CODEPOINT);
            check(mask, MAX_CODEPOINT, MAX_CODEPOINT);
        }
        for _ in 0..5000 {
            let mask = next() & next() & ALL_CATEGORIES;
            let (a, b) = (next() % (MAX_CODEPOINT + 1), next() % (MAX_CODEPOINT + 1));
            check(mask, a.min(b), a.max(b));
            check(next() & ALL_CATEGORIES, a.min(b), a.max(b));
            check(mask, 0, MAX_CODEPOINT);
        }
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

    // Independent per-bit reference for `extract_intervals`.
    #[allow(clippy::arithmetic_side_effects)]
    fn extract_reference(bits: &AsciiMask) -> Vec<Interval> {
        let mut out: Vec<Interval> = Vec::new();
        for cp in 0..ASCII_BOUND {
            if bits[(cp / 64) as usize] & (1u64 << (cp % 64)) != 0 {
                match out.last_mut() {
                    Some((_, hi)) if *hi + 1 == cp => *hi = cp,
                    _ => out.push((cp, cp)),
                }
            }
        }
        out
    }

    #[test]
    fn test_extract_intervals() {
        // Edge cases: empty, word edges, runs crossing word boundaries, full mask.
        let cases: [AsciiMask; 9] = [
            [0, 0, 0, 0],
            [1, 0, 0, 0],
            [1 << 63, 0, 0, 0],
            [1 << 63, 1, 0, 0],  // run crossing the 63/64 boundary
            [0, 0, 0, 1 << 63],  // last bit
            [u64::MAX, 0, 0, 0], // full first word
            [u64::MAX, u64::MAX, u64::MAX, u64::MAX], // all 256 bits
            [0xAAAA_AAAA_AAAA_AAAA, 0x5555_5555_5555_5555, 0, 0], // alternating, joins at edge
            [0, u64::MAX, u64::MAX, 0], // run spanning two interior words
        ];
        for bits in cases {
            assert_eq!(
                extract_intervals(&bits),
                extract_reference(&bits),
                "{bits:?}"
            );
        }
        // Deterministic fuzz over dense and sparse masks.
        let mut state = 0x1234_5678_9abc_def0u64;
        let mut next = || {
            state = state
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            state
        };
        for _ in 0..5000 {
            let dense = [next(), next(), next(), next()];
            let sparse = [
                next() & next(),
                next() & next(),
                next() & next(),
                next() & next(),
            ];
            assert_eq!(
                extract_intervals(&dense),
                extract_reference(&dense),
                "{dense:?}"
            );
            assert_eq!(
                extract_intervals(&sparse),
                extract_reference(&sparse),
                "{sparse:?}"
            );
        }
    }
}
