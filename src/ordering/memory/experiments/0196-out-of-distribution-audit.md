# 0196/0197 — out-of-distribution audit of the ordering pipeline

## Question

Every iteration of this run has priced changes on the 300-row dev corpus and read the
verdict from remote receipts. The dev corpus has a structural hole above n = 10 000
(median nnz/n 5.7, max 39.6, and every row with nnz/n > 40 is below n = 1 000) while the
pipeline's time gates are keyed on `n`. So dev's margin map may describe only the families
the code was fitted to. Build the missing instrument.

## Instrument

`0196-tools/gen_ood_corpus.py` -> 27 rows, dev JSONL schema, families: 2D/3D grids, uniform
random sparse at fixed average degree (d = 6..41), block-angular KKT systems (dense blocks +
coupling rows), random geometric, scale-free, banded. Run:

    python3 src/ordering/memory/evidence/0196-tools/gen_ood_corpus.py /tmp/ood/patterns.jsonl
    SSI_CORPUS_FILE=/tmp/ood/patterns.jsonl cargo test --release -p ssi-candidate-worker \
        -- --ignored --nocapture --test-threads=1 probe_timing_and_score

## Findings

1. **Dev's worst row is 1.13 s; the same `order()` takes 10.26 s on a block-angular KKT row**
   (n = 40 400, nnz = 797 550, ratio 0.991), 6.34 s on scale-free n = 40 000, 4.50 s on
   random n = 300 000 (ratio exactly 1.0000 — 4.5 s for no gain), 2.69 s on random n = 20 000
   with nnz/n = 41.
2. **The cost is in the first stages**: `1.portfolio` 4.42 s of the 9.88 s KKT row and 5.50 s
   of the 6.30 s scale-free row; `1b.indep` 1.34/0.17 s; `9.reduce` 2.54 s.
3. **Dev's heavy rows are fast because they are fitted, not easy**: `pooling_sppc3pq`
   (n = 23 173, nnz = 893 724 — 12 % more nonzeros than the 10.26 s row) runs in 0.82 s.
4. **Monotonicity audit (negative result)**: a test-only `force_audit` seam compared the value
   `indep_first::run` returns with the re-scored exact flops at the stage-1b force-adoption.
   Over 300 dev + 27 OOD rows: 49 sites, 15 forced adoptions, **0 unsound, 0 mismatches** —
   the force path always installs a strictly better permutation.
5. **The ledger's "remaining identity windows" queue does not survive reading**: the subtree
   chain gate is already a measured cost/benefit gate (`SUBTREE_CHAIN_MAX_N`, `nnz <= 1.5M`;
   its comment records the chain moving the ratio by < 0.01 while costing 0.26-0.48 s on the
   rows nearest the cap) and the hub guard in `relabel_restarts_tuned` is a *time-saving*
   guard. Exactly one of the four is unmeasured added work — `sparse_large_tie` — and it is
   inert: 16 dev rows match the window, 2 are opened by it alone, disabling it changes 0/300
   dev flops (SCORE 0.792442 either way), and it opens 0 OOD rows alone. It is removed in 0196.

## Consequence for the candidate

0195's receipt (rejected at 0.842857 = the frontier, 0.00 %) prices the *substitution* half of
the removal class at zero, so removals cannot promote. The draw's value is real on dev and
lives below 12 000; every kill added work on a row above 10 000. 0197 therefore re-enables the
draw with its window at 10 000 — strictly below the band the chain owns — which keeps 2.16 of
the 2.30 available dev bips while leaving every row above 10 000 byte-identical to the
frontier (gt_10k bucket unchanged at 0.685651), and whose out-of-distribution cost is
+0.01..+0.09 s on the six rows it touches with zero flop changes on all 27 stress rows.
