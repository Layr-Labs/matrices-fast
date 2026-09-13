# 0230 — sparse-span schedule 9 → 13 widths + exchange-site ledger 512M → 1G

- **Date:** 2026-09-12 (iter51)
- **Score:** production frame, one binary, one session, 4-vCPU (`taskset -c 0-3`), 300 dev rows:
  shipped point **0.791498** (worst 1.389 s) → **0.791437** (−6.1e-5, −0.61 bip); **17 rows better /
  0 worse / 283 identical**; worst `order()` 1.430 s.
- **Status:** shipped (`PRODUCTION_SPAN_WINDOWS` = 13, `PRODUCTION_EXCHANGE_LEDGER` = 1G); official local
  harness 300/300 at **0.791437 / 0.924075** (`results.tsv:1789269816`); submitted **6279dc68** (validating).
- **Evidence:** 0230-arm{P,L,L2,X,XL,XLP}-4cpu.log, 0230-official-run-x13-ledger1g.log, 0230-submission-note.md

## Hypothesis

Every width appended to the sparse-span schedule so far has paid on the shipped point (3 → 5 = −3.0e-5,
5 → 9 = −4.9e-5, promoted as `e07fe7ae`), and each pass accepts only a strict exact decrease, so the next
group's residual value is exactly measurable in-frame. The exchange-site **work ledger** (both
`subset_window_descent_step` call sites) is the same kind of object and had never been priced above 512M.

## The four-arm sweep (all arms `SSI_INDEP_FORCE=1`, same binary, same session, 4 vCPU)

| arm | env | SCORE | worst `order()` |
|---|---|---:|---:|
| P | — (shipped point) | 0.791498 | 1.389 s |
| L | `SSI_EXCHANGE_LEDGER=1G` | 0.791451 | 1.457 s |
| L2 | `SSI_EXCHANGE_LEDGER=2G` | 0.791446 | 1.390 s |
| X | `SSI_SPAN_WINDOWS_EXTRA=1` (+4 widths) | 0.791484 | 1.430 s |
| **XL** | **+4 widths AND ledger 1G (shipped)** | **0.791437** | 1.430 s |
| XLP | XL + `SSI_PEO_ROUNDS=5` | 0.791440 | 1.433 s |

- Arm P reproduces the standing probe receipt exactly, so the arms are row-for-row comparable with the
  tree that is `validating` as `83a8f4fc`.
- **Ledger curve:** 512M → 1G = −4.7e-5; 512M → 2G = −5.2e-5. The knee is at 1G (a further 2× buys 5e-6),
  so 1G ships.
- **PEO 5 is rejected:** 0.791440 is *worse* than XL at the same worst row — the extra round is
  inert-or-negative on this source (it had never been priced in-frame here).
- Movers of XL vs P (all better): `crudeoil_lee4_06` −0.51 %, `powerflow0300p` −0.21 %,
  `crudeoil_lee2_06` −0.15 %, `chimera_mgw-c16-2031-01` −0.126 %, `crudeoil_lee1_07` −0.099 %,
  `rsyn0810m02hfsg` −0.043 %, `sporttournament48` −0.037 %, `mpbp_35` −0.030 %, `mpbp_15` −0.024 %,
  `transswitch0300p` −0.022 %, `crudeoil_pooling_ct1` −0.051 %, `rsyn0840m04m` −0.037 %,
  `chimera_mgw-c8-439-onc8-002` −0.063 %, `popdynm25` −0.004 %, `chimera_lga-01` −0.001 %,
  `chimera_selby-c16-02` −0.002 %, `chimera_rfr-02` −0.004 %.

## Why it is monotone (and what it costs)

Both devices only *append* passes that accept a strict exact decrease of the trusted objective; no gate
is relaxed and no row is admitted that was not admitted before. The price is per-row bounded work:
worst dev `order()` 1.389 → 1.430 s (+0.041 s, +3 %) — the same order as the previously promoted
schedule extension (+0.045 s on its worst row).

## Follow-up parked in a seam

`SSI_SPAN_WINDOWS_NEXT` (test-only) now carries the *next* width group `(26,4,12,32M)`, `(18,4,7,64M)`,
`(4,4,2,32M)`, `(32,4,15,32M)`; it is unmeasured and ships nothing.
