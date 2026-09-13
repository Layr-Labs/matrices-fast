//! Exact scheduling of eight selected vertices among a fixed sequence.
//! States retain the eliminated subset and the fixed-prefix position.
fn eliminate(g: &mut [u64], n: usize, v: usize) {
    let w = n.div_ceil(64);
    let row = super::patch::row_words(&g[v * w..(v + 1) * w]);
    for &(j, mut bits) in &row {
        while bits != 0 {
            let u = j * 64 + bits.trailing_zeros() as usize;
            bits &= bits - 1;
            super::patch::fill_row(&mut g[u * w..(u + 1) * w], &row, u, v);
        }
    }
    g[v * w..(v + 1) * w].fill(0);
}
fn closure(mut c: usize, allowed: usize, adj: &[usize]) -> usize {
    loop {
        let next = c | (adj[c] & allowed);
        if next == c {
            return c;
        }
        c = next;
    }
}
pub(super) fn schedule(
    g: &[u64],
    n: usize,
    total: usize,
    selected: &[usize],
) -> Option<(Vec<usize>, u64)> {
    schedule_impl(g, n, total, selected, None)
}

pub(super) fn schedule_weighted(
    g: &[u64],
    n: usize,
    total: usize,
    selected: &[usize],
    weights: &[u64],
    cliques: &[bool],
) -> Option<(Vec<usize>, u64)> {
    if weights.len() != n || cliques.len() != n || weights.contains(&0) {
        return None;
    }
    schedule_impl(g, n, total, selected, Some((weights, cliques)))
}

fn schedule_impl(
    g: &[u64],
    n: usize,
    total: usize,
    selected: &[usize],
    weighted: Option<(&[u64], &[bool])>,
) -> Option<(Vec<usize>, u64)> {
    let k = selected.len();
    if k == 0 || k > 12 || n > 1024 || total < k || total > n {
        return None;
    }
    let w = n.div_ceil(64);
    let states = 1usize << k;
    let full = states - 1;
    let fixed = total - k;
    if (fixed + 1) * states * (w + k) > 4_000_000 {
        return None;
    }
    if weighted.is_some() && (fixed + 1) * states * (n + k) > 8_000_000 {
        return None;
    }
    let mut ids = selected.to_vec();
    let mut inverse = vec![usize::MAX; n];
    for (j, &v) in selected.iter().enumerate() {
        if v >= total || inverse[v] != usize::MAX {
            return None;
        }
        inverse[v] = j;
    }
    for v in 0..n {
        if inverse[v] == usize::MAX {
            inverse[v] = ids.len();
            ids.push(v);
        }
    }
    let weights: Vec<u64> = ids
        .iter()
        .map(|&v| weighted.map_or(1, |(w, _)| w[v]))
        .collect();
    let mut cliques: Vec<bool> = ids
        .iter()
        .map(|&v| weighted.is_none_or(|(_, c)| c[v]))
        .collect();
    let mut nonunit = vec![0u64; w];
    for (v, &weight) in weights.iter().enumerate() {
        if weight != 1 {
            nonunit[v / 64] |= 1 << (v % 64);
        }
    }
    let has_nonunit = nonunit.iter().any(|&bits| bits != 0);
    let weight_of = |mut bits: u64, word: usize| -> u64 {
        let mut sum = bits.count_ones() as u64;
        bits &= nonunit[word];
        while bits != 0 {
            sum += weights[word * 64 + bits.trailing_zeros() as usize] - 1;
            bits &= bits - 1;
        }
        sum
    };
    let block_cost = |d: u64, v: usize, clique: bool| -> u64 {
        if weights[v] == 1 {
            (d + 1).pow(2)
        } else if clique {
            super::patch::sum_squares(d + weights[v]) - super::patch::sum_squares(d)
        } else {
            weights[v] * (d + 1).pow(2)
        }
    };
    let mut h = vec![0u64; n * w];
    for v in 0..n {
        for j in 0..w {
            let mut bits = g[v * w + j];
            while bits != 0 {
                let u = j * 64 + bits.trailing_zeros() as usize;
                bits &= bits - 1;
                h[inverse[v] * w + inverse[u] / 64] |= 1u64 << (inverse[u] % 64);
            }
        }
    }
    let inf = u64::MAX / 4;
    let mut dp = vec![inf; states];
    dp[0] = 0;
    let mut back = vec![-2i8; (fixed + 1) * states];
    let mut unions = vec![0u64; states * w];
    let mut adj = vec![0usize; states];
    let mut boundary = vec![0u64; states];
    let mut degree = vec![0u64; states];
    let mut cache = vec![u64::MAX; states];
    let mut next = vec![inf; states];
    let mut selected_changed = true;
    for i in 0..=fixed {
        if selected_changed {
            for s in 1..states {
                let bit = s & s.wrapping_neg();
                let b = bit.trailing_zeros() as usize;
                let prev = s ^ bit;
                let mut count = 0u64;
                for j in 0..w {
                    unions[s * w + j] = unions[prev * w + j] | h[b * w + j];
                    count += weight_of(unions[s * w + j], j);
                }
                adj[s] = unions[s * w] as usize & full;
                boundary[s] = count - weight_of((adj[s] & s) as u64, 0);
            }
        }
        if i < fixed {
            cache.fill(u64::MAX);
            let u = k + i;
            let touched = h[u * w] as usize & full;
            for s in 0..states {
                let c = closure(touched & s, s, &adj);
                if cache[c] == u64::MAX {
                    let mut d = 0;
                    for j in 0..w {
                        let mut z = h[u * w + j] | unions[c * w + j];
                        if j == 0 {
                            z &= !(c as u64);
                        }
                        if j == u / 64 {
                            z &= !(1u64 << (u % 64));
                        }
                        d += weight_of(z, j);
                    }
                    cache[c] = d;
                }
                degree[s] = cache[c];
            }
        }
        next.fill(inf);
        for s in 0..states {
            if dp[s] == inf {
                continue;
            }
            let mut rem = full ^ s;
            while rem != 0 {
                let bit = rem & rem.wrapping_neg();
                rem &= rem - 1;
                let b = bit.trailing_zeros() as usize;
                let c = closure(bit, s | bit, &adj);
                let val = dp[s] + block_cost(boundary[c], b, cliques[b] || c != bit);
                if val < dp[s | bit] {
                    dp[s | bit] = val;
                    back[i * states + (s | bit)] = b as i8;
                }
            }
            if i < fixed {
                let u = k + i;
                let clique = cliques[u] || (h[u * w] as usize & s) != 0;
                let val = dp[s] + block_cost(degree[s], u, clique);
                if val < next[s] {
                    next[s] = val;
                    back[(i + 1) * states + s] = -1;
                }
            }
        }
        if i < fixed {
            selected_changed = h[(k + i) * w] as usize & full != 0;
            if has_nonunit {
                for j in 0..w {
                    let mut bits = h[(k + i) * w + j] & nonunit[j];
                    while bits != 0 {
                        cliques[j * 64 + bits.trailing_zeros() as usize] = true;
                        bits &= bits - 1;
                    }
                }
            }
            eliminate(&mut h, n, k + i);
            std::mem::swap(&mut dp, &mut next);
        }
    }
    let cost = dp[full];
    let (mut i, mut s) = (fixed, full);
    let mut answer = Vec::with_capacity(total);
    while i > 0 || s != 0 {
        let b = back[i * states + s];
        if b == -1 {
            if i == 0 {
                return None;
            }
            answer.push(ids[k + i - 1]);
            i -= 1;
        } else {
            if b < 0 || s & (1usize << b) == 0 {
                return None;
            }
            answer.push(ids[b as usize]);
            s ^= 1usize << b;
        }
    }
    answer.reverse();
    Some((answer, cost))
}
