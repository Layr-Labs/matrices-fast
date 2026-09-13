//! Exact recognition of L(K_t) joined to universal vertices, followed by
//! the closed-form singleton-merge optimum. No matrix labels are consulted.
pub(super) fn order(cp: &[usize], ri: &[usize]) -> Option<Vec<usize>> {
    let n = cp.len().checked_sub(1)?;
    if n < 10 || n > 10000 {
        return None;
    }
    let degree: Vec<_> = cp.windows(2).map(|a| a[1] - a[0]).collect();
    let universal: Vec<_> = (0..n).filter(|&v| degree[v] == n - 1).collect();
    let vertices: Vec<_> = (0..n).filter(|&v| degree[v] != n - 1).collect();
    let m = vertices.len();
    let mut t = 5usize;
    while t * (t - 1) / 2 < m {
        t += 1;
    }
    if t * (t - 1) / 2 != m
        || vertices
            .iter()
            .any(|&v| degree[v] != 2 * (t - 2) + universal.len())
    {
        return None;
    }
    let adjacent = |u: usize, v: usize| ri[cp[u]..cp[u + 1]].binary_search(&v).is_ok();
    let e = vertices[0];
    let neighbors: Vec<_> = vertices
        .iter()
        .copied()
        .filter(|&v| adjacent(e, v))
        .collect();
    let x = *neighbors.first()?;
    let common: Vec<_> = neighbors
        .iter()
        .copied()
        .filter(|&v| adjacent(x, v))
        .collect();
    let ys: Vec<_> = common
        .iter()
        .copied()
        .filter(|&v| !common.iter().any(|&u| adjacent(v, u)))
        .collect();
    if ys.len() != 1 {
        return None;
    }
    let mut a = vec![x];
    a.extend(common.into_iter().filter(|&v| v != ys[0]));
    a.sort_unstable();
    if a.len() != t - 2 {
        return None;
    }
    let mut label = vec![None; n];
    label[e] = Some((0, 1));
    for (j, &v) in a.iter().enumerate() {
        label[v] = Some((0, j + 2));
    }
    for &v in &vertices {
        if label[v].is_none() {
            let z: Vec<_> = a
                .iter()
                .enumerate()
                .filter(|(_, u)| adjacent(v, **u))
                .map(|(j, _)| j + 2)
                .collect();
            if adjacent(e, v) {
                if z.len() != 1 {
                    return None;
                }
                label[v] = Some((1, z[0]));
            } else {
                if z.len() != 2 {
                    return None;
                }
                label[v] = Some((z[0], z[1]));
            }
        }
    }
    let mut pair = vec![usize::MAX; t * t];
    for &v in &vertices {
        let (a, b) = label[v]?;
        if pair[a * t + b] != usize::MAX {
            return None;
        }
        pair[a * t + b] = v;
        pair[b * t + a] = v;
    }
    // Degree, unique labels and adjacency inclusion imply exact reconstruction.
    for &v in &vertices {
        let (a, b) = label[v]?;
        for &u in &ri[cp[v]..cp[v + 1]] {
            if let Some((c, d)) = label[u] {
                if a != c && a != d && b != c && b != d {
                    return None;
                }
            }
        }
    }
    // The split gap relative to singleton merges is
    // a*b*(a-1)*(b-1)*(t-a-b)*(t-a-b-1)/2 >= 0.
    // Thus split 1+(k-1) always attains the recurrence minimum. Emit that
    // construction directly, preserving the former DP's first-minimum ties.
    let mut out = Vec::with_capacity(n);
    for a in (0..t - 1).rev() {
        for b in a + 1..t {
            out.push(pair[a * t + b]);
        }
    }
    out.extend(universal);
    if out.len() == n {
        Some(out)
    } else {
        None
    }
}
