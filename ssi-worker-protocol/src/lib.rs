//! Shared binary protocol for the trusted parent/candidate worker boundary.
//!
//! Pattern format (all little-endian u64):
//! `n | col_ptr_len | col_ptr[..] | row_idx_len | row_idx[..]`
//!
//! Permutation format (all little-endian u64):
//! `count | permutation[..]`

use ssi_scoring::Pattern;
use std::fs;
use std::io::{Read, Write};
use std::path::Path;

pub fn write_pattern(path: &Path, pat: &Pattern) -> std::io::Result<()> {
    let total = 8 + 8 + pat.col_ptr.len() * 8 + 8 + pat.row_idx.len() * 8;
    let mut buf = Vec::with_capacity(total);
    buf.extend_from_slice(&(pat.n as u64).to_le_bytes());
    buf.extend_from_slice(&(pat.col_ptr.len() as u64).to_le_bytes());
    for &value in &pat.col_ptr {
        buf.extend_from_slice(&(value as u64).to_le_bytes());
    }
    buf.extend_from_slice(&(pat.row_idx.len() as u64).to_le_bytes());
    for &value in &pat.row_idx {
        buf.extend_from_slice(&(value as u64).to_le_bytes());
    }
    fs::File::create(path)?.write_all(&buf)
}

pub fn read_pattern(path: &Path) -> std::io::Result<Pattern> {
    let mut bytes = Vec::new();
    fs::File::open(path)?.read_to_end(&mut bytes)?;
    let invalid =
        |message: &str| std::io::Error::new(std::io::ErrorKind::InvalidData, message.to_string());

    let mut cursor = 0usize;
    let read_u64 = |cursor: &mut usize| -> std::io::Result<u64> {
        let end = cursor
            .checked_add(8)
            .ok_or_else(|| invalid("offset overflow"))?;
        if end > bytes.len() {
            return Err(invalid("truncated pattern file"));
        }
        let value = u64::from_le_bytes(bytes[*cursor..end].try_into().unwrap());
        *cursor = end;
        Ok(value)
    };

    let n = read_usize(read_u64(&mut cursor)?, "pattern dimension")?;
    let col_ptr_len = read_usize(read_u64(&mut cursor)?, "column pointer length")?;
    if col_ptr_len > bytes.len() / 8 {
        return Err(invalid("column pointer length exceeds input size"));
    }
    let mut col_ptr = Vec::with_capacity(col_ptr_len);
    for _ in 0..col_ptr_len {
        col_ptr.push(read_usize(read_u64(&mut cursor)?, "column pointer")?);
    }

    let row_idx_len = read_usize(read_u64(&mut cursor)?, "row index length")?;
    if row_idx_len > bytes.len().saturating_sub(cursor) / 8 {
        return Err(invalid("row index length exceeds remaining input"));
    }
    let mut row_idx = Vec::with_capacity(row_idx_len);
    for _ in 0..row_idx_len {
        row_idx.push(read_usize(read_u64(&mut cursor)?, "row index")?);
    }

    if cursor != bytes.len() {
        return Err(invalid("trailing bytes after pattern"));
    }
    if col_ptr.len().checked_sub(1) != Some(n) {
        return Err(invalid("col_ptr length != n+1"));
    }
    if col_ptr.first() != Some(&0) {
        return Err(invalid("col_ptr[0] != 0"));
    }
    if col_ptr.last().copied() != Some(row_idx.len()) {
        return Err(invalid("col_ptr[n] != row_idx.len()"));
    }

    Ok(Pattern {
        n,
        col_ptr,
        row_idx,
    })
}

pub fn write_permutation(path: &Path, permutation: &[usize]) -> std::io::Result<()> {
    let mut buf = Vec::with_capacity(8 + permutation.len() * 8);
    buf.extend_from_slice(&(permutation.len() as u64).to_le_bytes());
    for &index in permutation {
        buf.extend_from_slice(&(index as u64).to_le_bytes());
    }
    fs::File::create(path)?.write_all(&buf)
}

/// Read a permutation the worker wrote, given the dimension `expected_n` of the
/// pattern the parent staged. A valid permutation is exactly `8 + 8*expected_n`
/// bytes, so any longer file is rejected from its length ALONE before its
/// contents are allocated. This runs in the trusted parent against a file an
/// untrusted worker controls: without the bound, a worker that writes a
/// multi-GB self-consistent file would drive an equally large allocation here.
pub fn read_permutation(path: &Path, expected_n: usize) -> std::io::Result<Vec<usize>> {
    let max_bytes = expected_n
        .checked_mul(8)
        .and_then(|body| body.checked_add(8))
        .ok_or_else(|| invalid_data("expected permutation size overflows this platform"))?;

    let file = fs::File::open(path)?;
    let file_len = file.metadata()?.len();
    if file_len > max_bytes as u64 {
        return Err(invalid_data(&format!(
            "permutation file is {file_len} bytes, exceeds {max_bytes} for n={expected_n}"
        )));
    }

    // Bound the read to the budget even if the length check above raced a
    // growing file: `take` caps the allocation regardless of the real size.
    let mut bytes = Vec::with_capacity(max_bytes);
    file.take(max_bytes as u64).read_to_end(&mut bytes)?;
    if bytes.len() < 8 {
        return Err(invalid_data("permutation file too short"));
    }
    let count_u64 = u64::from_le_bytes(bytes[0..8].try_into().unwrap());
    let count = read_usize(count_u64, "permutation length")?;
    let expected = count.checked_mul(8).and_then(|size| size.checked_add(8));
    if expected != Some(bytes.len()) {
        return Err(invalid_data(&format!(
            "permutation file length mismatch: header says {count}"
        )));
    }

    let mut permutation = Vec::with_capacity(count);
    for index in 0..count {
        let offset = 8 + index * 8;
        let value = u64::from_le_bytes(bytes[offset..offset + 8].try_into().unwrap());
        permutation.push(read_usize(value, "permutation index")?);
    }
    Ok(permutation)
}

fn read_usize(value: u64, description: &str) -> std::io::Result<usize> {
    usize::try_from(value)
        .map_err(|_| invalid_data(&format!("{description} does not fit this platform")))
}

fn invalid_data(message: &str) -> std::io::Error {
    std::io::Error::new(std::io::ErrorKind::InvalidData, message.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_file(name: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("ssi-worker-protocol-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        dir.join(name)
    }

    #[test]
    fn pattern_round_trip() {
        let pattern = Pattern {
            n: 2,
            col_ptr: vec![0, 1, 2],
            row_idx: vec![1, 0],
        };
        let path = temp_file("pattern.bin");
        write_pattern(&path, &pattern).unwrap();
        let actual = read_pattern(&path).unwrap();
        assert_eq!(actual.n, pattern.n);
        assert_eq!(actual.col_ptr, pattern.col_ptr);
        assert_eq!(actual.row_idx, pattern.row_idx);
    }

    #[test]
    fn permutation_round_trip() {
        let path = temp_file("permutation.bin");
        let permutation = vec![3, 1, 0, 2];
        write_permutation(&path, &permutation).unwrap();
        assert_eq!(
            read_permutation(&path, permutation.len()).unwrap(),
            permutation
        );
    }

    #[test]
    fn permutation_longer_than_expected_dimension_is_rejected() {
        // A self-consistent permutation file whose header count exceeds the
        // dimension the parent staged. The old length-only check accepted it and
        // allocated `count` usizes in the TRUSTED parent; a malicious worker that
        // wrote a multi-GB self-consistent file could exhaust parent memory. With
        // the expected dimension known, the read must reject before allocating.
        let path = temp_file("oversized-permutation.bin");
        let bloated = vec![0usize; 4096];
        write_permutation(&path, &bloated).unwrap();
        assert!(read_permutation(&path, 4).is_err());
    }

    #[test]
    fn malformed_lengths_are_rejected() {
        let pattern_path = temp_file("bad-pattern.bin");
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&1u64.to_le_bytes());
        bytes.extend_from_slice(&u64::MAX.to_le_bytes());
        std::fs::write(&pattern_path, bytes).unwrap();
        assert!(read_pattern(&pattern_path).is_err());

        let permutation_path = temp_file("bad-permutation.bin");
        std::fs::write(&permutation_path, u64::MAX.to_le_bytes()).unwrap();
        assert!(read_permutation(&permutation_path, 0).is_err());
    }
}
