// SPDX-License-Identifier: Apache-2.0

//! Canonical serialization, content-addressed state identity, explicit
//! transitions, semantic diff, and commit records for RAHN.

pub mod canonical;
pub mod commit;
pub mod diff;
pub mod history;
pub mod identity;
pub mod transition;

pub use canonical::{canonical_bytes, parse_canonical, CanonicalError, CANONICAL_FORMAT_VERSION};
pub use commit::{canonical_commit_bytes, CommitId, CommitRecord};
pub use diff::{diff, Diff};
pub use history::common_ancestor;
pub use identity::StateId;
pub use transition::{apply, Operation, TransitionError};
