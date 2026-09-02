# Experiment 0009: Envelope Expansion & High-Dimension Budget Scaling

- **Date**: 2026-09-02
- **Author**: Antigravity (Gemini 3.8 Flash)
- **Status**: Implemented & verified.
- **Score before**: 0.866980 (fill tiebreak: 0.958532)
- **Score after**: **0.865397** (fill tiebreak: **0.958052**)
- **Improvement**: **−0.001583** (−15.8 basis points / −18.3 bips relative)

---

## 1. Hypothesis & Rationale

Following the success of Experiment 0008 (unifying the relabel multi-start and cycling 6 AMD configurations), we observed two major opportunities:

1. **Expanding the Robust AMD and Relabelled AMF Envelopes**:
   - `ROBUST_MAX_NNZ` had been restricted to $130,000$. However, nonzeros between 130k and 600k contain large KKT matrices such as `cont6-qq` ($n = 120,395, \text{nnz} = 557,994$). By expanding `ROBUST_MAX_NNZ` to $600,000$, we allow non-aggressive and dense-detection disabled AMD variants to optimize these high-leverage graphs.
   - `RELABEL_AMF_MAX_NNZ` was similarly expanded from $130,000$ to $200,000$, enabling approximate minimum-fill multi-start passes on intermediate-density systems.

2. **Scaling High-Dimension Relabel Budgets**:
   - In `gt_10k` ($n \ge 10,000$), each matrix holds $4.36\times$ the benchmark score weight of an `lt_1k` matrix.
   - Many large matrices with $\text{nnz} \ge 250,000$ (such as `transswitch2383wpr` with $\text{nnz} = 277,562$) were receiving only 2 restart attempts under the previous $700,000\,\mu\text{s}$ budget.
   - We increased the `gt_10k` budget to $1,000,000\,\mu\text{s}$ (cap 64), `1k_10k` to $600,000\,\mu\text{s}$ (cap 48), and `lt_1k` to $500,000\,\mu\text{s}$ (cap 48).

---

## 2. Implementation

In [`src/ordering/mod.rs`](../mod.rs):
```rust
const ROBUST_MAX_NNZ: usize = 600_000;
const RELABEL_AMF_MAX_NNZ: usize = 200_000;

#[inline]
fn relabel_budget_and_cap(n: usize) -> (usize, usize) {
    if n >= 10_000 {
        (1_000_000, 64)
    } else if n >= 1_000 {
        (600_000, 48)
    } else {
        (500_000, 48)
    }
}
```

---

## 3. Results Across the 300-Matrix Dev Corpus

### Scores Across Size Buckets
| Bucket | Weight | Prior Best (`0008`) | This Revision (`0009`) | Net Delta |
| :--- | :--- | :--- | :--- | :--- |
| `lt_1k` | 0.30 | 0.906144 | **0.906119** | −0.003% |
| `1k_10k` | 0.30 | 0.895704 | **0.895582** | −0.014% |
| `gt_10k` | 0.40 | 0.816064 | **0.812217** | **−0.471%** |
| **Overall Score** | **1.00** | **0.866980** | **0.865397** | **−15.8 bips** |
| **Fill Tiebreak** | — | **0.958532** | **0.958052** | **−0.050%** |

### Key Matrix Breakthroughs
- **`transswitch2383wpr`** ($n = 59,853, \text{nnz} = 277,562$): Former persistent tie broken! Flop ratio reduced from 1.0000 to **0.9810**.
- **`arki0013`** ($n = 44,909, \text{nnz} = 160,172$): Flop ratio reduced from 0.6288 to **0.6159**.
- **`methanol400`** ($n = 23,999, \text{nnz} = 151,728$): Further reduced from 0.7410 to **0.7373**.
- **`hydroenergy2`** ($n = 2,092, \text{nnz} = 6,236$): Flop ratio reduced from 0.8905 to **0.8845**.
- **`space25a`** ($n = 634, \text{nnz} = 3,246$): Flop ratio reduced from 0.9885 to **0.9846**.

### Timing Profile
- Worst-case runtime across all 300 matrices: **0.917 s**.
- More than 2.1× headroom below the 2.000 s SIGKILL cap.
