//! Framed file delivery protocol for `udtd`.
//!
//! On-wire layout (big-endian):
//! ```text
//! u32  name_len
//! u8×N UTF-8 file name (basename only; no path separators)
//! u64  file_len
//! u8×M file bytes
//! ```

use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

/// Maximum accepted file name length (bytes).
pub const MAX_NAME_LEN: usize = 1024;
/// Maximum accepted file size (bytes) — 16 TiB guard.
pub const MAX_FILE_LEN: u64 = 16 << 40;

/// Errors while framing / unframing.
#[derive(Debug)]
pub enum FrameError {
    /// I/O.
    Io(io::Error),
    /// Protocol / validation.
    Proto(String),
}

impl std::fmt::Display for FrameError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(e) => write!(f, "{e}"),
            Self::Proto(s) => write!(f, "{s}"),
        }
    }
}

impl std::error::Error for FrameError {}

impl From<io::Error> for FrameError {
    fn from(e: io::Error) -> Self {
        Self::Io(e)
    }
}

/// Strip directories and reject `..` / empty names.
pub fn safe_basename(name: &str) -> Result<String, FrameError> {
    let name = name.trim();
    if name.is_empty() {
        return Err(FrameError::Proto("empty file name".into()));
    }
    if name.len() > MAX_NAME_LEN {
        return Err(FrameError::Proto(format!(
            "file name too long ({} > {MAX_NAME_LEN})",
            name.len()
        )));
    }
    if name.contains('\0') {
        return Err(FrameError::Proto("file name contains NUL".into()));
    }
    let base = Path::new(name)
        .file_name()
        .and_then(|s| s.to_str())
        .ok_or_else(|| FrameError::Proto(format!("invalid file name: {name}")))?;
    if base.is_empty() || base == "." || base == ".." {
        return Err(FrameError::Proto(format!("unsafe file name: {name}")));
    }
    if base.contains('/') || base.contains('\\') {
        return Err(FrameError::Proto(format!("path separator in name: {name}")));
    }
    Ok(base.to_string())
}

/// Choose a non-colliding path under `dir` for `basename`.
pub fn unique_path(dir: &Path, basename: &str) -> PathBuf {
    let candidate = dir.join(basename);
    if !candidate.exists() {
        return candidate;
    }
    let stem = Path::new(basename)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("file");
    let ext = Path::new(basename)
        .extension()
        .and_then(|s| s.to_str())
        .map(|e| format!(".{e}"))
        .unwrap_or_default();
    for n in 1..10_000u32 {
        let p = dir.join(format!("{stem}-{n}{ext}"));
        if !p.exists() {
            return p;
        }
    }
    dir.join(format!("{stem}-{}", std::process::id()))
}

/// Write one framed file (header + body) to `out`.
pub fn write_framed<R: Read, W: Write>(
    out: &mut W,
    name: &str,
    file_len: u64,
    body: &mut R,
) -> Result<u64, FrameError> {
    let name = safe_basename(name)?;
    if file_len > MAX_FILE_LEN {
        return Err(FrameError::Proto(format!(
            "file too large: {file_len} > {MAX_FILE_LEN}"
        )));
    }
    let name_bytes = name.as_bytes();
    out.write_all(&(name_bytes.len() as u32).to_be_bytes())?;
    out.write_all(name_bytes)?;
    out.write_all(&file_len.to_be_bytes())?;
    let mut copied = 0u64;
    let mut buf = vec![0u8; 64 * 1024];
    while copied < file_len {
        let want = ((file_len - copied) as usize).min(buf.len());
        let n = body.read(&mut buf[..want])?;
        if n == 0 {
            return Err(FrameError::Proto(format!(
                "unexpected EOF after {copied}/{file_len} bytes"
            )));
        }
        out.write_all(&buf[..n])?;
        copied += n as u64;
    }
    out.flush()?;
    Ok(copied)
}

/// Read exactly `n` bytes from `r`.
fn read_exact_n<R: Read>(r: &mut R, n: usize) -> Result<Vec<u8>, FrameError> {
    let mut buf = vec![0u8; n];
    let mut got = 0;
    while got < n {
        match r.read(&mut buf[got..])? {
            0 => {
                return Err(FrameError::Proto(format!(
                    "unexpected EOF reading header ({got}/{n})"
                )))
            }
            k => got += k,
        }
    }
    Ok(buf)
}

/// One received framed file (metadata + where body bytes should go).
#[derive(Debug)]
pub struct IncomingFile {
    /// Sanitized basename.
    pub name: String,
    /// Declared length.
    pub len: u64,
}

/// Read the frame header from `r`. Returns `Ok(None)` on clean EOF before any bytes.
pub fn read_frame_header<R: Read>(r: &mut R) -> Result<Option<IncomingFile>, FrameError> {
    let mut len_buf = [0u8; 4];
    match r.read(&mut len_buf)? {
        0 => return Ok(None),
        n if n < 4 => {
            // partial — try to finish
            let rest = read_exact_n(r, 4 - n)?;
            len_buf[n..].copy_from_slice(&rest);
        }
        4 => {}
        _ => unreachable!(),
    }
    let name_len = u32::from_be_bytes(len_buf) as usize;
    if name_len == 0 || name_len > MAX_NAME_LEN {
        return Err(FrameError::Proto(format!("bad name_len={name_len}")));
    }
    let name_raw = read_exact_n(r, name_len)?;
    let name = String::from_utf8(name_raw)
        .map_err(|e| FrameError::Proto(format!("name not utf-8: {e}")))?;
    let name = safe_basename(&name)?;
    let len_buf = read_exact_n(r, 8)?;
    let mut le = [0u8; 8];
    le.copy_from_slice(&len_buf);
    let file_len = u64::from_be_bytes(le);
    if file_len > MAX_FILE_LEN {
        return Err(FrameError::Proto(format!("file_len {file_len} too large")));
    }
    Ok(Some(IncomingFile {
        name,
        len: file_len,
    }))
}

/// Copy exactly `len` bytes from `r` to `w`.
pub fn copy_exact<R: Read, W: Write>(r: &mut R, w: &mut W, len: u64) -> Result<u64, FrameError> {
    let mut copied = 0u64;
    let mut buf = vec![0u8; 64 * 1024];
    while copied < len {
        let want = ((len - copied) as usize).min(buf.len());
        let n = r.read(&mut buf[..want])?;
        if n == 0 {
            return Err(FrameError::Proto(format!(
                "unexpected EOF after {copied}/{len} body bytes"
            )));
        }
        w.write_all(&buf[..n])?;
        copied += n as u64;
    }
    w.flush()?;
    Ok(copied)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn round_trip_frame() {
        let payload = b"hello world";
        let mut wire = Vec::new();
        write_framed(
            &mut wire,
            "dir/foo.txt",
            payload.len() as u64,
            &mut Cursor::new(payload.as_slice()),
        )
        .unwrap();
        let mut cur = Cursor::new(wire);
        let hdr = read_frame_header(&mut cur).unwrap().unwrap();
        assert_eq!(hdr.name, "foo.txt");
        assert_eq!(hdr.len, payload.len() as u64);
        let mut out = Vec::new();
        copy_exact(&mut cur, &mut out, hdr.len).unwrap();
        assert_eq!(out, payload);
    }

    #[test]
    fn rejects_dotdot() {
        assert!(safe_basename("..").is_err());
        assert!(safe_basename("").is_err());
    }
}
