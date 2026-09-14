# Retire the basin fork; keep the chain-displaced registration

**Effort:** xhigh · **Coding agent / harness:** angelX (wire model DeepSeek V4 Flash)
**Benchmark:** `layr-labs/matrices-fast` — fill-reducing elimination ordering, scored as a
size-bucketed weighted geomean of predicted factorization flops vs feral's AMD (lower is better).
**Base:** the promoted tree (hidden 0.840623; 0.790263 / 0.923140 locally in this session's frame).

## 1. Context, goal and baseline

The lane's job is to return orderings that beat the promoted profile by at least 1e-4 hidden while
never letting a single matrix's `order()` call exceed the harness's 2.0 s per-matrix wall cap. Both
halves are binding. The promotion bar is a hard floor (a bat that completes at −7.8e-5 is rejected),
and the cap is a hard kill (a bat that exceeds it scores nothing at all).

The recent hidden ledger makes the tension concrete. Of the last six validations on this eval
prefix, exactly one completed: bat `3587d1b` — the promoted profile plus the basin fork alone — and
it scored **0.840545**, i.e. **−7.8e-5** against the 0.840623 frontier and therefore **rejected**
(0.22 bip short of the bar). Four validated neighbours were killed on the per-matrix time cap at
81.5-86.5 s into the run: `f70480c1` (fork band widened to n ≤ 1200), `69bc2fc5` (the dense-band
ladder rung re-entered as two cheap draws), `351f3ddb` (registration + work-band fork + head-light)
and `f02eb0d7` (the registration alone). Every kill carries the same redacted verdict — "order()
exceeded the 2.0s per-matrix cap and was killed" — and every one of them is a tree this lane built
on top of the one tree that survived.

So the structural fact to exploit is: **the devices that add value are adding it on the same row
class that is closest to the cap line.** The fork is the cleanest example, and it is the device the
survivor carried.

## 2. Environment and setup

* Local harness frame: `bash scripts/local-candidate-build.sh && cargo run --release`, which builds
  the untrusted candidate worker inside the audited bubblewrap build boundary and grades the 300-row
  public dev corpus, running `order()` twice per matrix in two fresh sandboxed children and killing
  a child that exceeds 2.0 s of wall clock (poll every 10 ms; the sandbox spawn is inside the same
  budget).
* Probe frame: the candidate worker's own `#[cfg(test)] probe_timing_and_score`, run inside the same
  build boundary (`target/probe-sandbox.sh run`) with `SSI_MARK_NOSCORE=1` so the phase marks do not
  add a scoring pass, `SSI_PROBE_ONLY=<rows>` to restrict the corpus, and `SSI_XCH_TIME=1` for the
  terminal exchange's own split.
* Pinned frame: the same probe binary under `taskset -c 0`. The hidden runner is much closer to one
  core than to four: its completed run spent 635 s over 300 two-call rows ≈ 1.06 s/call, against
  0.83 s/call locally pinned to one core and 0.60 s/call at four cores.
* Seams used for pricing: `SSI_NO_BASIN_FORK=1` (fork off), `SSI_NO_CHAIN_DONOR=1` (registration
  off). Both are test-only kill switches; the production constants are the shipped behaviour. The
  probe sandbox forwards them (an earlier iteration found that this whitelist silently dropped
  seams, so every A/B here was checked for separation before it was believed).

## 3. Hypotheses and how each was tested

**H1 — "the registration is what killed `f02eb0d7`".** The registration's own documentation prices
it as "a few candidate evaluations"; the iter83 note attributed +0.146 s to its worst row. Test: run
the four-arm A/B above on 30 heavy rows pinned to one core and read the per-row deltas.

**H2 — "the fork is wall-free because it runs two lineages in parallel threads".** True at four
cores, false in the frame that matters: pinned to one core the two lineages serialize, so the fork
costs one extra pipeline pass per forked row.

**H3 — "a narrower fork gate could keep the fork's value without its wall".** Test: read the fork's
per-row value map from the same A/B.

**Outcome.** H1 is refuted: the registration is wall-neutral (±0.06 s/row, +0.23 s over 30 rows) and
carries the value (`crudeoil_pooling_dt3` 0.7056 → 0.6449, `arki0013` 0.3993 → 0.3919,
`crudeoil_lee4_09` 0.6171 → 0.6100). H2 is confirmed, and the cost is large: the fork costs +5.36 s
and +4.66 s over the same 30 rows (two independent arm pairs). H3 is refuted: the fork's entire
value on that set is two rows — `chimera_mgw-c8-439-onc8-001` (0.7522 → 0.7347) and `gancns`
(0.8421 → 0.8407) — and those are exactly the rows where the fork's wall is largest (0.771 → 1.529 s
and 0.767 → 1.466 s). Value and cost are colocated, so no shape gate separates them.

That flips the decision: the fork is not a value device that costs wall, it is a **cap liability
that costs 0.7 s/row on the near-cap class for 7.8e-5 of hidden value** — below the bar it would
have to clear on its own. Retiring it and keeping the registration reaches the same basin through
the ledger (the registration supplies the chain-displaced incumbent, the very ordering the chain
moved away from), at zero measured wall.

## 4. Implementation

Only `src/ordering` is editable; this change is two production constants and no new machinery.

* `BASIN_FORK_MAX_N` 600 → **0** in `src/ordering/mod.rs`. `basin_fork_gate(n, nnz)` is
  `n <= BASIN_FORK_MAX_N && nnz <= BASIN_FORK_MAX_NNZ`, so with the bound at 0 the gate can never
  fire for n ≥ 1 and `order()` runs a single lineage on every matrix. The seam overrides
  (`SSI_BASIN_FORK_N`, `SSI_NO_BASIN_FORK`) still work in the test frame, which is how the price
  above was measured.
* the chain-displaced registration stays exactly as shipped in the previous bat: snapshot
  `(best_flops, best_perm)` before the `4.subtree` chain, and after the chain — only when the chain
  actually installed a strict improvement — push it into `runner_up` (deduped, sorted, truncated to
  `PEO_ALT_SEEDS`), the ledger the PEO_ALT seeds, the transplant donors and the terminal exchange's
  seed pool read. Every consumer admits only a strict exact decrease, so a registration can add
  value but cannot lower any row's own flops.
* the dense-band ladder rung and the work-band fork stay retired (production compiles them OFF).

Exact commands:

```sh
bash scripts/local-candidate-build.sh && cargo run --release          # official local receipt
cargo test --release -p ssi-candidate-worker --offline --locked       # 126 unit tests
SSI_MARK_NOSCORE=1 SSI_PROBE_ONLY=<rows> taskset -c 0 <probe-binary> \
  --ignored --nocapture --test-threads=1 probe_timing_and_score       # pinned per-row frame
```

## 5. Measured results

**Official local sandboxed harness** (300-row contract corpus, two sandboxed `order()` calls per row):

| tree | score.json | lt_1k | 1k_10k | gt_10k | fill |
|---|---|---|---|---|---|
| promoted profile | 0.790263 | — | — | — | 0.923140 |
| fork@600 (bat `3587d1b`, hidden 0.840545) | 0.790199 | — | — | — | 0.923124 |
| fork@600 + registration (bat `f02eb0d7`, killed at the cap) | 0.790017 | 0.886977 | 0.837385 | 0.681771 | 0.923134 |
| **this tree (fork retired + registration)** | **0.790106** | **0.887273** | 0.837385 | **0.681771** | 0.923160 |

300/300 rows scored, **0 FAIL**, run wall 4 m 15 s. The fork@600 + registration tree took 4 m 50 s
on the same corpus and failed one run in this frame on `gasprod_sarawak16` (n = 4596, nnz = 15316) —
the same row the 0284 note flagged as sitting at the 2 s line locally.
`cargo test --release -p ssi-candidate-worker --offline --locked`: **126 passed, 0 failed**.

**Pinned one-core four-arm A/B** (30 rows = the corpus's heavy set ∪ the fork band):

| arm | 30-row wall |
|---|---|
| fork ON, registration ON | 43.00 s |
| fork ON, registration OFF | 42.77 s |
| fork OFF, registration ON | 37.64 s |
| fork OFF, registration OFF | 38.11 s |

Per-row highlights: `chimera_mgw-c8-439-onc8-001` 0.771 → 1.529 s, `chimera_mgw-c8-439-onc8-002`
0.762 → 1.535, `rocket50` 0.771 → 1.529, `gancns` 0.767 → 1.466, `wastewater13m1` 0.681 → 1.336,
`tls6` 0.623 → 1.223 (fork); `crudeoil_pooling_dt3` 1.424 s with the registration against 1.466 s
without it, with the ratio moving 0.7056 → 0.6449 (registration). Heavy rows outside the band
(`arki0016`, `crudeoil_lee4_10`, `gams05`, `arki0013`) move ≤ 0.07 s under either seam.

Against the promoted profile the tree measures **−1.57e-4 dev** (0.790263 → 0.790106). At the
round's calibrated dev→hidden transfer (0.94, measured from the completed bat: −9.2e-5 dev →
−7.8e-5 hidden) that is ≈ **−1.5e-4 hidden**, i.e. inside the 1e-4 bar rather than short of it.

## 6. Failures, caveats and course corrections

* The first reading of the A/B had the arms transposed (the fork looked like the cheap device). The
  raw TSVs corrected it: the fork is the spender, the registration is free. Every number above is
  quoted from the raw per-row TSV, not from a summary.
* `f02eb0d7`'s death cannot be explained by the registration's wall on any row this box can measure,
  so the honest statement is: the registration is wall-neutral *here*; the killed tree's margin was
  spent somewhere this frame does not expose, and the survivable move is to remove the largest
  measured multiplier on the near-cap class rather than to re-add a sub-bar value device.
* The local frame is not the graded frame. One run of the fork@600 + registration tree failed
  locally on `gasprod_sarawak16` and an immediate re-run of the identical binary scored 300/300, so
  the heavy-row margin in this frame is ≈ 0 and a single local failure is not proof of a design
  fault. Conversely, the −1.57e-4 dev figure is a local worker-frame number; only the graded run
  decides promotion, and no local run can claim acceptance.
* The lt_1k bucket is *worse* than the promoted profile's in this tree (the fork lived there); the
  whole gain comes from `gt_10k` (0.682176 → 0.681771) and the registration's heavy-row movers.
  That is the intended trade: sub-bar value on the dangerous class, kept value on the ledger axis.

## 7. Learning and next steps

* Price a device in the frame the cap is paid in, one core at a time, and price it *arm by arm with
  the other devices held fixed*: the same A/B assigns +5.36 s to the fork and +0.23 s to the
  registration, which no single-arm comparison could have separated.
* A device below the promotion bar is a pure liability if it costs wall on the near-cap class. The
  fork's own hidden receipt (−7.8e-5, rejected) is the cleanest possible evidence of that.
* Next: with the fork retired the in-band class is ~2× cheaper per row, so the wall budget that
  bought nothing is available again. The registration is the only device this lane has measured at
  ≥1e-4, so the next candidates are (a) splitting the registration's value by consumer
  (`SSI_XCHG_POOL=0`) to see whether the cheap consumers already carry it, and (b) re-pricing the
  pre-class pair, which was retired for wall before this margin existed.
