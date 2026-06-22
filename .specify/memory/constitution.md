<!--
Sync Impact Report
Version change: N/A -> 1.0.0
Modified principles: template -> concrete Headroom fork planning rules
Added sections: Rust proxy parity, CCR reversibility, production-image boundary, evidence-first gates
Removed sections: unresolved template placeholders
Templates requiring updates: spec-template.md reviewed; plan-template.md reviewed; tasks-template.md reviewed
Deferred items: none
-->

# Headroom Fork Constitution

## Core Principles

### I. Repo Reality First

All feature work MUST start from the current `hohsiang-lab/headroom` fork state,
not from upstream assumptions or old local clones. Plans MUST cite the concrete
Rust/Python files they touch and MUST keep unrelated upstream realignment work
out of scope.

### II. Reversible Compression Contract

Any lossy compression path that emits a CCR marker MUST back that marker with a
reachable CCR store entry. Missing store wiring, silent fallback to memory, or
markers that cannot be retrieved are release blockers.

### III. Rust Proxy Boundary

Rust proxy changes MUST stay inside the Rust crates unless the issue explicitly
authorizes Python production-image changes. Python packaging, Docker production
entrypoints, and deployment image behavior MUST remain unchanged for planning
scope that only targets Rust proxy CCR wiring.

### IV. Evidence Before State Movement

SpecKit artifacts MUST define tests or smoke checks for each acceptance point
before implementation starts. A Linear state move out of Todo requires spec,
plan, tasks, analyze evidence, and a docs-only draft PR.

### V. No Silent Operational Degradation

Startup/config failures for storage backends MUST surface as loud errors at the
proxy boundary. A configured persistent or remote backend MUST NOT degrade to a
different backend without explicit operator configuration.

## Governance

This constitution governs SpecKit planning artifacts for this fork worktree.
Amendments require a docs change in the same branch and must update this file's
version using semantic versioning: MAJOR for incompatible principles, MINOR for
new principles, PATCH for clarifications. Compliance is reviewed during
`speckit-plan` and `speckit-analyze`.

**Version**: 1.0.0 | **Ratified**: 2026-06-22 | **Last Amended**: 2026-06-22
