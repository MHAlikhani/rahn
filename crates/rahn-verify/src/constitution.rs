// SPDX-License-Identifier: Apache-2.0

//! The constitution (ADR 0006): a named collection of requirements, stored
//! as data, governing all valid states.
//!
//! v0.1 requirement vocabulary (deliberately minimal, deterministic):
//!
//! ```text
//! require-connectivity <node-a> <node-b>
//! ```
//!
//! Structural invariants (referential integrity, link endpoints, duplicate
//! links, self-loops) always apply and are not listed in the file.
//! A candidate state that violates any requirement MUST be rejected before
//! execution.

use std::fmt;

/// A parsed constitution.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Constitution {
    /// Ordered pairs `(from, to)` that must be connected (node level).
    pub connectivity_requirements: Vec<(String, String)>,
    /// Ordered pairs `(from, to)` that must NOT be connected (isolation).
    pub prohibited_connectivity: Vec<(String, String)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConstitutionError {
    pub line: usize,
    pub message: String,
}

impl fmt::Display for ConstitutionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "constitution line {}: {}", self.line, self.message)
    }
}

impl std::error::Error for ConstitutionError {}

impl Constitution {
    /// Parse the constitution text format: one requirement per line,
    /// `#` comments and blank lines allowed. Parsing is strict.
    pub fn parse(text: &str) -> Result<Constitution, ConstitutionError> {
        let mut c = Constitution::default();
        for (idx, raw) in text.lines().enumerate() {
            let line_no = idx + 1;
            let line = raw.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let mut parts = line.split_whitespace();
            let keyword = parts.next().unwrap_or_default();
            match keyword {
                "require-connectivity" => {
                    let a = parts.next().ok_or_else(|| ConstitutionError {
                        line: line_no,
                        message: "require-connectivity needs two node identifiers".into(),
                    })?;
                    let b = parts.next().ok_or_else(|| ConstitutionError {
                        line: line_no,
                        message: "require-connectivity needs two node identifiers".into(),
                    })?;
                    if parts.next().is_some() {
                        return Err(ConstitutionError {
                            line: line_no,
                            message: "too many arguments".into(),
                        });
                    }
                    rahn_core::model::validate_id(a).map_err(|e| ConstitutionError {
                        line: line_no,
                        message: e.to_string(),
                    })?;
                    rahn_core::model::validate_id(b).map_err(|e| ConstitutionError {
                        line: line_no,
                        message: e.to_string(),
                    })?;
                    c.connectivity_requirements
                        .push((a.to_owned(), b.to_owned()));
                }
                "prohibit-connectivity" => {
                    let a = parts.next().ok_or_else(|| ConstitutionError {
                        line: line_no,
                        message: "prohibit-connectivity needs two node identifiers".into(),
                    })?;
                    let b = parts.next().ok_or_else(|| ConstitutionError {
                        line: line_no,
                        message: "prohibit-connectivity needs two node identifiers".into(),
                    })?;
                    if parts.next().is_some() {
                        return Err(ConstitutionError {
                            line: line_no,
                            message: "too many arguments".into(),
                        });
                    }
                    rahn_core::model::validate_id(a).map_err(|e| ConstitutionError {
                        line: line_no,
                        message: e.to_string(),
                    })?;
                    rahn_core::model::validate_id(b).map_err(|e| ConstitutionError {
                        line: line_no,
                        message: e.to_string(),
                    })?;
                    c.prohibited_connectivity.push((a.to_owned(), b.to_owned()));
                }
                other => {
                    return Err(ConstitutionError {
                        line: line_no,
                        message: format!("unknown requirement {other:?}"),
                    })
                }
            }
        }
        Ok(c)
    }

    /// Render back to the text format (round-trip safe).
    pub fn to_text(&self) -> String {
        let mut out = String::new();
        out.push_str("# RAHN constitution\n");
        for (a, b) in &self.connectivity_requirements {
            out.push_str(&format!("require-connectivity {a} {b}\n"));
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_and_round_trip() {
        let text = "# comment\n\nrequire-connectivity api db\nrequire-connectivity api lb\n";
        let c = Constitution::parse(text).unwrap();
        assert_eq!(c.connectivity_requirements.len(), 2);
        assert_eq!(
            c.to_text(),
            "# RAHN constitution\nrequire-connectivity api db\nrequire-connectivity api lb\n"
        );
        assert_eq!(Constitution::parse(&c.to_text()).unwrap(), c);
    }

    #[test]
    fn unknown_keyword_rejected() {
        let err = Constitution::parse("require-magic a b").unwrap_err();
        assert!(err.message.contains("unknown requirement"));
    }

    #[test]
    fn missing_argument_rejected() {
        assert!(Constitution::parse("require-connectivity a").is_err());
    }

    #[test]
    fn extra_argument_rejected() {
        assert!(Constitution::parse("require-connectivity a b c").is_err());
    }

    #[test]
    fn invalid_id_rejected() {
        assert!(Constitution::parse("require-connectivity a 'bad id'").is_err());
    }
}
