# 0016 — Structural Dulmage-Mendelsohn (DM/BTF) Decomposition

**Date:** 2026-09-02
**Score:** `0.863575` → **`0.863231`** (−0.000344, −3.4 basis points); fill tiebreak `0.957088` → **`0.957193`**
**Status:** WIN — adapted from `ssi-ordering-challenge` (experiment 0037), verified on all 300 matrices.
**Matrix Moves:** 3 KKT/optimization network wins (`crudeoil_pooling_dt3` −4.29%, `ndcc12` −1.47%, `meanvar-orl400` −0.00%), 0 worse.
**Peak Latency:** 0.865 s (`crudeoil_lee4_10`), safely under the 2.000 s timeout.

---

## 1. Algorithmic Context & Motivation

In structured quadratic programming, optimal power flow, and process control networks, systems frequently exhibit bordered block-angular (KKT) structure:
$$\begin{bmatrix} H & A^T \\ A & 0 \end{bmatrix}$$
Standard degree-based ordering heuristics operate on the combined graph without distinguishing between primal decision variables and dual constraint multipliers. Because the dual-dual block is empty (an independent set), degree heuristics can prematurely eliminate primal variables, creating dense Schur complement fill across all dual rows.

In `dm_btf.rs`:
1. **Layout Detection**: Detects the maximal independent dual suffix $[d_0, n)$ and contiguous slack variables below $d_0$ in $O(nnz)$ time.
2. **Hopcroft-Karp Maximum Matching**: Computes a maximum bipartite matching between constraint duals and primal variables in $O(\sqrt{V} E)$ time.
3. **Dulmage-Mendelsohn Canonical Decomposition**: Decomposes the bipartite constraint graph into under-determined, square, and over-determined subgraphs.
4. **Tarjan SCC on Square Blocks**: Decomposes the square block into strongly connected components, yielding the fine block triangular form (BTF).
5. **Relabelled AMD**: Emits the fine DM blocks topological sinks-first with matched $(primal, dual)$ pairs adjacent, running AMD on the permuted pattern and composing back.

---

## 2. Results & Attributions

- `crudeoil_pooling_dt3` ($n = 30,660, nnz = 152,210$): 47,606,030 → 45,564,568 flops (**−4.29%**)
- `ndcc12` ($n = 974, nnz = 5,982$): 345,696 → 340,611 flops (**−1.47%**)
- Bucket `gt_10k` dropped from `0.8123` to **`0.8115`** (**−8 basis points**).
- Bucket `lt_1k` dropped from `0.9051` to **`0.9050`**.
- Overall composite score dropped to **`0.863231`** (cumulative −85.9 basis points from cloned baseline).
- Passed full containerized `yukon run` verification.
