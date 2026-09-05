# 0012 — Terminal Adjacent-Pair Descent (Local Search on Exact Objective)

**Date:** 2026-09-02
**Score:** `0.864899` → **`0.864652`** (−0.000247, −2.9 basis points); fill tiebreak `0.957753` → **`0.957588`**
**Status:** WIN — adapted from historical research (`rgreedy` adjacent-pair descent in `ssi-ordering-challenge`), verified on all 300 matrices.
**Matrix Moves:** 40 matrices strictly improved, 0 worse.
**Peak Latency:** 0.820 s (`crudeoil_lee4_10`), safely under the 2.000 s SIGKILL timeout.

---

## 1. Algorithmic Context & Motivation

All candidate generation mechanisms in `order()` (AMD, AMF, METIS, Scotch, KaHIP, MinFill, RCM, Sloan, and their relabelled multi-starts) generate complete elimination orders through global heuristics. However, because these heuristics make decisions based on localized degree or deficiency bounds, the resulting permutation often contains local ordering inversions among adjacent vertices.

If two adjacent vertices $(u, v)$ in the current permutation $\pi$ are also adjacent in the elimination graph $G_{\pi}$, their relative elimination order directly determines local fill:
- If $\pi$ eliminates $u$ before $v$, but $\text{deg}_{G}(v) < \text{deg}_{G}(u)$, swapping $(u, v) \to (v, u)$ eliminates the smaller-degree vertex first, strictly reducing the number of fill edges added to the remaining quotient graph.

In `rgreedy.rs`, `adjacent_pair_descent` performs alternating-parity sweeps over the incumbent permutation $\pi$. At each step, it simulates the exact forward elimination game using bitset graph representations:
- When adjacent vertices $(a, b)$ satisfy $(a \sim b)$ and $\text{deg}(b) < \text{deg}(a)$, they are transposed.
- The candidate is re-scored against the exact Cholesky objective $\sum c_j^2$.
- The candidate is accepted if and only if it strictly improves over `best_flops`.

Because it operates as a terminal phase on `best_perm` and accepts strictly lower flop counts, **it is mathematically monotonic** — zero regression is possible anywhere in the corpus.

---

## 2. Gating & Budget Constraints

To ensure strict compliance with the 2.000 s timeout:
- **Core Window**: $1,000 \le n \le 4,000$ and $nnz \le 60,000$, budget = 128,000,000 bitset operations (~0.02 s).
- **Extension Window**: $4,000 < n \le 12,000$, $nnz \le 30,000$, and non-hub gate `max_deg * 50 <= n`, budget = 48,000,000 bitset operations (~0.01 s).

---

## 3. Results & Attributions

40 matrices improved across the development corpus:
- `chimera_selby-c16-01`: 2,490,319 → 2,444,957 (−1.82%)
- `syn15m04m`: 25,606 → 25,403 (−0.79%)
- `nuclear25a`: 1,349,466 → 1,339,276 (−0.76%)
- `chimera_lga-01`: 546,478 → 543,497 (−0.55%)
- `syn40m04m`: 64,448 → 64,124 (−0.50%)
- `rsyn0810m04m`: 133,843 → 133,220 (−0.47%)
- `chimera_k64ising-02`: 382,544 → 381,060 (−0.39%)
- `syn40m04hfsg`: 74,769 → 74,526 (−0.33%)
- `chp_shorttermplan1a`: 331,297 → 330,292 (−0.30%)
- `space25`: 22,819 → 22,753 (−0.29%)
- `oil`: 36,159 → 36,058 (−0.28%)
- `hydroenergy2`: 60,552 → 60,380 (−0.28%)
- `pooling_sppa0pq`: 1,515,018 → 1,511,075 (−0.26%)
- `rsyn0840m02m`: 47,320 → 47,204 (−0.25%)
- `popdynm25`: 325,907 → 325,103 (−0.25%)
- `risk2bpb`: 71,579 → 71,424 (−0.22%)
- `blend718`: 64,050 → 63,914 (−0.21%)
- `kan_r3_h1_n4`: 103,781 → 103,601 (−0.17%)
- `gasprod_sarawak16`: 213,586 → 213,234 (−0.16%)
- `chp_partload`: 427,059 → 426,403 (−0.15%)

Bucket `1k_10k` dropped from `0.8947` to **`0.8939`**. Total score dropped to **`0.864652`**.
All containerized tests in `yukon run` passed cleanly.
