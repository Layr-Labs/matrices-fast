use super::*;

fn pattern(n: usize, edges: &[(usize, usize)]) -> ScoringPattern {
    let mut rows = vec![Vec::new(); n];
    for &(u, v) in edges {
        rows[u].push(v);
        rows[v].push(u);
    }
    let mut cp = vec![0];
    let mut ri = Vec::new();
    for row in &mut rows {
        row.sort_unstable();
        row.dedup();
        ri.extend_from_slice(row);
        cp.push(ri.len());
    }
    ScoringPattern {
        n,
        col_ptr: cp,
        row_idx: ri,
    }
}

#[test]
fn revisited_group_does_not_retain_deleted_neighbors() {
    VERIFY_INTERSECTIONS.with(|flag| flag.set(true));
    // Synthetic witness: deleting (3, 6) in a different group used to leave
    // stamp 7 at vertex 3. Revisiting group 6 then incorrectly included 3
    // in the common neighborhood of (6, 8), whose only common neighbor is 0.
    let sp = pattern(
        12,
        &[
            (0, 2),
            (0, 6),
            (0, 8),
            (1, 3),
            (1, 10),
            (3, 5),
            (3, 8),
            (3, 10),
            (3, 11),
            (4, 9),
            (5, 9),
            (7, 8),
            (8, 11),
        ],
    );
    let seed = [0, 4, 8, 2, 9, 3, 5, 1, 7, 11, 6, 10];
    let (candidates, completed) = minl_candidates(&sp, &seed).expect("nonempty descent");
    assert!(completed);
    for candidate in candidates {
        assert!(is_bijection(&candidate, sp.n));
        assert!(flops_of(&sp, &candidate) <= flops_of(&sp, &seed));
    }
}

#[test]
fn synthetic_descents_preserve_bijection_determinism_and_cost() {
    VERIFY_INTERSECTIONS.with(|flag| flag.set(true));
    let mut state = 0x51a7_9b03_u64;
    for n in [8, 16, 24, 32] {
        for density in [2, 4, 6] {
            for _ in 0..8 {
                let mut edges = Vec::new();
                for u in 0..n {
                    for v in u + 1..n {
                        if splitmix64(&mut state) % 10 < density {
                            edges.push((u, v));
                        }
                    }
                }
                let sp = pattern(n, &edges);
                let seed = relabel(n, splitmix64(&mut state));
                let first = minl_candidates(&sp, &seed);
                assert_eq!(first, minl_candidates(&sp, &seed));
                if let Some((candidates, _)) = first {
                    let base = flops_of(&sp, &seed);
                    // Only the first candidate is a PEO of the descended
                    // completion; the optional AMD realization is heuristic.
                    assert!(flops_of(&sp, &candidates[0]) <= base);
                    for candidate in candidates {
                        assert!(is_bijection(&candidate, n));
                    }
                }
            }
        }
    }
}
