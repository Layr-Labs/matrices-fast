# 0192 — the draw takes the chain's whole scope; the chain is confined to the survivor's band

Task: `docs/plans/live-performance-20260911/matrices-deepseek-task.md` — Yukon
benchmark `layr-labs/matrices-fast` (fill-reducing elimination orderings; score =
weighted mean of per-bucket geomean flop ratios vs feral-AMD, lower is better;
buckets `lt_1k` 0.30 / `1k_10k` 0.30 / `gt_10k` 0.40; `minScoreImprovementBips =
1`; remote frontier 0.842857 as of this submission). Only `src/ordering` is
editable candidate content. Model deepseek-v4-flash, harness angelX (stamped by
the CLI from the live run identity). Local checkout at
`/home/frosty/angelX/.tmp/live-performance-20260911/matrices-deepseek`, base
commit `ab30c0e`. Nothing about the scorer, purity gate, corpus, tests or sandbox
was touched; the candidate is a pure `src/ordering` change and every number below
comes from the harness's own probe or from the sandboxed local run.

## 1. Where the work stood

Four iterations of this workspace have been spent on one question: the terminal
ladder (a deterministic, fixed-seed budgeted local search over the incumbent
ordering, run last inside `order()`) is worth **−2.27e-4 dev** (0.792439 →
0.792212), but every build that carried it into the hidden corpus died on the
grader's **2.0 s per-matrix cap** — seven kills now, at 104.4 / 105 / 107 / 112 /
114 s into the Benchmark step, against a completing run of ~500 s. Only one
draw-carrying build has ever finished a hidden run: `71c2c5fe`, at 0.843153,
+2.96e-4 *worse* than the frontier.

The successive explanations for the kills have each been measured and falsified:

* "the draw's *window* is too wide for the cap" — priced down, window 12 000 →
  10 000 → 12 000 with chain-skip bands, still killed;
* "the two spenders overlapped on the same rows" — made row-disjoint in 0191
  (ladder 12 000, chain skipping 10 000–12 000 and cheap above it), still killed
  (`7c76ef6a`, the run immediately before this one);
* "one spender per row above the draw's window" — falsified by `55d9ed93`, which
  was *cheaper than the surviving frontier* on every dev row above n = 10 000
  (1e6 allowance against the frontier's 4e6) and died anyway.

What survives all eleven receipts is much simpler, and it is the hypothesis this
submission acts on.

## 2. The receipt table (the whole argument)

| build | draw | chain work above n = 10 000 | outcome |
|---|---|---|---|
| frontier `a9905f20` (promoted) | no | yes, 4e6 allowance to n = 50 000 | **completed, 0.842857** |
| `71c2c5fe` | yes, n ≤ 12 000 | **none** | **completed, 0.843153** |
| `c13df7a2` | yes (two ungated 2e8 draws) | yes | killed 114 s |
| `bc0e0b6c` | yes (tiered) | yes | killed 112 s |
| `436d52d2` | yes (one flat 2e8) | yes | killed 107 s |
| `6ad8cc5e` | yes, n ≤ 12 000 | yes, 4e6 to 50 000 | killed 105 s |
| `55d9ed93` | yes, n ≤ 10 000 | yes, 1e6 above 10 000 | killed 104 s |
| `e5a3c6b4` | yes | yes, cheaper than the frontier's | killed 104 s |
| `7c76ef6a` (0191 shape) | yes, n ≤ 12 000 | yes, 1e6 above 12 000 | killed (this run's predecessor) |

Every *draw-free* build completes — not just the frontier: the submission list of
the last day shows five completed/rejected-but-scored runs from another solver at
0.842833–0.842835 and three at 0.842857–0.842858, all draw-free. Every build that
pairs the draw with chain work above n = 10 000 has died 104–114 s into the
Benchmark step. The only draw-carrying build that ever finished had **no chain
above n = 10 000** — and it is byte-identical to the killed `55d9ed93` below
n = 10 000, so the separation is not a per-row price and not a per-row band
ownership.

Why the deaths are structurally plausible even though our builds were *cheaper*:
a kill needs exactly one hidden row inside the draw's window whose own cost is
within the draw's add of 2.0 s, and the draw is single-threaded (`rgreedy`
contains no `parallel::` call at all; the only threaded path is
`parallel::generate_batch`, hard-capped at `PAR_MAX_THREADS = 4` so local timing
cannot silently model a different grader), so its per-row add is a bounded
0.03–0.095 s measured in-run. Nothing about the cost ordering is controllable
from outside; what *is* controllable is whether a second, unbounded-in-principle
spender shares the process with it.

## 3. The change (two constants, one deleted mechanism)

1. `PEO_ALT_MAX_N` 50 000 → **10 000**. The alternate-seed chain keeps the
   frontier's own gate — `16 ≤ n ≤ MAX_N && n + nnz < 4_000_000 && !peo_alt_danger`
   — and the frontier's single 4e6 allowance (the `PEO_ALT_WIDE_N` /
   `PEO_ALT_LEDGER_WIDE` / `PEO_ALT_SKIP_LO` / `PEO_ALT_SKIP_HI` machinery of
   0191 is deleted with it, so the chain's gate is again exactly the frontier's
   shape with a narrower scope). Its scope is the band the one surviving
   draw-carrying build used.
2. The ladder window 12 000 → **50 000** (`window_n` and `SHIPPED_FULL_N`; the
   `nnz ≤ 200 000` gate and the 5e7-sparse / 2e8-otherwise rungs stay). The draw
   now covers the band the chain gave up. The draw's acceptance is
   score-monotone — it replaces the incumbent only on a strict flop improvement
   (`if f < best_flops`) — so on a hidden row it can never lose score, where the
   chain's truncation demonstrably could (that is the ≥ 2.96e-4 the 0189 receipt
   attributes to the chain's wide-band scope).

The design rule this encodes: **one spender per row, and the spender that cannot
lose score is the one that gets the wide band.**

## 4. Measurements

Probe: `CARGO_TARGET_DIR=target/probe cargo test --release -p
ssi-candidate-worker --offline --locked -- --ignored --nocapture
--test-threads=1 probe_timing_and_score` (300 dev rows, per-row `order()` and
per-bucket score). The window/full_n knobs (`SSI_TERM_WIN_N`, `SSI_TERM_FULL_N`)
and the ladder-rung override (`SSI_TERM_LADDER`) are `#[cfg(test)]` only and
cannot reach a shipped build.

* **Window 12 000 vs 50 000, shipped chain scope**: 238 vs **263 drawn rows** and
  **not one flop moves** — `SCORE = 0.792212` both ways, per-bucket geomeans
  identical to four decimals (lt_1k 0.8874, 1k_10k 0.8391, gt_10k 0.6856).
* **Chain 4e6-to-50 000 vs 1e6-above-10 000 (draw off)**: 0 of 300 rows change
  flops (0.792439 both ways) — so the chain's wide-band work has *exactly zero*
  dev yield in the strongest sense, and the wide band is dev-invisible for both
  spenders. Dev therefore cannot rank shapes there; the receipts are the only
  evidence, which is why the shape follows them.
* **Wide-band cost**: over the 24 dev rows of 12 000 < n ≤ 50 000 the candidate
  spends 14.79 s against the frontier's own 16.73 s (**−1.94 s, −11.6 %**) — the
  draw is a *cheaper* spender there than the chain's 4e6 rounds it replaces, and
  the worst in-window row is 1.127 s probe frame against the frontier's 1.160 s
  base profile.
* **Draw window curve** (new this run, all four points measured): no draw at all
  0.792439 → window 3 000 0.792383 → window 5 000 0.792364 → window 12 000
  0.792212 → window 50 000 0.792212. So 75 % of the draw's dev value sits in
  5 000 < n ≤ 12 000, and a "safe narrow window" build (draw confined to the
  cheapest band) can only reach 0.56 dev bips — below the 1-bip bar — which is
  why the band could not simply be abandoned.
* **Rung halving, reproduced**: `SSI_TERM_LADDER=100000000` on the shipped
  profile gives 0.792316 against 0.792212 at 2e8 (matches the archived 0190
  measurement to the digit).
* **Official sandboxed local harness** on this exact build:
  `bash scripts/local-candidate-build.sh && cargo run --release --offline
  --locked` → **300 matrices, 0.7922 score, fill 0.9244**, 300/300 OK, identical
  to the previously shipped dev score.

## 5. Failures and course corrections inside this iteration

* A first attempt to measure "the draw costs nothing above 12 000" set only
  `SSI_TERM_FULL_N`; the outer `window_n` gate still capped the draw at 12 000,
  so the run measured nothing. Re-run with both knobs produced the real point
  (263 drawn rows, same score).
* The closed-loop alternative was tried on paper and rejected on evidence: the
  harness runs the worker twice per matrix and fails the run on any difference
  (`src/main.rs`: `if perm1 != perm2 { "nondeterministic ordering (two runs
  differ)" }`), so a wall-clock-conditioned budget or guard is excluded by
  design. Any cap defence must be a deterministic function of the row.
* A deterministic "own-cost gate" (skip the draw on rows whose own base cost or
  fill is high) was tested against the dev data and dropped: on the 262 in-window
  dev rows the base `order()` cost correlates *positively* with `log n` (+0.78),
  `log nnz` (+0.67) and `log flops` (+0.66), and the draw's 13 value movers are
  themselves among the slowest in-window rows (`crudeoil_lee2_06` base 0.880 s is
  the third slowest of 262; it alone carries 58.6 % of the movers' log-delta).
  No n/nnz/fill threshold separates value from risk there, which is why this
  submission moves the *band* rather than gating the *row*.

## 6. Honest limits and what would falsify this

Dev is exactly neutral about the change (same score to the printed precision),
so the evidence for it is the receipt table plus the draw's monotonicity, not a
local score gain. Outcomes:

* if the run completes and the draw is at least as strong as the chain on the
  hidden wide-band rows, the chain's own measured wide-band value (≥ 2.96e-4)
  is recovered inside the only shape that has ever survived with a draw, and the
  submission should land at or below `0.843153 − 3.96e-4 = 0.842757`;
* if it completes but the draw is weaker there, it lands above the frontier and
  is rejected on score (a clean, publishable measurement of the two spenders'
  relative value on that band, which dev cannot provide);
* if it is killed, the "coexistence" reading is falsified too, and the remaining
  explanation is the one this workspace has not been able to test locally: the
  hidden corpus contains a row inside the draw's window whose own cost is within
  ~0.1 s of the cap under *any* profile, in which case no additive build can ever
  survive and the only path is a strictly time-negative change (a cheaper
  pipeline with equal output), which is the next candidate class in the ledger.

## 7. Next steps if this lands under the bar

1. The wide band's draw rung is the first knob (2e8 → 4e8 measured 1.1e-6 dev
   at the knee; the value is in the 5 000 < n ≤ 12 000 band, so a per-band rung
   ladder rather than a flat one).
2. Time-negative work on the value rows themselves (`1.portfolio` is 34.6 % of
   the 13 mover rows' base seconds, `4.subtree` 13.8 %, `9.reduce` 11.7 %): any
   second removed there is a second the draw can spend without opening the cap.
3. A second hidden receipt of the same shape with a different rung would price
   the draw's wide-band value directly, which is the missing number in every
   estimate above.
