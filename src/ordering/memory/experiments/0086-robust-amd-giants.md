# 0086 — Expand robust (non-aggressive) AMD onto nnz>600k giants

- **Date:** 2026-09-06
- **Base:** 4d86414 + 0085 (PEO_ALT_SEEDS 8 + extra-relabel seed capture)
- **Score:** 0.826784 → **0.826558** (−0.000226, 2.73 bip). gt_10k 0.752193 → 0.751781. gabriel10 1.000 → 0.976 (1365442310 → 1332170335). faclay75/acopf/kissing2 unchanged. 68 tests pass.
- **Status:** local green. Submitting vs hidden 0.859573 (1 bip bar ≲ 0.859487).

## Hypothesis

0085 was +0.74 bip, all in 1k_10k; gt_10k stayed 0.752193. The remaining
gt_10k 1.000s include `gabriel10` (n=244056, nnz=1.15M) which sits outside
`ROBUST_MAX_N=150k` / `ROBUST_MAX_NNZ=600k`. Non-aggressive AMD is a
different elimination order at AMD speed. One-row probe: gabriel10
1365442310 → 1332170335 (2.44%), 0.46 s; acopf and kissing2 unchanged;
worst of those three 0.52 s.

## What changed

- `ROBUST_MAX_N` 150_000 → 350_000
- `ROBUST_MAX_NNZ` 600_000 → 1_700_000
- Inner 0064 law unchanged: above nnz 150k only α-10 non-aggressive runs
- 0085 seed capture kept
- RCM/Sloan restore and AMF_SWEEP_MAX_N raise both probed dead; not shipped

## Result

pending full 300

Full 300: **0.826558** / fill 0.937286 (lt_1k 0.889730 / 1k_10k 0.863089 / gt_10k 0.751781). Only gabriel10 moved among the five newly admitted rows.
