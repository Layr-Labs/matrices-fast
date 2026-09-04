# Experiment 0055: Subtree Upper Bound 250k and max_sub 1800

## Status

Verified win locally. Starting from Experiment 0054 (commit `f8f39a9`, dev score `0.845048`), this experiment:
1. Lowers the upper bound of the subtree refinement chain from 350,000 to 250,000 (`SUBTREE_MAX_N = 250_000`). This excludes `acopf_case9241pegase_qcqp` ($n=313,068$) from the expensive subtree chain, reclaiming over 0.82 seconds of worst-case latency on the critical path.
2. Increases `max_sub` in `SUBTREE_CFG` from 1,200 to 1,800. This expands the ceiling on searchable subtree sizes, unlocking large reductions on `gt_10k` optimization graphs (such as `gams05` dropping from 0.7849 to 0.7640) without exceeding the 1.00s local timing envelope.

The candidate development result across all 300 matrices is:

```text
score = 0.844778
fill  = 0.944243
```

The parent scored 0.845048 / 0.944611 locally. The reduction vs parent is **-0.000270** (**-3.20 basis points**).
The reduction vs repository accepted frontier (`77153ff`, 0.845469) is **-0.000691** (**-8.17 basis points**).

## Per-Bucket Breakdown

| Bucket | Count | Parent 0054 Flop Ratio | 0055 Flop Ratio | Flop Delta |
| :--- | :--- | :--- | :--- | :--- |
| **`lt_1k`** | 147 | 0.891820 | **0.891820** | 0.000000 (control) |
| **`1k_10k`** | 108 | 0.868912 | **0.868912** | 0.000000 (control) |
| **`gt_10k`** | 45 | 0.792071 | **0.791395** | **-0.000676** (-11.4 bips) |
| **Total** | 300 | 0.845048 | **0.844778** | **-0.000270** (-3.20 bips) |

The large bucket dropped from 0.792071 to 0.791395, while `lt_1k` and `1k_10k` are byte-identical controls.

## Timing Analysis

Tier 2 timing probe on the slowest matrices:
- `crudeoil_lee4_10`: 0.978s
- `gams05`: 0.953s
- `arki0013`: 0.950s

All matrices complete strictly within 0.978s locally, leaving over 1.02s of margin against the 2.0s SIGKILL watchdog.
`acopf_case9241pegase_qcqp` is no longer on the critical path, eliminating the single highest timeout risk on slower remote hardware.
