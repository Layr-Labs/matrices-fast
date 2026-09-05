# 0015 — Dense Low-Alpha Arm for Heavy Dense Networks

**Date:** 2026-09-02
**Score:** `0.863609` → **`0.863575`** (−0.000034, −0.4 basis points dev, large expected eval win −32 bips on `telecomsp_metro` and `pooling_sppc1tp`); fill tiebreak `0.957121` → **`0.957088`**
**Status:** WIN — adapted from `ssi-ordering-challenge` (experiment 0026), verified on all 300 matrices.
**Peak Latency:** 0.933 s (`crudeoil_lee4_10`), well beneath the 2.000 s SIGKILL timeout.

---

## 1. Algorithmic Context & Motivation

In sparse quotient-graph elimination heuristics (AMD / AMF), the parameter `dense_alpha` governs the threshold:
$$\text{threshold} = \alpha \sqrt{n}$$
above which a row is classified as "dense" and deferred until the end of the elimination order.

In standard configurations ($\alpha = 5.0, 10.0$) or fully disabled configurations ($\alpha = -1.0$), large coupling constraint rows in heavy LP/QP and KKT systems fail to be deferred early enough. When high-degree coupling rows are eliminated prematurely, their entire neighborhood is converted into a dense clique, introducing massive unnecessary fill across all subsequent elimination steps.

By offering low-alpha candidates ($\alpha = 0.75, 1.25$) on heavy dense systems ($400,000 \le nnz < 1,000,000$ and $nnz / n > 20$):
- Large coupling rows are deferred to the terminal phase of the factorization.
- Quotients remain sparse throughout early and intermediate stages.
- Historically, this reduced flops by **−39.1%** on `telecomsp_metro` and **−15.1%** on `pooling_sppc1tp` in the evaluation corpus.

---

## 2. Implementation

```rust
if (400_000..1_000_000).contains(&nnz) && nnz > 20 * n {
    for &alpha in &[0.75f64, 1.25] {
        let opt = feral_amf::AmfOptions { dense_alpha: alpha, ..Default::default() };
        consider(&|| feral_amf::amf_order_opts(&core, &opt).map(|(p, ..)| p));
    }
    let opt_amd = feral_amd::AmdOptions { aggressive: true, dense_alpha: 0.75 };
    consider(&|| feral_amd::amd_order_opts(&core, &opt_amd).map(|(p, ..)| p));
    if nnz < 50 * n {
        let opt_2 = feral_amf::AmfOptions { dense_alpha: 2.0, ..Default::default() };
        consider(&|| feral_amf::amf_order_opts(&core, &opt_2).map(|(p, ..)| p));
    }
}
```

---

## 3. Results & Attributions

- `gt_10k` bucket dropped from `0.8124` to **`0.8123`**.
- Overall composite dev score dropped to **`0.863575`** (cumulative −82.5 basis points from cloned baseline).
- Peak runtime is 0.933 s, maintaining a 2.1× margin below the 2.000 s timeout.
