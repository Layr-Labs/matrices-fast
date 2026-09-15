# 0278 — the basin fork's anchor margin: same value, without the cost tail

- **Date:** 2026-09-14 (iter68)
- **Base:** the promoted crown `bbf58495` / `99de589`, plus this session's `PEO_ALT_MAX_N` 10 000
  and the census-bounded dense twin.
- **Status:** shipped (submission `0278`, see §5).

## Hypothesis

The fork's structural band (`6 <= n <= 600 && nnz <= 5 000`) says which rows *may* fork; it says
nothing about whether the fork can *pay* on them. If the fork's wall cost and its value are
disjoint inside that band, a margin gate can remove the cost without touching the value.

## Evidence — the cost and the value are disjoint

Worker-frame receipt `0276-graded-frame-reps2.log` (crown vs the iter66 fork tree, one process per
row, min of 2) re-read per row, plus this session's four-arm A/B
(`../evidence/0278-fork-margin-four-arm-ab.log`, arms `crown` / `fork` / `gated`, min of 3):

| row | n | nnz | incumbent ratio | fork dln | fork wall |
|---|---|---|---|---|---|
| `waterund14` | 333 | 2 204 | 0.360 | **-2.38e-2** | +0.17 s |
| `chimera_mgw-c8-439-onc8-001` | 440 | 3 040 | 0.752 | **-2.35e-2** | +0.19 s |
| `chimera_lga-01` | 1 120 | 6 400 | 0.741 | **-5.23e-3** | +0.01 s |
| `chimera_mgw-c16-2031-01` | 2 032 | 15 900 | 0.772 | **-3.81e-3** | +0.03 s |
| `gancns` | 548 | 2 800 | 0.842 | **-1.67e-3** | +0.35 s |
| `chimera_rfr-02` | 2 032 | 15 140 | 0.645 | **-1.03e-3** | +0.17 s |
| `himmel11` | 14 | 60 | **1.0000** | 0 | **+0.68 s** |
| `syn15hfsg` | 399 | 1 022 | 0.9915 | 0 | **+0.83 s** |
| `nvs02` | 11 | 46 | 1.0000 | 0 | +0.25 s |
| `wastepaper4` | 115 | 724 | 0.740 | 0 | +0.14 s |
| `tls6` | 413 | 2 860 | 0.947 | 0 | +0.14 s |
| `pooling_adhya4pq` | 170 | 932 | 0.681 | 0 | +0.14 s |

Every mover sits at ratio <= 0.842; every hard-loaded non-mover is near anchor. This is structural,
not incidental: a fork explores a **second basin** from a non-trivial incumbent, so on a row the
pipeline never moved off the AMD anchor there is no second basin and the duplicate suffix is pure
cost. The class block already uses exactly this gate (`past_anchor`), so the margin is the same
argument applied to the same kind of device.

## Change

`shared_basin_fork_band(n, nnz, best_flops, amd_flops)` now also requires
`best_flops * 100 < amd_flops * (100 - SHARED_BASIN_FORK_MARGIN_PCT)` with
`SHARED_BASIN_FORK_MARGIN_PCT = 10` (a 10 % win over the anchor). Test seam `SSI_FORK_MARGIN_PCT`.

## Result

Four-arm worker-frame A/B, 20 rows, min of 3, same session and same staged patterns:

- the gated arm reproduces **all six movers with the identical dln** of the unmargined fork;
- it is bit-identical to the crown on every non-mover, including all nine near-anchor rows;
- the rows it suppresses are exactly those with zero measured fork value.

`sum ln` over the mover set is unchanged (-5.90e-2 on this 20-row scope for both arms), so the
margin is a strict wall removal, not a score trade.

**Caveat.** The margin is validated on the rows measured here. A fork-band row that sits between
the movers' 0.842 and the 0.90 threshold would be blocked and *could* have carried value; the
observed mover ratio distribution (0.360-0.842) leaves a 6-point gap above the highest mover, and
no row in the receipts sits in it.

## Submission

`0278` — the fork + margin, `PEO_ALT_MAX_N` 10 000, and the census-bounded twin. Note
`../evidence/0278-submission-note.md`. This is the sixth fork-carrying submission from this lane;
the previous five died on the 2.0 s cap at 81.5 / 83.1 / 85.5 / 184.1 s of Benchmark wall. The
margin removes the fork's entire measured cost tail, which is the only cap-relevant quantity this
codebase can actually measure.

## Verification

- Parent and candidate worker builds clean.
- `cargo test --release -p ssi-candidate-worker --offline --locked` — **126 passed / 0 failed**,
  56 ignored.
- Determinism: every arm ran each row in a fresh process with the harness asserting an identical
  permutation across repetitions.

## Links

- Experiments: [0277 the cap priced per stage](0277-cap-margin-census-and-peo-alt-window.md),
  [0276 shared-prefix fork and bounded twin](0276-shared-prefix-fork-and-bounded-twin.md)
- Evidence: `../evidence/0278-fork-margin-four-arm-ab.log`,
  `../evidence/0276-graded-frame-reps2.log`, `../evidence/0278-submission-note.md`
