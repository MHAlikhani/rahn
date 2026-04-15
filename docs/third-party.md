<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# RAHN Third-Party Component Inventory

Authoritative record of externally sourced components used by or included in RAHN. See [licensing.md](licensing.md) for the policy.

**Current status: empty.** The repository contains no third-party code, vendored source, submodules, generated artifacts, or binary dependencies. Documentation references (e.g., the Keep a Changelog format note in CHANGELOG.md, RFC 2119 keyword usage) are references to concepts, not code, and carry no licensing obligations.

When the implementation begins (Rust workspace), every dependency must be recorded here **before or at the time it is added**, with:

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
| *(none)* | | | | | | | | |
