use feral_ordering_core::{quotient_graph::*, CscPattern, OrderingError};

pub(super) const WORK: usize = 128_000_000;

pub(super) fn front_work(ws: &Workspace, start: usize, end: usize) -> usize {
    let mut scan = (end - start).saturating_mul(8);
    for j in start..end {
        let u = ws.iw[j] as usize;
        scan = scan.saturating_add(4 * ws.len[u].max(0) as usize);
    }
    scan
}

pub(super) fn order(
    graph: &CscPattern<'_>,
    dense_alpha: f64,
    aggressive: bool,
    fill: bool,
    mut work: usize,
) -> Result<Vec<i32>, OrderingError> {
    let exhausted = || OrderingError::Internal("quotient work limit");
    let buckets = if fill {
        MinFill::n_buckets(graph.n)
    } else {
        MinDegree::n_buckets(graph.n)
    };
    let mut ws = Workspace::new_with_n_buckets(graph, &WorkspaceOptions { dense_alpha }, buckets)?;
    while ws.nel < ws.n {
        if fill && ws.mindeg >= ws.n {
            work = work.checked_sub(ws.n - ws.nel).ok_or_else(exhausted)?;
        }
        let Some(v) = (if fill {
            select_pivot_amf(&mut ws)
        } else {
            select_pivot(&mut ws)
        }) else {
            break;
        };
        let elements = ws.elen[v];
        let mut scan = ws.len[v].max(0) as usize + 1;
        for j in 0..elements.max(0) as usize {
            let e = ws.iw[ws.pe[v] as usize + j] as usize;
            scan = scan.saturating_add(ws.len[e].max(0) as usize);
        }
        work = work.checked_sub(scan).ok_or_else(exhausted)?;
        let (start, end, mass, degree) = if fill {
            create_element_amf(&mut ws, v)?
        } else {
            create_element(&mut ws, v)?
        };
        work = work
            .checked_sub(front_work(&ws, start, end))
            .ok_or_else(exhausted)?;
        if fill {
            finalize_step_amf(&mut ws, v, start, end, mass, degree, elements, aggressive);
        } else {
            finalize_step(&mut ws, v, start, end, mass, degree, elements, aggressive);
        }
    }
    Ok(finalize_permutation(&mut ws))
}
