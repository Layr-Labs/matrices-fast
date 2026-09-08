# One insertion unlocking several fill deletions — test-only screen

## Completed result

Remote34170108644 succeeded. Two tests passed, including4270 insertion-
certificate comparisons. The public screen completed300 rows (its first
REPAIR marker shares the test-header line; an anchored grep would omit it).
Score0.806559832->0.806559806, one winner arki0016:836603->836594. Isolated
added time totaled1.081510s; worst31.391ms on rsyn0820m04m. Runner was2logical
CPUs AMD EPYC9V74. This negligible gain is not a submission candidate.
Production capped grading was intentionally not part of the test workflow.
Next screen0106 tests exact neutral completion flips and subsequent deletions.

2026-09-07. Prepared alongside remote epoch experiment34168819206; not part
of that immutable run and not called by production order(). No local build,
test, benchmark or candidate execution. Remote validation is not yet dispatched.

Update: dispatched remote34170108644, lab commit e694b54f3f67ea163f87ca8fd1677e56bba6dfb4.
Candidate patch SHA256ae1b321ae54bc4635314f491a455c906cde58a355edcb28903d8dda6cd23b9d3,
verified byte-for-byte after upload. The remote workflow runs sandboxed builds,
two targeted tests, and the ignored public screen. It deliberately does not
run a capped production grader: the repair is not integrated in production.
Previous epoch experiment yielded no changed public flop counts. Results of
this new screen are pending; no improvement or runtime safety is claimed.

## Why leave the current completion?

An inclusion-minimal completion has no individually removable fill edge.
Repeating a subset-only minimalization cannot improve it. A different
completion may require adding an edge before removing other fill edges.

There is a useful no-go result for the simplest exchange. Let uv be a
nonremovable fill edge in chordal H. If H-uv+xy is chordal, the only missing
pair in C=N(u) intersect N(v) must be xy: every other missing pair would still
leave an induced four-cycle. Thus every vertex in C except x,y is adjacent
to both x and y. Consequently kxy=|N(x) intersect N(y)| is at least |C|,
because u,v replace x,y in that intersection. Using F=n+3m+2t, the exchange
changes F by 2*(kxy-|C|), which is nonnegative. A single-for-single repair is
therefore not a strict flop improvement in this setting. This is our direct
derivation, not a claimed novel literature theorem or a measured result.

One insertion followed by MULTIPLE deletions is different. For original
K_(2,4), the completion that makes the four-vertex side a clique costs80.
Adding the edge between the two other vertices permits deleting the six
unnecessary clique edges, yielding cost41. These are hand-derived synthetic
objectives, not public-corpus measurements. The regression test checks them
with the existing completion constructor and will run remotely.

## Prototype

separator_repair.rs is declared only under cfg(test). It collects a bounded
list of missing-edge witnesses from failed clique checks, prioritizing pairs
that recur. For at most four pairs, it checks that the common neighborhood
separates the endpoints; that certifies a chordality-preserving insertion.
The inserted edge is protected while the existing watcher tries removing
original fill edges. Each trial starts from the same completion. A new PEO is
kept only if its chordal graph cost strictly decreases; the public screen
also applies the original exact scorer to the resulting permutation.

Prototype limits: eight-million logical credits, one-million reserved census,
at most64 retained witness pairs, common-neighborhood census width256,
at mostfour one-million-credit deletion trials. The public screen admits
16<=n<10000, input nnz<130000, and completion fill<=600000. These are provisional
test settings, NOT measured production gates. Graph allocation, witness census,
connectivity, sorting, and realization can still cost significant wall time.

Two unrun tests cover the synthetic repair and exhaustive insertion-certificate
comparison on all five-vertex graphs that pass a chordality check. An ignored
300-public-matrix screen reports before/after flop counts and isolated added
runtime. No corpus identities or hidden information affect the algorithm.

## Sources and status

- [Chordal insertion certificate](../literature/deshpande-garofalakis-jordan-stepwise.md).
- [FLOPs versus fill](../literature/luce-ng-minimum-flops.md).
- [MCS-ETree considered but not implemented](../literature/heggernes-peyton-mcs-etree.md).

The primary-paper ideas were implemented independently using standard-library
containers and the already-existing watcher. No fetched code was copied.
Only source/diff review has occurred. Need remote compile, exhaustive tests,
public breadth and timing before even proposing a production integration.
