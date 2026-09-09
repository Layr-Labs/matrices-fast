# 0167 — subtree-chain to 50k, replacing late PEO-large

- **Date:** 2026-09-09
- **Score:** 0.793834 → **0.793834** (0.00)
- **Status:** miss / reverted. Not submitted.

## Hypothesis

0163 raised the ranked subtree chain to 150k as a first-round replacement of late PEO-large and moved three n>45k rows, but the score only fell 0.793834 → 0.793827 and was reverted. A 5k step should keep that replacement shape on the slice that already has a late pass, without the 150k leap.

## What changed

`src/ordering/mod.rs` only, then restored after the local run:

- `SUBTREE_CHAIN_MAX_N` 45_000 → 50_000
- follow-up rounds, the improved==0 retry, and the MINL subtree ticket stayed at 45_000
- for `45_000 < n <= 50_000` and `nnz < 1_200_000`, one sequential first-round `subtree_cfg_for` step (streams stay 1; no extra portfolio task) replaced late PEO-large
- historical chain `n <= 45_000 && nnz <= 1_500_000` unchanged
- no pair-EXT, mid_force, transplant, or extrarelbl change

Code was reverted after the local run. `SUBTREE_CHAIN_MAX_N` is 45_000 again. HEAD remains `74b6ccd`.

## Result

Yukon local **0.793834** vs tip rebaseline **0.793834** (`/tmp/yukon-run-0167.log`). Fill 0.9258. Buckets unchanged: lt_1k 0.8875, 1k_10k 0.8398, gt_10k 0.6891.

**0 better / 0 worse** / 300 same versus `/tmp/yukon-run-tip-74b6ccd.log`.

n>45k versus this tip: **0 better / 0 worse**.

- transswitch2383wpr (n=59853): unchanged
- transswitch2736spr (n=69651): unchanged
- cont6-qq (n=120395): unchanged
- unitcommit_200_100_1_mod_8 (n=146830): unchanged
- gabriel10 / faclay75 / acopf (n>150k): unchanged
- public corpus has no row in `(45_000, 50_000]`; nearest is arki0013 (n=44909), unchanged

Short of the keep bar (more than one n>45k row strictly better, none worse) and the submit bar (score < 0.793834 by at least 0.0001, plus that row condition). Not submitted. `src/ordering/mod.rs` restored to the tip.

## Why it won / lost

The replacement never touched a public row. The only dev matrices with n>45k sit at n≥59853, and arki0013 is already inside the 45k chain. Raising the first-round ceiling to 50k and dropping late PEO-large on an empty public slice cannot move the local table. Do not treat this as evidence that a larger cap is safe; 0163's 150k leap is still the measured bound and stayed under the 0.0001 submit bar.

## Follow-ups

- Do not repeat 0163's 150k leap.
- Do not restack a chain round on 45k < n without dropping the late pass that already covers that slice.
- A later cap step needs a public or hidden-facing band that actually contains n>45k rows.

## Links

- [0163](0163-subtree-chain-past-45k.md)
