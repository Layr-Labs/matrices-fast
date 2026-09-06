# 0069: Independent terminal core relabels on the refreshed leader

Base: a942ebd / submission 5bfc2263, hidden 0.861890. The earlier 0068
submission passed the official gates at hidden 0.862053, beating its starting
0.862386 leader, but was rejected after this concurrent frontier advance.

Preserve all of a942ebd: its extra core refinements, extra-depth recursive
refinement, tighter completion limits, and terminal-pass significance gate.
Preserve its exact-versus-proxy ranking decisions as well. The first package's
extra-depth exact-ranking change is not used here.

Evaluate the same four structural AMF relabel candidates on the K=3 residual
core (reverse / degree-sorted labels, alpha 2.5 / 10; core n 8..4000 and
nnz <=30000), but retain the best in a separate accumulator. Do not return it
to any existing intermediate pass. After every inherited pass, compare its
exact prefix-plus-core cost with the finished leader result. Only a strict
winner receives the existing bounded completion cleanup, independently of the
leader. Accept the cleaned result only on a strict exact improvement. This
keeps all inherited search seeds intact.

The earlier outer paired-swap/neutral-walk call is removed from production;
its helpers and historical regression tests remain. The forest certificate
remains as an optimal-score fast path. Redundant full-graph scoring of spliced
cores is replaced by the exact fixed-prefix plus core score, with the original
selection decisions preserved.

Fresh complete trusted sandboxed development runs, 300 matrices:

- Score: 0.832725 -> 0.832432; fill 0.938965 -> 0.938922.
- Two wins, zero losses, 298 exact ties.
- rsyn0830m04m: 193427 -> 181793 flops (-6.0147%).
- rsyn0820m04m: 177970 -> 167666 flops (-5.7897%).
- Buckets: lt_1k 0.889764 unchanged; 1k_10k 0.866276 -> 0.865300;
  gt_10k 0.764783 unchanged.
- All 62 unit tests pass; 18 diagnostic probes remain ignored.

A control that admits raw relabeled cores only at the end, without their
independent completion cleanup, scores 0.832652. The cleanup increases the
gain while preserving the finished baseline as incumbent. A separate fixed
seed expansion on the first package tied all 300 cases and was not submitted.

Local data and generated stress validation live outside the submission under
recon/codex-combined. Official outcome is pending; no promotion is inferred
from the local result.

Generated stress: all 40 sandboxed calls pass; maximum whole-process wall time
0.351503 s control / 0.356557 s candidate, ten patterns with n 32..8000. This is
local sample evidence only.
