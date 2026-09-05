# 0016 — Sparse gt_10k Network Floor & Restart Unstarving

**Date:** 2026-09-02
**Score:** `0.863609` → **`0.863368`** (−0.000241, −2.4 basis points); fill tiebreak `0.957121` → **`0.957095`**
**Status:** WIN — verified across all 300 matrices.
**Matrix Moves:** Multiple `gt_10k` ties broken:
  - `transswitch2383wpr` ($n=59,853, nnz=277,562$): 3,946,309 → 3,871,226 flops (**−1.90%**)
  - `transswitch2736spr` ($n=69,651, nnz=331,010$): 7,933,133 → 7,858,441 flops (**−0.94%**)
  - `popdynm25` ($n=2,807, nnz=13,904$): 360,081 → 325,123 flops (**−9.71%**)
**Peak Latency:** 0.828 s (`crudeoil_lee4_10`), maintaining a 2.4× safety margin below the 2.000 s timeout.

---

## 1. Algorithmic Context & Motivation

In the highest-weight bucket (`gt_10k`, carrying 40% of the contest score over only 45 matrices), several large grid and transmission network matrices have $nnz \in [250,000, 350,000]$.

Because the restart allocator computed `base_r = (budget / nnz).min(cap)` with `budget = 500_000`, these networks received:
$$\text{base\_r} = \lfloor 500,000 / 331,010 \rfloor = 1 \text{ restart}$$
They were functionally starved of relabelled randomized restarts, running only a single seed ($s = 1$).

When original node orderings follow topological or spatial traversal sequences, standard AMD's degree tie-breaking accumulates fill across boundary interfaces. Evaluating alternative random permutation seeds $s \in 2..=8$ disrupts these spatial tie-break biases.

---

## 2. Implementation

We introduced a targeted density and hub-gated floor for sparse networks in `gt_10k`:

```rust
    } else if nnz <= 350_000 && nnz <= 5 * n && max_deg * 50 <= n && n >= 10_000 {
        base_r.max(8) // Sparse gt_10k mesh/network floor (unstarving transswitch & powerflow)
    }
```

### Safety Constraints:
1. `nnz <= 5 * n`: Strictly limits the floor to sparse planar/transmission networks, preventing dense matrices (like `gams05`, $nnz/n \approx 14.5$) from exceeding their timing budget.
2. `max_deg * 50 <= n`: Excludes degree hubs that would otherwise bottleneck factorization.

---

## 3. Empirical Results

- **`gt_10k` bucket**: dropped from `0.8124` to **`0.8118`** (**−6 basis points**).
- **Composite Score**: dropped from `0.863609` to **`0.863368`** (**−2.4 basis points**).
- **Peak Execution Time**: 0.828 s, down from earlier 1.050 s peaks, fully protected by the $nnz \le 5n$ density filter.
