# Experiment 0013: 6-Way Cycled AMD Options & Small-Graph MinFill Restarts

- **Date**: 2026-09-02
- **Author**: Antigravity (Gemini 3.8 Flash)
- **Status**: Implemented & verified.
- **Score before**: 0.864899 (fill tiebreak: 0.957753)
- **Score after**: **0.864527** (fill tiebreak: **0.957503**)
- **Improvement**: **−0.000372** (−3.72 basis points / −4.30 bips relative)

---

## 1. Hypothesis & Rationale

Recent developments in the frontier portfolio (experiments 0011 and 0012) introduced hub-gated restart schedules and terminal adjacent-pair descent. However, two critical opportunities remained unexploited in the main relabel loop:

1. **6-Way Cycled AMD Parameter Diversity**:
   Prior to this change, the relabelled AMD loop only toggled between `aggressive = true` and `aggressive = false` with fixed `dense_alpha = 10.0`.
   As demonstrated in our earlier analysis, disabling dense row detection (`dense_alpha = -1.0`) and exploring moderate thresholds (`dense_alpha = 5.0, 2.0`) allows quotient-graph elimination to find radically different pivot sequences on KKT and saddle-point blocks.
   We schedule 6 configurations across restarts $r$:
   - `(aggressive: true, dense_alpha: 10.0)`
   - `(aggressive: false, dense_alpha: 10.0)`
   - `(aggressive: true, dense_alpha: -1.0)`
   - `(aggressive: false, dense_alpha: -1.0)`
   - `(aggressive: true, dense_alpha: 5.0)`
   - `(aggressive: false, dense_alpha: 2.0)`

2. **Expanded MinFill Restarts on Small Systems**:
   Experiment 0010 introduced 6 restarts of `minfill_order` for graphs with $n < 2,000$ and $\text{nnz} < 10,000$.
   For ultra-small systems ($n < 1,000$ and $\text{nnz} < 5,000$), `minfill_order` execution requires under $500\,\mu\text{s}$.
   Doubling the restart count to 12 on these small graphs produces immediate, strictly monotonic gains on tournament and communication graphs without adding any measurable latency.

---

## 2. Implementation

In [`src/ordering/mod.rs`](../mod.rs):
```rust
// 1. Scale minfill restarts on small matrices:
if n < 2_000 && nnz < 10_000 {
    let minfill_restarts = if n < 1_000 && nnz < 5_000 { 12 } else { 6 };
    for seed in 1..=minfill_restarts { ... }
}

// 2. Cycle AMD configurations across restarts:
let amd_configs = [
    feral_amd::AmdOptions { aggressive: true, dense_alpha: 10.0 },
    feral_amd::AmdOptions { aggressive: false, dense_alpha: 10.0 },
    feral_amd::AmdOptions { aggressive: true, dense_alpha: -1.0 },
    feral_amd::AmdOptions { aggressive: false, dense_alpha: -1.0 },
    feral_amd::AmdOptions { aggressive: true, dense_alpha: 5.0 },
    feral_amd::AmdOptions { aggressive: false, dense_alpha: 2.0 },
];
let amd_opt = &amd_configs[r % amd_configs.len()];
consider(&|| {
    let pb = feral_amd::amd_order_opts(&bcore, amd_opt).map(|(p, ..)| p)?;
    Ok(pb.iter().map(|&x| q[x as usize] as i32).collect())
});
```

---

## 3. Results Across the 300-Matrix Dev Corpus

### Scores Across Size Buckets
| Bucket | Weight | Prior Frontier (`0012`) | This Revision (`0013`) | Net Delta |
| :--- | :--- | :--- | :--- | :--- |
| `lt_1k` | 0.30 | 0.9051 | **0.9048** | −0.033% |
| `1k_10k` | 0.30 | 0.8947 | **0.8938** | −0.101% |
| `gt_10k` | 0.40 | 0.8124 | **0.8124** | 0.000% |
| **Overall Score** | **1.00** | **0.864899** | **0.864527** | **−3.72 bips** |
| **Fill Tiebreak** | — | **0.957753** | **0.957503** | **−0.026%** |

### Selected Matrix Improvements
- `chimera_lga-01` ($n=1,120$): flop ratio reduced from `0.8181` to **`0.8136`**.
- `space25` ($n=1,178$): flop ratio reduced from `0.9683` to **`0.9655`**.
- `chimera_mgw-c8-439-onc8-002` ($n=440$): flop ratio reduced from `0.9462` to **`0.9256`** (−2.18%).
- `sporttournament18` ($n=156$): flop ratio reduced from `0.7999` to **`0.7823`** (−2.20%).
- `rsyn0840m02m` ($n=3,486$): flop ratio reduced from `0.9956` to **`0.9932`**.
- `gasprod_sarawak16` ($n=4,596$): flop ratio reduced from `0.999` to **`0.997`**.
- `syn40m04hfsg` ($n=6,328$): flop ratio reduced from `0.981` to **`0.978`**.
- `hydroenergy2` ($n=2,092$): flop count dropped from `60,552` to **`60,380`**.
- `syn30m03m` ($n=2,508$): flop ratio reduced from `0.9962` to **`0.9952`**.

### Latency Profile
- Worst-case runtime across all 300 matrices: **0.877 s** (on `crudeoil_lee4_10`).
- Over 2.28× safety margin beneath the 2.000 s SIGKILL cap.
