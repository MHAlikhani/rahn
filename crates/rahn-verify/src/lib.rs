// SPDX-License-Identifier: Apache-2.0

//! Deterministic verification for RAHN: invariants, the constitution,
//! verification reports, and fail-closed semantic merge.

pub mod constitution;
pub mod invariants;
pub mod merge;

pub use constitution::{Constitution, ConstitutionError};
pub use invariants::{verify, InvariantReport, VerificationReport};
pub use merge::{merge, MergeConflict};
