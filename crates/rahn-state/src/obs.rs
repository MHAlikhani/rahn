// SPDX-License-Identifier: Apache-2.0

//! Deterministic observation model (ADR 0013).
//!
//! An [`Observation`] is a first-class record: `{ seq, time_ns, subject,
//! metric, value }`, canonically serialized and appended to an immutable
//! log. Timestamps are caller-supplied (no implicit clocks); `seq` is the
//! deterministic ingest position. No causal semantics live here (Stage 5).

pub mod log;

use std::fmt;

/// Target resource of an observation, addressed as in the topology model.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Subject {
    Node(String),
    Interface { node: String, iface: String },
}

impl Subject {
    pub fn parse(s: &str) -> Result<Self, ObsError> {
        let parts: Vec<&str> = s.split('/').collect();
        let bad = |id: &str| ObsError::InvalidSubject {
            subject: id.to_owned(),
        };
        match parts.as_slice() {
            [node] => {
                rahn_core::model::validate_id(node).map_err(|_| bad(s))?;
                Ok(Subject::Node(node.to_string()))
            }
            [node, iface] => {
                rahn_core::model::validate_id(node).map_err(|_| bad(s))?;
                rahn_core::model::validate_id(iface).map_err(|_| bad(s))?;
                Ok(Subject::Interface {
                    node: node.to_string(),
                    iface: iface.to_string(),
                })
            }
            _ => Err(ObsError::InvalidSubject {
                subject: s.to_owned(),
            }),
        }
    }

    /// Check the subject exists in the given network (association rule).
    pub fn exists_in(&self, net: &rahn_core::Network) -> bool {
        match self {
            Subject::Node(id) => net.node(id).is_some(),
            Subject::Interface { node, iface } => net
                .node(node)
                .map(|n| n.interface(iface).is_some())
                .unwrap_or(false),
        }
    }
}

impl fmt::Display for Subject {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Subject::Node(id) => write!(f, "node/{id}"),
            Subject::Interface { node, iface } => write!(f, "node/{node}/{iface}"),
        }
    }
}

/// Exact, float-free observation value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value {
    /// Monotonically non-decreasing count (by convention, not enforced).
    Counter(u64),
    /// Signed instantaneous reading.
    Gauge(i64),
    /// Discrete event text, bounded to 256 bytes of UTF-8.
    Event(String),
}

impl Value {
    const MAX_EVENT_BYTES: usize = 256;

    pub fn kind(&self) -> &'static str {
        match self {
            Value::Counter(_) => "counter",
            Value::Gauge(_) => "gauge",
            Value::Event(_) => "event",
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Counter(v) => write!(f, "counter:{v}"),
            Value::Gauge(v) => write!(f, "gauge:{v}"),
            Value::Event(v) => write!(f, "event:{v}"),
        }
    }
}

/// Structured rejection for observation ingest/parse.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ObsError {
    InvalidSubject {
        subject: String,
    },
    InvalidMetric {
        metric: String,
        reason: String,
    },
    EventTooLong {
        bytes: usize,
    },
    MalformedRecord {
        offset: usize,
        message: String,
    },
    /// Subject does not exist in the referenced state.
    UnknownSubject {
        subject: String,
    },
}

impl fmt::Display for ObsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ObsError::InvalidSubject { subject } => {
                write!(
                    f,
                    "invalid subject {subject:?} (expected node/<id> or node/<id>/<iface>)"
                )
            }
            ObsError::InvalidMetric { metric, reason } => {
                write!(f, "invalid metric name {metric:?}: {reason}")
            }
            ObsError::EventTooLong { bytes } => {
                write!(
                    f,
                    "event text is {bytes} bytes; limit is {}",
                    Value::MAX_EVENT_BYTES
                )
            }
            ObsError::MalformedRecord { offset, message } => {
                write!(
                    f,
                    "malformed observation record at byte {offset}: {message}"
                )
            }
            ObsError::UnknownSubject { subject } => {
                write!(
                    f,
                    "subject {subject} does not exist in the referenced state"
                )
            }
        }
    }
}

impl std::error::Error for ObsError {}

/// An immutable observation record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Observation {
    /// Ingest position (count of prior records). Deterministic, immutable.
    pub seq: u64,
    /// Caller-supplied nanoseconds since the UNIX epoch.
    pub time_ns: u64,
    /// State id (hex) this record was validated against (provenance).
    pub state_ref: String,
    pub subject: Subject,
    pub metric: String,
    pub value: Value,
}

impl Observation {
    /// Construct and validate a record. `state_ref` is the provenance hex.
    pub fn new(
        seq: u64,
        time_ns: u64,
        state_ref: String,
        subject: Subject,
        metric: &str,
        value: Value,
    ) -> Result<Self, ObsError> {
        rahn_core::validate_id(metric).map_err(|e| ObsError::InvalidMetric {
            metric: metric.to_owned(),
            reason: e.to_string(),
        })?;
        if let Value::Event(text) = &value {
            if text.len() > Value::MAX_EVENT_BYTES {
                return Err(ObsError::EventTooLong { bytes: text.len() });
            }
        }
        Ok(Observation {
            seq,
            time_ns,
            state_ref,
            subject,
            metric: metric.to_owned(),
            value,
        })
    }

    /// Canonical encoding (ADR 0003 rules: total order, no ambient data,
    /// no floats). Layout: seq, time_ns, state_ref, subject tag + parts,
    /// metric, value tag + payload.
    pub fn canonical_bytes(&self) -> Vec<u8> {
        let mut w = crate::canonical::Writer::new();
        w.u64(self.seq);
        w.u64(self.time_ns);
        w.string(&self.state_ref);
        match &self.subject {
            Subject::Node(id) => {
                w.u64(1);
                w.string(id);
            }
            Subject::Interface { node, iface } => {
                w.u64(2);
                w.string(node);
                w.string(iface);
            }
        }
        w.string(&self.metric);
        match &self.value {
            Value::Counter(v) => {
                w.u64(1);
                w.u64(*v);
            }
            Value::Gauge(v) => {
                w.u64(2);
                // Two's-complement round-trip: exact, no sign bit lost.
                w.u64(*v as u64);
            }
            Value::Event(text) => {
                w.u64(3);
                w.string(text);
            }
        }
        w.buf
    }

    /// Strict parse; malformed input is a hard error.
    pub fn parse(bytes: &[u8]) -> Result<Self, ObsError> {
        let mut r = crate::canonical::Reader { buf: bytes, pos: 0 };
        let err = |r: &crate::canonical::Reader, m: &str| ObsError::MalformedRecord {
            offset: r.pos,
            message: m.to_owned(),
        };
        let seq = r.u64().map_err(|e| err(&r, &e.message))?;
        let time_ns = r.u64().map_err(|e| err(&r, &e.message))?;
        let state_ref = r.string().map_err(|e| err(&r, &e.message))?;
        let subject = match r.u64().map_err(|e| err(&r, &e.message))? {
            1 => Subject::Node(r.string().map_err(|e| err(&r, &e.message))?),
            2 => {
                let node = r.string().map_err(|e| err(&r, &e.message))?;
                let iface = r.string().map_err(|e| err(&r, &e.message))?;
                Subject::Interface { node, iface }
            }
            t => return Err(err(&r, &format!("unknown subject tag {t}"))),
        };
        let metric = r.string().map_err(|e| err(&r, &e.message))?;
        let value = match r.u64().map_err(|e| err(&r, &e.message))? {
            1 => Value::Counter(r.u64().map_err(|e| err(&r, &e.message))?),
            2 => {
                let raw = r.u64().map_err(|e| err(&r, &e.message))?;
                // Two's-complement reinterpretation (exact round-trip).
                Value::Gauge(raw as i64)
            }
            3 => Value::Event(r.string().map_err(|e| err(&r, &e.message))?),
            t => return Err(err(&r, &format!("unknown value tag {t}"))),
        };
        if r.pos != bytes.len() {
            return Err(err(&r, "trailing bytes"));
        }
        Observation::new(seq, time_ns, state_ref, subject, &metric, value)
    }
}

/// Total ordering rule: `(time_ns, seq)` ascending.
pub fn order_key(o: &Observation) -> (u64, u64) {
    (o.time_ns, o.seq)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rec(seq: u64, time_ns: u64, metric: &str, value: Value) -> Observation {
        Observation::new(
            seq,
            time_ns,
            "aa".repeat(32),
            Subject::parse("web/eth0").unwrap(),
            metric,
            value,
        )
        .unwrap()
    }

    #[test]
    fn round_trip_exact() {
        for v in [
            Value::Counter(7),
            Value::Gauge(-3),
            Value::Event("up".into()),
        ] {
            let o = rec(1, 42, "latency_ns", v.clone());
            let b = o.canonical_bytes();
            assert_eq!(Observation::parse(&b).unwrap(), o);
        }
    }

    #[test]
    fn gauge_round_trips_negative() {
        let o = rec(0, 0, "queue_depth", Value::Gauge(i64::MIN));
        assert_eq!(Observation::parse(&o.canonical_bytes()).unwrap(), o);
    }

    #[test]
    fn trailing_bytes_rejected() {
        let mut b = rec(0, 0, "m", Value::Counter(1)).canonical_bytes();
        b.push(0);
        assert!(matches!(
            Observation::parse(&b),
            Err(ObsError::MalformedRecord { message, .. }) if message.contains("trailing")
        ));
    }

    #[test]
    fn unknown_tags_rejected() {
        let mut w = crate::canonical::Writer::new();
        w.u64(0);
        w.u64(0);
        w.string(&"a".repeat(64));
        w.u64(9);
        let bytes = w.buf;
        assert!(matches!(
            Observation::parse(&bytes),
            Err(ObsError::MalformedRecord { message, .. }) if message.contains("subject tag")
        ));
    }

    #[test]
    fn metric_and_subject_validation() {
        assert!(matches!(
            Subject::parse("a/b/c"),
            Err(ObsError::InvalidSubject { .. })
        ));
        assert!(Subject::parse("web").is_ok());
        assert!(Observation::new(
            0,
            0,
            "ab".repeat(32),
            Subject::parse("web").unwrap(),
            "has space",
            Value::Counter(1)
        )
        .is_err());
        assert!(matches!(
            Observation::new(
                0,
                0,
                "ab".repeat(32),
                Subject::parse("web").unwrap(),
                "ok",
                Value::Event("x".repeat(257))
            ),
            Err(ObsError::EventTooLong { bytes: 257 })
        ));
    }

    #[test]
    fn ordering_rule() {
        let a = rec(0, 10, "m", Value::Counter(1));
        let b = rec(1, 10, "m", Value::Counter(2));
        let c = rec(2, 9, "m", Value::Counter(3));
        let mut v = vec![a.clone(), b.clone(), c.clone()];
        v.sort_by_key(order_key);
        assert_eq!(v, vec![c, a, b], "(time_ns, seq) ascending");
    }

    #[test]
    fn subject_association() {
        let mut net = rahn_core::Network::empty();
        net.add_node("web", rahn_core::Metadata::new()).unwrap();
        net.add_interface("web", "eth0").unwrap();
        assert!(Subject::parse("web").unwrap().exists_in(&net));
        assert!(Subject::parse("web/eth0").unwrap().exists_in(&net));
        assert!(!Subject::parse("web/eth1").unwrap().exists_in(&net));
        assert!(!Subject::parse("ghost").unwrap().exists_in(&net));
    }
}
