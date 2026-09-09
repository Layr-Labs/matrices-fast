# 0183 — second-colour unbounded exclude on 18k–22k nnz≤80k

- **Date:** 2026-09-09
- **Base tip:** `74b6ccd` (local tip 0.793834; official best 0.843490, bar ≤0.843389)
- **Score:** 0.793834 → **0.793834** (0.00)
- **Status:** miss / reverted. Not submitted.

## Hypothesis

Second-colour sets are admitted only when `n <= 18_000 && nnz <= 80_000`.
Degree-cap 15 is in `extra_caps` when `n <= 20_000 || nnz <= 400_000`.
On the intersection `sc_ext = n > 18_000 && n <= 22_000 && nnz <= 80_000`,
same-count swap: drop degree-cap 15 and add one unbounded second-colour
exclude `greedy_independent_set_excluding(sp, usize::MAX, &g_inf)`. Do not
also push the cap-15/9/5 excluding sets. Caps 9/5/3 stay. Outside `sc_ext`,
extra_caps are unchanged (15 stays). The existing `n <= 18_000 && nnz <= 80_000`
second-colour block stays unbounded/15/9/5. Not a 0155 x9 widen on n>18k.
No 4th metric core.

Public Pattern rows in that slice (off-diagonal nnz, what `order()` sees):
crudeoil_pooling_dt2 (n=18742, nnz=75910), emfl100_5_5 (n=21925, nnz=52300).
More than one. lee4_10 (n=17809) is outside. gabriel09 nnz=89702 is outside.
nd_netgen-2000-3-4-b-a-ns_7 (n=22074) is outside `n <= 22_000`. On both
public rows the unbounded exclude, after budget trim against ledger
8_000_000, had a new `(xs, pairs)` versus the remaining admitted caps, so
the drop would have paid a new set. Slice not empty; yukon ran.

Keep only if more than one row in the slice moves, local beats 0.793834,
no worse rows, and worst order() does not rise. Submit only if score
≤0.793734, or (≥0.00008 better AND 0 worse), and more than one row in the
slice moves, and no row anywhere is worse. Drop if worst order() rises.

## What changed

`src/ordering/indep_first.rs` only. Reverted after the miss. HEAD remains
`74b6ccd`. probe.rs left untouched (trailing blank line). Light `amf_alphas`
left alone.

- `sc_ext = n > 18_000 && n <= 22_000 && nnz <= 80_000`
- on `sc_ext`, skip extra_caps degree-cap 15; push one
  `greedy_independent_set_excluding(sp, usize::MAX, &g_inf)`
- no second-colour 15/9/5 on that slice
- n≤18k second-colour block unchanged (unbounded/15/9/5)

## Result

Yukon local **0.793834** vs tip **0.793834** (`/tmp/yukon-run-0183.log`).
Fill 0.9258 → 0.9258. Buckets unchanged at 4 d.p.: lt_1k 0.8875 / 0.9599,
1k_10k 0.8398 / 0.9468, gt_10k 0.6891 / 0.8844.

Flops-exact vs tip (`/tmp/yukon-run-tip-74b6ccd.log`): **0 better / 0 worse / 300 same**.

Slice 18k<n≤22k && nnz≤80k: **0/2 moved**.

- crudeoil_pooling_dt2 (n=18742, nnz=75910): flops 9947938 → 9947938, ratio 0.664
- emfl100_5_5 (n=21925, nnz=52300): flops 221650 → 221650, ratio 1.000

Does not beat 0.793834. Short of score ≤0.793734 and short of ≥0.00008
better with 0 worse. Public table redacts all 300 times as capped; no
per-matrix cap kill. Not submitted. `indep_first.rs` reverted.

## Why it won / lost

The new unbounded exclude was a distinct admitted `(xs, pairs)` on both
public slice rows (crudeoil exclude (6866, 146191) vs remaining caps;
emfl exclude (9350, 27250)), so the same-count drop of cap 15 did pay a
new AMD walk. That walk never beat the incumbent portfolio on either row,
and no other public row moved. The second-colour ticket that wins below
18k does not transfer to this band as a replacement for degree-cap 15.
Do not retry this same-count swap, and do not widen it to second-colour
15/9/5 on n>18k.

## Follow-ups

- none

## Links

- Techniques: independent-set-first lift
