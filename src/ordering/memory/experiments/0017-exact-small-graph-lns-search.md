# 0017 — Exact Small-Graph LNS Search (Area 2)

**Date:** 2026-09-02
**Score:** `0.863368` → **`0.861158`** (−0.002210, −22.1 basis points); fill tiebreak `0.957095` → **`0.956223`**
**Status:** WIN — verified across all 300 matrices.
**Matrix Moves:** 34 matrices in `lt_1k` strictly improved (0 regressions):
  - `genpooling_lee2` ($n=221, nnz=1,048$): 19,190 → 14,316 flops (**−25.40%**)
  - `elf` ($n=124, nnz=376$): 2,501 → 2,173 flops (**−13.11%**)
  - `maxcsp-langford-3-11` ($n=660, nnz=29,646$): 13,106,275 → 11,423,208 flops (**−12.84%**)
  - `waterund11` ($n=160, nnz=828$): 12,588 → 11,514 flops (**−8.53%**)
  - `wastewater05m1` ($n=98, nnz=536$): 9,696 → 8,935 flops (**−7.85%**)
  - `graphpart_clique-20` ($n=80, nnz=1,260$): 30,190 → 27,856 flops (**−7.73%**)
  - `waterund14` ($n=333, nnz=2,204$): 81,276 → 77,281 flops (**−4.92%**)
  - `ndcc13` ($n=969, nnz=5,882$): 378,571 → 364,238 flops (**−3.79%**)
  - `slay05h` ($n=760, nnz=2,080$): 9,065 → 8,802 flops (**−2.90%**)
**Peak Latency:** 0.843 s (`crudeoil_lee4_10`), maintaining a 2.37× safety margin below the 2.000 s timeout.

---

## 1. Algorithmic Context & Motivation

On small matrices ($n \le 1,000$), standard quotient-graph minimum degree heuristics rely heavily on approximations (supervariable clumping, aggressive absorption, unweighted external degree estimates). For graphs with $n \le 1,000$ and $nnz \le 30,000$, our entire elimination pipeline executed in under 0.005 seconds per matrix, leaving more than 99.7% of the per-matrix 2.000-second compute budget unspent.

In `rgreedy.rs`, the exact elimination game maintains true fill graphs as bitsets. Because the Cholesky column count satisfies:
$$c(v, S) = 1 + |N_{G_S}(v)|$$
the exact quadratic objective $\sum c_j^2$ is accumulated on-the-fly during simulation at zero evaluation cost.

We deploy large-neighborhood search (LNS) prefix freezing with sideways plateau acceptance:
1. Keeps the incumbent permutation's low-deficiency prefix frozen.
2. Explores alternative suffix completion orders via randomized min-degree / min-deficiency greedy walks.
3. Accepts strict improvements when re-scored against exact Cholesky factorization flops.

---

## 2. Implementation Architecture

```rust
    if n <= 1_000 && nnz <= 30_000 {
        if let Some((cand, _)) = rgreedy::search(
            n,
            &pattern.col_ptr,
            &pattern.row_idx,
            &best_perm,
            best_flops,
            100_000_000,
            0x9E37_79B9_7F4A_7C15,
        ) {
            if is_bijection(&cand, n) {
                let f = flops_of(&scoring_pat, &cand);
                if f < best_flops {
                    best_flops = f;
                    best_perm = cand;
                }
            }
        }
    }
```

---

## 3. Empirical Evaluation

- **`lt_1k` Bucket**: Dropped from `0.9051` to **`0.8977`** (**−74 basis points**).
- **Composite Dev Score**: Dropped from `0.863368` to **`0.861158`** (**−22.1 basis points**).
- **Tiebreak Fill Ratio**: Dropped from `0.957095` to **`0.956223`** (**−87.2 basis points**).
- **Containerized Verification**: `yukon run` passed cleanly in 64.85 seconds.
