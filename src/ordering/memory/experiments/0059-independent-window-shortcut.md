# Independent-window shortcut inside the five-pivot work allowance

Date: 2026-09-04. Model: GPT-6 Astra. Harness: Codex.

## Result

The shortcut passes all **300 trusted public cases** and **43 active tests**. Exact score is **0.8441439854819426**, fill **0.943972**. Against the separately qualified plain-five variant from [0058](0058-exact-five-pivot-cleanup.md), five matrices improve, zero worsen, and 295 are identical. Against pristine source `649c230a5d60e2122263bd3b653e508938e9fca2`, the result is 30 better, 16 worse, and 254 identical, a **0.6091 basis-point** relative public gain from 0.84419540581772. This remains below the manifest's one-basis-point threshold on the public comparison; no hidden result or leaderboard improvement is claimed.

The earlier plain-five source tree, patch, worker, tests, full log and score were preserved before this edit. This is one separately qualified follow-up, with no change to the original caller gate, 128M/48M allowance, offsets, preceding portfolio stages, or dependencies.

## Exact shortcut and budget split

If all five internal-neighbor masks are zero, the window vertices are independent in the current filled graph. Eliminating any subset of them cannot change another window pivot's neighborhood. Their shared **live external neighbors** cannot mediate paths through eliminated vertices: eliminating a window pivot only joins external neighbors, and never creates an edge to another independent window pivot. All 120 window orders therefore have the same local cost, so retaining the incumbent is exact.

The loop first reserves **512 units** and computes the 25 adjacency-bit checks. This covers at most 16 index/load/mask/branch/update units per check, mask initialization and the all-zero scan, with slack. A nonempty internal graph reserves the remainder `five_window_work(words)-512` and builds the subset tables from the masks already computed. Its total charge is exactly the former `640*words+16384`, and the 25 checks are not repeated. An independent window skips both table construction and DP, paying only the probe charge. Replay and offset ordering remain unchanged.

Consequently the new variant makes the same choices as the qualified plain-five variant until the latter exhausts its budget, then can make additional strict improvements. Since this is the final cleanup stage, its returned ordering cannot have higher flop cost than the plain-five result. The public comparison explicitly asserts this invariant; no regression occurred.

## Validation

All 40 previously active tests remain, including the original four-pivot controls and the 120-permutation five-pivot Boolean-clique oracle. Three new tests cover:

- Five independent vertices sharing both external live neighbors: every valid transition has width three and every complete order costs 45.
- Both sides of the 512-unit probe gate and the nonempty remainder gate. A fixture with five isolated pivots followed by a star proves that actual saved work reaches a later strict gain under an allowance where the frozen control cannot reach it. Nonempty first-window boundary behavior matches the control.
- 108 deterministic fixture/budget comparisons against a literal test-only copy of the qualified plain-five descent, checking bijection and canonical nonworsening.

Required sandbox tests and production build passed. The unchanged full harness ran each ordering twice, passed all 300 matrices, and enforced the frozen two-second cap. No separate direct timing probe was run; there is no claimed worst-call estimate or hidden runtime guarantee.

## Paired distribution

| Bucket | Pristine exact geomean | Shortcut exact geomean | Better / worse / same versus pristine |
|---|---:|---:|---:|
| n < 1,000 | 0.8904926105665446 | 0.8903606478713713 | 9 / 0 / 138 |
| 1,000 <= n < 10,000 | 0.8675534877246537 | 0.8675140493005690 | 21 / 16 / 71 |
| n >= 10,000 | 0.7919539408259014 | 0.7919539408259014 | 0 / 0 / 45 |

All five incremental gains over plain-five occur in the medium bucket. Both alternating name-sorted public halves improve versus pristine. Replacing the top five relative wins with baseline values still gives 0.8441828712714862, slightly better than pristine. These checks do not alter production behavior.

## Provenance and limits

Qualified worker SHA256: `b1e4dd37631ac7bb2d93f3d61b0038d352bcc6c5bc74e98ded6fbb0d80696c79`. Qualified `rgreedy.rs` SHA256: `5bd7ad94d07978e4f035891c2195609586b6c1e0d13c96de43f1074dee5914ad`. `mod.rs` remains `b934a7a606ed0bdbb4013954165625b8623a120b0dbd674a89e53f48b381ce02`. Only research documentation was added after qualification.

Campaign evidence under `work/yukon/research/`: `matrices-five-noedge-tests.log`, `matrices-five-noedge-candidate-build.log`, `matrices-five-noedge-full.log`, `matrices-five-noedge-score.json`, `matrices-five-noedge-comparison.json`, and `matrices-five-noedge-compare.py`. The frozen control is `matrices-five-frozen-ordering/` plus `matrices-five-worker` and the plain-five logs. Reproduction uses the unchanged setup and candidate-sandbox scripts followed by the default full-corpus trusted run.

A separate campaign candidate, submission `3f22af18`, failed the hidden two-second cap. Its incumbent-score fixes are absent from this pristine-649c230 branch. That failure supplies no hidden instance or score and was not used to alter this variant's gates or budget. This variant still needs its own official evaluation to establish any hidden improvement.
