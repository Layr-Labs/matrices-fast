# 0014 — Custom Quotient-Graph Pivot Metrics (SqDiv and SqPure)

**Date:** 2026-09-02
**Score:** `0.864462` → **`0.863609`** (−0.000853, −8.5 basis points); fill tiebreak `0.957488` → **`0.957121`**
**Status:** WIN — adapted from `ssi-ordering-challenge` (experiments 0023 and 0034), verified on all 300 matrices.
**Matrix Moves:** 3 large optimization network wins (`pooling_sppa9pq` −28.51%, `pooling_sppa9tp` −0.86%, `meanvar-orl400` −0.00%), 0 worse.
**Peak Latency:** 1.050 s (`crudeoil_lee4_10`), safely under the 2.000 s timeout.

---

## 1. Algorithmic Context & Motivation

Standard Minimum Degree (AMD) heuristics select pivots based on minimizing the degree of the eliminating vertex in the current quotient graph. However, the true Cholesky factorization cost charged in this competition is:
$$\text{Cost} = \sum_{j=1}^n c_j^2$$
where $c_j$ is the column count (number of non-zero entries in column $j$ of the Cholesky factor $L$).

In `custom_metrics.rs`, alternative quotient-graph pivot selection scoring formulas are driven through the vendor `feral_ordering_core::quotient_graph` loop (`select_pivot_amf`, `create_element_amf`):
- **`SqDiv`**: Computes $\text{deg}^2 / (\text{nv} + 1)$ for each candidate supervariable. If supervariable $v$ with multiplicity $nv$ is eliminated now, each of its soon-to-be-created columns contributes $\approx \text{deg}^2$ to the Cholesky flop metric. Dividing by $(nv + 1)$ normalizes by prospective multiplicity.
- **`SqPure`**: Computes $\text{deg}^2$ directly.

On constrained optimization models and process networks (such as `pooling_sppa9pq` and `pooling_sppa9tp`), selecting pivots according to prospective squared degree avoids the premature creation of wide dense columns that AMD's un-squared degree heuristic fails to anticipate.

---

## 2. Gating & Implementation

```rust
if nnz <= 300_000 && nnz >= 10 * n {
    for &variant in &[
        custom_metrics::ScoreVariant::SqDiv,
        custom_metrics::ScoreVariant::SqPure,
    ] {
        for &alpha in &[1.0, 10.0] {
            consider(&|| {
                custom_metrics::order_variant(&core, alpha, true, variant)
            });
        }
    }
}
```

- Bound by $nnz \le 300,000$ and density $nnz / n \ge 10$ to target dense coupling structures.
- Evaluates four fast quotient-graph traversals with zero matrix allocations (reusing vendor quotient-graph workspace).
- Integrated into `consider()` best-of portfolio floor: strictly zero risk of regression.

---

## 3. Results & Attributions

- `pooling_sppa9pq` ($n = 5,030, nnz = 120,730$): 21,896,437 → 15,653,402 flops (**−28.51%**)
- `pooling_sppa9tp` ($n = 5,040, nnz = 121,302$): 48,009,421 → 47,597,914 flops (**−0.86%**)
- Bucket `1k_10k` dropped from `0.8933` to **`0.8904`** (−29 basis points).
- Overall composite score dropped to **`0.863609`** (cumulative −82.2 basis points from baseline).
- Passed full containerized `yukon run` verification.
