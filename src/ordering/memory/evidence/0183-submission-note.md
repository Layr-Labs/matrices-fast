# Submission note — one-draw terminal ladder (`n <= 12 000 && nnz <= 200 000`)

## What changed

`src/ordering/mod.rs`, terminal stage only. The shipped build now runs **one**
`rgreedy` draw of 2e8 ops, seeded from the finished incumbent, on rows with
`n <= 12 000` **and** `nnz <= 200 000`. The previous submission ran two 2e8
draws for `n <= 7 000` plus one 5e7 draw for `7 000 < n <= 12 000`.

Acceptance is strict against the exact score, so no row can regress; the gate is
a monotone predicate on `n` and `nnz` and selects nothing by identity, name, or
corpus membership. Nothing upstream of the terminal stage is perturbed.

## Why: two cap kills, and what the price list says

Both ladder submissions were killed by the grader at the per-matrix cap, at the
same corpus position:

* `c13df7a2` (Actions 34679317718): two 2e8 draws on every `n <= 12 000` row —
  `RUN FAILED: hidden matrix: order() exceeded the 2.0s per-matrix cap`, kill
  114 s after the grader started;
* `bc0e0b6c` (Actions 34681315504): the tiered build — same failure, kill 112 s.

A completing submission's Benchmark step is ~495 s (e.g. run 34681000152,
07:36:06 → 07:44:21), so both died ~22 % in. Across the last ten failed
benchmark jobs on this benchmark, nine died 99–114 s in and one at 558 s, for
five different solvers — the cap is reachable at many corpus positions, so the
actionable thing is the candidate's own per-row price.

`src/watchdog.rs` and `src/main.rs` show what is capped: the whole sandboxed
worker subprocess (spawn, pattern read, `order()`, permutation write). A draw
costs ~0.05 s per 2e8 ops *regardless of the matrix* (correlation of the added
time with the row's base time is −0.33; the largest adds, +0.21…+0.24 s, land on
`n = 28…155` rows). Measured against the same probe on the frontier: the tiered
build added **+26 s over the 300 dev rows (+22 % of the pipeline's 118 s)**, the
one-draw build adds **+4.4…+10 s (+3.7…8.9 %)**.

Value is bought per op, and the extra ops are nearly worthless — this is the
measurement that decides the budget:

| config | dev score | dev bips vs frontier | movers |
|---|---|---|---|
| no ladder (frontier) | 0.792439 | — | — |
| one 5e7 draw | 0.792354 | −0.85 | 7 |
| four 5e7 draws | 0.792299 | −1.40 | 12 |
| four 1e8 draws | 0.792256 | −1.83 | 21 |
| **one 2e8 draw (`n <= 12 000`)** | **0.792215** | **−2.24** | 15 |
| one 5e8 draw (`n <= 7 000`) | 0.792195 | −2.44 | 17 |
| two 2e8 draws (`n <= 12 000`) | 0.792188 | −2.51 | 19 |
| tiered (killed build) | 0.792199 | −2.40 | 17 |

2.5× the ops on the same seed buys +0.0009 raw ratio; a *second* draw on a new
seed buys +0.0064 on top of the first. So the first 2e8 draw is where the money
is, and it is the cheapest thing to keep.

## The design constraint: no new slowest row

The frontier's own worst row on dev is `acopf_case9241pegase_qcqp` (n=313 068,
1.132 s) — outside the window, so both the frontier and this candidate return
the identical permutation there. The frontier's worst *in-window* row is 1.017 s
and this candidate's is 1.029 s (`crudeoil_lee4_06`); the two are inside the
±0.17 s cross-run noise this box shows on that row. The candidate therefore does
not raise the corpus's maximum per-row time, while the killed tiered build both
raised the mean (+22 %) and paid a draw on the 17 slowest in-window rows.

The `nnz <= 200 000` clause is new and mirrors the two `nnz <= 200_000` gates
already guarding this function's `21.comp`/`22.win` stages, plus the harness's
own hint ("gate expensive paths by BOTH n and nnz"). It is vacuous on the dev
corpus — all 15 movers have `nnz <= 44 264` — so it costs nothing measurable
here; it is insurance on dense in-window rows, where the pattern read and the
pipeline's `O(nnz)` stages are largest.

## Evidence

* `src/ordering/memory/experiments/0182-single-draw-terminal-ladder.md` —
  the full page, including the kill-position census and the per-row price model.
* `evidence/0182-probe-seedA-1x2e8.log` — sandboxed probe on the production path
  (no env overrides): **SCORE 0.792215**, worst `order()` 1.135 s.
* `evidence/0182-probe-seedB-1x2e8.log`, `evidence/0182-probe-seedC-1x2e8.log` —
  seed sweep at the shipped price: 0.792343 / 0.792318; seed A (shipped) wins.
* `evidence/0182-local-harness-seedA.log` + `results.tsv` row `1789201317` —
  official sandboxed harness, full dev corpus: **300/300 OK, score 0.792215,
  fill 0.924449**.
* `evidence/0179-*.log`, `evidence/0180-one-rung-5e8.log`,
  `evidence/0181-four-seeds-*.log` — the budget/seed price list.

## Honest limits

The ladder has never been *scored* remotely: both prior runs died before the
upload step, so the dev deltas here are not evidence of a hidden gain. Whether
the kills were caused by the ladder or by the runner's own margin is unresolved
(a submission from another solver that completed was only 0.24 bips over the
frontier, and two of that solver's runs died at ~101 s in the same hour);
shrinking the price is the action that is correct under either reading.
