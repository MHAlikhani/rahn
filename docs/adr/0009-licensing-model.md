<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# ADR 0009: Licensing Model — Apache-2.0 Code / CC-BY-4.0 Documentation

- Status: Accepted
- Date: 2026-04-15
- Deciders: project licensing policy (charter: licensing & IP policy)

## Context

RAHN is intended for long-term adoption by individual developers, researchers, infrastructure teams, operators, cloud providers, hardware vendors, and commercial companies. Licensing must therefore support open-source adoption, commercial adoption, contributor friendliness, patent clarity, and machine detectability, while cleanly separating software from documentation, specifications, trademarks, and third-party material. The repository was bootstrapped with a provisional MIT LICENSE before this policy existed.

## Decision

1. **Source code** (including tests, build scripts, CI configuration, schemas, executable examples): **Apache-2.0**.
2. **Documentation, research materials, specification prose, white paper**: **CC-BY-4.0**.
3. **Third-party code** retains its original license; provenance is tracked in `docs/third-party.md`.
4. **Trademark** treatment is separate from copyright; no registration is claimed; no artificial trademark restrictions on code.
5. **Contributions** are made under the same license as the contributed material; **no CLA**; no copyright transfer; DCO only if contributor volume justifies it later (via ADR).
6. **No dual licensing, no commercial licensing, no custom restrictions** at launch; relicensing only via public proposal + ADR.
7. Machine detectability: canonical `LICENSE` (unmodified Apache-2.0 text), `LICENSES/` with canonical texts, SPDX identifiers in headers where practical, valid SPDX identifiers in package metadata.
8. Accuracy rules: describe Apache-2.0 accurately (permissive license with explicit patent provisions; not "more free"; does not eliminate all patent risk); never invent copyright ownership, legal obligations, or patent claims; flag material legal decisions **[LEGAL REVIEW]**.

## Consequences

- The provisional MIT root LICENSE is replaced by canonical Apache-2.0 text; all bootstrap documentation gains CC-BY-4.0 SPDX headers; README and CONTRIBUTING state the split model.
- Documentation of software internals in docs/ is CC-BY-4.0, so code that ends up *in* docs must be marked Apache-2.0 or moved to source files.
- Every future dependency addition requires a license check and an entry in the third-party inventory.
- Apache-2.0's patent grant applies to the software; no broader patent claims are made on behalf of the project.

## Alternatives considered

- **MIT (provisional)**: simpler, but lacks explicit patent provisions — material for a project expecting vendor and infrastructure adoption.
- **Dual licensing (Apache-2.0 OR GPL-3.0)**: rejected; no concrete requirement, adds friction.
- **Copyleft (GPL/AGPL)**: would impede the embeddability/commercial-adoption goals and ecosystem growth.
- **Single license for everything (all Apache-2.0 or all CC-BY)**: conflates software and documentation semantics; CC-BY is unsuitable for code, Apache-2.0 alone leaves documentation attribution/normalization semantics mismatched.

## Conformance checklist (pre-first-release)

- [x] LICENSE at root with canonical Apache-2.0 text
- [x] Canonical license texts in LICENSES/
- [x] SPDX identifiers in documentation files
- [x] README states source-code and documentation licenses
- [x] CONTRIBUTING.md states contribution licensing
- [x] docs/licensing.md exists (this policy's parent document)
- [x] Third-party inventory exists (currently empty)
- [x] Package metadata uses valid SPDX expressions (`license.workspace = true` → `Apache-2.0`)
- [x] License scan / audit tooling (CI `license-check` job: canonical license files + SPDX headers)

## Implementation amendment (2026-04-24)

Initial copyright ownership confirmed: Mohammad Hossein Alikhani. The
[LEGAL REVIEW] placeholder in docs/licensing.md is resolved accordingly;
the crate `authors` field remains the neutral "The RAHN Project
Contributors" (attribution listing, not an ownership claim).

## References

- docs/licensing.md (authoritative policy); docs/third-party.md; README License section; CONTRIBUTING "Licensing of Contributions"
