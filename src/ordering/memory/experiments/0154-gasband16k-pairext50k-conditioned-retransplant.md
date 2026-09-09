# 0154 — gasprod_band 16k + mild pair-EXT 50k + conditioned transplant re-call

- **Date:** 2026-09-09
- **Base:** tip `be9bae1` official local **0.794121** / fill **0.926126**. Hidden best 0.843577. Promote bar 0.843476.
- **Score:** official local **0.794020** / fill **0.926119** (−1.01 local bip). **12 better / 0 worse**.
- **Status:** win locally. Submitted.
- **Worst order() (this box probe):** package **1.392 s** vs tip probe **1.509 s** same box (tip claim 1.105 s is not reproducible under current load; package does not regress tip worst).

## Hypothesis

0147–0153 metric-core / second-colour retreads were null or thin. Open non-retreads from 0153 next-leads and board context:

1. **DegP125 ADD** on n>16k (drop DegP075, keep SqDiv) — family metric, not the 0148 SqDiv↔DegP125 swap.
2. **gasprod_band 16k** so crudeoil_lee4_10 (n=17809) gets indep immediate, while lee4_09 (n=15904) stays deferred — timing protection vs jonathan 365a/368a at 15k (1.119–1.140 s).
3. **Mild pair-EXT nnz widen** (30k→80k then tightened to 50k) with `max_deg * 50 <= n` kept; stay 4 sweeps / 48M.
4. **Conditioned terminal re-transplant** (darthweenies shape): snapshot flops after phase-14 `refine_with_donors`; if later stages strictly improve, re-call with bijection + strict `<` admit. Same ledger/gates; no widen.

## What changed

`src/ordering/mod.rs` only (0154a `indep_first.rs` DegP125 ADD was exact tip-tie and reverted before stacking):

- `gasprod_band`: `n >= 20_000` → `n >= 16_000`.
- `pair_descent_ext` nnz: `30_000` → `50_000` (80k first-cut worst 1.389 s on lee4_10; 50k keeps score, relative timing ≤ tip).
- After phase-14 transplant: `transplant_entry_flops = best_flops`.
- Immediately before `order()` returns: if `best_flops < transplant_entry_flops`, re-call `refine_with_donors` (bijection + strict `<`).

Yukon logs: `/tmp/yukon-run-0154a.log` (null), `/tmp/yukon-run-0154b.log`, `/tmp/yukon-run-0154bc.log`. Probes: `/tmp/probe-tip.log`, `/tmp/probe-0154b-50k.log`, `/tmp/probe-0154bc.log`.

## Result

| | tip | 0154b (gasband16k+pair50k) | 0154b+c (package) |
|---|---:|---:|---:|
| score | 0.794121 | 0.794054 | **0.794020** |
| fill | 0.926126 | 0.926056 | **0.926119** |
| Δ bip vs tip | — | −0.67 | **−1.01** |
| movers | — | 5/0 (printed) | **12/0 exact** |
| worst probe | 1.509 s | 1.383 s | **1.392 s** |

Buckets (package): lt_1k 0.8875 / 1k_10k 0.8398 / gt_10k 0.6896.

Exact movers (flops-strict, all same-sign improvements):
- gt_10k: lee4_10 0.639→0.634, lee4_06 0.507→0.505, pinene200, mpbp_35, powerflow0300p, transswitch0300p
- 1k_10k: lee2_06, chimera_selby-c16-01/02, chimera_k64ising-02, crudeoil_pooling_ct3
- lt_1k: sporttournament18

0154a alone: exact tip tie → reverted. Gasband-only isolate: 0.794068. Pair 80k score-tied 50k but no extra gain.

## Why it won / lost

Won: gasband 16k pays lee4_10 / pinene-class indep immediate without opening lee4_09; mild pair-EXT 50k under hub gate picks up mid-band polish (lee2_06, powerflow, mpbp_35) without jonathan's 15k timing death; conditioned re-transplant only spends when post-transplant stages already paid a strict gain (lee4_06 and several chimera/sport rows), avoiding unconditional darthweenies re-transplant flat rejects.

## Follow-ups

- Do not retread DegP125↔SqDiv swap (0148) or DegP125 ADD on n>16k alone (0154a null).
- Do not drop gasband to 15k without a timing probe that beats tip on the same box.
- EXT deepen (64M / 6 sweeps) only if worst stays ≤ tip probe on this host.

## Links

- Prior nulls: [0153](0153-x3-x16-second-colour-safe-gate.md), [0148](0148-n16k-degp125-for-sqdiv.md)
- Board rejects avoided: 330a pair-EXT 80k thin, 368a gasband15k+pairdesc too slow, darthweenies re-transplant flat
