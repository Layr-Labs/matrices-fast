# 0059 — Narrow 16→0.5 swap: `5k ≤ n < 25k && nnz ≤ 80k`

- **Date:** 2026-09-04
- **Score:** `0.845281` → **`0.844270`** (−10.11 bips); fill **0.944357**
- **Status:** local full-run WIN, all 300 `(capped)`; both unique 0.5 prizes kept; `crudeoil_lee4_10` stays on 16.0

## Why 0058 died

Global 16→0.5 (`d6e006a`) is `failed`, same class as 0057 extras. `crudeoil_lee4_10` is n=17809 **nnz=120632**, inside the 130k AMF sweep. α=0.5 defers fewer dense rows than 16.0, so that cap-owner does more fill-work. H2 kept 16.0 there and passed hidden.

## Gate

Same three tickets everywhere. Swap 16.0 for 0.5 only when

```text
n >= 5_000 && n < 25_000 && nnz <= 80_000
```

| matrix | n | nnz | in gate? |
|---|---:|---:|---|
| `crudeoil_pooling_dt2` | 18742 | 75910 | **yes** (the gt_10k prize) |
| `chp_partload` | 5211 | 16740 | **yes** |
| `crudeoil_lee4_10` | 17809 | 120632 | **no** (nnz>80k; keeps 16.0) |

No fourth ticket. No 130k ceiling raise. No subtree change.

## Follow-ups

If this fails hidden, **stop**. α=0.5 is closed on this envelope, narrow or global. Revert to H2 `{1, 16, −1}` and do not retry H3/R5/0057 extras.
