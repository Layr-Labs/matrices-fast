//! Bitset union kernel: OR-assign + count-newly-set-bits, with a portable
//! scalar body and an x86_64 hardware-POPCNT path selected at runtime.
//!
//! Sandbox measurements (`artifacts/assessment-next/bit-kernels.log`):
//! POPCNT measured 1.79-1.87x the scalar body at 16..188 words (1.14-1.32x
//! below that). An AVX2 OR-merge variant measured slower than POPCNT at
//! every tested size, and slower than scalar at length 1 — so AVX2 stays
//! test-only (below) and is not wired into any production call site.
//!
//! Legality: `ssi-purity::scan_source` forbids only `extern`/`no_mangle`/
//! `proc_macro`/`#[link]`/the `asm!` family/`include!`/`#[path]`/build.rs;
//! it does not scan for `unsafe`, `std::arch`, `target_feature`, or
//! `is_x86_feature_detected!`. That is evidence this file (checked against
//! its exact tokens) is not blocked by that gate, not a guarantee beyond it.

/// OR-assign `add` into `acc` (`acc[i] |= add[i]`) and return the number of
/// bits newly set (`popcount(add[i] & !acc[i])` summed over `i`, evaluated
/// before the assignment), in one fused pass. `acc` and `add` must have
/// equal length (`debug_assert_eq!`, matching this crate's convention of
/// trusting release-mode callers).
///
/// Dispatches to the POPCNT path only when the CPU supports it and the
/// slice has at least 8 words; below that the runtime-check overhead is
/// not worth it (measured gains start at 16 words), so the portable
/// scalar fallback runs directly.
pub(crate) fn fused_or_assign_count_new(acc: &mut [u64], add: &[u64]) -> u64 {
    debug_assert_eq!(acc.len(), add.len());
    #[cfg(target_arch = "x86_64")]
    if acc.len() >= 8 && is_x86_feature_detected!("popcnt") {
        // SAFETY: feature confirmed present immediately above.
        return unsafe { fused_or_assign_count_new_popcnt(acc, add) };
    }
    fused_or_assign_count_new_scalar(acc, add)
}

/// Portable fallback; identical body to [`fused_or_assign_count_new_popcnt`]
/// so the two agree by construction, not merely by test.
pub(crate) fn fused_or_assign_count_new_scalar(acc: &mut [u64], add: &[u64]) -> u64 {
    debug_assert_eq!(acc.len(), add.len());
    let mut new_bits = 0u64;
    for i in 0..acc.len() {
        let new_here = add[i] & !acc[i];
        new_bits += new_here.count_ones() as u64;
        acc[i] |= add[i];
    }
    new_bits
}

/// # Safety
/// Caller must have confirmed `is_x86_feature_detected!("popcnt")`. Body is
/// ordinary safe Rust, identical to [`fused_or_assign_count_new_scalar`].
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "popcnt")]
unsafe fn fused_or_assign_count_new_popcnt(acc: &mut [u64], add: &[u64]) -> u64 {
    debug_assert_eq!(acc.len(), add.len());
    let mut new_bits = 0u64;
    for i in 0..acc.len() {
        let new_here = add[i] & !acc[i];
        new_bits += new_here.count_ones() as u64;
        acc[i] |= add[i];
    }
    new_bits
}

/// Test-only: exact fused OR+popcount, `sum_i popcount(a[i] | b[i])`. Not a
/// production call site; kept for the differential/benchmark tests below,
/// which is also where the AVX2 comparison point lives (see module doc:
/// measured slower than POPCNT everywhere, so it is not promoted further).
#[cfg(test)]
pub(crate) fn fused_or_popcount(a: &[u64], b: &[u64]) -> u64 {
    debug_assert_eq!(a.len(), b.len());
    #[cfg(target_arch = "x86_64")]
    {
        if is_x86_feature_detected!("avx2") && is_x86_feature_detected!("popcnt") {
            // SAFETY: both features confirmed present immediately above.
            return unsafe { fused_or_popcount_avx2(a, b) };
        }
        if is_x86_feature_detected!("popcnt") {
            // SAFETY: feature confirmed present immediately above.
            return unsafe { fused_or_popcount_popcnt(a, b) };
        }
    }
    fused_or_popcount_scalar(a, b)
}

#[cfg(test)]
pub(crate) fn fused_or_popcount_scalar(a: &[u64], b: &[u64]) -> u64 {
    debug_assert_eq!(a.len(), b.len());
    let mut total: u64 = 0;
    for i in 0..a.len() {
        total += (a[i] | b[i]).count_ones() as u64;
    }
    total
}

/// # Safety
/// Caller must have confirmed `is_x86_feature_detected!("popcnt")`.
#[cfg(all(test, target_arch = "x86_64"))]
#[target_feature(enable = "popcnt")]
unsafe fn fused_or_popcount_popcnt(a: &[u64], b: &[u64]) -> u64 {
    debug_assert_eq!(a.len(), b.len());
    let mut total: u64 = 0;
    for i in 0..a.len() {
        total += (a[i] | b[i]).count_ones() as u64;
    }
    total
}

/// # Safety
/// Caller must have confirmed `is_x86_feature_detected!("avx2")` and
/// `is_x86_feature_detected!("popcnt")`. Vectorizes only the OR-merge (4
/// words/256 bits per `_mm256_or_si256`, safe for any alignment via the
/// `u` load/store variants); AVX2 has no 256-bit popcount instruction, so
/// counting still goes through scalar `count_ones` per extracted lane —
/// which is why this measured no better than the POPCNT-only path.
#[cfg(all(test, target_arch = "x86_64"))]
#[target_feature(enable = "avx2,popcnt")]
unsafe fn fused_or_popcount_avx2(a: &[u64], b: &[u64]) -> u64 {
    use std::arch::x86_64::{_mm256_loadu_si256, _mm256_or_si256, _mm256_storeu_si256};
    debug_assert_eq!(a.len(), b.len());
    let len = a.len();
    let chunks = len / 4;
    let mut total: u64 = 0;
    for c in 0..chunks {
        let base = c * 4;
        // Safe: `base + 4 <= len` by construction of `chunks = len / 4`.
        let a_chunk = &a[base..base + 4];
        let b_chunk = &b[base..base + 4];
        // SAFETY: loadu/storeu accept any alignment; each chunk addresses
        // exactly 32 valid bytes.
        let (va, vb) = unsafe {
            (
                _mm256_loadu_si256(a_chunk.as_ptr().cast()),
                _mm256_loadu_si256(b_chunk.as_ptr().cast()),
            )
        };
        let vor = _mm256_or_si256(va, vb); // safe: pure register op, no memory access
        let mut merged = [0u64; 4];
        unsafe { _mm256_storeu_si256(merged.as_mut_ptr().cast(), vor) };
        for w in merged {
            total += w.count_ones() as u64;
        }
    }
    for i in (chunks * 4)..len {
        total += (a[i] | b[i]).count_ones() as u64;
    }
    total
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Minimal deterministic PRNG so these tests need no external `rand`
    /// dependency (this file has none).
    struct Xorshift64(u64);
    impl Xorshift64 {
        fn next(&mut self) -> u64 {
            let mut x = self.0;
            x ^= x << 13;
            x ^= x >> 7;
            x ^= x << 17;
            self.0 = x;
            x
        }
    }

    #[derive(Clone, Copy)]
    enum Density {
        Empty,
        Sparse,
        Medium,
        Dense,
    }

    fn gen_words(rng: &mut Xorshift64, len: usize, density: Density) -> Vec<u64> {
        (0..len)
            .map(|_| match density {
                Density::Empty => 0,
                Density::Sparse => rng.next() & rng.next() & rng.next(), // biased toward zero bits
                Density::Medium => rng.next(),
                Density::Dense => rng.next() | rng.next() | rng.next(), // biased toward set bits
            })
            .collect()
    }

    fn reference_or_popcount(a: &[u64], b: &[u64]) -> u64 {
        a.iter().zip(b).map(|(x, y)| (x | y).count_ones() as u64).sum()
    }

    fn reference_new_bits(acc: &[u64], add: &[u64]) -> u64 {
        acc.iter()
            .zip(add)
            .map(|(a, b)| (b & !a).count_ones() as u64)
            .sum()
    }

    const DENSITIES: [Density; 4] = [
        Density::Empty,
        Density::Sparse,
        Density::Medium,
        Density::Dense,
    ];

    /// Differential: scalar, dispatch, and (when the running CPU actually
    /// supports the feature) POPCNT/AVX2 must all return the same value for
    /// every length 0..=188 words across dense/sparse/medium/empty
    /// patterns. Accelerated bodies are only invoked after confirming their
    /// feature is present.
    #[test]
    fn fused_or_popcount_matches_reference_across_lengths_and_density() {
        let mut rng = Xorshift64(0x9E37_79B9_7F4A_7C15);
        for len in 0..=188usize {
            for density in DENSITIES {
                let a = gen_words(&mut rng, len, density);
                let b = gen_words(&mut rng, len, density);
                let expected = reference_or_popcount(&a, &b);

                assert_eq!(fused_or_popcount_scalar(&a, &b), expected, "scalar len={len}");
                assert_eq!(fused_or_popcount(&a, &b), expected, "dispatch len={len}");

                #[cfg(target_arch = "x86_64")]
                {
                    if is_x86_feature_detected!("popcnt") {
                        assert_eq!(
                            unsafe { fused_or_popcount_popcnt(&a, &b) },
                            expected,
                            "popcnt len={len}"
                        );
                    }
                    if is_x86_feature_detected!("avx2") && is_x86_feature_detected!("popcnt") {
                        assert_eq!(
                            unsafe { fused_or_popcount_avx2(&a, &b) },
                            expected,
                            "avx2 len={len}"
                        );
                    }
                }
            }
        }
    }

    /// Exercises `_mm256_loadu_si256` at every residue relative to a
    /// 32-byte boundary via offsets 0..8 words into a shared buffer.
    #[test]
    fn fused_or_popcount_handles_unaligned_subslices() {
        let mut rng = Xorshift64(0xD1B5_4A32_D192_ED03);
        let base_a = gen_words(&mut rng, 400, Density::Medium);
        let base_b = gen_words(&mut rng, 400, Density::Medium);
        for offset in 0..8usize {
            for len in [0usize, 1, 2, 3, 4, 5, 7, 9, 16, 17, 188] {
                if offset + len > base_a.len() {
                    continue;
                }
                let a = &base_a[offset..offset + len];
                let b = &base_b[offset..offset + len];
                let expected = reference_or_popcount(a, b);
                assert_eq!(fused_or_popcount(a, b), expected, "offset={offset} len={len}");
                #[cfg(target_arch = "x86_64")]
                if is_x86_feature_detected!("avx2") && is_x86_feature_detected!("popcnt") {
                    assert_eq!(
                        unsafe { fused_or_popcount_avx2(a, b) },
                        expected,
                        "avx2 offset={offset} len={len}"
                    );
                }
            }
        }
    }

    /// Differential for the production OR-assign+count-new kernel: checks
    /// both the returned new-bit count and that the mutated accumulator
    /// equals the elementwise OR, across the same length/density sweep
    /// (spanning both sides of the `len >= 8` dispatch gate).
    #[test]
    fn fused_or_assign_count_new_matches_reference_and_mutates_correctly() {
        let mut rng = Xorshift64(0xA24B_AED4_963E_E407);
        for len in 0..=188usize {
            for density in DENSITIES {
                let add = gen_words(&mut rng, len, density);
                let acc0 = gen_words(&mut rng, len, density);
                let expected_new = reference_new_bits(&acc0, &add);
                let expected_or: Vec<u64> = acc0.iter().zip(&add).map(|(a, b)| a | b).collect();

                let mut acc_scalar = acc0.clone();
                let got_scalar = fused_or_assign_count_new_scalar(&mut acc_scalar, &add);
                assert_eq!(got_scalar, expected_new, "scalar new-bits len={len}");
                assert_eq!(acc_scalar, expected_or, "scalar OR-mutation len={len}");

                let mut acc_dispatch = acc0.clone();
                let got_dispatch = fused_or_assign_count_new(&mut acc_dispatch, &add);
                assert_eq!(got_dispatch, expected_new, "dispatch new-bits len={len}");
                assert_eq!(acc_dispatch, expected_or, "dispatch OR-mutation len={len}");

                #[cfg(target_arch = "x86_64")]
                if is_x86_feature_detected!("popcnt") {
                    let mut acc_popcnt = acc0.clone();
                    let got_popcnt =
                        unsafe { fused_or_assign_count_new_popcnt(&mut acc_popcnt, &add) };
                    assert_eq!(got_popcnt, expected_new, "popcnt new-bits len={len}");
                    assert_eq!(acc_popcnt, expected_or, "popcnt OR-mutation len={len}");
                }
            }
        }
    }

    /// Opt-in relative-timing comparison (see `probe.rs` for this crate's
    /// `#[ignore]` convention). Batches many repetitions to dilute `Instant`
    /// overhead; `black_box` prevents hoisting the invariant loop body.
    #[test]
    #[ignore]
    fn bench_fused_or_popcount_variants() {
        use std::hint::black_box;
        use std::time::Instant;

        const WORDS: usize = 1 << 16;
        const REPEATS: usize = 200;

        let mut rng = Xorshift64(0x2545_F491_4F6C_DD1D);
        let a = gen_words(&mut rng, WORDS, Density::Medium);
        let b = gen_words(&mut rng, WORDS, Density::Medium);
        let mut sink: u64 = 0;

        for _ in 0..10 {
            sink ^= fused_or_popcount_scalar(black_box(&a), black_box(&b));
        }
        let t0 = Instant::now();
        for _ in 0..REPEATS {
            sink ^= fused_or_popcount_scalar(black_box(&a), black_box(&b));
        }
        let scalar_elapsed = t0.elapsed();
        println!("scalar: {scalar_elapsed:?} for {REPEATS} reps over {WORDS} words");

        #[cfg(target_arch = "x86_64")]
        {
            if is_x86_feature_detected!("popcnt") {
                for _ in 0..10 {
                    sink ^= unsafe { fused_or_popcount_popcnt(black_box(&a), black_box(&b)) };
                }
                let t1 = Instant::now();
                for _ in 0..REPEATS {
                    sink ^= unsafe { fused_or_popcount_popcnt(black_box(&a), black_box(&b)) };
                }
                let popcnt_elapsed = t1.elapsed();
                println!(
                    "popcnt: {:?} for {REPEATS} reps; scalar/popcnt ratio = {:.3}",
                    popcnt_elapsed,
                    scalar_elapsed.as_secs_f64() / popcnt_elapsed.as_secs_f64().max(1e-12)
                );
            }
            if is_x86_feature_detected!("avx2") && is_x86_feature_detected!("popcnt") {
                for _ in 0..10 {
                    sink ^= unsafe { fused_or_popcount_avx2(black_box(&a), black_box(&b)) };
                }
                let t2 = Instant::now();
                for _ in 0..REPEATS {
                    sink ^= unsafe { fused_or_popcount_avx2(black_box(&a), black_box(&b)) };
                }
                let avx2_elapsed = t2.elapsed();
                println!(
                    "avx2: {:?} for {REPEATS} reps; scalar/avx2 ratio = {:.3}",
                    avx2_elapsed,
                    scalar_elapsed.as_secs_f64() / avx2_elapsed.as_secs_f64().max(1e-12)
                );
            }
        }
        println!("sink={sink} (kept live via black_box to prevent dead-code elimination)");
    }
}
