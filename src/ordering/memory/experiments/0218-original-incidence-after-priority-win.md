# 0218 — One bounded original-incidence MCS after a strict priority win

Implemented with GPT 6 through Codex at high reasoning effort.

Effort: high.

**Preceding base FAILED hidden 2.0 s cap**: 074ed992 in workflow34723170151,
Benchmark104.86s. The positive replay below is on that invalid submitted base,
not a deployable improvement. This stage was never enabled in production or
uploaded. All prototype code and evidence remain preserved for a future
screen on a validated source. Next production drops wide MCS/LexBFS and the
medium producer fence; campaign2 rejected/8 failed/0 own promotions.

**Research only before the preceding official hidden grade.** The algorithm
and selected replay are cfg(test) only at this checkpoint. No production
candidate or hidden gain is claimed here until the selection and full checks
below are completed.

## Objective and controls

The primary score is predicted LDL transpose factorization flops relative to
feral AMD at 1.00; lower is better. It is the weighted mean of three size-bucket
geometric means, with fill as the secondary tie-break. The hidden worker also
requires a deterministic bijection in at most 2.0 seconds and 4GiB per pattern.
This proposal adds one different chordal-completion tie rule, only after the
preceding degree-priority pass has found a strict exact full-flop decrease.

External promoted frontier at the latest check is **fe4f40c**, **newjordan**,
hidden **0.841858**, fill **0.944586**, immutable source
**256152b3da9b08028ab82c90f373c9e64429c18f**, successful official workflow
[34721278191](https://github.com/Layr-Labs/matrices-fast/actions/runs/34721278191).
The copied terminal span/ledger/PEO schedule in our current base is credited
to **newjordan**. Its disabled fanout and unrelated memory are not imported.

Immediate submitted base is **074ed992-338a-441c-b6f1-782566f5f425**,
tested/uploaded **5b867796027e46630aad4baa14d763111bb6e0eb**, remote
**74ff8114d805c500659bdc3c2ab20ac6eb2d57b6**. The entire ordering archive
matches the tested source. Official
[workflow 34723170151](https://github.com/Layr-Labs/matrices-fast/actions/runs/34723170151)
is in progress; this is not yet a validated hidden base. Its public exact
score is **0.790176632480**, required sandbox **0.790177**, fill **0.923617**.
[0217](0217-promoted-window-schedule-integration.md) records all 300 exact
permutations, symbolic counts, 126 active release tests and 64 generated calls.

The current base reduces ordinary PEO continuation to cases where degree-
priority MCS strictly wins, and calls LexBFS5 only when that priority helper
returns a strict winner. Separate full-function screens preserve all 300 exact
public permutations under both gates. A focused finished-completion kernel
comparison cuts ordinary PEO reconstructions 302->16 and LexBFS calls 290->4,
total 1.144744->0.617229 seconds, with identical outputs. This is local ARM
kernel evidence rather than a hidden cap guarantee.

The earlier 100M/64 producer-prefix attempt **2ee07107** fails the hidden
2.0-second cap, workflow 34722194684, Benchmark 102.96 s, no hidden score.
Our campaign has **2/4 scored rejections, 7 failed workflows, 0 own promotions**
before the pending base's outcome. Four scored rejections in this fresh
campaign are the user's stop condition; failed workflows are separate.
Our own promoted 07f0e8a2 at 0.842377 and valid raw de17cd31 at 0.842374 remain
preserved alongside the newer external frontier.

## Tie policy and independent implementation

The original degree-priority MCS changes static ties among vertices with the
largest number of already visited completion neighbors. This proposal keeps
that completion-cardinality criterion primary, but adds a dynamic secondary
key derived from the original pattern's incidences. As original neighbors are
visited, the secondary key decreases, favoring vertices with more remaining
original incidences among vertices of equal completion cardinality. Static
lower completion degree and incumbent position then break remaining ties.
Production selection is one fixed policy3; there is no per-row policy oracle.

The queue is implemented independently with one live item per vertex, inverse
heap positions and 128-bit integer keys. Completion cardinality is in bits 64
and above, original incidence weight starts in bits 32 and above, and the
unique static rank occupies low bits. Standard-library vectors, comparisons
and shifts implement maximum extraction and key updates. Original incidence
decrements sift down; completion-cardinality increments sift up. All updates
for a selected vertex finish before the next maximum is extracted.

For remaining incidences, the starting secondary weight is total original
nnz plus original column degree. The offset prevents underflow even with
directed or duplicate incidences, without changing secondary ordering.
Dimension 50k and input 180k fit the separate key fields, as does each completion
cardinality. Cardinality remains primary throughout. The selected visitation
order is reversed into a PEO of the prepared chordal completion. The candidate
is independently scored on the original pattern; a non-strict result is kept
out of the continuation, and only a strict full-flop improvement replaces the
incumbent. Existing ordinary PEO gets at most two further rounds after that win.

Primary MCS provenance is the 1984 Tarjan/Yannakakis paper linked in
[the MCS literature note](../literature/0207-mcs-structural-ties.md). The dynamic
original-incidence tie rule, integer layout and queue implementation are our
independent heuristic proposal. No fetched implementation is copied. The
independent reference uses explicit cardinality and secondary-label arrays and
a full maximum scan over the tuple of labels and static priority.

## Smaller work scope and acceptance-state admission

The proposed bonus is considered only after the existing priority helper
returns an actual strict winner. Thus no original-incidence queue or completion
is built on no-op priority cases. The root's existing original AMD<=1B guard
still applies. Additional bounds are **n=6..50000**, original input nnz<=180k,
exact incumbent factor nnz<=750k and exact incumbent flops<=20B. The original
broader prototype had input1.3M/factor1M; these smaller bounds retain its four
winning rows on the current public base.

The helper prepares one bounded completion, runs one fixed original policy3,
and scores one resulting permutation. Its ordinary PEO continuation starts
only after a strict full-score decrease, stops on a no-op round and retains
the same smaller dimension/input/factor limits. There is no wider factor path,
unbounded iteration, extra random stream, repeated independent-set driver or
METIS producer. Work is finite and determined by integer structure and accepted
graph state, not elapsed time or an external request.

At 750k factor nnz, completion-neighbor storage has fewer than 1.5M entries;
original updates have at most 180k raw incidences. Each queue operation traverses
at most the heap height. The queue's key, position and heap arrays need 32 bytes
per vertex, plus temporary static priorities and leading weights. These
structural estimates describe the selected helper; the official whole-pipeline
2.0-second result is still required.

## Completed oracles and selected public replay

All graphs and incumbent permutations through n=5 test all four original-
incidence modes against the independent explicit-label maximum-scan oracle.
Each candidate must also be a PEO of the independently simulated completion
and must not worsen the independently scored original graph. These checks
are included with the existing twelve indexed MCS and six LexBFS policies.

A further active oracle check uses 20 generated graphs at dimensions 7,17,31,64,
127 and four densities. Heuristic incidence columns deliberately include
asymmetric deletions, duplicate entries and diagonal duplicates. All four modes
still match the explicit-label reference, retain completion PEO status and
respect the base graph's independent flop floor. This checks integer field
separation, decreasing-key repairs and larger inverse heaps. It passes in
**0.03 seconds**. The completed graph remains the actual base graph's
completion; the perturbed incidence columns test the secondary key mechanics.

The selected public replay starts from all 300 independently verified submitted-
base permutations. It uses test-only acceptance flags captured during the
separate full-function screen, then applies the smaller bounds and one fixed
mode3. Those stored flags reproduce graph-state admission in this diagnostic;
no lookup table or name will be used by the production caller. An independent
composition of raw mode3 and conditional ordinary PEO2 is compared to the
selected helper on every exact output permutation, including closed gates.

Result **0.790176632480 ->0.790098120936**, **4 wins,0 losses,296 ties**;
**13 eligible public completions**. Single selected-helper calls total
**0.191469 seconds**, maximum **0.056107 seconds**, test completion **1.08s**.
The independent replay precedes each measured helper call, so this is a
warm local diagnostic, not a paired speed ratio or Linux/x86 cap prediction.
[All300 selected replay rows](../evidence/0218-selected-original-screen.tsv).

| Public diagnostic identity | Base full flops | Selected mode3 and conditional PEO2 |
| --- | ---: | ---: |
| crudeoil_lee4_10 |176348241|174752490|
| crudeoil_lee4_06 |32474991|32467193|
| crudeoil_lee4_09 |129101152|128840457|
| arki0013 |150283404|150041934|

All four have a previously accepted priority and LexBFS win, input<=160172
and factor<=683350, hence fit the smaller fixed limits. Public identities
appear only in diagnostic evidence. The order function does not inspect a
name, hash, corpus membership, stored answer, hidden result or persistent state.

## Reproduction and pending production decision

Run from the standalone benchmark root with the installed Yukon, gcc,
Cargo/Rust1.98.1 and cargo-deny0.20.2. Git LFS preceded the original clone;
setup and baseline were completed earlier. No restart or hook setup is needed.

```sh
SSI_ALLOW_UNSANDBOXED_WORKER=1 SSI_INDEP_FORCE=1 SSI_MARK_NOSCORE=1 \
cargo test --release --offline --locked -p ssi-candidate-worker \
ordering::peo_extract::tests::original_incidence_queue_larger_directed_duplicate_weights \
-- --exact --nocapture --test-threads=1

SSI_ALLOW_UNSANDBOXED_WORKER=1 SSI_INDEP_FORCE=1 SSI_MARK_NOSCORE=1 \
cargo test --release --offline --locked -p ssi-candidate-worker \
ordering::probe::campaign::probe_selected_original_incidence_bonus \
-- --ignored --exact --nocapture --test-threads=1
```

Production activation is deferred until the pending base clears its complete
official grade. Then the selected helper, module and caller can be enabled
with the actual priority-winner boolean, followed by fresh all 300 exact
permutation comparisons, complete release checks, generated pairs and the
required unchanged sandbox `yukon run`. CPU diagnostics and scoring remain
sequential. Verified evidence, score and metadata must precede any upload.
The inherited schedule's author newjordan remains a submission coauthor.

All changes are under `src/ordering/`. New algorithm code uses Rust standard
library only and existing trusted pattern/scoring helpers. No manifest,
dependency, trusted scorer, corpus, native source, worker, workflow, sandbox,
purity rule or objective changes. Production will read no environment, clock,
filesystem or network and use no persistent cache or stored answer. Test-only
oracle/corpus/cache/timing controls do not enter the graded function. Later
selection, verification and official outcome are appended when complete.
