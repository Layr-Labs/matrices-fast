//! JSONL corpus loader for the trusted/grader side.
//!
//! The corpus is `patterns.jsonl`: one JSON object per line, each a full
//! symmetrized CSC sparsity pattern. `pattern_from_jsonl_line` is the SINGLE
//! definition of "corpus line -> Pattern"; both the harness and the grader
//! parse through it, so there is no second parser that can silently disagree
//! — one parser at the parsing boundary.
//!
//! VALUES NEVER ENTER. The corpus format is pattern-only — a line carries `n`,
//! `indptr`, and `indices`, with no value column anywhere — so NARROW INPUT
//! holds by construction: there is nothing for the loader to
//! drop. The diagonal IS present in the stored CSC and is stripped here to
//! match the contract's `Pattern` (off-diagonal both-triangle storage).

use crate::pattern::Pattern;
use std::fmt;
use std::path::Path;

/// Failure to parse a JSONL corpus line into a `Pattern`.
#[derive(Debug)]
pub enum LoadError {
    /// A `patterns.jsonl` line was malformed or structurally invalid.
    Json(String),
}

impl fmt::Display for LoadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LoadError::Json(m) => write!(f, "failed to parse JSONL pattern: {m}"),
        }
    }
}

impl std::error::Error for LoadError {}

/// Parse ONE line of `patterns.jsonl` into `(source, Pattern)`.
///
/// The pipeline (`corpus-generation`) emits a full *symmetrized* CSC pattern
/// that INCLUDES the diagonal (column `j`'s indices contain `j`). The contract
/// `Pattern` is off-diagonal, both-triangle, so we drop `i == j` and rebuild
/// via `Pattern::from_adjacency` (which sorts, dedups, and validates symmetry).
///
/// This is the SINGLE definition of "corpus line -> Pattern". The harness loads
/// the whole corpus through it; a future grader can address one line by index
/// and route it through this same core — one parser at the parsing boundary.
pub fn pattern_from_jsonl_line(line: &str) -> Result<(String, Pattern), LoadError> {
    let err = |m: &str| LoadError::Json(format!("{m} in line: {}", truncate(line, 80)));

    let n = parse_usize_field(line, "\"n\"").ok_or_else(|| err("missing/invalid \"n\""))?;
    let indptr = parse_int_array(line, "\"indptr\"").ok_or_else(|| err("missing \"indptr\""))?;
    let indices = parse_int_array(line, "\"indices\"").ok_or_else(|| err("missing \"indices\""))?;
    let source = parse_string_field(line, "\"source\"").unwrap_or_default();

    if indptr.len() != n + 1 {
        return Err(err("indptr length != n+1"));
    }
    if indptr.first() != Some(&0) {
        return Err(err("indptr[0] != 0"));
    }
    if indptr.last().copied() != Some(indices.len()) {
        return Err(err("indptr[n] != indices.len()"));
    }

    // Expand CSC -> per-column adjacency, dropping the diagonal.
    let mut adj: Vec<Vec<usize>> = vec![Vec::new(); n];
    for j in 0..n {
        let (lo, hi) = (indptr[j], indptr[j + 1]);
        if lo > hi || hi > indices.len() {
            return Err(err("non-monotone indptr"));
        }
        for &i in &indices[lo..hi] {
            if i >= n {
                return Err(err("row index out of range"));
            }
            if i != j {
                adj[j].push(i);
            }
        }
    }
    Ok((source, Pattern::from_adjacency(n, &mut adj)))
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max { s.to_string() } else { format!("{}…", &s[..max]) }
}

/// Find `"key":<digits>` and parse the integer. Returns None if absent/invalid.
fn parse_usize_field(line: &str, key: &str) -> Option<usize> {
    let start = line.find(key)? + key.len();
    let rest = line[start..].trim_start_matches([':', ' ']);
    let end = rest.find(|c: char| !c.is_ascii_digit()).unwrap_or(rest.len());
    rest[..end].parse().ok()
}

/// Find `"key":"value"` and return the string value (no escapes expected).
fn parse_string_field(line: &str, key: &str) -> Option<String> {
    let start = line.find(key)? + key.len();
    let rest = line[start..].trim_start_matches([':', ' ']);
    let rest = rest.strip_prefix('"')?;
    let end = rest.find('"')?;
    Some(rest[..end].to_string())
}

/// Find `"key":[<comma-separated ints>]` and parse into a Vec<usize>.
fn parse_int_array(line: &str, key: &str) -> Option<Vec<usize>> {
    let start = line.find(key)? + key.len();
    let rest = line[start..].trim_start_matches([':', ' ']);
    let rest = rest.strip_prefix('[')?;
    let end = rest.find(']')?;
    let body = &rest[..end];
    if body.trim().is_empty() {
        return Some(Vec::new());
    }
    body.split(',').map(|t| t.trim().parse::<usize>().ok()).collect()
}

/// Load an entire `patterns.jsonl` corpus into `(source, Pattern)` pairs, in
/// file order. Blank lines are skipped. Used by the local harness.
pub fn load_corpus_jsonl(path: &Path) -> Result<Vec<(String, Pattern)>, LoadError> {
    let text = std::fs::read_to_string(path)
        .map_err(|e| LoadError::Json(format!("{}: {e}", path.display())))?;
    let mut out = Vec::new();
    for (i, line) in text.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let (source, pat) = pattern_from_jsonl_line(line)
            .map_err(|e| LoadError::Json(format!("{}:{}: {e}", path.display(), i)))?;
        out.push((source, pat));
    }
    Ok(out)
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn jsonl_line_parses_and_drops_diagonal() {
        // n=4 line from the real corpus (source st_e09). indices include the
        // diagonal (each column j lists j); the contract Pattern must drop it.
        let line = r#"{"n":4,"nnz":12,"indptr":[0,3,6,8,12],"indices":[0,1,3,0,1,3,2,3,0,1,2,3],"hash":"9cce0c0e","source":"st_e09"}"#;
        let (source, p) = pattern_from_jsonl_line(line).unwrap();
        assert_eq!(source, "st_e09");
        assert_eq!(p.n, 4);
        // Column 0 raw = {0,1,3}; diagonal 0 dropped -> {1,3}.
        assert_eq!(p.col(0), &[1, 3]);
        // Column 2 raw = {2,3}; diagonal 2 dropped -> {3}.
        assert_eq!(p.col(2), &[3]);
        // Column 3 raw = {0,1,2,3}; diagonal 3 dropped -> {0,1,2}.
        assert_eq!(p.col(3), &[0, 1, 2]);
        // 12 raw entries - 4 diagonal = 8 off-diagonal.
        assert_eq!(p.nnz(), 8);
    }

    #[test]
    fn jsonl_line_rejects_malformed() {
        assert!(pattern_from_jsonl_line("not json").is_err());
        assert!(pattern_from_jsonl_line(r#"{"n":2}"#).is_err());
    }

    #[test]
    fn load_corpus_parses_all_lines() {
        let jsonl = "\
{\"n\":2,\"nnz\":3,\"indptr\":[0,2,3],\"indices\":[0,1,1],\"hash\":\"a\",\"source\":\"m0\"}
{\"n\":4,\"nnz\":12,\"indptr\":[0,3,6,8,12],\"indices\":[0,1,3,0,1,3,2,3,0,1,2,3],\"hash\":\"b\",\"source\":\"m1\"}
";
        let dir = std::env::temp_dir().join("ssi-scoring-jsonl-io-test");
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("patterns.jsonl");
        std::fs::write(&path, jsonl).unwrap();

        let corpus = load_corpus_jsonl(&path).unwrap();
        assert_eq!(corpus.len(), 2);
        assert_eq!(corpus[0].0, "m0");
        assert_eq!(corpus[1].0, "m1");
        assert_eq!(corpus[1].1.n, 4);
    }
}
