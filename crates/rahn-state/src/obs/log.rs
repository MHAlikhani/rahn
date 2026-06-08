// SPDX-License-Identifier: Apache-2.0

//! Append-only observation log with canonical framing (ADR 0013).
//!
//! Framing: each record is `u64 LE length + canonical bytes`. Reads parse
//! strictly; malformed framing is a loud error — never silent truncation.
//! The log itself is pure storage; `seq` assignment and validation happen
//! at the ingest API.

use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use super::{ObsError, Observation};

pub struct ObsLog {
    path: PathBuf,
}

impl ObsLog {
    pub fn path(root: &Path) -> PathBuf {
        root.join("observations.log")
    }

    /// Open (or lazily create) the log inside a repository root.
    pub fn open(root: &Path) -> Result<Self, ObsError> {
        let path = Self::path(root);
        if !path.exists() {
            fs::File::create(&path).map_err(io_err)?;
        }
        Ok(ObsLog { path })
    }

    /// Read all records strictly, in stored order.
    pub fn read_all(&self) -> Result<Vec<Observation>, ObsError> {
        let bytes = fs::read(&self.path).map_err(io_err)?;
        let mut out = Vec::new();
        let mut pos = 0usize;
        while pos < bytes.len() {
            if bytes.len() - pos < 8 {
                return Err(ObsError::MalformedRecord {
                    offset: pos,
                    message: "truncated frame length".into(),
                });
            }
            let len = u64::from_le_bytes(bytes[pos..pos + 8].try_into().unwrap()) as usize;
            pos += 8;
            if len > bytes.len() - pos {
                return Err(ObsError::MalformedRecord {
                    offset: pos,
                    message: "frame exceeds file size".into(),
                });
            }
            out.push(Observation::parse(&bytes[pos..pos + len])?);
            pos += len;
        }
        Ok(out)
    }

    /// Append one record (framing applied here). Caller supplies `seq`.
    pub fn append(&self, o: &Observation) -> Result<(), ObsError> {
        let payload = o.canonical_bytes();
        let mut file = fs::OpenOptions::new()
            .append(true)
            .open(&self.path)
            .map_err(io_err)?;
        file.write_all(&(payload.len() as u64).to_le_bytes())
            .map_err(io_err)?;
        file.write_all(&payload).map_err(io_err)?;
        file.flush().map_err(io_err)?;
        Ok(())
    }

    pub fn len(&self) -> Result<usize, ObsError> {
        Ok(self.read_all()?.len())
    }

    /// Whether the log holds no records (falls under clippy's
    /// len-without-is-empty convention; `len` is fallible, so this
    /// mirrors it).
    pub fn is_empty(&self) -> Result<bool, ObsError> {
        Ok(self.len()? == 0)
    }
}

fn io_err(e: io::Error) -> ObsError {
    ObsError::MalformedRecord {
        offset: 0,
        message: format!("io error: {e}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::obs::{Subject, Value};

    fn temp_root(tag: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!(
            "rahn-obs-{tag}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&d).unwrap();
        d
    }

    fn rec(seq: u64, time_ns: u64) -> Observation {
        Observation::new(
            seq,
            time_ns,
            "ab".repeat(32),
            Subject::parse("web").unwrap(),
            "latency_ns",
            Value::Counter(time_ns),
        )
        .unwrap()
    }

    #[test]
    fn append_read_round_trip() {
        let root = temp_root("rt");
        let log = ObsLog::open(&root).unwrap();
        for seq in 0..3u64 {
            log.append(&rec(seq, 100 + seq)).unwrap();
        }
        let all = log.read_all().unwrap();
        assert_eq!(all.len(), 3);
        assert_eq!(all[2].seq, 2);
        assert_eq!(log.len().unwrap(), 3);
        fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn malformed_framing_is_loud() {
        let root = temp_root("bad");
        let log = ObsLog::open(&root).unwrap();
        log.append(&rec(0, 1)).unwrap();
        // Corrupt the file: truncate mid-frame.
        let path = ObsLog::path(&root);
        let bytes = fs::read(&path).unwrap();
        fs::write(&path, &bytes[..bytes.len() - 1]).unwrap();
        assert!(matches!(
            log.read_all(),
            Err(ObsError::MalformedRecord { message, .. })
                if message.contains("frame exceeds") || message.contains("truncated")
        ));
        fs::remove_dir_all(&root).ok();
    }
}
