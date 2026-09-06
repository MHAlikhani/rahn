<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# RAHN Third-Party Component Inventory

Authoritative record of externally sourced components used by or included in RAHN. See [licensing.md](licensing.md) for the policy.

**Current status: one dependency.** `sha2` (SHA-256 for content-addressed state identity and object integrity; ADR 0002). License verified from the crate metadata at time of addition: `Apache-2.0 OR MIT` — compatible with the project's Apache-2.0 default; neither alternative imposes obligations beyond attribution.

Every dependency must be recorded here **before or at the time it is added**, with:

- component name and version
- source URL
- license name and SPDX identifier
- whether it is vendored / dynamically linked / statically linked
- whether source modifications were made
- required notices and where they are preserved
- compatibility notes

Rules enforced by review (from docs/licensing.md):

- Cargo availability does not imply Apache-2.0 compatibility; verify each license.
- MIT, BSD-2-Clause, BSD-3-Clause, Apache-2.0 are generally acceptable; GPL/LGPL/AGPL/MPL/SSPL, proprietary, source-available, and custom licenses require explicit documented compatibility analysis.
- Copied or adapted external code must retain its license notices and attribution and be recorded here — never silently relicensed.
- Submodules and vendored directories retain upstream licensing and provenance.

## Registry

| Component | Version | Source | License (SPDX) | Vendored | Linking | Modified | Notices | Notes |
|---|---|---|---|---|---|---|---|---|
| sha2 | 0.10.9 | https://github.com/RustCrypto/hashes | Apache-2.0 OR MIT | No | Static (Rust crate) | No | None required beyond license text | SHA-256; used by rahn-state (identity) and rahn-store (object integrity) |
| block-buffer / crypto-common / digest / generic-array / typenum / cfg-if / cpufeatures / version_check / libc | (transitive of sha2) | https://github.com/RustCrypto (block-buffer, digest, crypto-common); github.com/RustCrypto/traits (generic-array/typenum); crates.io (cfg-if, cpufeatures, libc, version_check) | Apache-2.0 OR MIT (RustCrypto stack); Apache-2.0 OR MIT (cfg-if); Apache-2.0 OR MIT (cpufeatures, libc); Apache-2.0 OR MIT (version_check) | No | Static (Rust crates) | No | None required beyond license texts | Transitive closure of the single direct dependency; verified via Cargo.lock at v1.0.0 audit |
