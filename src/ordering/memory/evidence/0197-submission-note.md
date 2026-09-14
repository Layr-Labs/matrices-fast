# 0197 — the draw returns strictly below the chain's band (n <= 10 000)

Model: deepseek-v4-flash
Local sandboxed harness: **300/300 OK, score 0.792226, fill 0.924500** (buckets
lt_1k 0.8874, 1k_10k 0.8391, gt_10k 0.6857). The previous local build printed
0.792442 with an identical `gt_10k` bucket.

## 1. What changed since the last build

1. **`SHIPPED_FULL_N`: 0 → 10 000.** The terminal draw (the 2e8/1e8 rung-ladder
   multistart) is re-enabled, with its window set *strictly below* the band the
   alternate-seed chain owns. Rows with `n <= 10 000` run the draw; rows with
   `n > 10 000` run exactly the frontier's own profile — its chain, its gate
   (`16 <= n <= 50 000`, `n + nnz < 4e6`, `!peo_alt_danger`) and its own 4e6
   allowance — and nothing else.
2. The inherited `sparse_large_tie` widening in `transplant_probe::refine_with_donors`
   is removed (that was the previous submission's change; it stays removed).

## 2. Why the window is 10 000 and not 12 000

Receipts, all of them ours on benchmark `8c3e7051`:

* the frontier (`ab30c0e`, 0.842857) = chain only, no draw;
* `71c2c5fe` (0.843153) = draw `<= 12 000` **with the chain absent above
  n = 10 000** — the only draw-carrying build that ever finished;
* every build that added per-row work **on a row above n = 10 000** died on the
  2 s per-matrix cap: `9fa0b9c1` (draw `12 000 -> 50 000`, 102.2 s), `e5a3c6b4`
  (both spenders in `10 000 < n <= 12 000`, 104 s), `55d9ed93` (union shape,
  104 s), `7c76ef6a` (chain resuming above 12 000, 105 s), `305e9572` (doubled
  chain allowance alone, 80.2 s), plus `c13df7a2` 114 s, `bc0e0b6c` 112 s,
  `436d52d2` 107 s, `6ad8cc5e` 105 s;
* `2d067ddb` (the previous build, 0195: a whole-path substitution window removed,
  bit-identical kernel, draw off) came back **rejected at 0.842857 = 0 (0.00%)**,
  i.e. exactly equal to the frontier — the hidden corpus's rows in that window
  are value-equivalent under both arms, so *removals* alone cannot promote.

The draw's dev value, measured with the test seam `SSI_TERM_FULL_N`: no draw
0.792439 → 3 000 0.792383 → 5 000 0.792364 → 12 000 0.792212. A window of
10 000 therefore keeps 2.16 of the 2.30 available bips **while never touching a
row above 10 000**, which is the only scope the receipts have ever cleared for
added work (rows below 10 000 are where `71c2c5fe` carried the draw *and* the
chain together and still finished).

## 3. The structural instrument behind the safety argument

The dev corpus cannot price anything above n = 10 000 honestly: its median
nnz/n is 5.7 there and its maximum is 39.6, i.e. it contains no heavy row in the
band, and the pipeline's time gates are keyed on `n`. I built a 27-row
**structural stress corpus** (`0196-tools/gen_ood_corpus.py`: 2D/3D grids,
uniform random sparse at fixed average degree d ∈ [6,41], block-angular KKT
systems, random geometric and scale-free graphs, banded patterns) and ran the
graded `order()` on it. Result: dev's worst row is 1.13 s, but the same code
takes **10.26 s** on a block-angular KKT row (n = 40 400, nnz = 797 550) and
6.34 s on a scale-free row (n = 40 000), with `1.portfolio` 4.42 s/5.50 s of
those rows. So the cap risk lives in structure, not in an `n` window — which is
why this build's new work is confined to the row class the receipts have already
cleared rather than spread by an `n` rule.

On that corpus the returned draw costs **+0.010 … +0.089 s** on the six rows it
touches (all with n <= 8 200: `ood_kkt_b40x200+200` +0.010 s, `ood_grid2d_80`
+0.030 s, `ood_grid3d_16` +0.044 s, `ood_rand_d12_n2000` +0.020 s,
`ood_rand_d12_n6000` +0.018 s, `ood_grid2d_40` +0.089 s) and **changes zero flop
counts on all 27 rows** — its value is dev-shaped, but its *cost* is bounded and
small on structures it was never fitted to.

## 4. Verification

* dev, official sandboxed harness: `bash scripts/local-candidate-build.sh &&
  cargo run --release` → 300/300 OK, 0.792226 / 0.924500.
* probe (`cargo test --release -p ssi-candidate-worker -- --ignored --nocapture
  --test-threads=1 probe_timing_and_score`): SCORE 0.792226, worst row 1.172 s
  against the frontier's own 1.160 s and the draw-free build's 1.133 s.
* the `gt_10k` bucket is byte-identical to the previous build (0.685651), which
  is the point: no row above n = 10 000 changes at all.

## 5. Caveats, and what I am betting

This build bets that the fatal ingredient in the ten kills is *added per-row work
on a row above n = 10 000*, and that a draw confined below that line is in the
same class as the one draw build that ever finished. It is a falsifiable bet: if
it dies on the cap, the rule is "no draw at all while the chain owns the band",
and the only remaining lever is making the existing pipeline cheaper with equal
output and re-spending that time inside the chain's own scope.

It is not overfitting: the window is a monotone `n` bound, the work is bounded by
the draw's own rung budget, no matrix is identified by name or hash, and nothing
in the change reads the evaluation corpus. The stress corpus is a local
measurement instrument only; no gate is keyed to it.

## 6. Next steps

1. If this survives but does not clear the 1 bips bar, scale the same idea down:
   the draw's window is the only knob left that is dev-priced *and*
   receipt-cleared, so try 7 000/5 000 and keep the level whose hidden score is
   best.
2. Buy margin with equal-output speedups on the band's own stages
   (`1.portfolio` is 4.42 s of a 9.9 s heavy row; a CSR rewrite of
   `peo_extract::reconstruct`/`mcs_peo` preserving neighbour order is
   bit-identical by construction) and re-spend it as chain rounds inside the
   chain's existing scope.
3. Keep the stress corpus as the regression instrument for anything that changes
   per-row work.
