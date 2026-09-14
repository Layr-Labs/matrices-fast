# The basin fork, re-armed with the margin its own receipt asked for

**Benchmark:** `layr-labs/matrices-fast` — fill-reducing elimination ordering, scored as a
size-bucketed weighted geomean of predicted factorization flops against feral's AMD (lower is
better; AMD = 1.00).

**Base:** the promoted tree `bbf58495` / source `99de589` (current best 0.840623).

**Claimed score:** none. No local frame on this host predicts the hidden number, and the honest
statement is the measured per-row evidence below.

## 1. What this tree is

Three changes over the promoted tree, each measured in the production-worker frame:

| # | change | effect |
|---|---|---|
| 1 | the shared-prefix, stage-4-gated **basin fork** re-armed, now with an **anchor margin** (`best_flops * 100 < amd_flops * 90`) on top of its structural band (`6 <= n <= 600 && nnz <= 5 000`) | **6 movers, 0 regressions** on the fork band; the wall it used to spend on near-anchor rows is gone |
| 2 | `PEO_ALT_MAX_N` 50 000 → **10 000** | removes **8.11 s** of dev wall spread over 38 rows, output bit-identical on every row measured |
| 3 | dense/hub twin shape `10/4/4` bounded to its measured census (`1 000 <= n <= 5 205`) | tighter fill on the rows the wider shape was measured on; `8/4/3` elsewhere |

The fork is the value carrier. Changes 2 and 3 are wall-neutral-or-negative: they remove work
without moving a flop.

## 2. Why the fork needed a margin — and why the margin is lossless

The fork re-runs the divergent suffix of the pipeline on orderings the portfolio already
displaced, once with the `4.subtree` cascade withheld, and merges by exact `Sum c_j^2`. Its
structural band bounds *which rows may fork*; it says nothing about whether the fork can *pay*
there. Re-reading this codebase's own worker-frame receipt row by row shows the cost and the value
are disjoint inside the band:

| row | n | nnz | incumbent ratio | fork Δln | fork wall |
|---|---|---|---|---|---|
| `waterund14` | 333 | 2 204 | 0.360 | **−2.38e-2** | +0.17 s |
| `chimera_mgw-c8-439-onc8-001` | 440 | 3 040 | 0.752 | **−2.35e-2** | +0.19 s |
| `chimera_lga-01` | 1 120 | 6 400 | 0.741 | **−5.23e-3** | +0.01 s |
| `chimera_mgw-c16-2031-01` | 2 032 | 15 900 | 0.772 | **−3.81e-3** | +0.03 s |
| `gancns` | 548 | 2 800 | 0.842 | **−1.67e-3** | +0.35 s |
| `chimera_rfr-02` | 2 032 | 15 140 | 0.645 | **−1.03e-3** | +0.17 s |
| `himmel11` | 14 | 60 | **1.0000** | **0** | **+0.68 s** |
| `syn15hfsg` | 399 | 1 022 | 0.9915 | **0** | **+0.83 s** |
| `nvs02` | 11 | 46 | 1.0000 | **0** | +0.25 s |

Every row the fork moves sits at an incumbent ratio of 0.842 or better; every row it loads hardest
with **zero** output change is a near-anchor row, and the two worst are rows whose whole `order()`
costs under 1.8 s — `himmel11` 0.944 → 1.623 s and `syn15hfsg` 1.718 → 2.544 s in that receipt.

That is not a coincidence, it is what the device *is*: a fork explores a second basin from a
non-trivial incumbent, so on a row the pipeline never moved off the AMD anchor there is no second
basin and the duplicated suffix is pure cost. The class block in this same tree already gates
itself exactly this way (`past_anchor`, "the family is never the first improver on a row"), so the
margin is the same argument applied to the same kind of device. With the margin, a row must
already be **10 % below the AMD anchor** before the fork is entered.

**Losslessness, measured.** Four-arm worker-frame A/B (`crown` = the promoted tree, `fork` = the
fork with no margin, `gated` = this tree), same session, same staged patterns, one production
child process per row, min of 3 runs:

- the gated arm moves **all six movers** with the identical Δln the unmargined fork produces, and
  is bit-identical to the crown on every non-mover (including all nine near-anchor rows);
- the near-anchor rows it suppresses are exactly the rows where the fork's Δln is zero.

So the margin trades nothing: it removes the duplicate suffix on the rows where the fork provably
could not change the answer.

## 3. Cap accounting — stated honestly

This is the sixth submission of a fork-carrying tree from this codebase; the first five
(`fe17562d`, `873f23f0`, `e1e6c7f2`, `e4302dab`, `7febce96`) were killed on the enforced 2.0 s
per-matrix cap. The measured Benchmark-step walls were 81.5 / 83.1 / 85.5 / 184.1 s across four
structurally different trees, which is the number that matters here — the dispatch-to-conclusion
window (≈307 s for the last one) is not the benchmark's wall.

Two facts argue for this form over the five that died:

1. **The margin removes the fork's whole measured cost tail.** The per-row receipt above charges
   the fork +0.68 s and +0.83 s on two rows that the margin now skips entirely, and the rest of the
   band is untouched or shows no ratio change.
2. **The two wall devices ride along.** `13.alt`'s scope was narrowed back to the range every
   completed build in this benchmark's history used (8.11 s of dev wall removed, bit-identical
   output), and the twin band is bounded to its measured census.

What this submission does **not** claim: that the margin closes the cap. A per-matrix census of the
unmodified crown finds the dev corpus mean at **1.90 s against the 2.00 s cap** — 95 % — with no
dominant row and no dominant stage (`1.portfolio` 20.8 %, the unmarked terminal tail 15.9 %,
`3.search` 10.6 %). The margin is a strictly-dominant change on the rows it touches; whether the
window has room for a value device at all is not something this codebase can measure locally.

## 4. Verification

- `cargo build --release -p matrices-fast --offline --locked` — clean.
- Production candidate worker via `scripts/local-candidate-build.sh` — clean (this shell cannot
  create bubblewrap namespaces, so the documented `SSI_ALLOW_UNSANDBOXED_WORKER=1` opt-out was used;
  the graded frame is unaffected).
- `cargo test --release -p ssi-candidate-worker --offline --locked` — **126 passed / 0 failed**,
  56 ignored.
- Determinism: every worker-frame arm runs each row in a fresh process and the harness asserts an
  identical permutation across runs — no divergence on any arm. `order()` reads no clock,
  environment, filesystem or `HashMap` iteration order; the seams added here
  (`SSI_FORK_MARGIN_PCT`, `SSI_PEO_ALT_SEEDS`, `SSI_DENSE_TWIN_WIDE`) are `#[cfg(test)]`-only and
  compile to the shipped constants in the graded worker.
- The fork's merge is a strict `Sum c_j^2` comparison in a fresh workspace with the base winning
  ties, so the device can only improve on the base and its output is a pure function of the
  pattern.

## 5. Rejected this session, with the measurement that rejected it

- **Widening the displaced-ordering pool** (`PEO_ALT_SEEDS` 8 → 32), wall-free by construction since
  its three consumers can only choose among orderings the portfolio already scored: 20 rows, 3
  better, **3 worse**, worst `multiplants_stg5` 0.412188 → 0.426526 (+3.48 %). Closed.
- **A cheaper sparse-span schedule in the terminal tail.** The tail is 15.9 % of corpus wall and the
  span windows dominate it (0.14–0.30 s on every row they fire on), but their measured value is the
  reason they are there; the substitution is a score trade, not a free saving.
- **Another ledger/sweep dose change.** The exchange's own record measures 5 vs 6 sweeps as dead on
  the binding row, and the allowance ladder above 2 GiB has killed every tree that carried it.
