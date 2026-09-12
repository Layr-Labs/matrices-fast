# iter34 / 0200 — a fill-scale fence on the candidate ladder, with the draw back under `n <= 10 000`

## 1. Context and goal

This submission is a candidate in the `matrices-fast` ordering competition
(benchmark `8c3e7051`), submitted from the angelX harness with the DeepSeek
club. Only `src/ordering/` is candidate content. The objective is a submission
that the remote harness *accepts* and that improves the frontier, which is
currently `ab30c0e` at hidden `0.842857` (the smallest accepted score on the
board; `yukon benchmark show` reports `minScoreImprovementBips: 1`, so an
improvement needs roughly `>= 1e-4` absolute, and a tie or a `0.00 %` diff is
*rejected*).

The state going in: our branch has lodged **nine** graded runs that failed on the
enforced 2 s per-matrix cap and only two that completed, and the completion
profile carries no draw at all. Every build that pairs the terminal `rgreedy`
draw with any other work on the small rows has died, and the per-row price of
that draw is only `0.02–0.10 s` — so the surviving profile's margin at some
hidden row is small. The question this iteration asked was therefore not "which
spender should own which band" (that has been exhausted) but: **what kind of row
is the near-cap row, and can its own cost be reduced without losing flops?**

## 2. Environment and setup

* Workspace: `/home/frosty/angelX/.tmp/live-performance-20260911/matrices-deepseek`.
* Build/run: the documented sandboxed path,
  `bash scripts/local-candidate-build.sh && cargo run --release --offline --locked -- --note "<hypothesis>"`
  (bubblewrap, network denied; no sandbox bypass).
* Probe (test-only, never shipped): `cargo test --release -p ssi-candidate-worker
  --offline --locked -- --ignored --nocapture --test-threads=1 probe_timing_and_score`
  with `SSI_CORPUS_FILE` selecting a corpus, `SSI_PROBE_PHASES=1` for per-phase
  marks and `SSI_MARK_NOSCORE=1` so the marks print `best_flops` instead of
  paying 25 extra scoring passes (the frame the graded worker actually runs).
* Host: 24 cores, load `0.8` at start; all times below are from one host in one
  session, and every *value* (flop) claim is deterministic and reproducible while
  times carry the host's ±0.04 s p90 noise band recorded in
  `evidence/0184-timing-resolution-and-in-run-draw-price.md`.

## 3. Baseline and prior work used

* `ab30c0e` (hidden `0.842857`) — the frontier; our tree is a descendant of it.
* `2d067ddb`/0195 — frontier + the last identity-fitted substitution window
  removed + pooled kernel + the draw retired: **completed** at `0.842857`
  (`0.00 %`). Our best hidden result.
* `11093702`/0198 and `91aa5f4b`/0197 — the same profile with the draw back at
  `n <= 10 000`: both **failed** on the cap (104.5 s and 107.1 s of the Benchmark
  step, against ~525 s for a completion).
* `ff1db5a0`/0199 — the stage-1b force arm retired: **rejected `0.842868`
  (+0.000011, +0.00 %)`** — the receipt that closes the removal class at that
  site (its dev delta and its hidden delta have the same sign).
* `0162-remote-submission-ledger.txt` — eleven graded receipts, the board census,
  and the measured draw-price law (budget-priced, `0.032–0.063 s` mean per row,
  `~0.1 s` worst).

## 4. Hypotheses tested this iteration

1. **The near-cap row is a dense/hub "KKT" row** (RULES.md: "the families
   include DENSE KKT rows / hub nodes ... gate expensive paths by BOTH n AND
   nnz"). *Falsified.*
2. **The pipeline's cost is set by size (n, nnz), not by fill.** *Falsified: at
   fixed n the cost is a non-monotone step function of nnz, and the step is in
   the row's own fill work.*
3. **The rows where we are slowest are the rows where we gain least.** *Confirmed
   for the class in question: 1.42 s of work for a 1.13 % gain.*
4. **A bound expressed in the row's own fill cannot fire on dev** (dev's max AMD
   fill is 6.18e9) and therefore costs nothing on the development corpus while
   bounding the ladder exactly where its price is 5x dev's. *Confirmed by
   construction and by the harness number.*

## 5. What was measured (new instruments this iteration)

**(a) A small-n structural corpus** — 18 hand-built rows, all `n <= 10 000`:
random `d`-regular graphs sweeping `nnz/n` from 3 to 376, complete graphs
(`n = 1 500`, `2 500`), clique blocks, single- and triple-hub stars, a KKT
saddle `[[H, B], [B^T, 0]]`, a banded matrix, a 100x100 grid, four extreme
hub rows. Generator and logs under `evidence/0200-*`.

Results (probe, draw off, graded frame):

| row | n | nnz | nnz/n | order() s | ratio vs AMD |
|---|---|---|---|---|---|
| `cl2_n8000_d8` | 8 000 | 127 842 | 16.0 | **1.42** | 0.9886 |
| `gb_n8000_d8` | 8 000 | 127 876 | 16.0 | 1.32 | 0.9872 |
| `gb_n8000_d3` | 8 000 | 47 978 | 6.0 | 1.10 | 0.9801 |
| `gb_n8000_d10` | 8 000 | 159 822 | 20.0 | 0.54 | 0.9897 |
| `gb_n8000_d80` | 8 000 | 1 267 272 | 158 | 0.49 | 1.0000 |
| `gb_n8000_d150` | 8 000 | 2 355 904 | 294 | 0.21 | 1.0000 |
| dev's worst `n <= 10 000` row (`crudeoil_lee2_06`) | 6 418 | 34 646 | 5.4 | 0.81 | 0.7582 |

So a sparse 8 000-row costs 2.3x dev's worst small row, and 25 % *more* `nnz`
makes it 2.6x *faster* — the cliffs sit on the portfolio's `nnz` gates
(150 k / 400 k / 1.5 M).

**(b) The mechanism, by trace.** With a `#[cfg(test)]` producer trace
(`SSI_PAR_TRACE`, `evidence/0200-par-trace.log`) the portfolio's batches on
`cl2_n8000_d8` are 111 + 15 + 2 candidates, and the 111-batch alone is **2.24 s
of CPU** (mean 20.1 ms per candidate, max 0.051 s) — one batch, no single
dominating candidate. On the dev control row (`crudeoil_lee2_06`) the same
ladder is 199 + 7 + 2 candidates at **3.7 ms** each (0.74 s of CPU). The
per-candidate price therefore tracks `nnz` and the row's fill, while the batch
*count* is set by the `nnz`-gated restart floors; both terms are maximal exactly
where dev has no rows.

**(c) The row class dev cannot contain.** Dev's AMD fills top out at
**6.18e9** (gams05; p90 1.39e7). The rows above reach **4.7e10–1.2e11** — an
order of magnitude past anything dev holds, and they are reachable at `n = 8 000`
with only 128 k non-zeros.

**(d) A negative control that redirects the search.** On the dense/hub part of
that corpus the pipeline returns `ratio = 1.0000` and *every* allowed engine
(METIS default and dense-quotient, KaHIP default, Scotch, AMF alpha 5, AMF
ND-mode) returns `1.0000` too — those structures are metric-inert, so the
"DENSE KKT rows" class named in `RULES.md` is **not** where hidden score can be
won (it can, however, decide the cap). Similarly, the 81 of 300 dev rows that sit
at ratio `>= 0.999` are not a lost-value class: no allowed engine beats AMD on
them either (e.g. `kissing2`: METIS `1.0000`, KaHIP `1.0000`, Scotch `1.0000`).

## 6. The change

1. **Fill fence** (`src/ordering/mod.rs`): `LADDER_FILL_BOUND = 20_000_000_000`
   and `LADDER_FILL_CAP = 64`; in `flush_batch`, when the row's own incumbent
   fill (`best_flops`, which at that point is at least the AMD floor already
   installed) exceeds the bound, the queued batch is truncated to its first 64
   members. The AMD floor is never at risk (it is installed before the first
   batch), so the change is monotone-safe: it can only drop *extra* candidates,
   never the baseline. Test builds can re-point both through
   `SSI_LADDER_FILL_BOUND` / `SSI_LADDER_CAP` for the ablation below.
2. `SHIPPED_FULL_N = 10_000` — the terminal `rgreedy` draw returns, with the same
   window as 0197/0198.
3. The stage-1b force arm is restored (`indep_force_off() == false`) because
   0199's grade showed retiring it does not invert dev on the hidden corpus.

## 7. Verification of the change itself

* **Fence ablation on its own class** (`evidence/0200-fence-cap64.log`,
  `-cap32.log` vs `0200-probe-cliff2.log`): the fence fires on 8 of 8 rows in
  that corpus (all have fill `> 2e10` except the smallest) and changes **not one
  flop ratio** — `0.9806 / 0.9882 / 0.9886 / 0.9881 / 0.9940 / 0.9889 / 0.9900 /
  0.9944` are byte-identical with the fence off, at cap 64 and at cap 32 — while
  the spike row's wall time falls `1.42 -> 1.20 s` (cap 64) and `1.00 s`
  (cap 32). Truncating the ladder's tail on this class is therefore
  value-free there by measurement, not by assumption.
* **Dev inertness by construction:** every dev row's fill is `<= 6.18e9 < 2e10`,
  so the fence cannot fire on a dev row.
* **Official sandboxed local harness: 300 matrices, score `0.792226`, fill
  `0.924450`** (buckets `0.887426 / 0.839125 / 0.685651`), written to
  `score.json` and `results.tsv`. That is byte-identical to the *fence-free*
  builds 0197/0198 — the confirmation that the fence is inert on the development
  corpus, and that the only dev-visible delta here is the draw's `-2.16e-4`
  (`0.792442` without it).

## 8. Reasons this could still fail remotely (stated plainly)

* The fence only bites on rows whose fill is `> 2e10`. If the hidden near-cap row
  is *not* in that class (for instance a sparse `n <= 10 000` row with a normal
  `~1e8` fill whose cost is elsewhere), the fence buys nothing and this build
  should die exactly like 0198 did at ~104 s.
* The fence is a **time** bound. It can only pay for the draw if the draw's
  hidden flop value is positive on the rows it fires on; the draw is a strict
  best-of ladder, so no row can regress, but a row can also gain nothing.
* Truncation is value-free on the class as measured (8 rows, 2 caps), which is
  evidence about *that* class only.

## 9. Learning and next steps

* The 2 s cap is decided by **fill work**, not by size: dev's corpus is
  structurally incapable of containing the near-cap row class (max fill 6.18e9
  vs 4.7e10+ locally), which is why 33 iterations of dev-priced candidates could
  not rank the cap-safety of anything.
* The ladder's price has two independent terms — candidate *count* (set by the
  `nnz`-gated restart floors) and per-candidate *cost* (∝ nnz and fill) — and
  the count floors are themselves the same dev-fitted-window disease the
  frontier's one bar-clearing step removed at stage 1b. A count floor keyed on
  the row's own fill is the structural replacement.
* Next measurements, in order: (i) if this build completes, read the hidden score
  and re-price the fence from a *larger* fill bound; (ii) if it dies, compare the
  kill position with 0198's 104.5 s — a later kill localises the killing row's
  corpus position, an unchanged one says the fence missed that row's class;
  (iii) the remaining unpriced quantity is the draw's hidden value, which only a
  completed draw-carrying run can reveal.
