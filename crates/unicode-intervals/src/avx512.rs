//! AVX-512 steps of the multi-category union.
use crate::query::{CATEGORY_BITS, CATEGORY_MASK};

/// Membership bits of 64 runs, 16 per step: shift each run's category bit of `mask` down
/// and test it.
#[target_feature(enable = "avx512f,popcnt")]
#[allow(
    clippy::arithmetic_side_effects,
    clippy::cast_sign_loss,
    clippy::cast_possible_wrap
)]
pub(crate) unsafe fn members(chunk: &[u32; 64], mask: u32) -> u64 {
    use std::arch::x86_64::{
        _mm512_and_si512, _mm512_loadu_si512, _mm512_set1_epi32, _mm512_srlv_epi32,
        _mm512_test_epi32_mask,
    };
    let mask = _mm512_set1_epi32(mask as i32);
    let category_mask = _mm512_set1_epi32(CATEGORY_MASK as i32);
    let one = _mm512_set1_epi32(1);
    let mut members = 0u64;
    for (idx, lanes) in chunk.as_chunks::<16>().0.iter().enumerate() {
        // SAFETY: `lanes` holds 16 `u32`, exactly one unaligned 512-bit load.
        let runs = unsafe { _mm512_loadu_si512(lanes.as_ptr().cast()) };
        let bits = _mm512_srlv_epi32(mask, _mm512_and_si512(runs, category_mask));
        members |= u64::from(_mm512_test_epi32_mask(bits, one)) << (idx * 16);
    }
    members
}

/// Start codepoints of the runs selected by `edges`, clamped to `min_codepoint` and packed
/// to the front of `boundaries` with `vpcompressd`, 16 runs per step. Returns how many were
/// written.
#[target_feature(enable = "avx512f,popcnt")]
#[allow(
    clippy::arithmetic_side_effects,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap
)]
pub(crate) unsafe fn boundaries(
    chunk: &[u32; 64],
    edges: u64,
    min_codepoint: u32,
    boundaries: &mut [u32; 64],
) -> usize {
    use std::arch::x86_64::{
        _mm512_loadu_si512, _mm512_maskz_compress_epi32, _mm512_max_epu32, _mm512_set1_epi32,
        _mm512_srli_epi32, _mm512_storeu_si512,
    };
    let min_codepoint = _mm512_set1_epi32(min_codepoint as i32);
    let mut count = 0;
    for (idx, lanes) in chunk.as_chunks::<16>().0.iter().enumerate() {
        let selected = (edges >> (idx * 16)) as u16;
        // SAFETY: `lanes` holds 16 `u32`, exactly one unaligned 512-bit load.
        let runs = unsafe { _mm512_loadu_si512(lanes.as_ptr().cast()) };
        let starts = _mm512_max_epu32(_mm512_srli_epi32::<CATEGORY_BITS>(runs), min_codepoint);
        let packed = _mm512_maskz_compress_epi32(selected, starts);
        // SAFETY: at most `idx * 16` values are written so far, so these 16 end within the
        // 64 of `boundaries`.
        unsafe { _mm512_storeu_si512(boundaries.as_mut_ptr().add(count).cast(), packed) };
        count += selected.count_ones() as usize;
    }
    count
}
