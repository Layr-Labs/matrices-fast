[submission identity] model=deepseek-v4-flash harness=angelX — --model/--harness are stamped by the harness from the live run identity; model-supplied values are discarded
# Submission note — density-shaped terminal ladder (sparse band priced down)

## Context and goal

Task: `docs/plans/live-performance-20260911/matrices-deepseek-task.md` — produce
independently accepted, frontier-improving submissions for the Yukon benchmark
`layr-labs/matrices-fast` (fill-reducing elimination orderings; score = weighted
mean of per-bucket geomean flop ratios vs feral-AMD, lower is better; buckets
`lt_1k` 0.30 / `1k_10k` 0.30 / `gt_10k` 0.40; `minScoreImprovementBips = 1`;
remote frontier at the time of writing 0.842857). Only `src/ordering` is editable
candidate content. Environment: this repo at `/home/frosty/angelX/.tmp/
live-performance-20260911/matrices-deepseek`, base commit `ab30c0e`, cargo/rust
release builds, the harness's own bubblewrap worker sandbox for the official
local runs, and the test-only `probe` module
(`cargo test --release -p ssi-candidate-worker --offline --locked --
--ignored --nocapture --test-threads=1 probe_timing_and_score`, with
`CARGO_TARGET_DIR=target/probe`) for per-row timing and score.

## Baseline and the failures that frame this change

* Frontier `ab30c0e`: dev `SCORE = 0.792436` (147/108/45 rows per bucket,
  geomeans 0.8874 / 0.8397 / 0.6856), worst in-process `order()` 1.085 s.
* Three prior ladder submissions were killed by the grader at the per-matrix
  wall-clock cap, all in the same 99–114 s band of the hidden run:
  `c13df7a2` (two ungated 2e8 draws) 114 s, `bc0e0b6c` (tiered 2×2e8 / 5e7)
  112 s, and `436d52d2` (one flat 2e8 draw, the immediately preceding
  submission) 107 s — Actions run `34683124829`, scored run 08:26:37.43 →
  `RUN FAILED: hidden matrix: order() exceeded the 2.0s per-matrix cap and was
  killed` 08:28:24.47. Ten failed benchmark jobs from five different solvers sit
  in the same band, while the per-row adds of those three builds differ by
  2–4×; the cap is enforced by `watchdog::run_capped` around the **whole
  sandboxed worker subprocess**, so spawn + pattern read + `order()` + write all
  count (`src/main.rs:74`, `src/watchdog.rs:111`).

## Hypothesis chain

1. The draw's *value* is known and reproducible (flops are deterministic); its
   *price* was only ever measured by differencing two probe runs. If that
   estimator's noise floor is comparable to the price, every per-row price and
   worst-row claim made so far is unproven, and any window design built on it is
   arbitrary.
2. If the price is instead measured inside a single process, it should have a
   simple law (budget × a per-row constant), and that law decides *where* extra
   budget is cheap — which is where the ladder should spend.
3. If the sparse band's value saturates at a much smaller budget than the mid
   band's, the shipped flat 2e8 can be priced down on sparse rows for free, i.e.
   a rare change that improves the score **and** lowers the quantity the cap
   measures.

## Measurements (all reproducible from the committed logs)

**Noise floor of the old estimator.** Two back-to-back runs of the *same*
binary: `0184-probe-repro-a.log` / `-b.log`. Score and all 300 `COUNTS` flop
counts byte-identical (0.792215); per-row times differ by median 0.0098 s, p90
0.040 s, max 0.156 s; printed `WORST order()` 1.196 s vs 1.118 s. Corpus
`order()` totals 125.3 s vs 127.8 s. Across the 21 archived probe tables of this
session, `acopf_case9241pegase_qcqp` (n=313 068, never inside any gate) reads
0.4846 s … 1.196 s — a 2.5× spread on an untouched row; on rows that received no
draw in any run, cross-run spread exceeds 0.1 s on 267/300 rows.

**In-run price instrument.** The probe prints the pipeline's elapsed time at the
ladder gate (before the draw) and the row's `order()` total; their difference is
the draw's own price, with no cross-run subtraction. It repeats to median
0.0010 s / p90 0.0051 s / max 0.015 s between the two runs above. Results: one
2e8 draw = 0.0537 s mean / 0.0733 p90 / 0.0957 max over 238 in-gate dev rows
(12.8 s corpus-wide); one 5e8 draw = 0.1396 / 0.1956 / 0.2625 (33.5 s); the
20 m/50 m/100 m rungs read 0.0068 / 0.0149 / 0.0275 s mean, i.e. the price is
**linear in the budget** at ~0.00027 s per 1e6 ops. It is **density-shaped, not
size-shaped**: sparse (`nnz/n < 3`) 0.315 ns/op, mid 0.275, dense
(`nnz/n >= 12`) 0.150; `corr(price, nnz/n) = −0.445`, `corr(price, nnz) =
−0.398`, `corr(price, n) = −0.123`. The sparse band therefore holds both the
dearest per-row draws (max 0.0957 s at 2e8) and the corpus's worst per-row add.

**Saturation of value in the sparse band.** Splicing per-row deterministic
results (`COUNTS` flops) of `0175-budget-50m.log` onto the sparse rows of this
build's own log predicts a dev score of ~0.79221; the built and measured result
is 0.792212 with all three sparse movers kept (`rsyn0820m04m`, `rsyn0830m04m`,
`rsyn0840m02m`). The mid band's ten movers need the full 2e8; the dense band
keeps 2e8 as well because it is the cheapest per op there.

## Implementation and exact result

`src/ordering/mod.rs`, terminal stage only, inside the existing
`n <= 12_000 && nnz <= 200_000` window: `budget = 50_000_000` when
`nnz < 3 * n`, else `200_000_000`, same seed `0x9E37_79B9_7F4A_7C15`, same seed
from the finished incumbent, same strict acceptance `f < best_flops` (so no row
can regress). Gate is a monotone predicate on `nnz` and `n` — no identity,
name, or corpus-membership selection, no hidden-corpus access, no scorer or
sandbox changes.

| build | dev SCORE | vs frontier 0.792436 | price mean | p90 | max | corpus add |
|---|---|---|---|---|---|---|
| flat one 2e8 (previous submission) | 0.792215 | −2.21 bips | 0.0537 s | 0.0733 s | 0.0957 s | 12.8 s |
| **density-shaped (this)** | **0.792212** | **−2.24 bips** | **0.0414 s** | 0.0717 s | **0.0902 s** | **9.9 s** |

Exactly three rows change flops versus the flat build (`rsyn0830m04m` 171801 →
171821, `rsyn0820m04m` 152992 → 152996, `rsyn0840m02m` 45077 → 45000 — the third
more than pays for the first two). The sparse band's 66 rows go 0.0629 →
0.0166 s mean and 0.0957 → 0.0375 s max. Verification: the test-only probe on the
production path reports `SCORE = 0.792212`, worst `order()` 1.168 s; the official
sandboxed local harness (`bash scripts/local-candidate-build.sh && cargo run
--release`) completes **300/300 matrices**, `score.json` 0.792212, fill
0.924447, `results.tsv` row `1789202595` — matching the probe exactly.

## Caveats and course corrections

* This submission does **not** claim to have fixed the cap. The three kills sit
  at 107/112/114 s while their per-row adds differ 2–4×, so the kill position
  looks like a property of the corpus and the grader box, not of the ladder's
  price; what is controllable is the price, and this build spends 25 % less of
  it. A cheaper draw is a strictly better bet under that uncertainty, not a
  proof of survival.
* Earlier in the session the flat build's "max add 0.237 s" and "worst row
  1.135 s vs 1.085 s" claims were produced by cross-run differencing and are
  inside the noise floor measured above; they are superseded by the in-run
  figures here.
* The seed sweep (A 0.792215 / B 0.792343 / C 0.792318) is dev-corpus luck and is
  not presented as generalization: only the *price* law and the *saturation*
  observation are structural claims.
* Local wall-clock verdicts remain unreliable on this shared host (contention
  caused unrelated local cap FAILs during the session); the local run is used
  here for the score and the 300/300 determinism check, not for cap safety.

## Next steps

Re-read the verdict of this submission (and the kill position if it fails): if
the position moves with the price, the next lever is a per-row budget shaped by
the price law (predictable from `nnz/n` before the draw runs) to equalize the
add across bands, and an independent test would be repeating the same binary
twice to keep every timing claim inside the 0.001 s resolution of the in-run
instrument.
