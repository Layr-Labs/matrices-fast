# 0278 — the exchange's admission policy, the daily corpus rotation, and the fork's retirement

- **Date:** 2026-09-14 (iter68)
- **Base:** the promoted crown `bbf58495` / `99de589` (hidden 0.840623).
- **Status:** `PRODUCTION_XCH_ALLOC = 1` **shipped**; the basin fork **retired** (behind its seam).

## 1. The hidden corpus rotates DAILY — scores are not comparable across days

`.github/scripts/fetch-eval-corpus.sh` fetches `eval/current.txt` from a private bucket and its own
header says the pointer "lets the **daily rotation job** upload a new dated object and then flip
current.txt atomically". So the graded set is a **daily** object, not a per-submission draw and not
a fixed set.

Consequences that must survive into the next session:

* A verdict is comparable only with verdicts from **the same day**. `bbf5849`'s 0.840623 and this
  day's 0.840946 are different corpora.
* The kill times observed today (66.6 / 66.9 / 83.1 / 83.1 / 84.5 / 85.5 s of Benchmark wall) and
  the day's two completions at ≈640 s are consistent with a corpus whose hidden killer rows sit
  early in the order and whose *host* margin varies, not with a tree-quality ranking.
* Therefore: **rank trees by the local dev frame and by mechanism, never by a single day's hidden
  number.** A "the fork scored worse, so the fork is bad" inference needs the same tree measured on
  the same corpus, which is not available.

## 2. Shipped: policy 1 of the exchange's component admission (`PRODUCTION_XCH_ALLOC` 0 → 1)

`refine_window` precharges `2^k * (16k + 6w + 24)` against the window's deterministic ledger, then
walks the window's live connected components (2..=MAX_WIDTH positions) in ascending-position order.
Policy 0 ends the walk at the first component the ledger cannot fund; policy 1 skips it and keeps
walking. The precharge is untouched, so **policy 1 can only spend what policy 0 already budgeted**,
and every adoption still passes the pipeline's exact strict-`<` scorer — the device is monotone by
construction.

Measured, one binary, one session, all 300 dev rows, graded-closest `SSI_MARK_NOSCORE=1` frame
(`../evidence/0278d-xch-alloc-{0,1}.log`):

| arm | score | rows identical | movers | regressions |
|---|---|---|---|---|
| `SSI_XCH_ALLOC=0` | 0.790236 | — | — | — |
| `SSI_XCH_ALLOC=1` | **0.790230** | **299 / 300** | 1 (`crudeoil_lee1_07` 0.745866999 → 0.744049962, dln −2.44e-3) | **0** |

Worker-frame check (`SSI_STAGE_KEEP_ENV=1` so the seam reaches a real child process, one process per
row, min of 3, 13 rows): no systematic wall change; the mover reads 3.49/3.62/3.65 s at policy 0 and
3.50/3.82/3.88 s at policy 1, i.e. inside this host's own spread.

**Policy 2 is closed as a no-op.** `SSI_XCH_ALLOC=2` (additionally sort the components
smallest-first) gives **0.790230 — identical to policy 1 on all 300 rows**
(`../evidence/0278d-xch-alloc-2.log`). A peer submission shipped policy 1 alone and called policy 2
"deliberately not shipped: it deserves its own attribution receipt"; this is that receipt: there is
nothing to attribute.

**A second, unrelated seam is also closed.** `SSI_ENGINE_FLOOR` (the `k >= 5` floor that routes a
component to the `SignatureEngine` instead of the dense-union DP) at 2 instead of 5 gives
0.790230 → 0.790231, i.e. nothing, which confirms the tree's own note that dropping that floor is
wall-neutral *and* value-neutral.

## 3. Retired: the basin fork (behind `SSI_SHARED_BASIN_FORK=1`)

The fork's dev value is real (6 movers on its band, 0 regressions, ≈−1.2e-4 dev, +0.2 % median added
wall) and its hidden record today is:

| tree | fork | hidden outcome (2026-09-14 corpus) |
|---|---|---|
| `6506934a` | margin 10 % | completed, 0.840946 |
| `7dfe1ee7` | no margin | **killed at 66.6 s** |
| `5d0bd3e6` / `fb541b29` | none | killed at 83.1 / 84.5 s |

Two things follow. First, the fork is the only device in this tree whose cost can put a **cheap** row
over the wall: its own receipt charges **+0.68 s on `himmel11` (n = 14, nnz = 60)** and **+0.83 s on
`syn15hfsg` (n = 399, nnz = 1 022)**, both with zero output change. Second, the margin — which exists
purely to stop exactly that — is the difference between the day's one fork completion and a kill
*earlier than any other*, so the trade is real in the direction the receipt predicted.

Given the daily rotation (§1), the honest reading is not "the fork is bad" but "the fork buys
≈1.2e-4 of dev for a wall profile that this corpus window cannot afford". It is off in production
and the whole family stays reproducible from one seam.

## 4. The tree that ships

Crown behaviour + `XCH_ALLOC = 1` + the census-bounded dense twin (`10/4/4` for
`1_000 <= n <= 5_205`). The fork and the `PEO_ALT_MAX_N` narrowing are compiled in but off
(`SSI_SHARED_BASIN_FORK=1` re-arms; `PEO_ALT_MAX_N` is back at the promoted 50 000).

`126 passed / 0 failed`.

## Links

- Experiments: [0277 the cap priced per stage](0277-cap-margin-census-and-peo-alt-window.md),
  [0276 the shared-prefix fork](0276-shared-prefix-fork-and-bounded-twin.md)
- Evidence: `../evidence/0278d-xch-alloc-{0,1,2}.log`, `../evidence/0278d-submission-note.md`,
  `../evidence/0278b-restored-vs-scored-ab.log`, `../evidence/0278-fork-margin-four-arm-ab.log`
