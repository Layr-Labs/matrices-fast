# 0179 — DegDivNvWfP15 replaces DegP075 on 16k<n≤20k

- **Date:** 2026-09-09
- **Base tip:** `74b6ccd` (local tip 0.793834; official best 0.843490, bar ≤0.843406)
- **Score:** 0.793834 → **0.793834** (0.00)
- **Status:** miss / reverted. Not submitted.

## Hypothesis

On competitive cores the quotient-metric list is
`DegDivNvSqrtWf, DegPlusDegme, DegSqrt, SqDiv, DegDivNvDegme, DegP075`.
`metric_k` is 4 when `n≤16k` else 3. `METRIC_CORE_MAX_N` is 20k.

`DegDivNvWfP15` (`deg/(nv+1) + 0.1 * sign(wf)*|wf|^1.5`) is a same-family
hub-scale alternative to `DegP075`. Swap it in for `DegP075` only on
`16k<n≤20k`, where metric_k is already 3 and the core envelope already
stops at 20k. Leave `n≤16k` and `n>20k` on `DegP075`. Same count of six
variants; no new independent set; METIS untouched.

## What changed

`src/ordering/indep_first.rs` only. Reverted after the miss. HEAD remains `74b6ccd`.

When building the metric task list on competitive cores
(`metric_ok && cn <= METRIC_CORE_MAX_N && cnnz <= METRIC_CORE_MAX_NNZ`):

- if `16_000 < n && n <= 20_000`: last slot `ScoreVariant::DegP075` → `ScoreVariant::DegDivNvWfP15`
- else: last slot stays `ScoreVariant::DegP075`

Unchanged: `metric_k` (4 at n≤16k, 3 above), the other five variants
(`DegDivNvSqrtWf, DegPlusDegme, DegSqrt, SqDiv, DegDivNvDegme`),
`METRIC_CORE_MAX_N` 20k, METIS, set admission, exact-search seeds,
`SIMPLICIAL_PROMOTION_MAX_N`.

List during the run (exactly six, not seven):

- `n≤16k` and `n>20k`: `DegDivNvSqrtWf, DegPlusDegme, DegSqrt, SqDiv, DegDivNvDegme, DegP075`
- `16k<n≤20k`: `DegDivNvSqrtWf, DegPlusDegme, DegSqrt, SqDiv, DegDivNvDegme, DegDivNvWfP15`

## Result

Yukon local **0.793834** vs tip **0.793834** (`/tmp/yukon-run-0179.log`). Fill 0.9258 unchanged. Buckets: lt_1k 0.8875 / 0.9599, 1k_10k 0.8398 / 0.9468, gt_10k 0.6891 / 0.8844 — all unchanged.

Flops-exact vs tip (`/tmp/yukon-run-tip-74b6ccd.log`): **0 better / 0 worse / 300 same**.

Band `16k<n≤20k` (11 public rows): **0 moved**.

Unchanged in band: `chp_shorttermplan2d` 2108049, `supplychainr1_053050` 2173490, `faclay30` 12850431, `edgecross24-115` 28687145, `gams05` 3260266632, `nuclear10a` 58215556, `crudeoil_lee4_10` 186729717, `ringpack_30_2` 2877455, `pooling_sppb5pq` 213571545, `crudeoil_pooling_dt2` 9947938, `pinene200` 1134819.

Short of score `< 0.793734` (got 0.793834, delta 0). Zero band movers. Harness times are `(capped)`; worst `order()` was not separately probed because the score bar and the more-than-one-mover rule already failed. Not submitted. Quotient list restored to DegP075 in every slot.

## Why it won / lost

No public row in `16k<n≤20k` changed flops, so the DegP075 slot is not the ranking pass that wins those cores (or those graphs never take a competitive metric core inside the envelope). Replacing it with DegDivNvWfP15 is a no-op on this corpus. Leave the six-variant list alone.
