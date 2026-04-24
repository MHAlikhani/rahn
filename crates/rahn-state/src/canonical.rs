// SPDX-License-Identifier: Apache-2.0

//! Canonical serialization (ADR 0003).
//!
//! One logical state MUST have exactly one byte sequence. Rules:
//! 1. a u16 little-endian format version tag precedes all content;
//! 2. collections are written in their total (sorted) order;
//! 3. all lengths are u64 little-endian; strings are UTF-8;
//! 4. no ambient information (no timestamps, hosts, pointers);
//! 5. no floating point anywhere;
//! 6. parsing is total and strict: unknown versions, malformed lengths,
//!    invalid UTF-8, duplicate keys, dangling link endpoints, unsorted
//!    content, and trailing bytes are hard errors — never a partial state.
//!
//! `canonical_bytes(x)` is idempotent: re-encoding a parsed value reproduces
//! the same bytes (round-trip property, tested below).

use rahn_core::{Link, Metadata, Network, State, VerificationSummary};

/// Version tag of the canonical state encoding. Bump only via ADR.
pub const CANONICAL_FORMAT_VERSION: u16 = 1;

/// Strict canonical parsing failure. Carries the offset for diagnosis.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonicalError {
    pub message: String,
    pub offset: usize,
}

impl std::fmt::Display for CanonicalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "canonical parse error at byte {}: {}",
            self.offset, self.message
        )
    }
}

impl std::error::Error for CanonicalError {}

pub(crate) struct Writer {
    pub(crate) buf: Vec<u8>,
}

impl Writer {
    pub(crate) fn new() -> Self {
        Self { buf: Vec::new() }
    }

    pub(crate) fn u16(&mut self, v: u16) {
        self.buf.extend_from_slice(&v.to_le_bytes());
    }

    pub(crate) fn u64(&mut self, v: u64) {
        self.buf.extend_from_slice(&v.to_le_bytes());
    }

    pub(crate) fn bytes(&mut self, v: &[u8]) {
        self.u64(v.len() as u64);
        self.buf.extend_from_slice(v);
    }

    pub(crate) fn string(&mut self, v: &str) {
        self.bytes(v.as_bytes());
    }

    pub(crate) fn metadata(&mut self, md: &Metadata) {
        self.u64(md.len() as u64);
        for (k, val) in md {
            self.string(k);
            self.string(val);
        }
    }

    pub(crate) fn network(&mut self, net: &Network) {
        self.u64(net.node_count() as u64);
        for node in net.iter_nodes() {
            self.string(&node.id);
            self.metadata(&node.metadata);
        }
        self.u64(net.link_count() as u64);
        for link in net.iter_links() {
            self.string(&link.a);
            self.string(&link.b);
            self.metadata(&link.metadata);
        }
    }
}

pub(crate) struct Reader<'a> {
    pub(crate) buf: &'a [u8],
    pub(crate) pos: usize,
}

impl<'a> Reader<'a> {
    fn take(&mut self, n: usize) -> Result<&'a [u8], CanonicalError> {
        let end = self
            .pos
            .checked_add(n)
            .ok_or_else(|| self.err("length overflow"))?;
        if end > self.buf.len() {
            return Err(self.err("unexpected end of input"));
        }
        let out = &self.buf[self.pos..end];
        self.pos = end;
        Ok(out)
    }

    pub(crate) fn err(&self, message: &str) -> CanonicalError {
        CanonicalError {
            message: message.to_owned(),
            offset: self.pos,
        }
    }

    pub(crate) fn u16(&mut self) -> Result<u16, CanonicalError> {
        let b = self.take(2)?;
        Ok(u16::from_le_bytes([b[0], b[1]]))
    }

    pub(crate) fn u64(&mut self) -> Result<u64, CanonicalError> {
        let b = self.take(8)?;
        let mut arr = [0u8; 8];
        arr.copy_from_slice(b);
        Ok(u64::from_le_bytes(arr))
    }

    /// Read a length-prefixed byte string, refusing absurd lengths.
    pub(crate) fn bytes(&mut self) -> Result<&'a [u8], CanonicalError> {
        let len = self.u64()? as usize;
        if len > self.buf.len() - self.pos {
            // Length claims more than the remaining buffer: malformed.
            return Err(self.err("length prefix exceeds remaining input"));
        }
        self.take(len)
    }

    pub(crate) fn string(&mut self) -> Result<String, CanonicalError> {
        let b = self.bytes()?;
        String::from_utf8(b.to_vec()).map_err(|_| self.err("invalid UTF-8 in string"))
    }

    pub(crate) fn metadata(&mut self) -> Result<Metadata, CanonicalError> {
        let count = self.u64()? as usize;
        let mut md = Metadata::new();
        let mut last_key: Option<String> = None;
        for _ in 0..count {
            let k = self.string()?;
            let v = self.string()?;
            if let Some(prev) = &last_key {
                if k <= *prev {
                    return Err(self.err("metadata keys are not in strictly ascending order"));
                }
            }
            last_key = Some(k.clone());
            if md.insert(k, v).is_some() {
                return Err(self.err("duplicate metadata key"));
            }
        }
        Ok(md)
    }

    pub(crate) fn network(&mut self) -> Result<Network, CanonicalError> {
        let node_count = self.u64()? as usize;
        if node_count > self.buf.len() {
            return Err(self.err("node count exceeds input size"));
        }
        let mut net = Network::empty();
        let mut last_id: Option<String> = None;
        for _ in 0..node_count {
            let id = self.string()?;
            if let Some(prev) = &last_id {
                if id <= *prev {
                    return Err(self.err("node ids are not in strictly ascending order"));
                }
            }
            last_id = Some(id.clone());
            let metadata = self.metadata()?;
            net.add_node(&id, metadata)
                .map_err(|e| self.err(&e.to_string()))?;
        }
        let link_count = self.u64()? as usize;
        if link_count > self.buf.len() {
            return Err(self.err("link count exceeds input size"));
        }
        let mut last_key: Option<(String, String)> = None;
        for _ in 0..link_count {
            let a = self.string()?;
            let b = self.string()?;
            if a >= b {
                return Err(self.err("link endpoints are not normalized (a < b required)"));
            }
            if let Some(prev) = &last_key {
                if (a.clone(), b.clone()) <= *prev {
                    return Err(self.err("links are not in strictly ascending order"));
                }
            }
            last_key = Some((a.clone(), b.clone()));
            let metadata = self.metadata()?;
            // Strictness: canonical states enumerate every node before any
            // link that references it. Auto-creating endpoints would mask
            // corruption, so a missing endpoint is a hard error.
            if net.node(&a).is_none() {
                return Err(self.err(&format!("dangling link endpoint {a:?}")));
            }
            if net.node(&b).is_none() {
                return Err(self.err(&format!("dangling link endpoint {b:?}")));
            }
            let link = Link {
                a: a.clone(),
                b: b.clone(),
                metadata,
            };
            net.add_existing_link(link)
                .map_err(|e| self.err(&e.to_string()))?;
        }
        Ok(net)
    }
}

/// Encode a state canonically (ADR 0003).
pub fn canonical_bytes(state: &State) -> Vec<u8> {
    let mut w = Writer::new();
    w.u16(CANONICAL_FORMAT_VERSION);
    w.network(&state.network);
    w.buf
}

/// Strictly parse canonical state bytes. Malformed input is a hard error.
pub fn parse_canonical(bytes: &[u8]) -> Result<State, CanonicalError> {
    let mut r = Reader { buf: bytes, pos: 0 };
    let version = r.u16()?;
    if version != CANONICAL_FORMAT_VERSION {
        return Err(r.err(&format!("unsupported canonical format version {version}")));
    }
    let network = r.network()?;
    if r.pos != bytes.len() {
        return Err(r.err("trailing bytes after canonical state"));
    }
    Ok(State { network })
}

/// Canonical encoding of a verification summary (used inside commit records).
pub(crate) fn write_verification_summary(w: &mut Writer, v: &VerificationSummary) {
    w.u64(if v.passed { 1 } else { 0 });
    w.u64(v.failed_invariants.len() as u64);
    for id in &v.failed_invariants {
        w.string(id);
    }
}

pub(crate) fn read_verification_summary(
    r: &mut Reader,
) -> Result<VerificationSummary, CanonicalError> {
    let passed = r.u64()? == 1;
    let count = r.u64()? as usize;
    if count > r.buf.len() {
        return Err(r.err("failed-invariant count exceeds input size"));
    }
    let mut failed = Vec::with_capacity(count);
    for _ in 0..count {
        failed.push(r.string()?);
    }
    Ok(VerificationSummary {
        passed,
        failed_invariants: failed,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use rahn_core::ModelError;

    fn sample_state() -> State {
        let mut net = Network::empty();
        net.add_node("db-1", Metadata::new()).unwrap();
        let mut md = Metadata::new();
        md.insert("role".to_owned(), "api".to_owned());
        md.insert("zone".to_owned(), "a".to_owned());
        net.add_node("api-1", md).unwrap();
        net.add_link("api-1", "db-1").unwrap();
        State { network: net }
    }

    #[test]
    fn round_trip_is_exact() {
        let s = sample_state();
        let bytes = canonical_bytes(&s);
        let parsed = parse_canonical(&bytes).unwrap();
        assert_eq!(parsed, s);
        // Idempotence: re-encoding the parsed value reproduces the bytes.
        assert_eq!(canonical_bytes(&parsed), bytes);
    }

    #[test]
    fn empty_state_round_trips() {
        let s = State::empty();
        let bytes = canonical_bytes(&s);
        assert_eq!(parse_canonical(&bytes).unwrap(), s);
    }

    #[test]
    fn trailing_bytes_are_rejected() {
        let mut bytes = canonical_bytes(&sample_state());
        bytes.push(0);
        let err = parse_canonical(&bytes).unwrap_err();
        assert!(err.message.contains("trailing"));
    }

    #[test]
    fn unknown_version_is_rejected() {
        let mut bytes = canonical_bytes(&sample_state());
        bytes[0] = 0xFF;
        assert!(parse_canonical(&bytes).is_err());
    }

    #[test]
    fn truncated_input_is_rejected() {
        let bytes = canonical_bytes(&sample_state());
        for cut in [0, 1, 2, 5, 9, 17, bytes.len() - 1] {
            assert!(
                parse_canonical(&bytes[..cut]).is_err(),
                "cut at {cut} must fail"
            );
        }
    }

    #[test]
    fn unsorted_links_are_rejected() {
        // Hand-build a state whose links were not written in sorted order by
        // corrupting the writer: encode a state, then rebuild with swapped
        // link order through the raw encoder is not reachable via the public
        // API (ordering is enforced upstream), so test the reader directly
        // with a forged buffer: two links out of order.
        let mut w = Writer::new();
        w.u16(CANONICAL_FORMAT_VERSION);
        w.u64(2); // nodes: a, c
        w.string("a");
        w.metadata(&Metadata::new());
        w.string("c");
        w.metadata(&Metadata::new());
        w.u64(2); // links out of order: c-a first, then b-c? use (a,c) then (a,b)
        w.string("a");
        w.string("c");
        w.metadata(&Metadata::new());
        w.string("a");
        w.string("b");
        w.metadata(&Metadata::new());
        let err = parse_canonical(&w.buf).unwrap_err();
        assert!(err.message.contains("ascending"), "unexpected: {err}");
    }

    #[test]
    fn dangling_link_endpoint_is_rejected() {
        let mut w = Writer::new();
        w.u16(CANONICAL_FORMAT_VERSION);
        w.u64(1); // nodes: a only
        w.string("a");
        w.metadata(&Metadata::new());
        w.u64(1); // link a-b, but b does not exist
        w.string("a");
        w.string("b");
        w.metadata(&Metadata::new());
        let err = parse_canonical(&w.buf).unwrap_err();
        assert!(
            err.message.contains("dangling") || err.message.contains("exist"),
            "unexpected: {err}"
        );
    }

    #[test]
    fn unnormalized_endpoints_are_rejected() {
        let mut w = Writer::new();
        w.u16(CANONICAL_FORMAT_VERSION);
        w.u64(2);
        w.string("a");
        w.metadata(&Metadata::new());
        w.string("b");
        w.metadata(&Metadata::new());
        w.u64(1);
        w.string("b"); // reversed: a<b required
        w.string("a");
        w.metadata(&Metadata::new());
        let err = parse_canonical(&w.buf).unwrap_err();
        assert!(err.message.contains("normalized"), "unexpected: {err}");
    }

    #[test]
    fn self_loop_is_unrepresentable() {
        let mut net = Network::empty();
        net.add_node("a", Metadata::new()).unwrap();
        assert_eq!(
            net.add_link("a", "a"),
            Err(ModelError::SelfLoop { node: "a".into() })
        );
    }
}
