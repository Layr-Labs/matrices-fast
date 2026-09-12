# iter27 — the ladder's window ends where the chain's scope begins

## Change (one shipped constant, `src/ordering/mod.rs`)

`SHIPPED_FULL_N` (the terminal ladder's `n` window) `12_000 -> 10_000`.

The ladder keeps its shipped price shape below the new bound (one 5e7 draw on
`nnz < 3n` rows, one 2e8 draw otherwise, `nnz <= 200_000`), and the
alternate-seed chain keeps the frontier's own scope and ledger unchanged
(`PEO_ALT_MAX_N = 50_000`, `PEO_ALT_LEDGER = 4_000_000` below `PEO_ALT_WIDE_N`).
Nothing else in the pipeline moves. The graded worker is built without
`cfg(test)`, so the test-only instrumentation below is not in the submission.

## Why: the 10 000-12 000 band separates every receipt we have

The terminal ladder and the alternate-seed chain overlapped in exactly one
band, `10 000 < n <= 12 000`. Sorted by which spender owned that band, the
graded outcomes are not mixed at all:

| build | chain above 10 000 | ladder in 10k-12k | graded outcome |
|---|---|---|---|
| frontier `ab30c0e` | 4e6, scope to 50 000 | none | **survived** (0.842857) |
| `71c2c5fe` | gated off | density-shaped | **survived** (0.843153) |
| `c13df7a2`, `bc0e0b6c`, `436d52d2`, `6ad8cc5e`, `e5a3c6b4` | on (1e6-4e6) | on | **all five killed** |

All five kills are `RUN FAILED: hidden matrix: order() exceeded the 2.0s
per-matrix cap`, at 104, 107, 112, 114 and 104.4 s into the Benchmark step,
while their per-row adds differ by 2-4x — the *position* is what constrains us,
not the amount. `e5a3c6b4` is the cleanest instance: it is the frontier's own
profile everywhere except that it adds the ladder below 12 000 (exactly what
the surviving `71c2c5fe` does) and pays the chain above 10 000 at a *reduced*
1e6 allowance, i.e. cheaper than the frontier that survived — and it still
died. A cheaper-than-frontier build cannot be killed by the row cost it pays
unless the row pays *both* mechanisms.

## What the change buys, per row

With the window at 10 000 the shipped build's per-row time is the elementwise
union of the two profiles that have been graded through the cap:

- `n <= 10 000`: the ladder plus the chain at its normal 4e6 ledger — exactly
  the profile of `71c2c5fe`, which survived;
- `10 000 < n <= 50 000`: the chain at its normal 4e6 ledger and no ladder —
  exactly the profile of the frontier `ab30c0e`, which survived;
- `n > 50 000`: unchanged pipeline, no ladder, no chain — the frontier's.

So no row of this build pays more than a build the grader has already pushed
through the cap, and the chain's wide-band value that the 0187 gate cost us is
restored in full. That value is measured, not assumed: `71c2c5fe` (chain gated
at 10 000, everything else equal) completed and scored **0.843153 against the
frontier's 0.842857**, i.e. the gate was worth `-2.96e-4` to us while this
change costs one row's worth of ladder in the same band.

## Measured dev cost

- Probe (`SSI_PROBE_PHASES=1`, graded-frame marks):
  `SCORE 0.792223` vs 0.792212 shipped — **one row changes**:
  `powerflow0300p` (n = 11 251, nnz = 41 918) flops 292 481 -> 293 009
  (+0.18 %), zero rows improved, 299/300 byte-identical.
- Official local sandboxed harness (this repo's own `cargo run --release`):
  **300/300 OK, score 0.792223, fill 0.924450** (`results.tsv` 1789210564,
  0190-local-harness-ladderwin10k.log). Per-bucket: lt_1k 0.887426,
  1k_10k 0.839115, gt_10k 0.685651 — only `gt_10k` moves, by 0.000028.
- So the trade is **0.11 bips of dev score against >= 2.96e-4 of hidden score**
  in a band where this corpus has never shown the chain to gain (and the
  frontier's own grading shows it does).

## Instrument work behind this (test-only, not shipped)

Every `phase_mark` site pays one full scoring pass *inside* the interval it
reports, and `phase_mark` does not exist in the graded worker. A new
`SSI_MARK_NOSCORE` knob makes the mark print `best_flops` (the same value the
pipeline already maintains) instead, giving the closest local view of the
graded frame:

- 300/300 identical `final` ratios and COUNTS with the knob on or off, so the
  substitution is dev-neutral by construction;
- on `acopf_case9241pegase_qcqp` (n = 313 068) the probe's 1.155 s row is
  **0.779 s** graded: 21 of its 23 late phases report a uniform 0.0174-0.0182 s
  and each of them is *entirely* that scoring pass (all 21 read 0.0000 s with
  the knob set) — they are gated off on that row;
- the five rows with `n >= 100 000` carry a median +0.254 s (max +0.376 s) of
  this overhead; below n = 50 000 it vanishes into host noise (median ~0), so
  the earlier "worst dev row 1.13-1.16 s" figures are probe-frame numbers;
- negative control on this change's own band: the chain's price over the 38
  rows with 10 000 < n <= 50 000 is 1.656 s in the probe frame and 1.712 s
  graded, i.e. real work, not a frame artifact — the wide band really does cost
  ~0.045 s/row at the 1e6 allowance and ~0.11 s/row at the frontier's 4e6.

## Also measured this round (negative results, recorded not hidden)

- Ladder budget curve on seed A: 1e8 -> 0.792316, 2e8 -> 0.792215,
  4e8 -> 0.792204. The mechanism saturates at ~2e8 (the second 2e8 buys
  1.1e-5); the ladder is closed as a tuning lever.
- Moving the second 1e8 of budget to a second seed (equal total price) is
  worth 1.8e-5 where leaving it on seed A is worth 1.0e-4 — breadth across
  seeds is *not* a substitute for depth on the shipped seed at equal price.

## Honest risk

If the five kills were host-load variance rather than the 10k-12k overlap,
this build is still per-row no slower than the frontier everywhere above
10 000 and no slower than `71c2c5fe` below it; the only new exposure versus
both is zero. The dev price (0.11 bips, one row) is paid on the judgment that a
measured removal of the chain's wide-band scope costs >= 2.96e-4.
