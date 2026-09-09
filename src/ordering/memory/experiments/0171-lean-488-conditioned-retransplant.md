# 0171 — lean 488-like + conditioned re-transplant (timing-conservative)

- **Date:** 2026-09-09
- **Base tip:** `74b6ccd` (board best 0.84349 / 723bf797/475a; local tip 0.793834)
- **Score:** 0.793834 → **0.793785** (−0.49 bip)
- **Status:** miss / reverted. Not submitted.

## Hypothesis

jonathan 488a (mid_force 7–22k/nnz48k + FINAL_FIVE 14k/100k/192M + REDUCE deep 120k/250k + PAIR96M) got local **0.793790** / worst **1.114s** → hidden **0.843417** (0.73 bip, rejected thin). Need ~0.28 bip more than 488 and a tighter worst-case than jonathan’s ~1.14s prefer. Lean package: keep FINAL_FIVE_OPS at **128M** (not 192M), skip PAIR96M / SUBTREE / gasband / heavy-metric / extrarelbl, and add unused **0154c conditioned** terminal re-transplant (gain-gated — not the banned unconditional re-transplant).

Target: local ≤ **0.793734** (≥1 bip) or strong ≥0.8 bip with 0 worse; abort submit if worst order() > **1.105s**.

## What changed

`src/ordering/mod.rs` only:

1. **mid_force expand:** `(7_000..22_000).contains(&n) && nnz >= 48_000` (was 8_000..20_000 && nnz≥50_000).
2. **FINAL_FIVE widen, keep OPS:** `MAX_N=14_000`, `MAX_NNZ=100_000`, **OPS=128_000_000** (do not raise to 192M).
3. **REDUCE deep (487):** `REDUCE_RECURSE_MAX_NNZ` 150k→**250k**; `REDUCE_RECURSE_DEEP_MAX_CORE_N` 80k→**120k**; leave `DEEP_MAX_CORE_NNZ` at 250k.
4. **Conditioned terminal re-transplant (0154c):** after first phase-14 `refine_with_donors`, snapshot `transplant_entry_flops = best_flops` and clone `donor_perms` from `runner_up` when non-empty. Immediately before the final `best_perm` return (after subset_window stages), if `best_flops < transplant_entry_flops` (strict — later stages paid a gain), call `refine_with_donors` again; admit on bijection + strict `score < best_flops`. Skip if first transplant never ran / donors empty. Do **not** widen `TRANSPLANT_LEDGER`. Do **not** call unconditionally.

Unchanged: PAIR_DESCENT_* ops/nnz, SUBTREE_CHAIN_MAX_N, gasprod_band, HEAVY_METRIC_ORDER, indep_first metrics.

## Result

Yukon local **0.793785** vs tip rebaseline **0.793834** (`/tmp/yukon-run-0171.log`, tip `/tmp/yukon-run-tip-74b6ccd.log`). Short of the 1 bip local bar (need ≤ 0.793734) and the ≥0.8 bip strong-submit gate. Buckets: lt_1k 0.8875, 1k_10k 0.8398, gt_10k 0.6890.

Flops-exact vs tip: **8 better / 0 worse / 292 same**. Better: crudeoil_lee4_06 (−154155), nuclear10a (−104139), chimera_selby-c16-02 (−722), glider400 (−250), chimera_selby-c16-01 (−239), crudeoil_pooling_ct3 (−93), sporttournament18 (−30), chimera_k64ising-02 (−23). Ratio-strict 4/0. Timing not probed (0.49 bip < 0.5 bip probe gate; submit bar missed). `src/ordering/mod.rs` restored to tip `74b6ccd`.

## Why it won / lost

Lean 488 package without PAIR96M / 192M FINAL_FIVE still moves 8 rows with **0 worse** — cleaner than 0170's 6/1 and slightly better locally than jonathan 488a (0.793785 vs 0.793790) despite keeping FINAL_FIVE_OPS at 128M and skipping PAIR96M. Absolute geomean gain is still only −0.49 bip: far from the 1 bip / ≤0.793734 promote bar and from the ~0.28 bip-beyond-488 needed for a ≥1 bip hidden shot past 0.843389. Conditioned re-transplant did not add a large second-wave win visible in the public table; mid_force + FINAL_FIVE widen + REDUCE deep are the likely movers (lee4_06 / nuclear10a / glider400 sit near those gates).

## Follow-ups

- Do not submit. Keep page; tip restored.
- No mandatory 0172: no single-component regression to drop; package is 0-worse but thin.
- Further lean stacks that re-add PAIR96M or FINAL_FIVE 192M are closed (CI / timing). Conditioned re-transplant alone as an isolate remains unused-as-isolate on this tip if a future thin experiment wants it.

## Links

- Rival intel: jonathan 488a mid_force + FINAL_FIVE + REDUCE deep + PAIR96M → 0.793790 / 1.114s / hidden 0.843417
- Closed: 0163–0170; unconditional re-transplant; FINAL_FIVE_OPS>192M; PAIR_EXT aggressive
