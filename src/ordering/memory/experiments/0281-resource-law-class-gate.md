# 0281 — the terminal class gate as a resource law; and the union table cannot be narrowed

- **Date:** 2026-09-14 (iter72)
- **Base:** `dc627cd` (the iter71 tree: reset-first + `XCH_ALLOC=1` + twin band), which
  measures **0.790230** on the 300-row dev corpus.
- **Score:** **0.790230 → 0.790133 (−9.7e-5)**, movers 4, regressions 0.
- **Status:** **kept, submitted.** The companion union-table "optimisation" is a
  **verified negative** and is not shipped.

## 1. The class gate was two literals; the rows outside them are sparse

The terminal exact-kernel class was admitted by `n <= 45 000 && nnz <= 200 000`
(plus `nnz <= 16n`, `max_deg <= n/2`). The complement of those two literals on dev
is not the dense tail one would assume from the `nnz` key — it is a set of ordinary
**sparse** patterns, `nnz/n` between 4.6 and 15.6:

| row | n | nnz | nnz/n | excluded by | `n·⌈n/64⌉·8` |
|---|---:|---:|---:|---|---:|
| `gams05` | 17 364 | 252 910 | 15.57 | `nnz` | 37.8 MB |
| `nuclear104` | 39 098 | 257 806 | 7.59 | `nnz` | 191.1 MB |
| `arki0013` | 44 909 | 205 081 | 4.57 | `nnz` | 252.2 MB |
| `transswitch2383wpr` | 59 853 | 337 415 | 5.64 | both | 448.2 MB |
| `transswitch2736spr` | 69 651 | 400 661 | 5.75 | both | 606.8 MB |

The exchange does **not** improve all of them; `arki0013` was already inside the
twin's reach and stays bit-identical. The change is worth −9.7e-5, and the promotion
bar is 8.4e-5.

## 2. What changed

Two literals became a law, with no new gate and no new budget:

* `rgreedy::MAX_N` **45 000 → 80 000** — the largest `n` at which the block's two
  bitset images (immutable pristine + mutable working, `n·⌈n/64⌉` u64 words each)
  still fit the graded 4 GiB `RLIMIT_AS` with headroom. At the new ceiling each
  image is 800 MiB; the largest admitted dev row pays 606 MiB.
* `CLASS_NNZ` (new constant) **200 000 → 400 000**, and `follow_nnz` now derives
  from it instead of repeating the literal.

Nothing else moved: same ledger (2 GiB), same `12/5/12` exchange schedule, same
nine sparse-span passes, same strict exact-flop acceptance, same `max_deg` and
`nnz <= 16n` density keys. Every adoption in the block is a strict flop decrease, so
admitting a row cannot worsen its ratio — the whole price is that row's seconds.

## 3. Result — the movers are exactly the newly admitted rows

One binary / one session / 300 dev rows, graded-closest frame
(`SSI_MARK_NOSCORE=1`). The gate was first priced through the existing test seams
before any code moved, so the measurement is of the *law*, not of a hopeful patch:

| arm | rows | score | lt_1k | 1k_10k | gt_10k |
|---|---:|---:|---:|---:|---:|
| shipped literals | 300 | 0.790230 | 0.8873 | 0.8372 | 0.6822 |
| widest gate (seams) | 300 | **0.790133** | 0.8873 | 0.8372 | **0.6819** |

`lt_1k` and `1k_10k` are untouched — every mover is `gt_10k`.

| row | n | nnz | ratio before → after | delta |
|---|---:|---:|---|---:|
| `transswitch2736spr` | 69 651 | 331 010 | 1.00000 → 0.98782 | −1.218 % |
| `transswitch2383wpr` | 59 853 | 277 562 | 1.00000 → 0.99853 | −0.147 % |
| `nuclear104` | 39 098 | 257 806 | 1.00000 → 0.99885 | −0.115 % |
| `gams05` | 17 364 | 252 910 | 1.00000 → 0.99887 | −0.113 % |

All four start at exactly the AMD anchor (1.00000): the block's first improvement on
those rows is also the row's first. 296 rows are bit-identical.

The three `n > 60 000` rows are the ones that need `MAX_N`; `gams05` and `nuclear104`
need only the `nnz` key. This is why raising one literal alone was worth −1.4e-5
when it was priced in iter58 and the pair is worth −9.7e-5.

## 4. Wall — the ablation receipt this region never had

The class region had no phase mark (`22.win` reads 0.000 s), so its wall share was
never measured in the graded frame. A test-only ablation seam switches the whole
region off, and `probe_slow_row_stage` runs **one real production worker process per
row** with the arms interleaved, 3 reps, `taskset -c 0-3`:

> **The seam is not in the shipped tree.** It was built behind a custom
> `--cfg ssi_tail_ablation` so the graded build could not see it, used for the
> receipt below, and then deliberately removed: even a `cfg`-gated environment read
> does not belong in the class gate. To re-run the receipt, reinstate
> `rgreedy::class_region_enabled` behind that cfg and AND it into
> `terminal_exchange` / `terminal_followup`; `probe_slow_row_stage` already
> exports `SSI_TAIL_OFF=1` for an arm tagged `tailoff`. The shipped score delta
> was separately re-verified *after* the removal.

| row | class on (median) | class off (median) | class share | flop ratio on | off |
|---|---:|---:|---:|---:|---:|
| `catmix400` | 3.562 s | 1.165 s | **67.3 %** | 0.99975 | 0.99975 |
| `chp_partload` | 10.209 s | 3.110 s | **69.5 %** | 0.73282 | 0.73701 |
| `edgecross24-115` | 4.427 s | 2.606 s | **41.1 %** | 0.80144 | 0.81733 |
| `arki0013` | 4.121 s | 2.763 s | 32.9 % | 0.39928 | 0.40209 |
| `methanol400` | 2.206 s | 1.508 s | 31.7 % | 0.69306 | 0.72081 |
| `pinene200` | 2.627 s | 1.898 s | 27.8 % | 0.83359 | 0.83495 |
| `powerflow0300p` | 3.408 s | 2.514 s | 26.2 % | 0.95182 | 0.96180 |
| `chimera_selby-c16-01` | 8.265 s | 6.730 s | 18.6 % | 0.66659 | 0.67764 |
| `crudeoil_pooling_dt3` | 2.724 s | 2.214 s | 18.7 % | 0.69149 | 0.69185 |
| `mpbp_15` | 3.062 s | 2.616 s | 14.6 % | 0.77715 | 0.80078 |
| **12-row total** | **51.58 s** | **35.45 s** | **31.3 %** | | |

Two numbers matter for the next session:

1. **The region is ~31 % of a binding row's wall**, up to **67–70 %** on the rows
   where the ledger binds hardest. That is far more than the 15.9 % the older
   census attributed to the *whole* unmarked terminal tail, and it means this region
   is the highest-leverage wall target on the tree.
2. **Switching it off costs real score** (`catmix400` unchanged, but `methanol400`
   +4.0 %, `powerbase300p` +1.0 %, `mpbp_15` +3.0 %, `powerflow0300p` +1.0 %). The
   region is not dead weight; a wall device for it has to make the same decisions
   more cheaply, not fewer.

`gasprod_sarawak81` and `chp_shorttermplan2d` read *slower* with the region off
(−28.8 %, −13.4 % "saving"): those two are the arms' measurement noise at this box's
load, and they are why the claim is the 31 % aggregate over 12 rows rather than any
single row.

### Where the region's seconds actually go

`COARSE`/`XCHTIME` instrumentation in the same binary, 53 rows with `n >= 9 000`
(one timestamp per *phase*, not per window):

| component | seconds | share of region |
|---|---:|---:|
| `refine_window` calls (enumeration + component admission + DP) | 20.6 | 62 % |
| interleaved `Game::eliminate` replay | 8.1 | 24 % |
| `Game::reset` | 2.6 | 8 % |
| solve scratch, measured inside `solve_component` | 6.9 | — |
| **region total** | **34.5** | **14.3 % of those rows' corpus wall** |

737 373 component solves across those 53 rows; `solve_component`'s own internals
(`salloc` + `spre` + `sunion` + `sbody`) account for **94–98 %** of `refine_window`
on the rows where it binds (`catmix400` 1.72 s of 1.76 s, `chimera_selby-c16-01`
1.35 s of 1.44 s) and 5–15 % on the rows where the signature engine carries it.
The DP is the region; the resets are noise (0.098 s of 4.50 s on `mpbp_07`), which
also retires the hypothesis that the nine sparse-span passes are worth optimising —
they are **14 of 469 sweeps and 6 of 137 934 components**.

## 5. Verified negative: the union table cannot be narrowed to the window

`solve_component` keeps `unions[mask]` as a **global-ID** bitset of `game.w` u64
words — `2^k · n/64` words, 28 MB at `n = 44 909, k = 14` — while
`SignatureEngine::run_dp` keeps the same union as a single `u16` k-bit mask over the
window's own positions. Narrowing the brute path to match the engine is the obvious
win: it removes essentially all of the table, and `sbytes` shows the class region
requesting 10.4 GB of solve scratch across 53 rows.

**It is wrong.** A window is *not* closed under "neighbours of its vertices". An
isolated window vertex has all of its neighbours outside the window, so a
window-local image cannot see the external boundary. Diagnosed with a test-only
`SSI_UNION_DIAG` that recomputes the old global width for every merged mask:

```
UNIONDIAG  n=11659  k=2  mask=2  old=4  new=2  nbhd=[2, 1]  verts=[4806, 10363]
```

`inside = [0, 0]`: the two window vertices are not adjacent, so the component
`{4806}` is a single vertex whose column count is 4 (itself + one internal-boundary
vertex + two external neighbours). The window-local form reads 2. **26 of 53 measured
rows changed their returned flop record**, all of them worse.

The signature kernel uses the local image legitimately because it carries the
external boundary *separately*, through a per-component `hist` histogram and its zeta
transform — not because the window is closed. So the global bitset is load-bearing
and the width must stay `|union_global| − |mask| + 1`.

This is recorded because the change looks obviously safe, is the first thing a
reader of `run_dp` will try, and cost an hour to disprove.

## 6. What was priced and rejected

* **Reusing one `Game` across the nine sparse-span passes.** Exactly the handoff's
  §5.2 suggestion. Measured: the spans are 14 of 469 sweeps and 6 of 137 934
  components on 53 slow rows — the whole family costs 0.02–0.05 s per row against a
  region of 0.5–2.0 s. Not worth the lifetime plumbing.
* **A recycled-scratch pool for `solve_component`'s five vectors.** Bit-identical by
  construction (every vector is fully written before it is read), and worth 5–15 % of
  `refine_window` on the binding rows — but `refine_window` is 62 % of a region that
  is 31 % of a binding row, so the end-to-end ceiling is ~1–2 %, against ~150 lines
  of pool with an `unsafe`-free but delicate type split. Rejected on cost/benefit,
  not on correctness.
* **An earlier iteration's `plateau` stop below `n = 10 000`** (iter60, kept closed):
  unchanged.

## 7. Reproduce

```sh
# price the law with the existing seams, before touching the constants
SSI_TERM_CLASS_N=80000 SSI_CLASS_NNZ=400000 SSI_MAX_N=80000 SSI_MARK_NOSCORE=1 \
  <probe> --ignored --nocapture --test-threads=1 probe_timing_and_score

# the ablation receipt (needs a build that opts in; the shipped build cannot see it)
RUSTFLAGS="--cfg ssi_tail_ablation" cargo build --release -p ssi-candidate-worker
SSI_STAGE_REPS=3 SSI_STAGE_KEEP_ENV=1 \
  SSI_GRADED_WORKERS="on=<probe>,tailoff=<probe>" SSI_PROBE_ONLY=<rows> \
  <probe> --ignored --nocapture --test-threads=1 probe_slow_row_stage
```

Evidence: `iter72/gate-wide.log` (the seam-priced law), `iter72/ship-full.log`
(the production constants), `iter72/tail-ablation.log`, `iter72/coarse2.log`,
`iter72/big-phases.log`.

## Links

- Context: [0280 reset-first](0280-reset-first-adjacency-copy.md),
  [0277 cap census](0277-cap-margin-census-and-peo-alt-window.md),
  [0238 shared class engine ceiling](0238-shared-class-engine-ceiling-45k.md)
- The `MAX_N` ceiling curve that produced the old 45 000 literal is quoted in
  `rgreedy.rs` (0264 receipts); this page moves one point on that curve and adds
  the `nnz` key.
