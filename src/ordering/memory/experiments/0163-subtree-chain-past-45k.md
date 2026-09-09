# 0163 — subtree-chain past 45k, replacing late PEO-large

Base: `74b6ccdf697140bc3d04c0409eb1d32b099edcfd`.

Rows with `n > 45_000` sat outside the ranked subtree chain. The late PEO-large pass (`nnz < 1_200_000`) already runs on that band.

Replacement, not a stack: `SUBTREE_CHAIN_MAX_N` is 150_000. For `45_000 < n ≤ 150_000` and `nnz < 1_200_000` (chain nnz cap still 1_500_000), one sequential first-round `subtree_cfg_for` step runs and the late PEO-large pass is dropped. Follow-up chain rounds and the MINL subtree ticket stay at 45_000. Gate is n and nnz only. No pair-EXT, mid_force, transplant, or extrarelbl change.

## Result

Local score 0.793834 → **0.793827** (delta −0.000007, under the 0.0001 submit bar). Fill 0.925766. gt_10k 0.6891.

n>45k versus this tip: **3 better / 0 worse**.

- cont6-qq (n=120395): flops 581918900 → 581899076
- transswitch2736spr (n=69651): 7281507 → 7277230 (ratio 0.918 → 0.917)
- transswitch2383wpr (n=59853): 3861509 → 3859526 (ratio 0.979 → 0.978)
- unitcommit_200_100_1_mod_8 (n=146830, inside the band): unchanged
- gabriel10 / faclay75 / acopf (n>150k): unchanged

No other row moved versus the tip remeasure of this commit. Public yukon table redacts times as capped; the run finished with no per-matrix cap kill. Not submitted.
