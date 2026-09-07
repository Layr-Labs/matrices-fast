# Stage-6 move sampling and alternate-seed controls

This is the local experiment formerly numbered 0085. Upstream 0085 denotes
[wider alternate-seed capture](0085-widen-alt-seed-capture.md).

The baseline was `4d86414`, dev 0.826784. Stage 6 uses 512 paired-swap draws
at seed `0x917ad73`, followed by 1024 plateau draws at `0xa839d37`, inside
`12 <= n <= 300 && nnz <= 3000`.

The measured bundle with up to 32 stream pairs, an estimated 150M-unit
allowance based on starting fill, eight alternate seeds, and stage-7 hygiene
scored **0.826637**. Submission `6ce0721` returned **failed**, without a
public score or identified failure category. The stage-6 multi-set component
was removed. The remaining pool-eight/hygiene bundle scored **0.826723**;
submission `f8941f7f` completed at hidden **0.859571**, versus frontier
0.859573, and was rejected below the promotion floor.

The complete multi-set bundle raised 104 of 300 measured rows by more than
25 ms, with a 153.7 ms maximum increase. This is evidence of broad runtime
exposure, not a proof of the hidden failure's cause. Completion of the smaller
bundle on one hidden run is not a future safety certificate.

The post-hoc 32-stream treatment gained about 0.31 absolute dev bips; placing
it before later stages gained about 0.86 bips. A changed intermediate can alter
later heuristic basins, so a post-hoc result is not a general bound on a
mid-pipeline implementation.

SmallScore work depends on fill produced by each trial. An estimate using
starting fill does not itself bound every proposal's operations. Each stream,
core, setup pass, and repeated reduction depth needs explicit accounting.

The current frontier `996e8d6` already includes eight alternate seeds and wider
capture. The local 32-stream treatment remains withdrawn. The residual-core
10M screen is recorded separately in [0089](0089-production-core-stage6-screen.md).

The original detailed local record is preserved in Git stash
`98e7de7a6e4da2e238849725a04c488361232b9f` at its former filename, and in the
original `autoresearch-r4b` branch history. This summary uses the current
experiment numbering and distinguishes observations from causal hypotheses.
