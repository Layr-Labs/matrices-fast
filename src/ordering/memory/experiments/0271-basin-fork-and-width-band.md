# 0271 — basin fork on the cheap tier, and the exchange's width *band*

- **Date:** 2026-09-14
- **Base:** the synced frontier tree `bbf5849` (hidden **0.840623**), which is
  `MAX_N` 45 000 + `PRODUCTION_EXCHANGE_LEDGER` 2 GiB + 12 exchange sweeps +
  the shared `Pristine`/`Game` memo + the `ADJ_POOL`.
- **Score (dev, this box, production-mirror probe frame, 300 rows):**
  **0.790260 → 0.790100** (see the run section; the three devices below are
  additive and act on disjoint row sets).
- **Status:** shipped.

## The measurement frame, fixed first

Two of this tree's own test seams **defaulted to a different program than
production**, which is the trap `0239` warns about, and it was still live here:

| seam | test default (before) | production |
|---|---|---|
| `SSI_PRECLASS_WIN` / `SSI_PRECLASS_STEP` | `true` | `false` |
| `SSI_EXCHANGE_SWEEPS` | `6` | `12` |

So a plain probe ran two extra exchange passes the graded worker never runs, and
a half-length sweep cycle. Both seams are now **opt-in**, with the unset default
equal to the production value; a probe that wants the old frame asks for it. A
production-mirror probe run is now `SSI_MARK_NOSCORE=1` and nothing else, and on
the untouched tree it reads **0.790260** — 2.8e-5 from that lane's official
`0.790288` on the same tree, which is the frame's error bar.

## Device 1 — exchange width is a *banded* decision, not a constant

`step` coprime to `width` makes `width` sweeps a **complete alignment cycle**:
12/5/12 visits all twelve tilings of the permutation once, and 14/5/14 visits
all fourteen. The two are alternatives, not a sequence, and a wider window can
reach components of 13–14 vertices that a 12-window can never contain.

Which cycle to buy is a *ledger* question. In-frame (one binary, one session,
all 198 below-anchor class rows, `SSI_MARK_NOSCORE=1`) 14/5/14 **everywhere** is
**+1.01e-4 dev**, and the per-bucket split says why:

| bucket | weighted Δln |
|---|---|
| `lt_1k` | −6.6e-7 |
| `1k_10k` | **−7.45e-5** |
| `gt_10k` | **+2.03e-4** |

Every row that regressed has `n >= 10 429` (`chp_shorttermplan2d` 16 364,
`crudeoil_lee4_09` 15 904, `crudeoil_lee4_10` 17 809, `crudeoil_lee4_06` 10 429,
`crudeoil_pooling_dt3` 30 660, `arki0013` 44 909) — exactly the rows where
2 GiB truncates the sweep loop. Measured directly with the in-tree split
instrument: at 2 GiB `crudeoil_pooling_dt3` completes ~1.6 of 12 sweeps
(`windows=4031` of `n=30660`), `crudeoil_lee4_10` ~2.4, `mpbp_48` ~6; at 8 GiB
they complete 12 (`windows=17651 / 14372 / 28367`) at 3–4x the DP wall. A
truncated 14-cycle is strictly *less* search than a complete 12-cycle, which is
the whole regression.

**Shipped:** `EXCHANGE_WIDE_MAX_N = 10 000` — rows at or below it run 14/5/14,
rows above keep 12/5/12 and are byte-identical. Worth **−5.94e-5 dev**.

## Device 2 — the dense/hub twin still ran the shape the class block outgrew

The class block's gate sends `nnz > 16n || max_deg > n/2` rows to the *dense/hub
twin*, which had kept `8/4/3` across two later window decisions at the main
site. Priced on the twin's own 24-row census in-frame:

| twin shape | 24-row geomean |
|---|---|
| 8/4/3 (shipped) | 0.776753 |
| 12/4/5 | 0.776776 |
| 12/12/5 | 0.776629 |
| **10/4/4** | **0.776456** |

Movers `chimera_lga-01` −0.52 %, `chimera_mgw-c16-2031-01` −0.38 %,
`chimera_rfr-02` −0.10 %; `sporttournament48` and `torsion50` unchanged. Worth
**−2.21e-5 dev**. Note the width axis here is *not* the class block's: on these
hub/dense rows a partial 10/4/4 coverage beats both 8/4/3 and a full cycle, so
the two sites are priced separately.

## Device 3 — a second basin for cheap rows (fork + exact merge)

The middle of the pipeline is a chain of **greedy** acceptances; each one is a
commitment, and the stages after it are monotone from wherever it lands. A
greedy step that improves the incumbent can still be a trap for the final
ordering, and the fix is not to patch the chain (its value is measured and
large) but to run a **second lineage whose middle commitment is withheld** and
return whichever lineage ends with fewer exact `Σ cⱼ²`.

Implemented in `order()`: for `n <= 600 && nnz <= 5 000` the pipeline runs
twice — once exactly as shipped, once with the whole-graph `4.subtree` cascade
suppressed by a thread-local flag — concurrently on a shared `&Pattern`, and the
two permutations are compared with `flops_of` (strict `<`, shipped lineage wins
ties). One-sided by construction, and a deterministic function of the pattern:
all pipeline state is `thread_local`, so the two lineages cannot see each other.

In-frame A/B over all 140 dev rows with `n <= 1 000 && nnz <= 20 000`, same
binary (fork on / fork off):

| row | n | off | on |
|---|---|---|---|
| `waterund14` | 333 | 0.3600 | **0.3516** (−2.35 %) |
| `chimera_mgw-c8-439-onc8-001` | 440 | 0.7522 | **0.7347** (−2.32 %) |
| `gancns` | 548 | 0.8421 | **0.8407** (−0.17 %) |

Σ Δln over the bucket = −3.33e-4 → **−7.90e-5 dev**, 3 rows moved, 0 regressions.
The same three rows are the ones the public record's own version of this device
names (submission `3587d1b`, PR-visible): that lane measured −8.3e-5 dev /
−7.8e-5 hidden for it, i.e. a transfer of ≈0.94 on this axis. The gate is
deliberately the narrow cheap band: the fork's added wall tracks the row's own
pipeline cost, and `nnz <= 5 000` excludes the dense small patterns whose `nnz`
at n = 600 is ~1.8e5 as well as every mid and crown row.

## The run

Full 300-row production-mirror probe on the stacked tree, one binary, one
session: **base 0.790260 → stacked 0.790086** (evidence
`0271-stack-prod-300rows.log`; the probe's own printed `SCORE` for each arm).
`cargo test --release -p ssi-candidate-worker --offline --locked`: **126 passed
/ 0 failed**, 58 ignored.

| bucket | rows | base | stacked |
|---|---|---|---|
| `lt_1k` | 147 | 0.8873 | **0.8870** |
| `1k_10k` | 108 | 0.8373 | **0.8371** |
| `gt_10k` | 45 | 0.6822 | 0.6822 (byte-identical: every device's gate is below its rows) |

**25 rows move, 23 of them down.** The two that move up are the width band's and
the twin's own local-optimum changes and are worth 5.7e-6 between them
(`chimera_selby-c16-01` +0.084 %, `chimera_mgw-c8-439-onc8-002` +0.167 %):

| gain | n | Δ | | loss | n | Δ |
|---|---|---|---|---|---|---|
| `waterund14` | 333 | −2.350 % | | `chimera_selby-c16-01` | 2 031 | +0.084 % |
| `chimera_mgw-c8-439-onc8-001` | 440 | −2.324 % | | `chimera_mgw-c8-439-onc8-002` | 440 | +0.167 % |
| `crudeoil_lee1_07` | 3 670 | −0.815 % | | | | |
| `edgecross14-156` | 3 097 | −0.755 % | | | | |
| `chimera_lga-01` | 1 120 | −0.522 % | | | | |
| `chimera_mgw-c16-2031-01` | 2 032 | −0.380 % | | | | |
| `mpbp_15` | 9 858 | −0.310 % | | | | |
| `chimera_k64ising-02` | 1 225 | −0.255 % | | | | |
| `gancns` | 548 | −0.199 % | | | | |
| `chimera_selby-c16-02` | 2 031 | −0.136 % | | | | |

The fork's own wall, A/B on all 119 rows of its band in the graded worker frame
(one process per arm, same patterns): **113.2 s → 138.3 s, +22 %**, worst single
row `chimera_mgw-c8-439-onc8-001` ×1.55, median ×1.2. Every gated row is one the
2 GiB tree orders in a small fraction of the cap.

## Rejected this session (all measured, none shipped)

- **A new neighbourhood: exact reordering of `{v} ∪ N(v)`** (`star_window_descent`,
  new code in `rgreedy/window_dp.rs`). One pass over the permutation keeps the
  live graph and, at every position, exactly reorders the pivot's closed live
  neighbourhood inside its own slots — a set the position-window sites cannot
  reach when a vertex's neighbours are scattered. **Zero wins over 189 rows with
  `n <= 2 000`.** The instrument was validated first: seeded from a
  relabelled/reversed permutation instead of the incumbent it improves **119 of
  140** rows (Σ Δln = −192.8), so the zero is a property of the shipped
  incumbent, not of the code: the pipeline's output is already star-optimal.
- **Seeding the exact class from a different basin on rows AT the AMD anchor.**
  24 at-anchor class-key rows × 6 relabelled-AMD seeds, 12-sweep exchange at the
  2 GiB ledger, exact scoring: **0 improvements in 144 attempts**, on every row
  `best == mine`. This *extends* the `iter65` anchor gate's justification
  ("the family is never the first improver") from the incumbent seed to
  relabelled seeds, and closes the "at-anchor rows are the frontier" hypothesis.
- **Nested dissection on the giants.** Every partitioner on every row with
  `n > 45 000`: METIS default and tuned, Scotch, KaHIP — ratios **2.2x to
  9500x** AMD (`faclay75` METIS 2.23x / 24 s, `gabriel10` Scotch 3.30x,
  `transswitch2736spr` hand-rolled ND 9804x), plus the hand-rolled `nd_order` /
  `ndfm_order` at 0.08–1.7 s each and equally bad ratios. The rings are why the
  gates exist; ND is not the headroom on this corpus's large rows.
- **Exchange `step = 1`** (a sliding window instead of a 5-stride): +1.05e-4 dev
  over the rows it reached, dropped.
- **`n <= 1 000` fork band** instead of `n <= 600`: the two rows it adds
  (`multiplants_mtg1b`, `sonet24v5`) are worth ~0.11 bip together and push the
  forked wall on those rows to 1.05–1.21 s, i.e. into the band the hidden cap
  owns; not taken.

## Why the width band is not just a dev fit

The mechanism is an accounting one and therefore structural: the ledger
truncates the sweep loop on rows where the class's charged work exceeds 2 GiB,
and charged work grows with `n` (both the replay term `n·(deg+1)(3w+6)` and the
DP term `2^k(16k+6w+24)`). The band boundary is where the truncation starts, and
the census puts every regressing row above it. On the hidden corpus the same
inequality holds — what changes is *where* the boundary sits for a given row,
which is a second-order effect next to "complete cycle beats truncated cycle".

## Follow-ups

- The fork is a *diversity* device, not a search device: a third lineage
  (withholding a different commitment) is the obvious next axis, and the
  mid-band extension needs a deterministic work fence before it is affordable.
- `open-questions.md`: the ledger's unit of account (see below) is still open.

## Links

- Techniques: [best-of-portfolio](../techniques/best-of-portfolio.md),
  [nested-dissection](../techniques/nested-dissection.md)
- Experiments: [0239 public-board receipts](0239-public-board-receipts.md),
  [0218 production-frame mirror](0218-production-frame-mirror.md),
  [0230 span schedule 9 → 13](0230-span13-ledger1g.md)
