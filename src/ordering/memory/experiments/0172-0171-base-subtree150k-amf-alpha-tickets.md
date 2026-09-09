# 0172 — 0171 base + subtree-chain 150k + additive AMF α tickets

- **Date:** 2026-09-09
- **Base tip:** `74b6ccd` (board best 0.84349 / 723bf797/475a; local tip 0.793834)
- **Score:** 0.793834 → **0.793808** (−0.26 bip)
- **Status:** miss / reverted. Not submitted.

## Hypothesis

0171 lean 488 package (mid_force 7–22k/nnz48k + FINAL_FIVE 14k/100k@128M + REDUCE deep 120k/250k + conditioned re-transplant) was clean **8/0** at local **0.793785** (−0.49 bip) but thin vs ≤0.793734 / ≥0.8 bip submit bar. Reuse 0171 as base stack and add two complementary pieces that previously moved public rows without worse:

- **0163** first-round `SUBTREE_CHAIN_MAX_N` 45k→150k replacing late PEO-large on 45k<n≤150k nnz<1.2M (−0.07 bip, 3/0 on n>45k).
- **Additive AMF α tickets** `da ∈ {0.5, 2.5}` with distinct seeds (do **not** reshape `amf_alphas` — modulo reshuffle risk). Best-of floor ⇒ score-safe; timing is the risk.

Target: local ≤ **0.793734** with 0 worse, or ≥0.8 local bip / 0 worse with worst order() ≤ **1.105s**. Hidden promote bar ≤ **0.843389**.

## What changed

`src/ordering/mod.rs` only:

**Restore 0171 stack (A–D):**
1. **mid_force:** `(7_000..22_000).contains(&n) && nnz >= 48_000`
2. **FINAL_FIVE:** `MAX_N=14_000`, `MAX_NNZ=100_000`, **OPS=128_000_000** (not 192M)
3. **REDUCE deep:** `REDUCE_RECURSE_MAX_NNZ` 150k→**250k**; `REDUCE_RECURSE_DEEP_MAX_CORE_N` 80k→**120k**; `DEEP_MAX_CORE_NNZ` stays 250k
4. **Conditioned terminal re-transplant (0154c):** after phase-14 `refine_with_donors`, snapshot `transplant_entry_flops` and clone `donor_perms` when non-empty; before final return, re-call only if `best_flops < transplant_entry_flops`; admit on bijection + strict `<`. No `TRANSPLANT_LEDGER` widen; not unconditional.

**NEW E — subtree-chain past 45k (0163 semantics):**
- `SUBTREE_CHAIN_MAX_N` 45_000→**150_000**
- `SUBTREE_CHAIN_ROUNDS_MAX_N` = **45_000** (follow-up rounds + improved==0 retry + MINL subtree ticket stay here)
- Historical full chain: `n ≤ 45k && nnz ≤ 1.5M`
- Extended first-round only: `45k < n ≤ 150k && nnz < 1.2M` (streams=1 via `subtree_cfg_for`); late PEO-large skipped on that slice

**NEW F — additive AMF α tickets:**
- After existing relabelled-AMF loop (`nnz <= RELABEL_AMF_MAX_NNZ`), extra `consider_cached` for `da in [0.5, 2.5]` × `r in 0..amf_restarts.min(4)` with seeds `10_000+r` / `20_000+r`
- Existing `amf_alphas` array untouched

Unchanged / closed this run: PAIR ops, RELABEL_METRIC 180k, FINAL_FIVE_OPS 192M, extrarelbl, gasband16k, heavy-metric invention, unconditional re-transplant.

## Result

Yukon local **0.793808** vs tip rebaseline **0.793834** (`/tmp/yukon-run-0172.log`, tip `/tmp/yukon-run-tip-74b6ccd.log`). Fill 0.925837. Buckets: lt_1k 0.8875, 1k_10k 0.8398, gt_10k 0.6891.

Flops-exact vs tip: **12 better / 2 worse / 286 same**.

Better (tip→0172 cand flops): crudeoil_lee4_06 (−154155), nuclear10a (−104139), cont6-qq (−19824), chimera_selby-c16-01 (−12719), transswitch2736spr (−4277), transswitch2383wpr (−1983), chimera_selby-c16-02 (−722), glider400 (−250), crudeoil_pooling_ct3 (−93), chp_shorttermplan2a (−57), sporttournament18 (−30), chimera_k64ising-02 (−23).

Worse: **chp_shorttermplan2d** (+14831, ratio 0.516→0.519), **pooling_sppa0pq** (+72).

n>45k vs tip: **3 better / 0 worse** (cont6-qq, transswitch2736spr, transswitch2383wpr) — matches 0163 isolate.

vs 0171 log: 5 better / 2 worse / 293 same. The 0171 8/0 clean set is retained on those rows; E adds the three n>45k wins; F (or path interaction from F) introduces both regressions (both n≤45k, outside E's extended gate).

Short of ≤0.793734 (1 bip) and ≥0.8 bip strong-submit. Absolute gain −0.26 bip is **worse** than 0171 alone (−0.49 bip) because the two worse rows outweigh E's tiny −0.07 bip. Timing not probed (bar missed). `src/ordering/mod.rs` restored to tip `74b6ccd`.

## Why it won / lost

0171 stack still moves the mid_force/FINAL_FIVE/REDUCE rows. 0163-style first-round chain past 45k again moves exactly the three public n>45k rows with 0 worse on that slice. Additive AMF α{0.5,2.5} tickets are best-of at consider-time but path-dependent polish produced two final regressions (shorttermplan2d dominates the geomean loss). Stacking E+F on 0171 therefore lost the 0-worse property that made 0171 the clean thin package.

## Follow-ups

- Do not submit. Keep page; tip restored.
- Optional 0173 (0171+E, drop F) would likely restore 0-worse and add ~0.07 bip on top of 0171's −0.49 → still ~−0.56 bip, short of ≤0.793734. Skipped this turn (not a lone one-row regression; bar still out of reach).
- Do not reshape `amf_alphas` array; additive tickets need a structural gate if revisited (e.g. nnz/n band that excludes shorttermplan2d).

## Links

- [0171](0171-lean-488-conditioned-retransplant.md) base stack
- [0163](0163-subtree-chain-past-45k.md) subtree-chain replacement semantics
