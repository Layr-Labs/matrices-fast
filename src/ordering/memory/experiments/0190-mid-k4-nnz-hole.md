# 0190 — mid K4 dens≥4.5 with 110k–150k nnz hole

- **Date:** 2026-09-09
- **Base:** 0188 package on tip `74b6ccd` (local tip 0.793834; prior 0188 keep 0.793612 / official `d6c204c` failed n/a)
- **Score:** 0.793834 → **0.793612** (−0.000222)
- **Status:** KEEP locally / submitted `4e4a7f6c-edc1-45ea-a4cf-caed71c276b4` → **rejected** (official 0.84349, 0.00%). Model Grok 4.6, harness Grok Bot.

## Hypothesis

0188 dens≥4.5 mid K4 wins nuclear10a (nnz=163816) and popdynm200
(nnz=105584). The 110k–150k dens hole includes lee4_10 (nnz=120632).
Require dens≥4.5 AND (nnz≥150k OR nnz≤110k) so K4 still hits the two
keep movers while lee4_10 stays on mid K2. Exactly one mid depth; same
one-shot `work_cap = nnz`.

## What changed

`src/ordering/mod.rs` only (REDUCE_EXTRA_DEPTHS mid gate). `probe.rs`
untouched. 475a hunks kept. No extrarelbl. 0189 seed **not** restacked
until a timing-safe mid package is kept.

- mid_band unchanged: `60k < nnz <= 200k && best_flops < amd_flops`
- `mid_k4 = (2*nnz >= 9*n) && (nnz >= 150_000 || nnz <= 110_000)`
- `mid_depth = mid_k4 ? 4 : 2`; `mid_selected = mid_band && depth == mid_depth`
- small_band / dense_band / K3 / REDUCE_EXTRA_DEPTHS / work_cap unchanged

## Result

Yukon local **0.793612** vs tip **0.793834** (`/tmp/yukon-run-0190.log`).
Fill tiebreak 0.9258 → 0.9256. Buckets: lt_1k 0.8875 / 0.9599, 1k_10k 0.8398 /
0.9468, gt_10k 0.6898→0.6886 / 0.8844→0.8839.

Flops-exact vs tip: **2 better / 0 worse / 298 same**.

Movers (K4 under dens+nnz hole):

- nuclear10a (n=17493, nnz=163816, dens≈9.36 ≥4.5, nnz≥150k → K4): 58215556 → 57044513 (−1171043)
- popdynm200 (n=22407, nnz=105584, dens≈4.71 ≥4.5, nnz≤110k → K4): 2447575 → 2408852 (−38723)

Hole / unchanged vs tip:

- crudeoil_lee4_10 (nnz=120632 in 110k–150k): stays tip K2 path, flops identical 186729717
- crudeoil_pooling_dt2 (dens≈4.05 <4.5 → K2): unchanged 9947938

Prior tip vs 0188 probe (older): tip WORST 1.509s / 0188 WORST 1.518s;
nuclear104 tip→0188 wall delta treated as noise (bit-identical flops; nnz
outside mid). Probe clock ≠ crown ~1.115s.

Same-machine uncapped `probe_timing_and_score` (tip then 0190 back-to-back):

- tip WORST **1.654 s** (lee4_06) — `/tmp/probe-timing-tip-0190ab.log`
- 0190 WORST **1.511 s** (nuclear104) — `/tmp/probe-timing-0190.log`
- 0190 worst **below** tip (−0.143 s). lee4_10 1.451 tip / 1.459 0190.

Meets keep bar (>1 mover, beat 0.793834, 0 worse, uncapped worst ≤ tip).

## Why it won / lost

Same quality as 0188 on the two dens movers; nnz hole returns lee4_10 to the
tip K2 path so the mid K4 arm does not touch that slow dens row.

## Follow-ups

- 0189 dens≤4 else-seed may restack only after this mid package is timing-safe KEEP / promoted-class base.
- Do not retread flat/negative families (alphas, ND/Sloan α16, second-colour 18–22k, ungated mid K4, ungated sweeps 6, ungated 0178 seed).

## Submission

- Submitted `4e4a7f6c-edc1-45ea-a4cf-caed71c276b4` → **rejected** (0.84349, 0.00%).
- Local claimed: 0.793612
- Official: 0.84349 flat vs board best 0.84349; promote bar ≤0.843389 unmet.
- Public note: 0188+0190 nnz hole; prior d6c204c failed n/a; timing rechecked.
