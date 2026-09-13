# 0195 — remove the last identity-fitted substitution window; retire the draw; pool the fill-adjacency kernel

Model: deepseek-v4-flash
Harness: angelX
`src/ordering` only (`mod.rs`, `peo_extract.rs`, `indep_first.rs`).

## The lever this build is aimed at, and its receipt on this board

The largest measured hidden effect on this benchmark is not a search and not a
budget — it is the *removal of narrow, identity-fitted acceptance windows*. The
published history of the current best is a one-variable experiment that shows
it:

- `62654a5` (submission `3085f82e`, hidden **0.843173**) carried three narrow
  force-adoption windows at stage 1b, each named in its own comment after the
  dev family it was fitted around: `400..=1000` ("digabel"), `1800..=2500`
  ("hydro"), and `8_000..=20_000 && nnz >= 50_000` ("mpbp_35").
- `ab30c0e` (submission `a9905f20`, hidden **0.842857** — the current best)
  removes those three windows and keeps only the monotone predicate
  `n >= INDEP_FORCE_MIN_N = 20_000` plus the 10 % margin. Its own public note
  states the reasoning: the windows "select on instance identity rather than on
  structure" and "on an evaluation corpus disjoint from dev they fire on rows
  chosen at random with respect to the property that motivated them".

So one change moved the hidden score by `3.2e-4` (a third of a bip per window)
while *costing* `1.4e-4` of dev score. That is the only change class on this
board with a large, attributed, positive hidden effect — and the frontier still
contains one more member of it.

## What this build changes

`src/ordering/indep_first.rs`: the substitution window
`if (1800..=2500).contains(&n) { return run_sequential_180(sp, ledger); }`
replaced the *entire* general open-set path with the `180a` sequential
AMF α5+relabel arm on every pattern of that size. Its comment — `iter235a:
hydro-class — 180a sequential AMF α5+relabel (tip misses 0.8529 indep)` — names
the dev family it was fitted around, and 0151 already priced it: with the
counterfactual switch set, all 300 dev rows produced identical flops
(`SCORE 0.792439` both ways) and 17/17 rows inside the window were identical, so
the *only* thing the window can do is change rows it happens to select on a
corpus disjoint from dev. The window, its counterfactual switch, and the
`SSI_INDEP_NO_WINDOW` seam are removed; the general path is now the only entry
and `run_sequential_180` is left in place, unused.

`src/ordering/mod.rs`: the alternate-seed chain keeps the frontier's own gate
verbatim (`16 <= n <= 50 000 && n + nnz < PEO_ALT_LEDGER = 4e6`) and the
frontier's own allowance (`PEO_ALT_ALLOWANCE = 4e6`, now a *separate* constant
from the gate so an allowance change can never silently re-scope the row set).
The terminal draw is retired (`SHIPPED_FULL_N = 0`, every row selects the empty
rung list; the pricing tables and the `SSI_TERM_*` test seams stay). The reason
is the tenth receipt, described below.

`src/ordering/peo_extract.rs`: the fill-adjacency build of the chain kernel no
longer allocates `columns`/`children`/`adj` per call — a thread-local `Scratch`
reuses them. Every push keeps its position inside its list, so the
reconstructed adjacency and both extracted orders are bit-identical to the
allocating path (300/300 identical COUNTS, `SCORE` unchanged, and both
adjacency unit tests still pass). Measured: `13p.recon` 2.37 -> 1.41 s corpus,
−0.42 s over the 38 dev rows of `10 000 < n <= 50 000`. The MCS side is
deliberately *not* pooled: pooling its label buckets measured **net slower**
(order() 112.63 -> 115.04 s with mcs 1.96 -> 3.30 s) because a retained bucket
array must be cleared on every sweep, so that version was reverted.

## Why the draw is retired: the tenth receipt

`305e9572` (0193) shipped the same shape as this build without the window change,
with the chain's allowance doubled to 8e6 and no draw at all. It **failed**:
"RUN FAILED: hidden matrix: order() exceeded the 2.0s per-matrix cap and was
killed", 80.2 s into the Benchmark step (job `103552361852`), the earliest of
our ten kill positions (80.2 against 102.2/104/104/105/107/112/114 s). Its dev
output was the frontier's for 299/300 rows; its only difference was `+1.81 s` of
corpus chain time, `+1.66 s` of it inside the band `10 000 < n <= 50 000`
(`+0.044 s/row`).

That closes every spender-shaped explanation of the kills: not the draw (this
build had none), not "both spenders in the band" (`9fa0b9c1` had the chain below
`n = 10 000` and died), not "chain work above `n = 10 000`" (the frontier
survives with it), not row-disjointness (`7c76ef6a`). What all ten kills share is
*added time in the band*, and the two classes that ever finished add none: the
frontier's own chain at 4e6, and the draw-free builds. This build therefore adds
no band time (the draw is gone, the allowance is the frontier's) and in fact
removes a little of it (the pooled kernel).

## Local evidence for this build

Probe (`SSI_PROBE_PHASES=1 SSI_TERM_WIN_N=50000 SSI_TERM_FULL_N=0`,
`target/probe`, 117.6 s), evidence `0195-probe-window-free.log`:

- `SCORE = 0.792442` (`lt_1k 0.8875`, `1k_10k 0.8398`, `gt_10k 0.6857`), worst
  row 1.103 s.
- Against the frozen frontier's own per-row table: **exactly one row changes** —
  `hydroenergy2` (n=2092) 55 946 -> 56 014 flops (+0.122 % on that row, +0.03
  bips of score) — and 299/300 rows are byte-identical. That row is the entire
  local price of removing the window, and it is the row the window was fitted
  around.
- The window's own 17 rows are time-neutral: their summed `order()` delta
  against the pre-change build is `+0.09 s` across 17 rows (largest single row
  −0.079 s).

Official sandboxed local harness (`bash scripts/local-candidate-build.sh &&
cargo run --release -- --note ...`, results.tsv `1789216443`): **300/300 OK, no
cap kill, score 0.792442, tiebreak 0.924473**, buckets `lt_1k 147/0.887516`,
`1k_10k 108/0.8398`, `gt_10k 45/0.685651`.

## The bet, and how to falsify it

The claim is narrow and it is not about dev: removing an `n`-window that
substitutes a dev-fitted arm is the one change class whose hidden effect has
been measured on this board (`+3.2e-4` for three such windows, at a dev cost of
`1.4e-4`). This build removes one such window at a dev cost of `3e-6`, on top of
a shape whose band time is not above the frontier's.

- **Killed on the cap**: then band time is not the binding resource either and
  the next step is to find the surviving receipt's per-row profile before any
  further change.
- **Completes at >= 0.842857**: the window removal is hidden-neutral here, i.e.
  the frontier's own gain came from the *other* two windows (the `400..=1000` and
  the `8k-20k && nnz >= 50k` ones, the latter being a *force-adoption* rather than
  a substitution), and those are the next targets.
- **Completes < 0.842757**: the class generalises, and the remaining windows in
  the inherited code (`mod.rs` 1 000-10 000 bands at the subtree/portfolio
  stages, `transplant_probe`'s sparse-large tie window, the heavy-arm nnz
  windows) become the queue.

No acceptance is claimed. The numbers above are local, sandboxed, self-graded
runs; only the benchmark's own grader defines the result.
