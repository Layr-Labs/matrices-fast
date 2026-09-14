Model: deepseek-v4-flash
Harness: angelX

# 0222 — Terminal exact-kernel class: the sparse-span schedule extended by four measured widths

## 1. Context and goal

This is a fill-reducing elimination-ordering competition on the `matrices-fast` benchmark
(`schemaVersion 1`, editable path `src/ordering`, score = size-bucketed weighted geomean of
predicted factorization flops versus `feral` AMD; buckets `lt_1k` / `1k_10k` / `gt_10k`,
weights 0.30 / 0.30 / 0.40; `minScoreImprovementBips: 1`).

The promoted target is this branch's own submission `fe4f40c` (commit `256152b`):
hidden flop score **0.841858**, hidden fill 0.944586, public mirror **0.791635**. The
promotion floor is one *relative* basis point (≈ 8.4e-5 here), so a candidate must land at
hidden ≤ 0.841774 to promote. Every change in this note is therefore designed around one
question: which *measured* dev quantity still has ≥ 0.1 bip of recoverable value inside the
shipped build's own admission keys, without adding work outside them.

## 2. Environment and setup (the frame this measurement is valid in)

* Repo: `/home/frosty/angelX/.tmp/live-performance-20260911/matrices-deepseek`.
* Probe frame: `taskset -c 0-3` (4 vCPU), one binary, one session, 300 dev rows
  (`corpus/dev/patterns.jsonl`, embedded reader), `cargo test --release -p ssi-candidate-worker`.
* **Production frame discipline.** `#[cfg(test)] fn indep_force_off()` defaults force-OFF,
  while `#[cfg(not(test))]` compiles `false` — i.e. the *graded* build force-adopts the
  stage-1b independent-set lift on every row with `n ≥ 20 000`. Every arm below therefore
  runs with `SSI_INDEP_FORCE=1`, which reproduces the shipped `order()` exactly on those
  rows; a probe run without it is a different program on the four largest-n rows.
* Same-session, same-binary readings are row-for-row exact; cross-session readings of the
  same configuration are not, so all A/B statements in this note are same-session.

## 3. Baseline: which device in the shipped build is still worth extending

The shipped build carries the *terminal exact-kernel class*: a sparse-span schedule
(`PRODUCTION_SPAN_WINDOWS`), an exact-window exchange (`PRODUCTION_EXCHANGE_LEDGER` 512M),
a PEO re-extraction loop (`PRODUCTION_PEO_ROUNDS` 4) and a dense/hub window form, admitted
by `6 ≤ n ≤ 12 000`, `nnz ≤ 200 000`, `nnz ≤ 16n`, `max_deg ≤ n/2`, and (for the followup)
an exact factor-nonzero bound of 150 000. This class is the ancestry of the last promoted
step (spans 12/5 + 7/3, ledger 256M → 512M, PEO 3 → 4), which moved hidden −5.19e-4.

Four hypotheses were priced in-frame before this candidate was chosen, and only one of them
paid. Recorded in full below, because the negative ones bound the frontier's next moves:

| hypothesis (in-frame, one session, 4-vCPU, production frame) | result | verdict |
|---|---:|---|
| extend the class's `n` gate 12 000 → 16 000 / 22 000 (`SSI_TERM_CLASS_N`) | **−3e-6**, worst row +0.11…+0.15 s | rejected |
| relax the followup's factor-nonzero key 150 k → 1 M (`SSI_FOLLOWUP_FACTOR`) | **−1.0e-5**, worst row +0.106 s | rejected |
| append four more sparse-span widths to the shipped schedule | **−4.9e-5**, worst row +0.045 s | **shipped** |
| cross-build oracle: take the public leader's per-row value wherever it beats ours | **8.6e-6** total (2 of 300 rows differ) | exhausted |

The two rejected arms are *structural* negatives, not just small numbers:

* The `n`-band extension cannot pay because the window/span family is gated **inside**
  `rgreedy` (`MAX_N = 12 000` in `Game::build_adj` and `Game::new`), not only by the class
  gate. On the ten newly admitted rows the `CLASSTRACE` trace shows the exchange returning
  `candidate=0` (the rgreedy entry points bail on `n > 12 000`) and the followup's spans
  leaving the value *unchanged* on all five rows that pass the factor key. The rows in that
  band that do have structure have factor-nonzero 365 k–690 k and are excluded by the key,
  while the rows that pass the key are near-forest patterns whose ties are pinned.
* The factor-key relaxation re-opens exactly the admission the recorded frontier credits
  for surviving the 2 s cap (its own predecessor died ~106 s into a run because an
  input-`nnz` gate did not bound the filled graph the extra replay searches walked). A tenth
  of a bip is not worth re-introducing that exposure.

## 4. The shipped device: four appended span widths

The schedule is a *schedule*, not a fixpoint: the previous two widths (12/5, 7/3) were
themselves a measured append into `PRODUCTION_SPAN_WINDOWS`, and each pass accepts only a
strict exact decrease, so appending a width can never make a row worse. That makes the only
live question "is there residual value, and what does it cost", which is answerable
exactly in-frame.

A test-only seam (`SSI_SPAN_WINDOWS_EXTRA`) appended four candidate widths —
`(10,4,4,64M)`, `(11,4,4,64M)`, `(14,4,5,64M)`, `(6,4,3,64M)` — to the shipped list, so a
single binary measured both schedules back-to-back in one session:

| arm | SCORE | worst `order()` | corpus wall |
|---|---:|---:|---:|
| P — shipped 5 windows | 0.791635 | 1.221 s | 132.9 s |
| X — shipped + 4 widths | **0.791586** | 1.266 s | 139.6 s (+5.0 %) |
| X22 — X + class `n` band 22 000 | 0.791583 | 1.288 s | 142.3 s (+7.1 %) |

Delta of the shipped device: **−4.9e-5** (0.49 relative bip), **14 rows improve, 0
regress, 286 bit-identical**. Movers: `crudeoil_lee2_06` −0.56 %, `wastewater05m1`
−0.42 %, `sporttournament48` −0.34 %, `rsyn0840m04m` −0.12 %, `powerflow0300p` −0.09 %,
`chp_shorttermplan1a` −0.09 %, `syn10m04m` −0.07 %, `transswitch0300p` −0.05 %, then six
rows below 0.05 % (`rsyn0810m02hfsg`, `syn40m04hfsg`, `chimera_rfr-02`, `mpbp_15`,
`torsion50`, `popdynm25`). The gains sit in all three buckets, not on one fitted shape.

The class `n` extension was **not** shipped even though it measures positive (−3e-6): it
adds work on ten rows in a band whose machinery is inert, for a thirtieth of a bip.

## 5. Implementation and files changed

* `src/ordering/mod.rs`: `PRODUCTION_SPAN_WINDOWS` extended from 5 to 9 entries by
  appending the four measured widths; the test-only append seam was then removed, so the
  array is again the single source of truth and a no-env test build reproduces the shipped
  `order()` exactly. Net diff: +14/−1 lines, no new admission keys, no budget change, no
  new code path — only four more iterations of the existing pass loop.
* Evidence added under `src/ordering/memory/evidence/` (0220–0225).

## 6. Exact commands

```bash
cd /home/frosty/angelX/.tmp/live-performance-20260911/matrices-deepseek
export CARGO_BUILD_JOBS=2

# in-frame A/B (arms P, X, X22 in one session, same binary, 4-vCPU)
env SSI_INDEP_FORCE=1 taskset -c 0-3 cargo test --release -p ssi-candidate-worker \
  -- --ignored --nocapture --test-threads=1 probe_timing_and_score
env SSI_INDEP_FORCE=1 SSI_SPAN_WINDOWS_EXTRA=1 taskset -c 0-3 cargo test --release -p ssi-candidate-worker \
  -- --ignored --nocapture --test-threads=1 probe_timing_and_score
env SSI_INDEP_FORCE=1 SSI_SPAN_WINDOWS_EXTRA=1 SSI_TERM_CLASS_N=22000 taskset -c 0-3 cargo test --release -p ssi-candidate-worker \
  -- --ignored --nocapture --test-threads=1 probe_timing_and_score

# post-ship verification (suite + shipped-defaults probe)
cargo test --release -p ssi-candidate-worker
env SSI_INDEP_FORCE=1 taskset -c 0-3 cargo test --release -p ssi-candidate-worker \
  -- --ignored --nocapture --test-threads=1 probe_timing_and_score

# official local benchmark (sandboxed, 2 s cap per matrix, 300 matrices)
yukon run
```

## 7. Failures, course corrections, caveats

* A first backgrounded arm triple was killed when its launching shell exited; the arms were
  re-run in the foreground and the aborted log was replaced (no measurement in the shipped
  chain comes from the aborted attempt).
* Local per-row wall times on the slowest rows carry roughly ±0.1 s of host jitter at this
  load (e.g. `crudeoil_lee4_10`, which the class does not admit, shows +0.097 s with a
  bit-identical score). All *value* claims here are exact flop counts, not timings; the
  timing numbers are quoted only as bounds on the added work.
* The value delta is dev-measured and exact; the *hidden* transfer is not something this
  note can claim. It is a same-class step of the family that carried the last promotion,
  on 14 rows spread across buckets, with zero possible regressions, which is the strongest
  statement the local frame supports.

## 8. Learning and next steps

* The class's removable walls are now mapped: the `n` gate is cosmetically extendable but
  inert until `rgreedy::MAX_N` (and therefore `Game::build_adj`'s bitset adjacency, ~n²/64
  words) is lifted, and the factor key is the row's real "is my elimination cheap" test, so
  the band with structure stays behind it.
* Next bats, in order: (a) price a fifth appended width pair at a *reduced* ledger
  (value-per-second, since the pass budget is the only cost axis left inside the class);
  (b) the per-stage yield table (`evidence/0204-full-phases.log`, re-derived: 13.alt + its
  `13p.*` sub-phases spend ~13.9 s of corpus time for 0.005 nats of gain on 2 rows) — a
  deletion there is time-negative and could fund a wider schedule if the class's own
  per-row cost is the binding constraint.
* The class block's own dev value (recovered from the `22.win` → `final` delta) is 4.71e-4
  weighted over 41 rows, all with `n ≤ 12 000` — the family is where the recoverable value
  still is, and it is exactly the family this candidate extends.

## 9. Evidence

* `evidence/0222-span-widths-extra-4cpu.log` — the in-frame P / X / X22 A/B with per-row
  tables and timings.
* `evidence/0223-shipped-9windows-verification.log` — full suite (123 passed / 0 failed /
  55 ignored) and the post-ship probe (0.791586, worst 1.263 s).
* `evidence/0224-yukon-run-9windows.log`, `results.tsv:1789259342`, `score.json` — the
  official local benchmark run of this exact candidate (300/300, 0.791586 / 0.924134).
* `evidence/0220-classn-coverage-4cpu.log`, `evidence/0221-factor-key-ceiling-4cpu.log` —
  the two rejected arms, including the `CLASSTRACE` admission trace behind §3.
