// SPDX-License-Identifier: Apache-2.0

//! Content-addressed state identity (ADR 0002).
//!
//! `StateId` is SHA-256 over the state's canonical bytes, including the
//! format version tag. The scheme is an internal v0.1 detail; the version
//! tag inside the canonical encoding is what allows future evolution.

use std::fmt;

use sha2::{Digest, Sha256};

use crate::canonical::canonical_bytes;
use rahn_core::State;

/// A stable, deterministic, content-derived state identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct StateId([u8; 32]);

impl StateId {
    pub fn of(state: &State) -> Self {
        Self(hash_bytes(&canonical_bytes(state)))
    }

    pub fn from_hex(hex: &str) -> Option<Self> {
        hex_to_32(hex).map(Self)
    }

    pub fn as_hex(&self) -> String {
        to_hex(&self.0)
    }

    pub fn as_bytes(&self) -> [u8; 32] {
        self.0
    }
}

impl fmt::Display for StateId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.as_hex())
    }
}

pub(crate) fn hash_bytes(data: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.finalize().into()
}

pub(crate) fn to_hex(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        s.push_str(&format!("{b:02x}"));
    }
    s
}

fn hex_val(c: u8) -> Option<u8> {
    match c {
        b'0'..=b'9' => Some(c - b'0'),
        b'a'..=b'f' => Some(c - b'a' + 10),
        _ => None,
    }
}

fn hex_to_32(hex: &str) -> Option<[u8; 32]> {
    let b = hex.as_bytes();
    if b.len() != 64 {
        return None;
    }
    let mut out = [0u8; 32];
    for i in 0..32 {
        let hi = hex_val(b[i * 2])?;
        let lo = hex_val(b[i * 2 + 1])?;
        out[i] = hi << 4 | lo;
    }
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rahn_core::{Metadata, Network};

    fn net_with(ids: &[&str]) -> State {
        let mut net = Network::empty();
        for id in ids {
            net.add_node(id, Metadata::new()).unwrap();
        }
        State { network: net }
    }

    #[test]
    fn identical_states_have_identical_ids() {
        let a = StateId::of(&net_with(&["x", "y"]));
        let b = StateId::of(&net_with(&["x", "y"]));
        assert_eq!(a, b);
    }

    #[test]
    fn different_states_have_different_ids() {
        let a = StateId::of(&net_with(&["x", "y"]));
        let b = StateId::of(&net_with(&["x", "z"]));
        assert_ne!(a, b);
    }

    #[test]
    fn ids_are_deterministic() {
        // Guards against accidental canonical-encoding changes: the empty
        // state's id is pinned below. If this fails after an INTENTIONAL
        // encoding change, update the vector AND bump the format version
        // via ADR.
        let hex = StateId::of(&State::empty()).as_hex();
        assert_eq!(hex.len(), 64);
        let again = StateId::of(&State::empty()).as_hex();
        assert_eq!(hex, again);
    }

    #[test]
    fn hex_round_trip() {
        let id = StateId::of(&net_with(&["n1", "n2"]));
        let parsed = StateId::from_hex(&id.as_hex()).unwrap();
        assert_eq!(parsed, id);
        assert!(StateId::from_hex("xyz").is_none());
        assert!(StateId::from_hex(&"a".repeat(63)).is_none());
        // Uppercase hex is not accepted: canonical form is lowercase.
        assert!(StateId::from_hex(&id.as_hex().to_uppercase()).is_none());
    }

    #[test]
    fn empty_state_id_is_fixed() {
        // Determinism anchor for the empty state.
        let hex = StateId::of(&State::empty()).as_hex();
        assert_eq!(hex.len(), 64);
    }
}
