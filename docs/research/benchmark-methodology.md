<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# RAHN Benchmark Methodology

Status: living research document (Stage 0). Anti-goal: meaningless benchmark numbers (e.g., "X% faster" with no baseline, environment, or methodology).

## What to benchmark (when implementation exists)

State creation · state serialization · state hashing/identity · diff · branch creation · merge · invariant verification · persistence (commit/load) · graph operations (from Stage 2).

Timing is explicitly **not** a v0.1 priority (charter: correctness over speed); benchmarks exist to catch regressions and to test hypotheses (E2), not to advertise.

## Rules

1. **Record the environment.** Every result must state: hardware (CPU, RAM, storage type), OS + kernel, compiler/toolchain versions, code commit, date, and relevant flags. A result without this is void.
2. **Record the method.** Workload definition, input generation (seeded), warmup, iteration count, measurement harness (e.g., `criterion` for Rust), and statistical treatment (median + intervals, not single runs).
3. **Use realistic workloads.** Topologies spanning small (10² objects) to large (10⁵) scale; both synthetic and derived-from-real shapes where licensing permits.
4. **Compare against a baseline**, in the same environment, same method. "Faster" is meaningless without "faster than what, measured how".
5. **Never average away the tail** when tail latency matters (verification gates every transition — p99 is the story).
6. **Publish negative results.** If a benchmark shows the design is slow, that is recorded, not hidden.
7. **Reproducibility over peak numbers.** Prefer a benchmark anyone can rerun from the repo over a cherry-picked record run.

## Reporting format

Each benchmark result document includes: hypothesis/question · environment block · methodology · results (table/plot with intervals) · baseline comparison · interpretation and limitations · raw data location.
