# At most two terminal PEO extraction rounds

Status: FULL TRUSTED PUBLIC PASS; final source passes 68 active tests. Official hidden evaluation pending.
Baseline source: a07cc9c7c903fd808b95ef2b92f8a049ba7c934e.
Date: 2026-09-06. Reasoning effort: ultra.

Full trusted benchmark passed with exit 0 on all 300 cases. score.json reports score 0.830047 and fill ratio 0.938301; total trusted run time was 149.616 seconds. All source hashes remained unchanged during the trusted run. The final 68 active tests also pass. Official hidden evaluation is pending; no promotion is claimed.

## Final algorithm

Run at most two terminal rounds after the complete inherited pipeline. A round reconstructs the current incumbent's chordal completion once and tries two deterministic bucket MCS extractions. Initialize the zero-weight bucket from incumbent order; use forward adjacency traversal for the first candidate and reverse traversal for the second. Reverse each MCS visit sequence, validate the full bijection and score on the original graph. Only strict exact gain changes the incumbent. The second round runs only after a strict first-round improvement; reconstruction refusal or no gain ends the loop.

The second round reconstructs the induced completion of the first round's best permutation, which may be smaller than the starting completion. It uses exactly the same gates, caps, tie policies, helper, and scoring path. There is no third round and no extra edge-erasure or ordering search. An unproductive or refused second round returns the exact one-round result.

Placement follows residual reductions, core selection/refinement, both terminal completion paths, independent terminal candidate acceptance, and small cutoff refinements. The inherited forest certificate, all baseline seeds and ordering calls, and the 6M/8M plus independent 2M watcher allocations are unchanged. completion.rs is byte-identical. Each round derives the current exact objective from fresh symbolic counts rather than inherited best_flops, which can be stale after late baseline stages.

## Containment and objective argument

For original graph G and incumbent chordal completion H, elimination of G under a PEO of H produces K contained in H. Inductively, live original/fill adjacency is contained in live H. A PEO vertex's later neighbors form a clique in H, so all new fill stays within H.

For a chordal graph under a PEO, the squared-column-count objective is sum_v (1+d(v))^2 = n + 3|E| + 2T, where d(v) is later-neighbor count and T counts triangles. Edges contribute once to sum d(v), and triangles once to sum binomial(d(v),2). This objective is independent of which PEO is chosen. K has no more edges or triangles than H, so its score cannot be higher. The argument applies separately to each round, and production still requires exact strict acceptance.

This proof is our derivation. Tarjan and Yannakakis's 1984 paper introduced MCS for simplified linear-time chordality recognition; its inspected abstract is background evidence for that claim, not evidence for the new tie policy, induced-subcompletion proof, or score benefit. The inherited code already uses MCS extraction, and the sibling helper was independently implemented without fetched source. See the accompanying literature draft, tarjan-yannakakis-1984-mcs.md.

## Bounds

Gate: 16 <= n <= 30,000; original directed nnz <= 180,000; reconstructed exact Lnnz <= 300,000 per round. Validate CSR dimensions/pointers/indices, the permutation, parent dimensions/increasing indices, each symbolic count in 1..=n-j, checked total Lnnz, and every reconstructed column length.

Each retained column is scanned once again at its unique parent, so reconstruction is O(n + original nnz + Lnnz). For M completion edges, adjacency has 2M entries. Each lazy bucket MCS scans those entries, inserts n initial entries plus at most M weight updates, and pops at most that many entries including stale ones. Bucket indices are <=n-1. MCS work and memory are O(n+M).

At most two rounds each add a permuted symbolic analysis and two exact original-graph scores, with no added AMF/AMD/MinFill, subtree/local descent, heap, pair enumeration, or fill erasure. Scratch is scoped to a round; adjacency is released before scoring and only the best output permutation survives. Peak scratch does not accumulate. These bounds are not a hidden two-second runtime proof.

## Final native evidence

Final source: 68 active tests pass, zero failures, 20 ignored probes. The two extractor tests include all labeled simple graphs and all incumbent permutations for n=1..5, totaling 124,469 graph/order pairs. Literal clique insertion independently checks completion reconstruction, duplicates and the two PEOs; vendor symbolic scoring independently checks non-increase. A center-first three-vertex path gives a strict 14 -> 9 witness. Malformed inputs are refused. The extractor and its tests are unchanged between one and two rounds; final-source tests were rerun.

The final 300-case sandboxed direct probe independently rescores production order(). All identifiers, n, original nnz and AMD counts match the frozen baseline.

| Measure | Baseline | One round | At most two rounds |
|---|---:|---:|---:|
| Public direct score | 0.8317720492495939 | 0.8304574664260687 | 0.8300471638388927 |
| Better / worse / same versus baseline | — | 18 / 0 / 282 | 18 / 0 / 282 |
| Worst direct seconds | 0.6283 | 0.6557 | 0.6642 |

Bucket counts are 147 / 108 / 45. Baseline bucket geomeans: 0.8897304931511659 / 0.8636457748272273 / 0.7643979221401898. Final: 0.8897304931511659 / 0.8634669213066553 / 0.7602198487538658. Final two-half score deltas are -0.002051697402077668 and -0.0015155430332454145. After dropping the five largest contributing wins, delta remains -0.0001344666376857928. Summed final direct times are 68.0459 s; single-run timing variance applies. Direct tests/probes are distinct from the separately passed trusted watchdog run; neither establishes the official hidden outcome.

## Excluded controls

Four controls matched all 300 baseline final scores: omit redundant K4 reduction while retaining its ledger debit; refund only that skipped ticket; exact-rank already produced extra-core permutations while preserving the completed baseline; and bounded cleanup of a distinct core runner-up. Broad exact-ranking replay found 37 raw-score opportunities among 346 admitted cores, masked by the complete baseline. A separate six-pivot subset-DP treatment reached 0.8317713038285726 with five gains and no losses, a much smaller independent improvement. None of those controls is part of this candidate. No automated GPT Pro/browser dispatch occurred for these trials.

## Reproduction and source

Full trusted command from the repository root, using prepared pinned dependencies:

```sh
bash scripts/local-candidate-build.sh
cargo run --release --offline --locked -- --note "At most two terminal PEO extraction rounds, second conditional on strict gain"
```

The source edit is limited to mod.rs and new peo_extract.rs under src/ordering. Final SHA256:

- mod.rs: 368804a99e7bab55c16d6c1174c9da06848346db6a6ddd0c76e3c3ea71b95a22.
- peo_extract.rs: 46bdc5926d56fdfb39a5c7b4b2e795de26589e527402cef94f965b9c702818a5.
- Unchanged completion.rs: 9b8077126be911c9c2bb524b36a8a7e95e1060cc762442f3cf435eb5da36051f.

Local receipts: native/peo2-tests-run.json, native/peo2-probe-run.json, native/peo2-comparison.json and corresponding logs/source archives. One-round control: native/peo-comparison.json. Six-pivot control: native/six-comparison.json. Source hashes remained unchanged during final tests/probe. Design: peo2-design.md and peo-design.md. Public submission draft: peo-submission-note.md.

This is a record-only draft ready for public memory/submission finalization. No source edit, native run, commit, or publishing occurred during drafting; the controller performed and verified the native validation reported above.
