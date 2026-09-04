# Exact five-pivot replacement under the existing cleanup allowance

Date: 2026-09-04. Model: GPT-6 Astra. Harness: Codex.

## Result

The isolated replacement passes the full trusted local run on all 300 public matrices. Exact flop counts reconstruct score **0.8441459761788914**, versus **0.84419540581772** for pristine promoted source `649c230a5d60e2122263bd3b653e508938e9fca2`: a **0.5855 basis-point** relative public improvement. Rounded fill improves from **0.944012** to **0.943974**. This is a local result; no hidden score or promotion is claimed. The manifest requires at least one basis point for promotion, and a public delta does not predict the hidden delta.

The existing four-pivot implementation, its mathematical argument, and atomic work guards are inherited from the work documented in [0057](0057-exact-four-pivot-cleanup.md). No earlier portfolio stage, dependency, gate, seed, or allowance was changed. The unrelated current campaign's incumbent-score fixes are not included in this isolated experiment.

## Mechanism and accounting

Replace the final four-pivot cycle with exact five-pivot subset DP. A subset of eliminated window vertices determines the residual graph; the current pivot's component through eliminated vertices determines its live neighbor union. External live vertices never participate in that component closure. The kernel stores 31 nonempty subset unions/counts, solves 32 states and 80 transitions, and reconstructs the exact optimum. A `u16` path code with three-bit digits provides deterministic lexicographic strict-gain ties; an optimum tied with the input retains its original order.

All five offsets share the same existing 128M ordinary or 48M extended allowance. The kernel reserves `640*ceil(n/64)+16384` before evaluation, covering the larger subset/transition work. Existing validation, graph setup, resets and pivot replay retain their precharges. Exhaustion returns completed strict gains with the remainder untouched. The n=4 case retains the original four-pivot helper. The original four tests remain controls. No second cleanup phase or time-dependent stopping was added.

The reservation is a conservative deterministic primitive envelope, not a wall-time guarantee. Larger windows cost more and can leave fewer offsets explored. No per-matrix causal attribution to lost coverage was measured in this experiment.

## Paired public results

| Bucket | Count | Baseline exact geomean | Five exact geomean | Better / worse / same |
|---|---:|---:|---:|---:|
| n < 1,000 | 147 | 0.8904926105665446 | 0.8903606478713713 | 9 / 0 / 138 |
| 1,000 <= n < 10,000 | 108 | 0.8675534877246537 | 0.8675206849570650 | 21 / 17 / 70 |
| n >= 10,000 | 45 | 0.7919539408259014 | 0.7919539408259014 | 0 / 0 / 45 |

Overall: **30 better, 17 worse, 253 identical**. Both alternating name-sorted public halves improve. Replacing the five largest relative wins with their baseline results still gives 0.8441848620230799, a small improvement over baseline. These are diagnostic robustness checks, not production selectors.

## Validation and reproducibility

Required sandbox builds passed. All **40 active tests** passed; 17 manual probes were ignored. A separate Boolean-clique elimination oracle checked **1,084 windows, 86,720 transition widths, and 130,080 complete permutations**. It covers all 1,024 simple five-vertex graphs plus fixed-seed filled-prefix states and word boundaries. Additional explicit tests cover live external bridges, prefix-created fill crossing bit 63/64, optimal and strict-gain ties, malformed input, exact budget cutoffs, partial-gain retention, and the n=4 fallback. The canonical scorer confirms deterministic strict whole-order improvements in bounded fixtures.

The unchanged trusted full harness ran both determinism invocations for every public matrix under the frozen two-second cap and passed. No separate direct timing probe was run, so this note reports no worst-call estimate or hidden timing guarantee. Source remained unchanged between tests, production build and full qualification.

The production worker SHA256 is `082140aacffbf77242e59f492c2f08e6cb664d8b2527da9a69cf4229530b121e`. Candidate source hashes at qualification:

- `rgreedy.rs`: `3b556fdf6e68677f2e581114989cda85f521d745ce486faa47a51a6aca9bc99a`
- `mod.rs`: `b934a7a606ed0bdbb4013954165625b8623a120b0dbd674a89e53f48b381ce02`

Campaign artifacts are saved under `work/yukon/research/`: `matrices-five-tests.log`, `matrices-five-candidate-build.log`, `matrices-five-full.log`, `matrices-five-score.json`, `matrices-five-comparison.json`, and `matrices-five-compare.py`. Baseline exact rows are in `matrices-649c230-original-full.log`. Reproduction uses the unchanged prepare/build scripts, the candidate sandbox, and the default full-corpus harness; no corpus override is needed.

## Next bounded hypothesis

A window with no internal edges has order-independent pivot widths. An honestly charged 25-adjacency-bit precheck could avoid subset/DP work in that structural case and spend the unchanged allowance on more windows. This has not been implemented or measured in the qualified variant above. Do not infer that all observed five-window regressions arise from this particular wasted work.
