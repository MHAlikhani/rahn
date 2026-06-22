// SPDX-License-Identifier: Apache-2.0

//! Append-only causal edge log (ADR 0014); same framing rules as the
//! observation log — u64 length + canonical bytes, loud corruption
//! refusal, records immutable once written.

use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use crate::causal::{CausalEdge, CausalError, CausalGraph};

pub struct CausalLog {
    path: PathBuf,
}

impl CausalLog {
    pub fn path(root: &Path) -> PathBuf {
        root.join("causal.log")
    }

    pub fn open(root: &Path) -> Result<Self, CausalError> {
        let path = Self::path(root);
        if !path.exists() {
            fs::File::create(&path).map_err(io_err)?;
        }
        Ok(CausalLog { path })
    }

    /// Read all edges and rebuild the DAG (re-asserting acyclicity).
    pub fn load(&self) -> Result<CausalGraph, CausalError> {
        let bytes = fs::read(&self.path).map_err(io_err)?;
        let mut edges = Vec::new();
        let mut pos = 0usize;
        while pos < bytes.len() {
            if bytes.len() - pos < 8 {
                return Err(CausalError::MalformedRecord {
                    offset: pos,
                    message: "truncated frame length".into(),
                });
            }
            let len = u64::from_le_bytes(bytes[pos..pos + 8].try_into().unwrap()) as usize;
            pos += 8;
            if len > bytes.len() - pos {
                return Err(CausalError::MalformedRecord {
                    offset: pos,
                    message: "frame exceeds file size".into(),
                });
            }
            edges.push(CausalEdge::parse(&bytes[pos..pos + len])?);
            pos += len;
        }
        Ok(CausalGraph::new(edges))
    }

    /// Append one edge. The caller MUST have validated it against the
    /// current graph (anchors + DAG) before appending.
    pub fn append(&self, e: &CausalEdge) -> Result<(), CausalError> {
        let payload = e.canonical_bytes();
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
}

fn io_err(e: io::Error) -> CausalError {
    CausalError::Io(e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::causal::{Anchor, Status};

    fn temp_root(tag: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!(
            "rahn-causal-{tag}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&d).unwrap();
        d
    }

    fn edge(seq: u64, from: u64, to: u64) -> CausalEdge {
        CausalEdge::new(
            seq,
            Anchor::Observation(from),
            Anchor::Observation(to),
            Status::Hypothesis,
            "note".into(),
        )
        .unwrap()
    }

    #[test]
    fn append_load_round_trip() {
        let root = temp_root("rt");
        let log = CausalLog::open(&root).unwrap();
        log.append(&edge(0, 1, 2)).unwrap();
        log.append(&edge(1, 2, 3)).unwrap();
        let g = log.load().unwrap();
        assert_eq!(g.edges().len(), 2);
        assert_eq!(g.incident_around(&Anchor::Observation(2)).len(), 3);
        fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn corruption_is_loud() {
        let root = temp_root("bad");
        let log = CausalLog::open(&root).unwrap();
        log.append(&edge(0, 1, 2)).unwrap();
        let path = CausalLog::path(&root);
        let bytes = fs::read(&path).unwrap();
        fs::write(&path, &bytes[..bytes.len() - 1]).unwrap();
        assert!(log.load().is_err());
        fs::remove_dir_all(&root).ok();
    }
}
