# 0206 — Factor-bounded terminal PEO and smaller sparse windows

Implemented with GPT 6 through Codex at high reasoning effort.

Effort: high.

## Official hidden result — promoted

Submitted **`07f0e8a2-d0fc-4d37-ab5f-ff5349768add`**, remote commit
**`52affcba3180dbebd8693d28fd879ae09b161efe`**, in
[workflow 34711543900](https://github.com/Layr-Labs/matrices-fast/actions/runs/34711543900).
The entire remote `src/ordering/` is verified byte-identical to tested local
commit **`433fa88`**. Yukon confirms **PROMOTED, current best**, at hidden
flop score **0.842377**, fill **0.944856**. This beats the prior **0.842526**
by **0.000149**, approximately **1.77 relative basis points**, clearing the
one-basis-point promotion floor. Fill also decreases by **0.000089**.

The complete workflow succeeds. Benchmark runs from **18:36:19 to 18:45:33
UTC on 2026-09-12**, **554 seconds**, with all hidden per-matrix caps and
determinism checks passed; score upload succeeds at 18:45:34 UTC. The shorter
step than the prior successful run is not an isolated runtime comparison and
is not claimed as a kernel speedup. Subsequent local updates affect only
Markdown metadata. The winning Rust remains identical to submitted `52affcb`.

## Baseline and reason for this revision

The promoted target remains our **`c5e6c2ff` / `fb35d11`**, hidden flop score
**0.842526**, fill **0.944945**. Its exact winning Rust remains preserved on
`fenced-terminal-window`. The public baseline is **0.792029985254**, rounded
to **0.792030**, fill **0.924364**, on all 300 matrices. Size-bucket counts
are 147 / 108 / 45 and weights are 0.30 / 0.30 / 0.40. All work continues
in the established benchmark repository; setup, LFS corpus and tooling are
already installed. The host is ARM macOS using the existing release toolchain.

The first post-search completion/sparse-span continuation, recorded in
[0205](0205-post-search-completion-and-sparse-spans.md), passed all 300 public
matrices at 0.791769, all 123 active tests, and its paired generated screen.
Nevertheless, submission **`026c5a9c-00cb-4a1a-b8ed-03885d3f4bf5`** failed
the hidden **2.0 s per-matrix cap**. Workflow
[34710571494](https://github.com/Layr-Labs/matrices-fast/actions/runs/34710571494)
failed at Benchmark, from 18:17:57 to 18:19:44 UTC on 2026-09-12, about
106 seconds. There is no hidden score or improvement to claim from that run.
The failed submitted ordering was verified identical to tested `a427c32`.

This is a changed continuation, not an identical retry. The fresh submission
list still identifies `c5e6c2ff` as the promoted best. Benchmark metadata says
lower is better, Discussions are disabled, and the promotion floor is one
relative basis point. The claimed-score field is recorded only.

## Hypothesis and cost controls

The final ordering after the promoted greedy search and width-eight exact
window can expose a new chordal completion. Re-extracting PEOs and solving
small components of wider contiguous spans can reduce its actual flop score.
That mechanism remains valid, but the first continuation spent too much work
somewhere on the hidden corpus. Its original input-nnz gate did not limit the
filled graph on which the extra replay searches operated.

The revision adds an **exact 150,000-factor-nonzero admission bound** before
any new extraction or window search. It retains `6 <= n <= 12,000` and
`nnz <= 200,000`. The existing terminal scalar ledger first rejects rows
above the **20-billion-flop** threshold without an added scoring pass. For
remaining rows, a fresh exact incumbent score and the scoring arena's current
symbolic factor count decide admission; no stale scalar is used to accept a
candidate. The ledger check is conservative and does not repair or alter any
of the winning prefix's earlier adoption rules.

The new continuation **removes the added fill-edge watcher entirely** and
halves the window allowance from **128M to 64M**:

1. For dense/hub patterns excluded by the original terminal window gate,
   apply its same width 8 / four sweeps / step 3 / 64M refinement.
2. Extract forward/reverse MCS PEO candidates for at most two rounds, stopping
   immediately on a round without a strict exact gain. The extraction limits
   are **12k dimension / 200k input nonzeros / 150k factor nonzeros**.
3. Run span 48 / four sweeps / step 19 / **16M**, then width 9 / four sweeps /
   step 4 / **16M**, then width 8 / four sweeps / step 3 / **32M**.

Every candidate is independently scored on the original pattern and accepted
only on a strict decrease. These replacements cannot affect earlier seeds or
donor selection. The original terminal width-eight pass remains unchanged.

## Exact sparse-span mechanism

The span helper from 0205 uses a u64 position mask, accepting at most 64
contiguous positions. It finds connected components of their current live
induced graph. Components of size two through fourteen use the inherited
exact subset solver; singleton and oversized components keep their original
positions and order. The component ceiling is still fourteen, not the span
width. This avoids an exponential dependence on 48 or 64 positions.

Each solved component stays in its own slots. No component has an internal
edge to another, and external vertices remain live through the span, so
reordering one component cannot affect the other components' pivot costs.
Eliminating the same complete span leaves the same suffix graph. This is why
an independently measured local gain is also a global gain. The existing
shared work counter includes preparation, component scans, DP and elimination
replay; completed gains survive exhaustion with the unvisited suffix retained.
No wall-clock threshold, external state, matrix identity or cached answer is
used in production.

The entire promoted prefix is preserved: first-64 producer fence above
20-billion flops, all original portfolio/window stages, donor transplantation,
full 50k / 4M alternate-PEO scope, terminal greedy stream and exact runtime
kernels. The new production code uses standard-library containers and the
already present scoring/PEO helpers. No dependency, manifest, trusted harness,
corpus or file outside `src/ordering/` is edited.

## Independent public admission screen

All six arms start from the same promoted final ordering on every public
matrix. Its actual score and symbolic factor nnz are recomputed before any
gate. The exact baseline aggregate is asserted as 0.792029985254. Original
AMD scores are independently recomputed; unadmitted patterns count unchanged.
Every completed candidate is a bijection and asserted non-worsening.

"Full" means PEO, watcher 4M and windows 32M + 32M + 64M; "small" means
PEO and windows 16M + 16M + 32M **without** the added watcher. Both use the
same original-window coverage extension. Times include candidate scoring and
are ARM diagnostics; they do not guarantee a hidden x86 deadline.

| Factor-nnz limit / continuation | Score | Wins by bucket | Extra CPU | Maximum |
|---|---:|---:|---:|---:|
| 300k / full | 0.791769290884 | 2 / 24 / 4 | 2.419344 s | 0.034257 s |
| 150k / full | 0.791791296162 | 2 / 23 / 3 | 2.297299 s | 0.033786 s |
| 75k / full | 0.791804400155 | 2 / 22 / 3 | 2.215953 s | 0.033911 s |
| 300k / small | 0.791847176761 | 2 / 19 / 4 | 1.201028 s | 0.017718 s |
| **150k / small** | **0.791864560331** | **2 / 18 / 3** | **1.155377 s** | **0.017906 s** |
| 75k / small | 0.791875469977 | 2 / 17 / 3 | 1.109019 s | 0.017829 s |

The selected 150k/small continuation has **23 wins / 0 losses / 277 unchanged**
and a decrease of **0.000165424923**, approximately **2.09 relative basis
points**. It gives up some measured public gain from the timed-out version
in return for a smaller graph class, no late watcher and half the added
window allowance. The 75k alternative saves very little extra public CPU
while losing another win; it is documented but not mixed into production.
The only full-version winners above 75k factor nnz were at factors 135815,
152488 and 235115. These measurements describe structural admission, not
production name-based exceptions.

The screen uses temporary test-only cached public baseline permutations to
avoid repeating the complete portfolio. They are independently rescored and
their exact aggregate asserted on every screen. That temporary cache is
excluded from the archive and never read by production. This distinguishes a
measurement fixture from a solver lookup table.

## Verification and commands

```sh
yukon run
SSI_ALLOW_UNSANDBOXED_WORKER=1 SSI_INDEP_FORCE=1 SSI_MARK_NOSCORE=1 \
  cargo test --release --offline --locked -p ssi-candidate-worker \
  probe_terminal_followup_stress -- --ignored --nocapture --test-threads=1
SSI_ALLOW_UNSANDBOXED_WORKER=1 SSI_INDEP_FORCE=1 SSI_MARK_NOSCORE=1 \
  cargo test --release --offline --locked -p ssi-candidate-worker \
  -- --test-threads=1
```

The scored candidate is rebuilt and run through Yukon's normal build and
per-worker sandboxes. The diagnostic opt-out is used only for the already
reviewed test-only probes on this local host. Final sandboxed scores, full
active suite and paired stress results are recorded below.
The existing seven active window tests include independent exhaustive and
boundary/tie/budget oracles, a 64-position/high-mask-bit case, skipped large
components and exact suffix graph equality.

## Completed local validation

The revised **sandboxed `yukon run` passes all 300** at **0.791865**,
fill **0.924246**. Its exact public aggregate is **0.791864560331**.
All 300 actual AMD/candidate flop counts match the selected 150k/small
diagnostic arm: **23 wins / 0 losses / 277 unchanged** relative to the
promoted baseline. Bucket flop geomeans are **0.887368 / 0.838227 /
0.685466**, with fill geomeans **0.959764 / 0.946298 / 0.881068**.
Thus the changed admission and smaller PEO construction limits are verified
in the scored worker, not merely predicted from the cached diagnostic.

The **full active release suite passes: 123 passed, zero failed, 55 ignored**,
in **78.71 s** with one test thread. The paired generated test also passes
all **44 complete orders** across eleven deterministic random/grid/hub
fixtures, alternating old/new execution order over two pairs per fixture.
Repeated permutations are identical; all scores equal the promoted control.
Every one of the eight random fixtures exceeds the new factor bound and
returns the exact same permutation as the control. Its factor sizes range
from **479783 to 16028994** entries, which demonstrates why original nnz
alone did not bound their filled graphs. The two grids are admitted but find
no new strict gain; the hub uses the original certificate fast path.

The largest new minimum same-host runtime is **0.993218 s**, versus
**0.993691 s** for the control on that fixture. This difference is timing
noise, not a claimed speedup. The largest measured minimum increase is
**0.010877 s** on the admitted 64-square grid. Above-bound and high-factor
scores remain equal; local timings do not establish a hidden guarantee.
Unlike the first version, large-factor random graphs run no continuation
search. The smaller bounded graph class is the intentional runtime tradeoff.

Evidence: [300 comparisons](../evidence/0206-final-screen.tsv),
[official local score](../evidence/0206-local-score.json),
[paired generated fixtures](../evidence/0206-generated-stress.tsv).
The test-only temporary permutation cache is excluded from the archive.
Generated `results.tsv` was restored. `git diff --check` passes and all
changed files are under `src/ordering/`. The latest full submission list
still confirms `c5e6c2ff` at **0.842526** as the promoted target. Two more
recent failed submissions do not change that target.

## Limits and next decision

The hidden grader enforces determinism, a two-second per-matrix cap and a
four-GiB address-space cap. Public average and minimum times do not prove
hidden safety; 0205 is direct evidence of that limitation. The revision's
factor count gate addresses filled-graph work rather than relying only on
original input density. Its fixed allowances and unchanged exact component
ceiling also bound the introduced search independently of any identity.

The successful public run justified submitting this changed implementation.
The completed workflow and official grade above now establish that the
reduced continuation passes the hidden cap and clears the promotion floor.
The failed larger version remains documented in 0205; this promotion is
specific to the factor-bounded, smaller continuation. The winning Rust and
previous promoted baseline are preserved on separate local branches.
