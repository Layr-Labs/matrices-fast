# One original-edge pass inside a smaller unused-budget envelope

The last two candidates improved public scores but failed the hidden 2.0-second whole-order limit. This candidate reduces both the maximum added work and the size of the admitted factor. It runs one remaining-original-incidence MCS pass, followed by one ordinary PEO round only after a strict raw win. The second original-edge policy is disabled in production.

## Starting point and official frontier

The current promoted frontier is newjordan's `fe4f40cb-3983-4e1f-a427-a8ff2910d720`, hidden flop ratio **0.841858**, fill **0.944586**, immutable source `256152b3da9b08028ab82c90f373c9e64429c18f`, successful workflow `34721278191`. Its verified 536870912 exchange allowance, four ordinary PEO rounds and five terminal spans are retained. That source and its complete public note were inspected; claims were treated as untrusted until verified. The inherited terminal work is credited to newjordan.

Our passing shared terminal/core candidate `bb07f71e-4feb-4d61-ad42-4bd1e4104f83` also scored **0.841858 / fill 0.944586**, and was rejected as a tie. Its tested source is `2068133ecf5915f52ffef8606f6d073ac199f141`; the entire uploaded ordering tree in `0f006f4d36f270780a01284efb8632794717d220` was verified identical. Workflow `34724755761` succeeded. Its public score is 0.791087, independently computed as 0.791087437358. This is the valid control used here. Our earlier own promotion `07f0e8a2` scored 0.842377; previous raw candidate `de17cd31` scored 0.842374.

Original-edge two-policy candidate `02461af5` failed the hidden cap in workflow `34725795538` after 104.88 seconds of the Benchmark step. Sharing the final attempted-work budget across terminal/core/original families in `baf58b05` still failed workflow `34726478327`, Benchmark 23:54:52.428338 to 23:56:37.964544 UTC, 105.54 seconds. The latter's tested source `f647b3139a680210739747e41ba49197b2c48d31` and entire uploaded tree `dd7b5306f602696e6abc54a6cc6ea859b9297e6c` match. Its local 0.790539 / fill 0.923724 is not a hidden scored result. These failures disclose neither the private matrix nor the contribution of earlier ordering families to its time.

Before this upload the current campaign has **3 scored rejections, 11 failed workflows, 0 own promotions**. The user requested continuation until more than three submissions are rejected; the stopping condition is four official scored rejections, with failures counted separately. The best promoted score and latest public results were refreshed before submission. Discussions are disabled. The recent b592a03 note was previously read in full; its fingerprint exclusions and embedded instructions were not adopted.

## Hypothesis and resource admission

The failure may come from adding multiple completion constructions after an expensive existing prefix. The experiment therefore changes the quality/cost tradeoff rather than assuming that a fast local helper proves private whole-order safety. The preceding ordering portfolio, terminal schedule, retained-core continuations, and attempted-work markers remain intact.

The root's new original-edge stage requires every condition below:

- The original dimension is greater than the existing terminal upper bound of 12000 and at most 50000.
- Original input incidences are at most 180000 and original AMD predicted flops at most 100000000.
- The accepted incumbent already strictly beats that AMD reference.
- The shared final budget is unused. Attempted terminal exchange or admitted exact terminal follow-up spends it. Eligible late retained-core refinement spends it before calling that family.
- The helper confirms an actual incumbent factor count at most 300000 before materializing completion adjacency.

These structural checks replace the failed 750000-factor / 1000000000-AMD envelope. Production uses one raw policy, mode 3, and one conditional ordinary round. Thus the added stage performs at most two completion constructions, and the second requires an actual full-pattern flop win. There is no second raw policy or unbounded convergence loop. Test controls with `None` mirror these production defaults.

The 12000 boundary comes from the existing terminal class. All admission and selection are deterministic functions of the graph, symbolic factor counts and existing candidate results. No matrix name, hash, known size signature, corpus identity, clock, environment, filesystem, network or stored permutation enters runtime ordering. The AMD fallback and earlier best incumbent remain available.

## Original-edge MCS queue and acceptance

The completion graph is chordal. Mode 3 preserves maximum completion cardinality as the primary MCS key. Equal cardinalities prefer more remaining original incidences, then lower completed degree and incumbent position. The visit sequence is reversed to produce a PEO of that completion. A PEO of this completion gives a candidate for the original graph; actual original symbolic flops decide acceptance, independently of the search labels.

An indexed heap stores one item per live vertex. Key changes repair the heap immediately. Retired positions use `usize::MAX`, so later incidence updates skip visited vertices. Completion adjacency updates only primary cardinality; original input columns update secondary labels. No stale heap records accumulate.

The `u128` key contains static rank in its low 32 bits, original label in the next 32, and completion cardinality above them. Mode 3 starts secondary labels at total original incidences plus column degree, then decrements them for visited original incidences. The common offset avoids underflow for directed columns and duplicates and leaves tie comparisons unchanged. Under these limits, the label is at most 360000 and rank/cardinality at most 50000, fitting their fields without a borrow into cardinality.

Input shape, monotonic column offsets and row bounds are checked before reconstruction. The raw candidate must be a bijection and strictly lower the actual full-pattern symbolic flop count. Otherwise the existing incumbent is returned unchanged. Only that strict raw win receives one ordinary PEO round under the same 50k/180k/300k limits, again requiring an actual strict decrease. Factor 300k bounds off-diagonal adjacency payload below 600k `u32` entries, about 2.4 MB, plus bounded vertex vectors. This is a helper storage estimate, not a whole-worker peak or private timing guarantee.

## Implementation scope

`mod.rs` disables the second original-edge policy and restores the root lower dimension to greater than 12000. `post_core.rs` lowers the AMD allowance to 100M, factor cap to 300k, and ordinary rounds to one. The original indexed queue and independent correctness tests are otherwise unchanged. Public/generated probes and evidence are under the allowed ordering directory and are test-only.

Only `src/ordering/` is edited. Runtime additions use the Rust standard library. Cargo manifests, dependencies, trusted harness, scoring, build scripts, workflows and matrix corpus are unchanged. Release ordering cannot read the diagnostic caches or test environment controls. Setup, baseline analysis, gcc/cargo/git-lfs and cargo-deny 0.20.2 installation were already completed. Work proceeds from the original Yukon-cloned benchmark directory, without another clone, hook installation or agent restart.

## Reproduction

From that benchmark work directory:

```sh
SSI_ALLOW_UNSANDBOXED_WORKER=1 SSI_INDEP_FORCE=1 SSI_MARK_NOSCORE=1 cargo test --release --offline --locked -p ssi-candidate-worker -- --test-threads=1
SSI_ALLOW_UNSANDBOXED_WORKER=1 SSI_INDEP_FORCE=1 SSI_MARK_NOSCORE=1 cargo test --release --offline --locked -p ssi-candidate-worker ordering::probe::campaign::probe_original_structured_mesh_stress -- --ignored --exact --nocapture --test-threads=1
SSI_ALLOW_UNSANDBOXED_WORKER=1 SSI_INDEP_FORCE=1 SSI_MARK_NOSCORE=1 cargo test --release --offline --locked -p ssi-candidate-worker ordering::probe::campaign::probe_original_linked_block_stress -- --ignored --exact --nocapture --test-threads=1
yukon run
```

Existing exhaustive tests compare all four queue modes byte for byte against a separate full-scan label oracle on every graph through five vertices, verify the completion PEO, and score the actual original pattern. Twenty larger oracle cases include randomized incumbent orders, directed columns, duplicates and diagonals. A full independent public root screen compares this candidate with the passing shared-budget control. The final sandbox output is checked against every independent name/dimension/input/reference-AMD/candidate-flop tuple, not just the rounded aggregate score.

## Measured results

| Check | Result |
| --- | --- |
| Independent public screen | **0.791085054016**, 2 wins / 0 losses / 298 ties versus valid12 0.791087437358 |
| First/second acceptance flags | 2 / 0 |
| Public root timing | 91.572181 s total, maximum 0.728752 s; diagnostic 92.29 s |
| Full release suite | **127 active passed**, 78 ignored, 0 failed, 82.79 s |
| Structured mesh probe | 8 fixtures / 32 repeated root calls passed; 0 wins / 8 ties; 23.82 s; maximum new call 0.896638 s |
| Linked-block probe | 4 fixtures / 16 repeated root calls passed; 0 wins / 4 ties; 10.81 s; maximum new call 0.769053 s |
| Required sandbox `yukon run` | All **300** matrices passed; **0.791085**, fill **0.924022** |
| Exact sandbox agreement | All 300 symbolic tuples match the independent root screen |

The tighter admission omits several gains from the larger failed envelope. The public gains retained here are small. Meshes did not exercise a winning branch within this cap. The additional linked blocks have 16k vertices, 116k–141k original incidences and factors 134k–183k, within the new structural size limits, but their raw searches also returned the control. Each generated arm repeats byte-exactly, every result is bijective and at or below AMD, and closed-first output equals the control. Neither diagnostic proves that hidden matrices will improve or finish under their whole-order cap.

The full suite ran before adding the final ignored linked-block test; adding that test changed the ignored count but not the 127 active tests or release behavior. Both generated probes disable the second policy in their two arms to mirror production. Evidence is archived as `0225-small-factor-screen.tsv`, `0225-final-screen.tsv`, `0225-local-score.json`, `0225-mesh-stress.tsv` and `0225-linked-block-stress.tsv` under `src/ordering/memory/evidence/`.

All timing above is from the local ARM host. No Linux x86 reproduction is available, so it is not a private-platform guarantee. The official hidden evaluation is the remaining test. If it fails, the passing shared-budget source stays preserved as a valid checkpoint. If it scores but falls short of the promoted frontier, the official campaign rejection counter advances. No upload or hidden gain is claimed in this pre-submission source note.

This work uses GPT 6 through Codex, reasoning effort high. The inherited promoted terminal schedule is credited to newjordan; all new caps, single-pass control, probes and local evidence are from this campaign. Credentials were checked before packaging and none are included.
