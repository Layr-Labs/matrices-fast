# 0191 — per-row attribution of our own dev delta, and the band the ladder's window owns

New instruments: `evidence/0191-tools/frontier_rowdiff.py` (per-row, per-bucket
diff of any two probe logs against each other, both ratio-table and
`order()`-seconds), plus the band/phase table below. Everything here is read off
archived logs plus one new probe run (`0191-probe-rowdisjoint.log`).

## (a) The frozen frontier's own probe table is comparable and the tool is calibrated

`0151-probe-baseline-ab30c0e.log` prints `SCORE = 0.792439` while its `COUNTS`
lines (299 rows, `slay06m` missing) reconstruct to `0.792222`; adding
`slay06m` at ratio exactly 1.0 to that set gives **0.792439**, so the frontier's
printed dev score is reproduced exactly and the one missing line is identified.
The same aggregation reproduces the shipped build's printed `0.792223` from its
own 300 `COUNTS` lines, so the tool's arithmetic is anchored on two independent
prints.

## (b) Where our dev gain actually is (vs the frozen frontier, `0151`)

`0190-probe-ladderwin10k-shipped.log` vs `0151`: **12 rows changed, all
improvements, zero regressions**; every mover has `n <= 6418`; the gt_10k bucket
is byte-identical (`0.685651` both). `crudeoil_lee2_06` (n=6418, nnz=34646,
-5.58 %) carries `-1.595e-04` of the total `-2.564e-04` bucket-weighted
log-delta, i.e. **62 % of the entire dev gain is one row**, and 7 of the 12 are
in 1k_10k.

Per-build dev deltas against the same frontier table (`delta = B - A`):

| build | score over common rows | delta |
|---|---|---|
| `0167-terminal-ladder-2e8-only` (draw only) | 0.791998 | -2.24e-4 |
| `0185-probe-density-shaped` | 0.791994 | -2.28e-4 |
| `0187-probe-altgate` (chain gated > 10 000) | 0.791994 | -2.28e-4 |
| `0190-probe-flat2e8` | 0.791998 | -2.24e-4 |
| `0190-probe-ladderwin10k` (shipped iter27) | 0.792005 | -2.17e-4 |
| `0191-probe-rowdisjoint` (this candidate) | 0.791994 | -2.28e-4 |

**The terminal draw owns essentially the whole dev delta**; the chain's gate,
scope and allowance changes are dev-neutral (0 to -4e-6), and the 10 000 window
cost 1.1e-5 (`powerflow0300p`).

## (c) Where our dev seconds actually are (probe frame, `order()` per row)

`0191` vs `0161-tail-attribution-300` (frontier) per-row seconds:

- the add is concentrated on the **smallest** rows: lt_1k `+6.33 s` over 146
  rows, 1k_10k `+4.09 s`, while `10 000 < n <= 50 000` is **-2.90 s** and
  `n > 50 000` **-1.28 s**.
- the worst *relative* adds are +79 % (`ex5_2_4`, n=18), +64 % (`ex8_1_7`,
  n=14), +62 % (`ex6_1_4`, n=10): the draw's charge is a near-constant share of
  a very cheap row.
- the draw's own price (row total minus the in-process `LADGATE` pre-draw mark)
  over its 238 drawn rows: **9.94 s total**, and it is *anti*-correlated with
  size — `corr(price, log10 flops) = -0.23`, `corr(price, log10 n) = -0.33`,
  top-24-fill rows mean 0.0226 s against bottom-24 0.0340 s. So the charge is
  bounded (~0.03-0.095 s) and does **not** grow with the row's fill over a
  6-order-of-magnitude fill range (44 to 3.3e7 flops).

## (d) The 10k-12k band, in seconds, for all four profiles

`order()` seconds of the eight dev rows with `10 000 < n <= 12 000`
(frontier `0161` vs `0191`):

| row | n | nnz | frontier | this build |
|---|---|---|---|---|
| `crudeoil_lee4_06` | 10429 | 55492 | 0.958 | 0.973 |
| `powerflow0300p` | 11251 | 41918 | 0.880 | 0.744 |
| `mpbp_35` | 11120 | 40790 | 0.924 | 0.732 |
| `transswitch0300p` | 11659 | 48446 | 1.017 | 0.725 |
| `mpbp_21` | 11716 | 37660 | 0.655 | 0.654 |
| `methanol200` | 11999 | 76128 | 0.602 | 0.604 |
| `mpbp_34` | 11556 | 40860 | 0.752 | 0.550 |
| `glider400` | 10017 | 49624 | 0.409 | 0.357 |
| **total** | | | **6.197** | **5.338** |

## (e) The 55d9ed93 receipt kills the "union of survivors" reading

`55d9ed93` (the iter27 build: window 10 000, chain at `1e6` above it) is the
**sixth** kill and its 10k-12k profile is the *cheapest ever submitted*:
frontier 6.197 s, `55d9ed93` 5.338 s (chain 0.34 s vs 1.16 s), `e5a3c6b4`
~5.74 s. The surviving frontier carries the most chain time in that band
(1.16 s) of the four and finished. So "which spender owned the band" does not
order the receipts by cost; what every kill shares is that the chain spends on a
row above the draw's window at all. This candidate is the first submission in
which the two budgeted searches are row-disjoint above the draw's window.
