# 0165 — BRKGA memetic random-key elimination: measured NEGATIVE (basin shift)

- **Date:** 2026-09-14
- **Score:** 0.789841 → **0.789848 (+7e-6 WORSE)** for up to +0.6 s of
  worst-row wall; wiring reverted with receipt, module + 2 unit tests kept.
- **Verdict: NO-GO.** The MatricesLit Rank-2 pick's medium-risk flag was
  correct: 0004's no-local-structure result extends to the crossover
  operator.

## What was built

`brkga.rs` — BRKGA over random-key vectors (RAIRO-RO 2023 family): decoder =
greedy min-degree elimination with `(degree, key, id)` tie-breaks — a
lottery DISTINCT from `relabel` (the keys drive the elimination itself, not
the vertex numbering a library tie-break reads). The decoder returns the
order AND its exact `Σ c_v²` (c_v = live degree at elimination), so the
population ranking is the true objective at zero extra cost. Incumbent
injected as elite #0 (inverse permutation as keys); elites copied, mutants
fresh, parametrized-uniform crossover (p=0.7) otherwise. Per-decode op
ledger with snapshot-tail degradation. Wired at pop 8 × 2 generations of
≤ 2M-op decodes on `500 ≤ n ≤ 6_000 ∧ nnz ≤ 30_000`, exact strict
acceptance.

## Results and reading

- Net **+7e-6 worse** (0.789848 vs 0.789841) across the full corpus. The
  insertion is strict-accept, so no row regresses AT THE SITE — but an
  early-accepted BRKGA candidate changes which incumbent the downstream
  subtree/exchange chain polishes (powerflow0118p +0.60 %, a handful of
  sub-0.1 % shifts), and the basin cost exceeds the lottery wins.
- The key-driven decoder IS a new lottery, but the portfolio's existing
  samples (minfill restarts, relabels, exact windows) dominate it on this
  band. Crossover adds no exploitable structure — consistent with 0004's
  "the relabeling→flops map has no exploitable local structure" for the
  crossover neighborhood too.

## Files

- `brkga.rs` (KEPT — instrument, 2 tests: determinism/bijection,
  empty-graph decode + starvation), mod.rs wiring REMOVED with the receipt
  comment.

## Reproduction

```sh
cargo test --release -p ssi-candidate-worker --offline --locked -- brkga
# the wired (negative) form: git show <0165 commit>:src/ordering/mod.rs
```
