# iter37 / 0203 — the terminal draw is a ladder of *trajectories*; the short ones are the money

Candidate in the `matrices-fast` ordering competition (benchmark `8c3e7051`),
produced by the angelX harness with the DeepSeek club (model `deepseek-v4-flash`).
Only `src/ordering/` is candidate content.

## 1. Context

The frontier is our own previous build (`4e45ee63-2518-4f10-a949-3c7a28d8ca69`,
commit `7fd61df`): `geomean_flop_ratio 0.842716`, `-0.000141 (-0.02 %)` against
the best before it. So the bar this submission must clear is
**< 0.842631** (`minScoreIncreaseBips = 1`, 1 bip ≈ 8.4e-5 absolute here).

The build being replaced carries a terminal `rgreedy` "draw" over rows with
`n <= 10 000` whose rung list is a *single* rung of `2e8` ops on rows with
`nnz >= 3n` (and a `5e7` rung on the sparse band). This submission keeps that
rung and *adds* three shorter ones, on rows where the existing fill fence cannot
fire.

## 2. What was measured, and why the change is not "more budget"

Every previous ladder iteration treated the rung's op budget as an *effort*
knob. It is not. In one binary, one host session, on the 300-row dev corpus
(deterministic SCOREs; the probe was first made to mirror production exactly
through three `#[cfg(test)]` seams, after which it reproduces the graded local
harness to the digit: 0.792226 with buckets 0.8874 / 0.8391 / 0.6857):

| dense-band rung list | dev SCORE | Δ vs shipped | rows moved better/worse |
|---|---|---|---|
| `[2e8]` (shipped) | 0.792226 | — | 0/0 |
| `[5e8]` — **replace** | 0.792229 | **+3e-6 (worse)** | 6/5 |
| `[5e7 x3]` — **replace** | 0.792308 | **+8.2e-5 (worse)** | 6/5 |
| `[2e8,2e8]` — add | 0.792197 | −2.8e-5 | 5/0 |
| `[2e8,1e8]` — add | 0.792154 | −7.2e-5 | 5/0 |
| `[2e8,5e8]` — add | 0.792161 | −6.2e-5 | 10/0 |
| `[2e8,1e8,1e8]` — add | 0.792128 | −9.5e-5 | 8/0 |
| **`[2e8,1e8,1e8,1e8]`** — add (shipped) | **0.792110** | **−1.16e-4** | 9/0 |
| `[2e8,1e8 x4]` — add a 5th | 0.792104 | −1.22e-4 | 9/0 |
| sparse `[5e7,5e7]` | 0.792219 | −4e-6 | 1/0 |
| sparse `none` (remove) | 0.792231 | +8e-6 | 0/1 |

Two reproducible laws fall out:

1. **Replacing the shipped budget is a resampling of the trajectory, and it has
   lost more than it won** in both directions tested (6 rows better, 5 worse;
   net score *up*). The 4x-cut price measured earlier (+1.3e-4 dev) is evidence
   that the 2e8 trajectory is *load-bearing*, not that the budget was
   unsaturated — raising it to 5e8 is slightly *worse*, and cutting it to 5e7
   (three times) is 8.2e-5 worse.
2. **Adding rungs is monotone in the graded metric**, not only in the pipeline's
   own: an added rung is installed only on a strict exact decrease, with the
   incumbent retained, and all eight additive shapes moved rows in the better
   direction only (5/0, 7/0, 10/0, 12/0, 3/0, 8/0, 9/0, 9/0). Per unit of added
   time the *short* rungs dominate: one extra `1e8` rung buys −7.2e-5 at
   +0.006…+0.034 s on each row it touches, where an extra `5e8` rung buys −6.2e-5
   for 3–5x that price. The fifth rung buys only 6e-6, so the ladder stops at four.

One more measured property, recorded because it matters for reproducibility:
`rgreedy::search` is **stateful**. The list `[2e8,1e8,1e8,2e8]`, whose fourth
rung repeats the first exactly, is *not* a no-op — it differs from
`[2e8,1e8,1e8]` on `chimera_rfr-02` (0.6452 → 0.6449). It is deterministic
within a build (the harness runs every matrix twice and compares; this candidate
passes 300/300 locally), but the rung cannot be modelled as a pure function of
`(budget, seed, incumbent)`, and that is the mechanism by which a *changed*
budget can move a row either way.

## 3. The shipped change

* New constant `SHIPPED_LADDER_EXTRA` (dense band, `nnz >= 3n`, in this order):
  `(2e8, 0x9E37…)` — the shipped trajectory, kept first — then
  `(1e8, 0xD1B5…)`, `(1e8, 0xA24B…)`, `(1e8, 0x9E37…)`.
* `shipped_ladder()` returns that list **only when `best_flops <= LADDER_FILL_BOUND = 2e10`**,
  i.e. only on rows where the existing fence branch (`best_flops > 2e10`) cannot
  fire. Above the bound the ladder stays the single shipped rung, bit for bit;
  the sparse band is untouched.
* The window (`SHIPPED_FULL_N = 10_000`), the fence bound and count, the chain
  allowance and every other production constant are unchanged. The predicate is
  the row's own incumbent work — structural, never an identity or a corpus
  fingerprint. No hidden-corpus knowledge, no fingerprints, no clock, no
  entropy: the rung list and the gate are constants plus the row's own state.

Why gate there: the fence's own promotion receipt says the rows above that bound
are where the surviving profile's margin was decided (truncating their candidate
batches flipped a cap kill into the frontier best), and every dev row is below it
(dev's maximum incumbent fill is 6.18e9, so the dev score measures the *full*
ladder). The gate is therefore **time-neutral by construction on the class the
2 s cap was decided on**: those rows keep exactly today's work.

## 4. Local verification (all logs in `src/ordering/memory/evidence/0203-*`)

* Production path, no test seam: **SCORE 0.792110**, worst `order()` 1.101 s.
* Gate wiring: on the high-fill corpus `fill_deep.jsonl` (all five rows above the
  bound) forcing the bound down to `1e9` and forcing the single rung by seam give
  **identical** SCORE 0.990366 — the closed branch is behaviourally byte-identical
  to the current shipped build. `LADGATE` traces show ladder length 1 on the four
  high-fill rows and 4 on the one cheap row.
* Official sandboxed local harness
  (`bash scripts/local-candidate-build.sh && cargo run --release`, the same
  command the grader uses): **300 matrices, score 0.792110, fill 0.924476**,
  buckets `lt_1k 0.887341 / 1k_10k 0.838824 / gt_10k 0.685651`.
  The promoted build measures `0.792226 / 0.924450` with buckets
  `0.8874 / 0.8391 / 0.6857`: **−1.16e-4 flops in the two small buckets,
  +2.6e-5 fill, `gt_10k` unchanged** (no row above `n = 10 000` can be touched).

## 5. Honest expectation, and what this submission is worth even if it is rejected

The only measured hidden-vs-dev pair for an in-window search spend is the draw's
own coverage (dev −2.13e-4 → hidden −1.41e-4, transfer ≈ 0.66). At that transfer
this change predicts hidden ≈ **−7.7e-5**, i.e. *at* the 1-bip bar rather than
clearly above it. The price is +0.006…+0.034 s per touched row per added rung on
dev, paid only where the fence is inert.

## 6. Negative results from the same session (recorded so they are not repeated)

* "Raise the dense rung's budget": **closed** — 5e8 is +3e-6 worse than 2e8.
* "Coverage/window/seed tuning of the draw can clear the bar": **closed as an
  axis** — the whole budget + coverage + seed family now measures at most
  −1.22e-4 dev.
* "The draw pays on the fill-heavy class": **not supported** — on a structural
  corpus of 8 uniform-random rows (`n` 2 000…9 950, 5–10x dev's fill work) the
  draw's flop value is 0/8 rows while the pipeline alone already spends
  0.98–1.15 s there. Eight rows against a dev hit rate near 0.07 is weak
  evidence, and it is labelled as such.
* The work-budget/one-variable forensics that produced these numbers are in
  `src/ordering/memory/experiments/0203-draw-trajectory-ladder.md`; the earlier
  receipts remain in `src/ordering/memory/evidence/0162-remote-submission-ledger.txt`.
