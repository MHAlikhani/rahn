// SPDX-License-Identifier: Apache-2.0

//! Commit records (ADR 0005): immutable, canonical, content-addressed.
//!
//! A commit record captures: the state it introduces, its parent(s), the
//! operations that produced the state, the commit message, and the
//! verification summary at commit time. The commit's own identity is the
//! SHA-256 of its canonical encoding.

use crate::canonical::{
    read_verification_summary, write_verification_summary, CanonicalError, Writer,
};
use crate::identity::{hash_bytes, to_hex, StateId};
use crate::transition::Operation;
use rahn_core::VerificationSummary;

/// A commit identity: SHA-256 over the canonical commit encoding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CommitId(pub(crate) [u8; 32]);

impl CommitId {
    pub fn from_hex(hex: &str) -> Option<Self> {
        StateId::from_hex(hex).map(|s| Self(s.as_bytes()))
    }

    pub fn as_hex(&self) -> String {
        to_hex(&self.0)
    }
}

impl std::fmt::Display for CommitId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.as_hex())
    }
}

/// Immutable record of one commit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommitRecord {
    /// The state this commit introduces.
    pub state_id: StateId,
    /// Parent commit ids. One parent for a normal commit; two for a merge.
    /// Empty only for the root commit.
    pub parents: Vec<CommitId>,
    /// Operations (in application order) that produced the state from the
    /// first parent. Empty for root and merge commits (merges record their
    /// parents; re-deriving their op set is a Stage 2 concern).
    pub operations: Vec<Operation>,
    pub message: String,
    /// Verification outcome at commit time. A commit with `passed == false`
    /// MUST NOT exist; the field is recorded so integrity checks can prove
    /// the gate was evaluated.
    pub verification: VerificationSummary,
}

/// Canonical encoding of a commit record (deterministic; hashed for id).
pub fn canonical_commit_bytes(rec: &CommitRecord) -> Vec<u8> {
    let mut w = Writer::new();
    w.bytes(&rec.state_id.as_bytes());
    w.u64(rec.parents.len() as u64);
    for p in &rec.parents {
        w.bytes(&p.0);
    }
    w.u64(rec.operations.len() as u64);
    for op in &rec.operations {
        write_operation(&mut w, op);
    }
    w.string(&rec.message);
    write_verification_summary(&mut w, &rec.verification);
    w.buf
}

pub(crate) fn write_operation(w: &mut Writer, op: &Operation) {
    match op {
        Operation::AddNode { id, metadata } => {
            w.u64(1);
            w.string(id);
            w.metadata(metadata);
        }
        Operation::RemoveNode { id } => {
            w.u64(2);
            w.string(id);
        }
        Operation::AddLink { a, b } => {
            w.u64(3);
            w.string(a);
            w.string(b);
        }
        Operation::RemoveLink { a, b } => {
            w.u64(4);
            w.string(a);
            w.string(b);
        }
    }
}

pub(crate) fn commit_id_of_bytes(bytes: &[u8]) -> CommitId {
    CommitId(hash_bytes(bytes))
}

impl CommitRecord {
    /// The commit's content-derived identity.
    pub fn id(&self) -> CommitId {
        commit_id_of_bytes(&canonical_commit_bytes(self))
    }

    /// Strictly parse a commit record from canonical bytes.
    pub fn parse(bytes: &[u8]) -> Result<Self, CanonicalError> {
        let mut r = crate::canonical::Reader { buf: bytes, pos: 0 };
        let state_bytes = r.bytes()?;
        let state_id =
            StateId::from_hex(&to_hex(state_bytes)).ok_or_else(|| r.err("invalid state id"))?;
        let parent_count = r.u64()? as usize;
        if parent_count > 2 {
            return Err(r.err("commit may have at most two parents"));
        }
        let mut parents = Vec::with_capacity(parent_count);
        for _ in 0..parent_count {
            let p = r.bytes()?;
            if p.len() != 32 {
                return Err(r.err("parent id must be 32 bytes"));
            }
            let mut arr = [0u8; 32];
            arr.copy_from_slice(p);
            parents.push(CommitId(arr));
        }
        let op_count = r.u64()? as usize;
        if op_count > bytes.len() {
            return Err(r.err("operation count exceeds input size"));
        }
        let mut operations = Vec::with_capacity(op_count);
        for _ in 0..op_count {
            operations.push(read_operation(&mut r)?);
        }
        let message = r.string()?;
        let verification = read_verification_summary(&mut r)?;
        if r.pos != bytes.len() {
            return Err(r.err("trailing bytes after commit record"));
        }
        Ok(CommitRecord {
            state_id,
            parents,
            operations,
            message,
            verification,
        })
    }
}

fn read_operation(r: &mut crate::canonical::Reader) -> Result<Operation, CanonicalError> {
    let tag = r.u64()?;
    match tag {
        1 => {
            let id = r.string()?;
            let metadata = r.metadata()?;
            Ok(Operation::AddNode { id, metadata })
        }
        2 => Ok(Operation::RemoveNode { id: r.string()? }),
        3 => Ok(Operation::AddLink {
            a: r.string()?,
            b: r.string()?,
        }),
        4 => Ok(Operation::RemoveLink {
            a: r.string()?,
            b: r.string()?,
        }),
        _ => Err(r.err(&format!("unknown operation tag {tag}"))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::identity::StateId;
    use rahn_core::{Metadata, Network};

    fn sample() -> CommitRecord {
        let mut net = Network::empty();
        net.add_node("a", Metadata::new()).unwrap();
        CommitRecord {
            state_id: StateId::of(&rahn_core::State { network: net }),
            parents: vec![],
            operations: vec![Operation::AddNode {
                id: "a".into(),
                metadata: Metadata::new(),
            }],
            message: "init".into(),
            verification: VerificationSummary::passed(),
        }
    }

    #[test]
    fn commit_round_trips() {
        let rec = sample();
        let bytes = canonical_commit_bytes(&rec);
        let parsed = CommitRecord::parse(&bytes).unwrap();
        assert_eq!(parsed, rec);
        assert_eq!(parsed.id(), rec.id());
    }

    #[test]
    fn merge_commit_round_trips() {
        let mut rec = sample();
        rec.parents = vec![rec.id(), rec.id()];
        rec.message = "merge b".into();
        let parsed = CommitRecord::parse(&canonical_commit_bytes(&rec)).unwrap();
        assert_eq!(parsed.parents.len(), 2);
    }

    #[test]
    fn three_parents_rejected() {
        let mut rec = sample();
        let id = rec.id();
        rec.parents = vec![id, id, id];
        assert!(CommitRecord::parse(&canonical_commit_bytes(&rec)).is_err());
    }

    #[test]
    fn trailing_bytes_rejected() {
        let mut bytes = canonical_commit_bytes(&sample());
        bytes.push(0);
        assert!(CommitRecord::parse(&bytes).is_err());
    }

    #[test]
    fn unverified_commit_is_representable_for_audit_only() {
        // The type can represent a failed summary (for integrity auditing);
        // creating one at commit time is prevented at the CLI/flow level.
        let mut rec = sample();
        rec.verification = VerificationSummary::failed(vec!["connectivity".into()]);
        assert!(!rec.verification.passed);
    }
}
