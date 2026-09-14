# 0164 — Flow-cutter Pareto bisection: measured NEGATIVE on the gt_10k ties

- **Date:** 2026-09-14
- **Score:** 0.789841 → **0.789841, ZERO of 300 rows moved**; wiring reverted
  with receipt, module + 3 unit tests kept as instruments.
- **Verdict: NO-GO.** The MatricesLit Rank-3 pick's honest-expectation-zero
  was correct on this corpus.

## What was built

`flowcutter.rs` — the Hamann/Strasser (BSD-2 reference
ben-strasser/flow-cutter-pace16) interleaved two-ball core WITHOUT the
push-relabel refinement: grow two BFS balls from seeded terminals one layer
at a time (smaller ball first), touch test each round, separator = the
smaller ball's boundary; recursive separator-last bisection with
`(degree, id)` local min-degree leaf/separator ordering; explicit op ledger
(edge scans); graceful degradation to snapshot tails. Wired as a portfolio
candidate on `n ≥ 10_000 ∧ nnz ≤ 2M` (the 22 gt_10k AMD ties that receive
only AMD), exact strict acceptance.

Unit tests: determinism/bijection on an 8×8 grid; separator quality (first
cut ≤ 12 vertices on the grid, min balanced ≈ 8); budget starvation still a
bijection. One real bug was caught and fixed by the corpus fixture (stale
sibling slot indices escaping the per-block `visited` bounds — order_block
now deactivates all slots per block).

## Results and reading

- **Zero rows moved** on the full corpus — the bisection-ND order never beat
  the incumbent on a single row, ties included. This is 0039's structural
  verdict repeating with a different ND engine: the gt_10k KKT/giant-MINLPLib
  graphs have no elimination-friendly small separators of this kind (METIS
  2.2–4.5× worse, Scotch 9519×, and a from-scratch bisection ND loses too).
- **Timing blocker found**: the leaf min-degree walk is NOT op-priced — 81.5 s
  worst on the 20M edge-scan ledger (faclay75 class). Any retry must charge
  the leaf ordering into the ledger (or cap leaf work structurally).
- The deeper reference port (push-relabel flow refinement + piercing + Pareto
  multistart, ~1–2k LOC) remains the open path, with EV still
  expectation-zero on this corpus family per this + 0039.

## Files

- `flowcutter.rs` (KEPT — instrument, 3 tests), mod.rs wiring REMOVED with
  the receipt comment at the insertion site.

## Reproduction

```sh
cargo test --release -p ssi-candidate-worker --offline --locked -- flowcutter
# the wired (negative) form: git show <0164 commit>:src/ordering/mod.rs
```
