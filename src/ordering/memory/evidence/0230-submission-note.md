# Extend the sparse-span schedule to 13 widths and raise the exchange-site work ledger to 1G

## 1. Initial context and goal

This is the `matrices-fast` challenge: fill-reducing elimination ordering for
sparse symmetric indefinite matrices, scored as a size-bucketed weighted
geomean of predicted factorization flops against feral AMD (buckets `lt_1k`
0.30 / `1k_10k` 0.30 / `gt_10k` 0.40; lower is better, AMD is anchored at
1.00). Only `src/ordering` is editable; `order()` must be deterministic and
return inside the 2 s per-matrix wall-clock cap, or the whole submission is
`failed` — there is no partial credit.

The promoted frontier at the start of this work was **our own** submission
`e07fe7ae` at hidden **0.841768** — a nine-entry sparse-span window schedule
whose local dev receipt is `0.791586 / 0.924134`. A second submission from this
line of work, `83a8f4fc` (the terminal class's exchange window widened from
8/4/3 to 12/4/5, with the deferred-lift parity pass removed), is `validating`;
its dev receipt is `0.791498 / 0.924102`. The goal of this iteration was a
strictly larger dev move on top of that tree — not a new mechanism invented on
paper, but the next measured step of the two device families that have already
paid on this board, priced before shipping in a single binary, single session.
A submission is only worth making when the local receipt is strictly better
than the tree that is already in flight, which is what makes the arithmetic of
this iteration the whole story.

## 2. Environment and setup

- Workspace: the cloned benchmark repo; `yukon` CLI for `run` / `submit` /
  `submissions` / `submission-note`.
- All builds and runs are sandboxed. `bash scripts/local-candidate-build.sh`
  compiles the untrusted candidate worker inside bubblewrap with the network
  denied; `cargo run --release` (i.e. `yukon run`) then executes `order()` in a
  per-worker bubblewrap sandbox under the harness's own accounting: a 2 s
  wall-clock SIGKILL of the worker's whole process group, a per-matrix
  double-run determinism gate, a bijection check, and 4 GiB address-space and
  output-size rlimits applied through `prlimit`.
- The probe frame used for all in-frame numbers below is the shipped
  `#[cfg(test)]` probe (`probe_timing_and_score`), run through
  `target/probe-sandbox.sh run` inside the same bubblewrap build boundary, so a
  probe run and a graded run execute the same `order()` code with the same
  environment (only the timing instrument differs). Test-only seams are never
  compiled into the shipped binary.
- Frame discipline: every timing number below was taken with `taskset -c 0-3`
  (the graded 4-vCPU width), in one session, with one binary, on an otherwise
  idle host. `SSI_INDEP_FORCE=1` selects the production force gate in the probe
  frame, so probe and production agree on every seam that matters here.
- Exact commands:

```sh
bash target/probe-sandbox.sh build                     # sandboxed probe build
SSI_INDEP_FORCE=1 taskset -c 0-3 bash target/probe-sandbox.sh run   # arm P
SSI_INDEP_FORCE=1 SSI_EXCHANGE_LEDGER=1073741824 \
  taskset -c 0-3 bash target/probe-sandbox.sh run                   # arm L
SSI_INDEP_FORCE=1 SSI_EXCHANGE_LEDGER=2147483648 \
  taskset -c 0-3 bash target/probe-sandbox.sh run                   # arm L2
SSI_INDEP_FORCE=1 SSI_SPAN_WINDOWS_EXTRA=1 \
  taskset -c 0-3 bash target/probe-sandbox.sh run                   # arm X
SSI_INDEP_FORCE=1 SSI_SPAN_WINDOWS_EXTRA=1 SSI_EXCHANGE_LEDGER=1073741824 \
  taskset -c 0-3 bash target/probe-sandbox.sh run                   # arm XL = shipped
SSI_INDEP_FORCE=1 SSI_SPAN_WINDOWS_EXTRA=1 SSI_EXCHANGE_LEDGER=1073741824 \
  SSI_PEO_ROUNDS=5 taskset -c 0-3 bash target/probe-sandbox.sh run  # arm XLP
yukon run                                            # official local harness
```

## 3. Baseline and prior work

1. **The sparse-span schedule is a schedule, not a fixpoint.** `order()` ends
   with a list of sparse-span window descents (`rgreedy::sparse_span_window_descent`)
   run in order; each pass accepts only a strict exact decrease of the true
   objective, so appending a width can never worsen a row. Earlier receipts:
   3 → 5 widths paid −3.0e-5; 5 → 9 widths paid −4.9e-5 in-frame
   (0.791635 → 0.791586; 14 rows better / 0 worse; worst row 1.221 → 1.266 s)
   and that tree is the promoted frontier `e07fe7ae`.
2. **The exchange ledger is a bounded *work* allowance, not a schedule.** Two
   sites call `rgreedy::subset_window_descent_step` (the terminal class's
   exchange pass and the dense/hub window site). Each call is charged against
   `PRODUCTION_EXCHANGE_LEDGER`, an ops budget. The 128M → 512M step had
   already paid ≈1.1e-4 in the record's history; the 512M → 1G step had never
   been priced.
3. **What is *not* in this tree, and why.** The deferred-lift parity pass (stage
   4b polishes the held independent-set-first lift identically to the
   incumbent and keeps the better *polished* pair) is the only device in the
   record worth ≥1 bip on dev (0.791586 → 0.791480). It was submitted twice
   (`41baf1c`, `ab5adbb`) and **failed remotely twice**, while the same tree
   without it promoted; production therefore ships the raw 4b rule. This
   submission does not re-open that seam.

## 4. Hypotheses and approach selection

- H1: the next four span widths still carry residual value, because every
  previous group did and the pass is monotone.
- H2: the exchange-site ledger at 512M is not yet saturated, because the
  previous step (128M → 512M) paid.
- H3 (cheap, priced in the same sweep): the PEO round count (`PEO_ROUNDS` 4 → 5)
  buys something.

Each is a one-variable change on the shipped point, and all are measurable in a
single binary through test-only env seams, so the sweep is the cheapest
possible form of evidence. The tradeoff being priced is always the same one:
value on the 300-row dev corpus versus wall-clock on the rows nearest the 2 s
cap. Everything here is a *bounded work* device — the span passes carry their
own ledgers and the exchange ledger is charged per call — so the cost is
per-row bounded work, not a schedule that can run away.

## 5. Implementation

`src/ordering/mod.rs`, three edits:

1. `PRODUCTION_SPAN_WINDOWS` 9 → 13 entries: appended `(13,4,6,64M)`,
   `(16,4,6,64M)`, `(5,4,3,64M)`, `(24,4,11,32M)`. The test-only append seam
   `SSI_SPAN_WINDOWS_EXTRA` was the arm that priced them and was then removed,
   so the array is the single source of truth again; a new, *unmeasured* next
   group is parked behind a fresh test-only seam (`SSI_SPAN_WINDOWS_NEXT`) and
   ships nothing.
2. `PRODUCTION_EXCHANGE_LEDGER` `536_870_912` → `1_073_741_824` (512M → 1G at
   both `subset_window_descent_step` sites).
3. Comment-only documentation of both receipts next to the constants.

No new dependencies, no FFI, no build scripts, no change to the harness,
scorer, purity gates or corpus; all changes live in the editable path
`src/ordering`.

## 6. Experiments and measured results

Four-arm sweep plus two follow-ups, one binary, one session, `taskset -c 0-3`,
`SSI_INDEP_FORCE=1`, 300 dev rows. `SCORE` is the harness's own aggregate over
the probe's exact per-row flops, and `worst` is the slowest `order()` call:

| arm | device | SCORE | worst `order()` |
|---|---|---:|---:|
| P | shipped point (9 widths, ledger 512M) | 0.791498 | 1.389 s |
| L | ledger 1G (both sites) | **0.791451** | 1.457 s |
| L2 | ledger 2G | 0.791446 | 1.390 s |
| X | + 4 span widths | 0.791484 | 1.430 s |
| **XL** | **+ 4 widths and ledger 1G (shipped)** | **0.791437** | 1.430 s |
| XLP | XL + `SSI_PEO_ROUNDS=5` | 0.791440 | 1.433 s |

Readings:

- Arm P reproduces the standing probe receipt (0.791498) exactly, so the frame
  is comparable row-for-row with the record that produced `83a8f4fc`.
- XL is **17 rows better / 0 worse** than P. Largest movers:
  `crudeoil_lee4_06` −0.51 %, `powerflow0300p` −0.21 %, `crudeoil_lee2_06`
  −0.15 %, `chimera_mgw-c16-2031-01` −0.126 %, `crudeoil_lee1_07` −0.099 %,
  `rsyn0840m04m` −0.037 %, `sporttournament48` −0.037 %, `mpbp_35` −0.030 %,
  `mpbp_15` −0.024 % — i.e. the wins land on the medium/large class rows the
  exchange and span passes admit, and no row regresses anywhere.
- The ledger curve is monotone and past its knee: 512M → 1G alone is −4.7e-5;
  512M → 2G is only a further −5e-6. Shipping 1G rather than 2G keeps the
  allowance at the point where the marginal value has already flattened.
- `SSI_PEO_ROUNDS=5` is **rejected**: 0.791440 is worse than XL (0.791437) at
  the same worst-row cost, so the extra round is inert-or-negative on this
  point and was not shipped.
- Attribution between the two shipped devices: the four-width group alone is
  −1.4e-5, the ledger step alone −4.7e-5, and together −6.1e-5 on the shipped
  point — the ledger step is the larger half, and the two are close to additive.

## 7. Verification

- **Official local harness** (`yukon run`: sandboxed candidate build, then the
  graded execution path with the 2 s SIGKILL, the double-run determinism gate,
  the bijection check and the trusted scorer): **300/300, `0.791437 /
  0.924075`**, buckets `lt_1k` 0.8873 / `1k_10k` 0.8374 / **`gt_10k` 0.6850**
  (the gt_10k bucket is where the device lands), `results.tsv:1789269816`. The
  official number is *identical* to probe arm XL, so the shipped production
  build and the measured arm are the same program on this seam.
- Purity/licence: unchanged — pure Rust in `src/ordering`, no new deps, no
  `include!`/`#[path]`/FFI/build script, so the Stage-A gate behaves as before.

## 8. Caveats and risks (stated, not hidden)

- This is a **dev-frame** result. It says nothing directly about the hidden
  corpus, which is rebaselined per round and scores ≈5 % worse for us than dev
  (frontier 0.841768 hidden vs 0.791586 dev).
- The worst dev `order()` moves 1.389 → 1.430 s (+0.041 s, +3 %). That is the
  same order of cost as the previously promoted schedule extension (+0.045 s on
  its worst row), but the 2 s cap is per row and wall-clock from spawn, so the
  real risk is a hidden row already sitting near the cap: a hidden row at
  ≥1.96 s would now be killed. Every device shipped here is a bounded work
  allowance on top of a monotone pass, so the added time is per-row bounded
  work, not an unbounded schedule.
- The in-flight submission `83a8f4fc` (exchange 12/4/5) is not superseded by
  this one in the "safe" direction: this tree is a strict improvement on the
  same dev frame and carries the same structural changes plus the ledger step.

## 9. Learning and next steps

- The dev-frame value left in this pipeline is genuinely sub-bip per device
  (−1.4e-5 for four more widths, −4.7e-5 for a 2× ledger), so the competition is
  now decided by (a) accumulating several such monotone steps before each
  submission and (b) not spending wall-clock margin that a hidden row needs.
- The one ≥1-bip device in the record remains the deferred-lift parity pass, and
  its per-row cost is now known on both frames: on dev it is +0.289 s at worst
  (`cont6-qq`), +0.110 s (`edgecross24-115`), +0.092 s on the dev peak row
  (`chimera_selby-c16-02`: 1.304 → 1.397 s); on structural rows dev cannot
  contain it is +0.232 s (+21 %) on `ood_grid3d_16`. That profile — not the
  device's value — is why the two parity-carrying submissions failed remotely.
- Next steps, in order: price the parked `SSI_SPAN_WINDOWS_NEXT` group in the
  same frame; keep every future device inside the measured worst-row budget; and
  re-open the parity seam only behind a cost cut (an early-exit cascade) that
  holds its value on the six rows it wins (`crudeoil_lee4_10` −0.76 %,
  `methanol200` −0.89 %, `torsion50` −0.20 %, `graphpart_3g-0244-0244`,
  `glider400`, `crudeoil_lee4_09`) while removing its cost on the rows where it
  loses (`wastewater05m1` +0.39 %).
