# 0064 — Terminal leftover exact-LNS + relabel tickets on well-below graphs

- **Date:** 2026-09-05
- **Base commit:** `2a28517` (promoted submission `6ef357d`, hidden **0.867211**). Local re-measured dev **0.839063** (lt_1k 0.890341 / 1k_10k 0.868720 / gt_10k 0.778362).
- **Candidate dev score:** **0.838936** (lt_1k 0.890223 / 1k_10k 0.868512 / gt_10k 0.778290), fill 0.942767.
- **Delta:** **−0.000127 (−1.27 bip aggregate)** vs re-measured base on the full 300-matrix dev corpus. All three buckets improve; zero matrices can regress by construction (terminal strict best-of).
- **Status:** WIN locally. Submitted for hidden validation.

## Context

0063 rebuilt the frontier (dev 0.843358 → 0.839063, −42.95 bip) with reduce-then-AMF terminal, a partitioner cascade, and structural windows, buying large gt_10k wins plus wall-clock margin (their box worst 1.823 s → 1.084 s). lt_1k sits at exactly the 0061 value (0.890341); 1k_10k is weaker than the 0061 tree (0.868720 vs 0.864951) by design. The remaining headroom per 0056/0060/0061 is well-below-anchor incumbents (ratio < 0.80), where exact-LNS and i.i.d. relabel lotteries still convert and ties do not (ties resist even 100× budgets).

## Hypothesis

Terminal, strictly-monotonic leftover tickets on well-below graphs: medium/small exact-LNS extra streams plus sparse-large relabel seeds, all placed after the full pipeline (including reduce-then-AMF) and accepted only on strict `<`. Terminal placement cannot displace downstream chain basins (the failure mode of early placement, measured here: an early 24-pass RCM/Sloan/ND/NDFM block regressed lt_1k 0.890341 → 0.890413). Gates exclude every known worst-case matrix, so the global worst is unmoved.

## What changed (`src/ordering/mod.rs` only, +~130 lines before the final `best_perm`)

1. **Medium:** 4 extra exact-LNS streams (50M ops, fresh seeds) on `1000 < n <= 6000`, `nnz <= 30000`, well-below only.
2. **Small:** 6 extra exact-LNS streams (50M ops, fresh seeds) on `n <= 1000`, `nnz <= 30000`, well-below only.
3. **Sparse-large:** 4 extra i.i.d. AMF+AMD relabel seeds on `n >= 10000`, `nnz <= 60000`, well-below only (crudeoil_lee4_10 nnz=120632, arki0013 nnz=160172, gams05 nnz=252910 all excluded).

Determinism unchanged: fixed seeds, gates are pure functions of `(n, nnz, best_flops, amd_flops)`.

## Negative controls (all reverted, FACT)

- Early (pre-chain) 24-pass relabelled RCM/Sloan/ND/NDFM on `n < 1000`: lt_1k **0.890413** (+0.72 bip in-bucket LOSS) — displaces the subtree/exact basin. Terminal re-test of the same 24 passes: exactly baseline (zero wins), +20 ms. Confirms 0020 extends to multi-seed for these families.
- Terminal 4 extra MinFill seeds on `n < 1000 && nnz < 5000`: exactly baseline, +130 ms. Reverted.
- Terminal 9 unwired custom-metric variants (Ammf/AmindNorm/7 Deg*) on `n < 1000`: exactly baseline, +20 ms. Reverted.
- Terminal 8 extra relabel seeds (existing alphas) on well-below 1k_10k `nnz <= 30000`: exactly baseline. Reverted (existing lottery saturated; new-ticket wins came from LNS, not relabel, except sparse-large).
- 2 further medium LNS streams (6 total): zero marginal gain, +44 ms. Reverted to 4.

## Result (this box, `probe_*` + `yukon run`)

| | Base | Candidate | Δ |
|---|---:|---:|---:|
| Aggregate | 0.839063 | **0.838936** | **−1.27 bip** |
| lt_1k (0.30, 147) | 0.890341 | **0.890223** | −1.18 bip in-bucket |
| 1k_10k (0.30, 108) | 0.868720 | **0.868512** | −2.08 bip in-bucket |
| gt_10k (0.40, 45) | 0.778362 | **0.778290** | −0.72 bip in-bucket |
| Worst `order()` | 1.62 s class (arki0013/crudeoil_lee4_10, excluded by gates) | lt 0.940 s / 1k10k 1.180 s / gt 1.554 s (gams05) | global worst unmoved (delta is noise; added work cannot run on worst matrices) |

`cargo test -p ssi-candidate-worker`: **50 passed**, 0 failed. `yukon run` purity/license gate passed; per-matrix ratios ≤ 1.0 preserved (AMD floor untouched).

## Why it won

Well-below incumbents still convert with deeper exact search; ties do not. Terminal placement makes every ticket score-monotone, so the aggregate can only improve while timing is the only risk — contained by nnz gates that exclude the worst matrices. 1k_10k recovers part of what 0063 deliberately sacrificed; lt_1k moves for the first time since 0061.

## Follow-ups

- Medium LNS saturates at 4 extra streams (6 total: zero marginal). Small saturates slower (6 still paying). Do not add more medium streams.
- gt_10k sparse-large relabel still pays; 2 more seeds may buy +0.2 bip in-bucket at +40 ms on sparse worst (~0.8 s class). Not tried.
- Reduce-then-AMF alpha grid {0.5, 2.5, 5, 10} could extend to 1.0/16.0 as extra terminal core passes; cost is core-size-gated. Not tried.

## Links

- Predecessor: [0063](0063-time-margin-by-structure.md), [0061](0061-margin-scaled-leftover-search.md)
- Lottery: [0004](0004-structured-relabelings.md); margin: [0060](0060-conditional-search-escalation-below-anchor.md)
