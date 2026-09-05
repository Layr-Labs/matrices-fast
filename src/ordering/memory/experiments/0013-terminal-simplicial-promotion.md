# 0013 — Terminal Simplicial Promotion (Zero-Deficiency Lookahead)

**Date:** 2026-09-02
**Score:** `0.864652` → **`0.864462`** (−0.000190, −2.2 basis points); fill tiebreak `0.957588` → **`0.957488`**
**Status:** WIN — adapted from Ost, Schulz, and Strash (2020) and `ssi-ordering-challenge`, verified on all 300 matrices.
**Matrix Moves:** 16 matrices strictly improved, 0 worse.
**Peak Latency:** 0.835 s (`crudeoil_lee4_10`), safely under the 2.000 s SIGKILL timeout.

---

## 1. Algorithmic Context & Motivation

A vertex $v$ in a graph $G$ is called **simplicial** if its neighborhood $\text{Adj}_G(v)$ induces a complete subgraph (clique). 
In the sparse Cholesky elimination game, eliminating a simplicial vertex creates **zero fill edges**:
$$\text{deficiency}(v) = 0$$

As proved by Ost, Schulz, and Strash (arXiv:2004.11315), simplicial vertices can be eliminated immediately without increasing the fill-in of any subsequent elimination step. In heuristics like AMD or partitioner-based orders, a vertex that becomes simplicial during the elimination process may be scheduled several positions later than optimal. In the meantime, eliminating non-simplicial neighbors can prematurely densify surrounding subgraphs.

In `rgreedy.rs`, `simplicial_promotion` tracks the exact forward elimination state:
- At each step $k$, it inspects candidate pivots in a lookahead window $k + 2 \dots \min(n, k + 16)$.
- If a future neighbor $w$ in the window has $\text{deficiency}(w) = 0$ and $\text{deg}(w) \le \text{deg}(\pi[k])$, $w$ is promoted to position $k$, shifting intervening vertices back by 1.
- The resulting permutation is scored with the canonical objective $\sum c_j^2$.
- The candidate is accepted if and only if it strictly reduces Cholesky factor flops below `best_flops`.

---

## 2. Gating & Budget Constraints

To ensure strict zero-regression behavior and maintain generous headroom below the 2.000-second timeout:
- **Envelope**: $1,000 \le n \le 6,000$, $nnz \le 100,000$, and density ceiling $nnz \le 24 \cdot n$.
- **Operation Budget**: 64,000,000 bitset operations (~0.015 s per matrix).
- **Execution Placement**: Runs immediately after `adjacent_pair_descent` on `best_perm`, comparing strictly against `best_flops`.

---

## 3. Results & Attributions

16 matrices improved across the development corpus:
- `chimera_selby-c16-01`: 2,444,957 → 2,397,052 (−1.96%)
- `nuclear25a`: 1,339,276 → 1,314,803 (−1.83%)
- `pooling_sppa0pq`: 1,511,075 → 1,496,775 (−0.95%)
- `rsyn0810m04m`: 133,220 → 132,199 (−0.77%)
- `edgecross14-156`: 1,421,400 → 1,412,844 (−0.60%)
- `rsyn0805m03m`: 60,455 → 60,299 (−0.26%)
- `mpbp_47`: 685,194 → 684,033 (−0.17%)
- `crudeoil_pooling_ct3`: 972,223 → 970,812 (−0.15%)
- `chimera_lga-01`: 543,497 → 542,732 (−0.14%)
- `gasprod_sarawak16`: 213,234 → 213,026 (−0.10%)
- `mpbp_46`: 486,579 → 486,111 (−0.10%)
- `sporttournament48`: 609,408 → 609,033 (−0.06%)
- `chimera_mgw-c16-2031-01`: 2,946,428 → 2,937,866 (−0.29%)
- `chp_partload`: 426,403 → 426,233 (−0.04%)
- `rsyn0840m02m`: 47,204 → 47,183 (−0.04%)
- `rsyn0815m02hfsg`: 43,372 → 43,303 (−0.16%)

Bucket `1k_10k` dropped from `0.8939` to **`0.8933`**. Total score dropped to **`0.864462`**.
All containerized tests in `yukon run` passed cleanly.
