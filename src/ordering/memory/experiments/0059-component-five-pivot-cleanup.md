# 0059 — Component widths and exact five-pivot cleanup

2026-09-05. Baseline: local commit `8cc942f05c36aabaf249547c71b7771ca4b9ff44`, official promoted submission `da03dc2c-cad7-4d07-9f59-c923c2aef395`, source `649c230a5d60e2122263bd3b653e508938e9fca2`. The preceding four-window/atomic candidate is officially first at hidden score **0.870307**. Its development score is **0.84419540581772**. See [0057](0057-exact-four-pivot-cleanup.md) and [index](../index.md). Old scores on other corpora are historical.

## Result

The selected five-pivot implementation scored **0.8440714862418132** on the same 300 development patterns: **42 better, 2 worse, 256 unchanged** versus the winner. Nine gains are in n<1000;33 gains and both losses are in 1000<=n<10000; the 45 large cases are unchanged. Final combined trusted validation passes all 300 cases, purity/license checks, two-run determinism, bijection and the 2-second watchdog. Trusted fill **0.943946**. Exact counts match the standalone five-pivot probe on every matrix. Official follow-up submission is pending.

The half-split score deltas are -0.00012782888554374594 and -0.00011984292783329131. Removing the five largest relative wins gives -0.000077963680786719. K5 worst observed direct call 0.9698s; preceding winner 0.987s. Timing varies with host load and is separate from exact counts. This does not establish hidden acceptance or a wall-clock bound.

## Primitive and implementation

For a live graph H, eliminated window subset S and next pivot x, let C be x's connected component in H[S union{x}]. Its next column width is `popcount(union adjacency rows of C)-|C|+1` for connected nonsingleton C, and cached degree+1 for singleton. This follows from filled paths through S. After all window vertices are eliminated, the external graph is independent of their order, so the window objective is an exact full-order delta with its prefix/suffix fixed.

The native kernel uses explicit fixed-arity two/three/four/five-row OR/popcount scans, 32 states and 80 transitions. It scans only connected nonsingletons (q <= 26), retains the incumbent on optimal ties and selects globally lexicographic strictly improving optima. One five-offset stride-five cycle replaces the preceding four-offset cycle at identical existing gate and 128M/48M allowances. A caller fallback preserves k4 for n<5. Earlier atomic Game guards remain.

The constructor reserves 8192 scalar units before graph metadata, then `(16+20*q)*w` before row scans. The reserve includes scalar connectivity/DP/decoding with audited slack. These are logical charges; they neither measure elapsed time nor include all other order() work. Every budget-refusal phase preserves complete accepted moves and the untouched suffix.

## Controls

| Experiment | Exact dev score | Better/worse/same | Conclusion |
| --- | ---: | ---: | --- |
| Boundary and score-cache fixes |0.84419540581772|0/0/300|Score-equivalent, include |
| Static component k4, original charge |0.84419540581772|0/0/300|Exact control |
| Static component k4,192*w+4096 |0.8441920499909364|7/0/293|Small gain |
| Static component k4,4096+(16+16*q)*w |0.8441672392354198|13/0/287|Positive alternative |
| Component k5 |0.8440714862418132|42/2/256|Selected |
| Rolling k4 schedule only |0.8441715930336796|18/16/266|Rejected: half/drop5 regress |

The first connected-parent tuple-plan kernel was correct but up to 2.764x slower in a large clique microbenchmark. Replacing dynamic per-word indexing with fixed-arity loops removed that slowdown on large tested cases. Small cases were mixed. Avoid assuming a mathematically smaller computation maps automatically to faster native code.

Reduced k4 evaluation charges preserve a deterministic strict-improvement sequence until the old cap and extend it; they should never regress a matrix. K5 changes the schedule and local-search basin, so pointwise dominance does not follow. The two small public losses are measured, not hidden by an AMD fallback. Their relative regressions are approximately 0.147% and 0.209%.

## Preparation repairs and evidence

Accepted subtree rounds 2/3 synchronize `best_flops` with their chosen permutations. Boundary collection rejects before adding the first distinct excess vertex beyond min(max_sub,MAX_N), preserving accepted S-first/first-seen order and clearing partial scratch via the touched list. These repairs alone reproduce all 300 baseline flop counts.

The combined source passes 44 active tests (18 ignored diagnostics). K5 is checked over all 1024 five-vertex graphs, with/without shared external context, against all 120 explicit elimination permutations: 2048 windows / 245760 permutations. Additional prefix/crossword fixtures, global tie rules, deterministic larger graphs, and metadata/scan/replay refusal tests pass. K4 has a frozen legacy comparison and independent Boolean oracle. Preparation checks 127 block subsets across 9 caps and a rejected hub followed by scratch reuse. Independent source and work-ledger audits found no concrete defect.

All candidate builds and executions used the supplied no-network sandbox. Final trusted scoring uses `yukon run`; no authored harness/dependency/corpus changes. Corpus SHA256 `faa3ecc29c4ef2c54fe08e4382cee0fef39b04b2cc0efa521d8ecc9c66b7c5b6`.

## Research provenance and follow-up

The user's two returned ChatGPT Pro bundles suggested component-based exact windows and adaptive subtree scheduling, based on a stale earlier triple baseline. Their six reference checks passed; the component identity was reimplemented and independently verified. Their showcase windows were already solved by our four-window winner. Adaptive resumption was not adopted: favorable scripted scheduling did not resolve delayed-payoff counterexamples and native ledger costs.

Implementation model GPT 6 Astra, effort ultra, harness Codex. The external Pro model variant was not supplied. A new owner-upload research packet asks for one bounded move escaping consecutive-window minima, an exact global gain certificate, a separation witness and native work ledger. It is a new algorithm experiment, not a submission review gate.
