# LexBFS and ordered partition refinement

Primary reference: Jesse Beisegel, Ekkehard Köhler, Robert Scheffler and
Martin Strehler, *Linear Time LexDFS on Chordal Graphs*, ESA 2020,
DOI 10.4230/LIPIcs.ESA.2020.13.
[Publisher PDF](https://drops.dagstuhl.de/storage/00lipics/lipics-vol173-esa2020/LIPIcs.ESA.2020.13/LIPIcs.ESA.2020.13.pdf).
Its section 2 and Algorithm 2 describe LexBFS and ordered partition refinement,
and cite Rose, Tarjan and Lueker's 1976 elimination-order work.

LexBFS visits the vertex with the lexicographically greatest list of previous
visited neighbors, giving earlier visits more significance. The reverse
search is a perfect elimination order on a chordal graph. Ordered partitions
represent equal labels. Splitting each class into neighbors and non-neighbors,
with neighbors first, implements the search in linear graph work. This supplies
a different completion extraction from MCS, which counts previous neighbors.

Related original reference: Michel Habib, Ross McConnell, Christophe Paul and
Laurent Viennot, *Lex-BFS and partition refinement, with applications to
transitive orientation, interval graph recognition and consecutive ones testing*,
Theoretical Computer Science 234 (2000), 59–84, DOI
10.1016/S0304-3975(97)00241-7.
[Author bibliography](https://www.cs.colostate.edu/~rmm/pubs.html),
[publisher abstract](https://www.sciencedirect.com/science/article/pii/S0304397597002417).
The abstract and bibliographic provenance were inspected; the full author-hosted
PDF link was unavailable. Implementation is derived independently from the
algorithmic description above, without copying fetched code.

Our challenge mapping: use the existing checked flat chordal completion,
fixed initial degree order, and fixed reverse neighbor traversal. Store each
partition class and each vertex in linked arrays. Recycle empty class IDs;
one pivot stamp prevents repeatedly splitting the same label class. These are
our implementation choices. Runtime allocation stays linear in vertices plus
the bounded completion. Return only a bijective permutation and accept it only
after exact scoring on the original graph; ordinary PEO may then continue a
strict win. No name, cache, environment or clock enters production.

The graph argument does not guarantee a 2-second wall-clock bound. Dimension,
input nnz and factor nnz all need independent caps, and full parent runtime
needs measurement. [0215 experiment](../experiments/0215-indexed-mcs-and-lex-bfs.md)
records the exhaustive independent label oracle, public policy selection and
complete production checks.
