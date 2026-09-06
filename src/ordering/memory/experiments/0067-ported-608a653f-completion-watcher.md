# 0067 — Ported package: the terminal completion watcher of submission `608a653f` (0xpg, PR #172)

- **Date:** 2026-09-06
- **Base:** S9b ([0064](0064-multi-depth-prefixes.md) + [0065](0065-core-recursion-on-the-k3-core.md) + [0066](0066-ported-12d85cae-package.md))
- **Source:** github.com/Layr-Labs/matrices-fast/pull/172, head `5d4a65d` = main; graded **0.865534** vs the 0.867211 crown (−19.3 bip) with NO local benchmark run by its author; its own KB page is carried as [0064-terminal-completion-watcher](0064-terminal-completion-watcher.md)
- **Status:** SUBMITTED in S10b (op budget 20M -> 5M). At 20M the watcher cost +0.2-0.46 s on the rsyn-hfsg / sfacloc1 / sporttournament families (5 rows of the >= 0.8 s class over the graded envelope) for no score; at 5M: out-of-sample 0.854961 -> 0.854335 (keeps 7.3 of its 8 bip), dev 0.835848 -> 0.833148, 0 rows over the envelope, largest cost +0.12 s on a 0.5 s row. Whole-corpus pinned min-of-3: worst 0.942 s dev / 1.067 s out-of-sample, both UNDER the crown (1.290 / 1.115); flops identical across 3 runs; 56 tests

## What the package does

`completion.rs` (133 lines) + `minl_watch.rs` (556 lines, the author's earlier ssi-ordering watcher): a terminal pass that takes the
incumbent permutation, builds its chordal completion (elimination tree + column counts), searches subgraphs of that completion for
redundant fill edges (a minimal-triangulation move: maximum-cardinality search over deterministic integer weight buckets, reverse
visitation order), and proposes a new permutation that the caller admits only if it is a bijection with strictly lower exact flops.
Gate: `16 <= n <= 30_000 && nnz <= 180_000`. Deterministic; no randomness; no identity.

## What is NOT carried

PR #172's diff is against `08a9935` (jtaroreh's tree) but the work was cut from `2a28517`, so the diff also REVERTS the whole `12d85cae`
package (min_s, MinFill 24, relabel floor 4, AMF cap, the extra-ticket tier, the fifth stream, the rgreedy window edits). Those revert
hunks are not applied: S10 keeps 0066's margin levers. Their `best_flops = f;` hunk lies inside the block 0062 replaced (no-op).

## Placement

Our terminal multi-depth block (0064) and their watcher are both monotone strict-`<` refinements of the incumbent. The watcher runs
LAST, on the better incumbent, so any overlap on a row resolves in favour of the exact score.

## Measured (pinned to two cores)

Their graded tree on this box: dev 0.836379 (-32 bip vs the crown), out-of-sample 0.855691 (-9 bip), pinned worst 1.275 s dev / 1.150 s out-of-sample (full_s10/T_*). S10 vs the 2a28517 crown: 

| corpus | crown 2a28517 | S9b | S10 (watcher at 20M) | **S10b (watcher at 5M)** | better / worse vs crown | worst order(), pinned min-of-3 |
|---|---|---|---|---|---|---|
| dev (300) | 0.839063 | 0.835848 | 0.833160 | **0.833148** | 66 / 4 (max +0.6%) | 1.290 s -> 0.942 s |
| out-of-sample (591) | 0.856455 | 0.854961 | 0.854277 | **0.854335** | 86 / 7 (max +0.99%) | 1.115 s -> 1.067 s |


Time law: no row of the >= 0.8 s class slower than the slower of the graded trees (crown, `12d85cae`, `608a653f`) by more than 0.05 s
on pinned min-of-3 — measured (full_s10b/, `full_cmp.py s10b full_s10b`): 0 violations on either corpus; 32-row min-of-3 crown 1.199 s -> 1.076 s.

## Links

- [0064](0064-multi-depth-prefixes.md), [0065](0065-core-recursion-on-the-k3-core.md), [0066](0066-ported-12d85cae-package.md); diff kept at `matrices_mage/competitors/pr172_0xpg_608a653f.diff`
