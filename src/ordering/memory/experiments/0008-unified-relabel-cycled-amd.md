# Experiment 0008: Unified Relabel Multi-Start with Cycled AMD Options & Expanded Dimension Budgets

- **Date**: 2026-09-02
- **Author**: Antigravity (Gemini 3.8 Flash)
- **Status**: Implemented & verified.
- **Score before**: 0.870672 (fill tiebreak: 0.960535)
- **Score after**: **0.866980** (fill tiebreak: **0.958532**)
- **Improvement**: **−0.003692** (−36.9 basis points, −42.4 bips relative)

---

## 1. Hypothesis & Rationale

Prior work (0003, 0005, 0006, 0007) established two independent multi-start restart loops:
1. Relabelled AMD, which alternated only between standard library AMD (`aggressive = true, dense_alpha = 10.0`) and non-aggressive AMD (`aggressive = false, dense_alpha = 10.0`).
2. Relabelled AMF, which cycled across 5 distinct `dense_alpha` values.

### The Redundancy
Because the two loops were executed sequentially, for each restart index $r \in [0, \text{restarts})$:
- Random permutation vector $q = \text{relabel}(n, \text{seed})$ was computed twice.
- The adjacency permutation `permute_pattern(&scoring_pat, &q)` was performed twice.
- Vector allocations (`bcp`, `bri`) and `CscPattern` construction were performed twice.

Since graph permutation is an $O(n + \text{nnz})$ allocation and indexing operation, redundant permutation consumed up to 30% of the relabel loop's latency budget without providing any algorithmic variety.

### Diversity of Elimination Objectives
While the baseline ordering portfolio recognized that disabling dense-row detection (`dense_alpha = -1.0`) yielded distinct, frequently superior elimination orders across KKT saddle-point blocks, this parameter was **never varied** during relabelled AMD restarts.

By cycling AMD across 6 parameter configurations:
1. `(aggressive: true, dense_alpha: 10.0)`
2. `(aggressive: false, dense_alpha: 10.0)`
3. `(aggressive: true, dense_alpha: -1.0)` (dense detection disabled)
4. `(aggressive: false, dense_alpha: -1.0)` (dense detection disabled, non-aggressive)
5. `(aggressive: true, dense_alpha: 5.0)`
6. `(aggressive: false, dense_alpha: 2.0)`

Each randomized topological draw explores structurally distinct quotient-graph elimination sequences. Furthermore, sharing the constructed `bcore` directly with the AMF multi-start allows both AMD and AMF passes to evaluate candidate orderings on the same permutation without duplicate allocation overhead.

Finally, because worst-case runtime across all 300 matrices was measured at only 0.57s (leaving >3.5× headroom under the 2.000s SIGKILL cap), we expanded the dimensional restart budgets:
- $n \ge 10,000$ (`gt_10k`): budget $700,000\,\mu\text{s}$, cap 48 (was $500,000\,\mu\text{s}$, cap 36)
- $1,000 \le n < 10,000$ (`1k_10k`): budget $500,000\,\mu\text{s}$, cap 36 (was $400,000\,\mu\text{s}$, cap 30)
- $n < 1,000$ (`lt_1k`): budget $400,000\,\mu\text{s}$, cap 32 (was $300,000\,\mu\text{s}$, cap 24)

---

## 2. Implementation

In [`src/ordering/mod.rs`](../mod.rs):
1. Merged the separate AMD and AMF loops into a single unified pass over $r \in [0, \text{restarts})$.
2. Reused `bcore` across both AMD and AMF candidate evaluations.
3. Added the 6-way `amd_configs` cycling schedule.
4. Scaled the dimensional restart budgets in `relabel_budget_and_cap`.

---

## 3. Results Across the 300-Matrix Dev Corpus

### Overall Score & Buckets
| Bucket | Weight | Baseline (`0007`) | New Candidate (`0008`) | Relative Change |
| :--- | :--- | :--- | :--- | :--- |
| `lt_1k` | 0.30 | 0.9064 | **0.9061** | −0.03% |
| `1k_10k` | 0.30 | 0.8963 | **0.8957** | −0.07% |
| `gt_10k` | 0.40 | 0.8247 | **0.8161** | **−1.04%** |
| **Overall Score** | **1.00** | **0.870672** | **0.866980** | **−36.9 bips** |
| **Fill Tiebreak** | — | **0.960535** | **0.958532** | **−0.21%** |

### Key Matrix Breakthroughs (26 matrices strictly improved)
- `methanol400` ($n = 23,999, \text{nnz} = 151,728$): Flop ratio reduced from 1.0000 to **0.7410** (−25.9% reduction in factorization flops).
- `crudeoil_lee4_09` ($n = 15,904, \text{nnz} = 101,792$): Flop ratio improved from 0.8938 to **0.8022** (−10.2%).
- `crudeoil_lee4_10` ($n = 17,809, \text{nnz} = 120,632$): Flop ratio improved from 0.7758 to **0.7450** (−3.97%).
- `nd_netgen-3000-1-1-b-b-ns_7` ($n = 33,155, \text{nnz} = 90,000$): Flop ratio improved from 0.9860 to **0.9676** (−1.87%).
- `chimera_selby-c16-02` ($n = 2,031, \text{nnz} = 10,878$): Flop ratio improved from 0.5803 to **0.5557** (−4.24%).
- `wastewater05m1` ($n = 98, \text{nnz} = 536$): Flop ratio improved from 0.8400 to **0.8294** (−1.26%).
- `chimera_mgw-c8-439-onc8-002` ($n = 440, \text{nnz} = 3,038$): Flop ratio improved from 0.9617 to **0.9477** (−1.46%).

### Latency Profile
- Worst-case runtime across all 300 matrices: **0.600 s** (on `arki0016`).
- Over 3.3× safety margin against the 2.000 s SIGKILL limit.
