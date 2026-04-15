# RAHN Licensing Policy

**Authoritative project licensing policy.** This document explains the licensing model; it does not modify any license text. It is not legal advice. Where a decision has material legal consequences it is marked **[LEGAL REVIEW]**.

Decisions are recorded in [adr/0009-licensing-model.md](adr/0009-licensing-model.md).

## Summary

| Material | License | SPDX |
|---|---|---|
| Source code (including tests, build scripts, CI configuration, schemas, executable examples) | Apache License 2.0 | `Apache-2.0` |
| Documentation, research notes, specification prose, white paper | Creative Commons Attribution 4.0 International | `CC-BY-4.0` |
| Experimental / generated files | Inherit the applicable parent artifact unless explicitly marked otherwise | — |
| Third-party code | Retains its original license | — |
| Trademarks (name, logo) | Not licensed by the software or documentation licenses | — |
| Contributions | Same license as the material contributed to | — |

The software implementation is licensed under Apache License 2.0. RAHN documentation and research materials are licensed under CC BY 4.0 unless otherwise stated. Third-party components remain under their respective licenses.

## Why Apache-2.0 for source

Apache-2.0 is a permissive open-source software license with explicit patent provisions. It was chosen because RAHN aims to be broadly reusable, embeddable, commercially usable, and compatible with large-scale infrastructure ecosystems, with explicit patent clarity for a long-lived project.

Accuracy rules: do not describe Apache-2.0 as "more free" than other licenses; do not claim it eliminates all patent risk; do not make broader patent claims than the license itself provides.

## Software vs. documentation boundary

The two licenses apply to different classes of material. Examples:

| Path | License |
|---|---|
| `crates/**` (future Rust source) | Apache-2.0 |
| `examples/*.rs`, `tests/**`, `scripts/*`, CI configs | Apache-2.0 |
| `README.md`, `ARCHITECTURE.md`, `DESIGN.md`, `docs/**` prose | CC-BY-4.0 |
| `docs/spec/**` specification prose | CC-BY-4.0 |
| `docs/whitepaper/RAHN-Whitepaper.md` | CC-BY-4.0 |
| Code snippets inside documentation | Apache-2.0 (marked where practical) |

Rules:

- The documentation license MUST NOT be interpreted as applying to source code, executable snippets, vendored software, binary artifacts, external material, or trademarks.
- When a Markdown file contains substantial executable code plus prose, the prose is CC-BY-4.0 and code examples indicate Apache-2.0 where practical. If ambiguity is unavoidable, move code examples into dedicated source files.
- Specifications must distinguish normative text, explanatory prose, reference implementation, examples, and test vectors. Specification prose defaults to CC-BY-4.0; reference implementations and example code remain Apache-2.0. If RAHN ever becomes an external standards candidate, revisit specification licensing as its own decision.

## SPDX policy

- Use valid SPDX identifiers: `Apache-2.0`, `CC-BY-4.0`. Never informal names; never invented identifiers.
- Source headers: `// SPDX-License-Identifier: Apache-2.0` (Rust, C, C++), `#` comment form (shell, YAML, TOML).
- Software-documentation Markdown: `<!-- SPDX-License-Identifier: CC-BY-4.0 -->` at the top of the file.
- Do not add SPDX headers where they would harm readability or violate a generated-file convention.
- Multi-license files use SPDX expressions (e.g., `Apache-2.0 AND MIT`) only when legally accurate.
- No per-file copyright years; centralize licensing documentation instead. Never bump copyright years automatically.

## Root license files

- `LICENSE` — canonical, unmodified Apache License 2.0 text. Never shortened, rewritten, or appended to.
- `LICENSES/Apache-2.0.txt`, `LICENSES/CC-BY-4.0.txt` — canonical texts for SPDX-conformance tooling.
- `NOTICE` — created **only** if legally or operationally meaningful attribution notices exist. Never a generic credits dump; never implies all contributors must be listed.
- `LICENSE-NOTES.md` — not currently needed; `docs/licensing.md` serves that role.

## Copyright ownership

- Contributors retain copyright in their contributions; the project does not claim contributor copyright.
- Copyright notices must be accurate. No entity (e.g., a "RAHN Foundation") may be named unless it actually owns the relevant rights. **[LEGAL REVIEW]** The initial copyright line is neutral ("The RAHN Project Contributors"); if/when an individual or entity is confirmed to hold initial rights, update the notice accordingly.
- As the community grows, keep ownership records accurate.

## Contributions

- Source-code contributions are made available under Apache-2.0.
- Documentation and other non-software contributions are made available under CC-BY-4.0, unless explicitly identified otherwise.
- **No CLA in the initial phase.** No copyright transfer is required. A DCO may be considered when contributor volume justifies it; a CLA only for concrete reasons (copyright aggregation, relicensing flexibility, legal risk, organizational governance). Any change must be recorded in an ADR.
- No dual licensing at launch; no commercial licensing unless a sustainable need, clear ownership, and community impact evaluation all exist (then via ADR).

## Third-party dependencies

See [third-party.md](third-party.md) for the inventory.

- Before adding any dependency, inspect its actual license — availability through Cargo does not imply Apache compatibility.
- MIT, BSD-2-Clause, BSD-3-Clause, Apache-2.0 are generally acceptable; perform explicit compatibility analysis for GPL/LGPL/AGPL/MPL/SSPL, proprietary, source-available, and custom licenses.
- Do not make legal compatibility claims without evidence. If a dependency introduces meaningful licensing uncertainty, stop and document the issue before adding it.
- Copied/adapted external code: identify source and license, determine compatibility, preserve notices and attribution, document provenance in the inventory, and record it there. Never remove upstream license notices; never relicense third-party code as Apache-2.0 without legal authority.
- Vendored code (future `vendor/`): retains upstream copyright, license, provenance, and required notices; always clearly separated from RAHN-original code.
- Git submodules: document their licenses, versions/commits, and source URLs; the parent repository's license does not apply to them.

## Generated code

For generated artifacts: identify the generator, its input, and its license; determine whether the output is copyrightable and what notices apply; document provenance. Add `SPDX-License-Identifier: Apache-2.0` to generated files only if the project has the legal right to license that output that way.

## Trademarks

Software licensing does not grant trademark rights. The RAHN name and logo are governed separately from copyright and are not licensed by Apache-2.0 or CC-BY-4.0. No trademark registration is claimed (none exists). A `TRADEMARKS.md` will be added when third-party branding matters; the future policy will distinguish use of RAHN software from use of the RAHN name/logo ("RAHN-compatible" under defined conditions vs. "Official RAHN" reserved; no implied endorsement). No artificial trademark restrictions may be imposed on code.

## Release, container, and SBOM policy

- Release artifacts must state: source code Apache-2.0; included documentation CC-BY-4.0 where applicable; third-party components retain their own licenses; bundled dependencies may carry additional notices. Required notices must never be silently omitted.
- Container images (future): component licensing is documented per component (RAHN software, base image, system libraries, package artifacts, external binaries); no universal license applies to an image.
- SBOM (SPDX or CycloneDX) is not required for v0.1 but the repository is structured so it can be added later.
- Automated license auditing (SPDX scanning, dependency license checks, SBOM generation, drift detection) is added when the repository is large enough to need it, following detect → classify → review → resolve rather than blind blocking.

## Relicensing

The license will not be changed casually, and no "never" promise is made. Any relicensing proposal must consider existing contributors, copyright ownership, third-party dependencies, downstream users, history, and legal compatibility, and must be a public proposal plus an ADR. Never silently replace the license.

## Open-source definition

No custom restrictions (non-commercial-only, no-competitors, field-of-use, no-cloud/SaaS clauses) may be added; such restrictions are incompatible with the open-source definition and with Apache-2.0. If restrictions are ever wanted, that is a fundamentally different strategy requiring its own decision.

## Legal disclaimer

This project provides licensing information for software, documentation, and third-party components. It is not legal advice. Do not overstate the project's legal certainty.
