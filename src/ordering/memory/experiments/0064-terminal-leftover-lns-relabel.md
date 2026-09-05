# 0064 — Terminal leftover exact-LNS tickets on below-anchor small graphs (v2)

- **Date:** 2026-09-05
- **Base commit:** `2a28517` (promoted submission `6ef357d`, hidden **0.867211**). Local re-measured dev **0.839063** (lt_1k 0.890341 / 1k_10k 0.868720 / gt_10k 0.778362).
- **Candidate dev score (full `yukon run`, 300 matrices):** **0.838921** (lt_1k 0.890015 / 1k_10k 0.868574 / gt_10k 0.778362), fill 0.942641.
- **Delta:** **−0.000142 (−1.42 bip aggregate)** vs re-measured base. lt_1k −3.26 bip + 1k_10k −1.46 bip in-bucket; gt_10k unchanged. **17 better / 0 worse / 283 same** per-matrix.
- **Status:** WIN locally. Submitted for hidden validation.

## Context

0063 rebuilt the frontier (dev 0.843358 → 0.839063) with reduce-then-AMF terminal, a partitioner cascade, and structural windows, buying large gt_10k wins plus wall-clock margin. The v1 of this experiment (medium 4-stream + small 6-stream well-below LNS + sparse-large relabel on the stale `784bfe5` base, dev −1.08 bip) was submitted as `f85679d3` and **FAILED hidden validation** (Benchmark step, exit 1 after ~7 min; logs private). The additive medium/large work fought 0063's margin design on exactly the tiers that own the slow rows, repeating the documented 0060 failure mode. v2 removes all work above n = 1000 except a 2-stream medium-cheap band that excludes the slow rows by nnz.

## Hypothesis

Terminal, strictly-monotonic leftover tickets where headroom is proven and timing is bounded: small exact-LNS extra streams (below-anchor, not just well-below), a small-dense band the base gates skip, post-ticket local-search cleanup, and a minimal medium-cheap band. Terminal placement cannot displace downstream chain basins (measured: early placement regressed lt_1k). Gates keyed on `(n, nnz, best_flops, amd_flops)`; ties excluded per 0056.

## What changed (`src/ordering/mod.rs` only, terminal, after reduce-then-AMF)

1. **Small (14 streams):** `n <= 1000`, `nnz <= 30000`, below-anchor; fourteen `rgreedy::search` 50M-op streams, fresh fixed seeds, sequential best-of.
2. **Small-dense (4 streams):** `n <= 1000`, `30000 < nnz <= 90000`, below-anchor; four 50M-op streams (covers qap-class dense small the base gates skip).
3. **Post-ticket cleanup:** pair descent (4 sweeps) + simplicial promotion on the same small gate (LNS moves expose fresh inversions).
4. **Medium-cheap (2 streams):** `1000 < n <= 6000`, `nnz <= 15000`, below-anchor; two 50M-op streams (excludes chimera_rfr-02 nnz=15140, crudeoil_lee1_07 nnz=19322 and every other slow medium row by nnz).

Determinism unchanged: fixed seeds, pure gates. Purity unchanged: stdlib + allowlisted feral crates only.

## Negative controls (all reverted, FACT)

- Early 24-pass relabelled RCM/Sloan/ND/NDFM: lt_1k LOSS (+0.72 bip). Terminal same: exactly baseline. 0020 extends to multi-seed.
- Terminal MinFill (4 seeds), 9 unwired custom-metric variants, 8-sweep pair, five/four post-ticket, 18-stream LNS (vs 14), small-core AMF-grid extension (1.0/16.0, ≤10k/200k core): all zero marginal. Reverted to protect margin.
- Medium 2-stream full-band (`nnz <= 30000`) on the new base: zero (its pipeline already covers it). Medium-cheap (`nnz <= 15000`) is what pays.
- v1 medium-4 + sparse-large-4 on old base: −1.45/−0.73 bip locally but FAILED hidden timing. Removed.

## Result (this box)

| | Base | Candidate | Δ |
|---|---:|---:|---:|
| Aggregate | 0.839063 | **0.838921** | **−1.42 bip** |
| lt_1k (0.30, 147) | 0.890341 | **0.890015** | 9 better / 0 worse |
| 1k_10k (0.30, 108) | 0.868720 | **0.868574** | 8 better / 0 worse |
| gt_10k (0.40, 45) | 0.778362 | 0.778362 | no work there by design |
| Worst `order()` | ~1.1–1.55 s class | lt 1.11 s / 1k10k 1.06–1.08 s / gt 1.55 s (gams05, untouched) | slow rows excluded by gate; global worst unmoved |

lt_1k wins: chimera_mgw-c8-439-onc8-001, graphpart_clique-70 (1.5%), maxcsp-langford-3-11, p_ball_10b_5p_4d_m (near-tie), sonet24v5, tls6 (1%), wastewater05m1 (new with 14 streams), waterund11, qspp_0_11_0_1_10_1 (dense band). 1k_10k wins: blend718, chimera_k64ising-02, chimera_lga-01, chimera_selby-c16-01, crudeoil_pooling_ct1/ct3, hydroenergy2, rsyn0805m03m.

`cargo test -p ssi-candidate-worker`: **50 passed**. `yukon run` purity/license gate passed.

## Why it should survive hidden

- No added work on n > 6000 at all; no added work on any 1k_10k row with nnz > 15000; small worst 1.11 s stays ~1 s clear of the cap even at 1.5× hidden slowdown; every ticket is strict best-of (17/0 movers).
- The v1 killer (medium-full + large additive work on the slow tiers) is gone.

## Follow-ups

- Small LNS saturates at 14 streams (18: zero marginal). Small-dense saturates at 4 (8: zero marginal).
- gt_10k has no added work; any future gt_10k ticket must be core-gated (like reduce-then-AMF), never additive on (n, nnz).

## Links

- Predecessor: [0063](0063-time-margin-by-structure.md); failed v1 analysis above.
- Lottery: [0004](0004-structured-relabelings.md); margin: [0060](0060-conditional-search-escalation-below-anchor.md), [0061](0061-margin-scaled-leftover-search.md).
