# 0178 — medium exact second-stream seed (else floor only)

- **Date:** 2026-09-09
- **Base tip:** `74b6ccd` (local tip 0.793834; official best 0.843490, bar ≤0.843406)
- **Score:** 0.793834 → **0.793807** (−0.27 bip / −0.000027)
- **Status:** miss / reverted. Not submitted.

## Hypothesis

The medium exact-search else branch (tip 4-ticket floor, also forced on
`danger_timing`) repeats seed `0xD1B5_4A32_D192_ED03` at 50M after the 100M
ticket of the same seed. A disjoint fixed seed on that second ticket can
walk a different plateau without adding a stream, changing a budget, or
moving `medium_exact_gate`. The well_below and n≤3k tables already use
other second tickets and stay untouched.

## What changed

`src/ordering/mod.rs` only. Reverted after the miss. HEAD remains `74b6ccd`.

Else branch (danger_timing and the default) only:

- second stream seed `0xD1B5_4A32_D192_ED03` → `0x6A09_E667_F3BC_C909` at the existing 50M budget
- 100M `0xD1B5_4A32_D192_ED03` ticket unchanged
- well_below table, n≤3k table, and small_streams unchanged
- no extra stream, no budget change, gate unchanged (`n > 1000 && n <= 6000 && (nnz <= 30_000 || well_below && nnz <= 50_000)`)

## Result

Yukon local **0.793807** vs tip **0.793834** (`/tmp/yukon-run-0178.log`). Fill 0.9258 → 0.9258 (0.925752). Buckets: lt_1k 0.8875 unchanged, 1k_10k 0.8398 → 0.8397, gt_10k 0.6891 unchanged.

Flops-exact vs tip: **4 better / 1 worse / 295 same**. All movers are inside `medium_exact_gate` and on the danger_timing else floor (`n>=1800`, `nnz>=9000`).

Better: `chimera_selby-c16-01` 2431621 → 2429028 (−2593), `mpbp_46` 448113 → 447665 (−448), `rsyn0840m02m` 45153 → 44761 (−392), `sfacloc2_4_80` 89175 → 88884 (−291).

Worse: `crudeoil_pooling_ct3` 867796 → 869779 (+1983).

Short of score `< 0.793734` (got 0.793807, delta 0.000027). One worse row. Harness times are `(capped)`; worst `order()` was not separately probed because the score bar and the no-worse rule already failed. Not submitted. Else-branch seed restored to `0xD1B5_4A32_D192_ED03`.

## Why it won / lost

Redrawing the 50M repeat of the 100M seed does move danger-band plateaus, but the one regression on `crudeoil_pooling_ct3` and the net −0.27 bip are below the keep bar. Same-count seed swaps on this floor are not free. Leave the exact-search seeds alone.

## Follow-ups

- Do not add a third else-branch stream, and do not retune this 50M seed again without a different payment.
- Do not touch the well_below or n≤3k medium tables as a follow-on to this miss.

## Links

- Medium exact search [0020](0020-medium-exact-search.md)
