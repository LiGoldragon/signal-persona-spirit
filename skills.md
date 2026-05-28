# skills — signal-persona-spirit

Read this before editing the ordinary spirit contract.

## Required Context

- `~/primary/skills/contract-repo.md`
- `~/primary/skills/component-triad.md`
- `~/primary/skills/architectural-truth-tests.md`
- `~/primary/skills/nix-discipline.md`
- this repo's `ARCHITECTURE.md`

## Boundary

This crate owns only the ordinary `persona-spirit` Signal vocabulary. It has no
runtime, no actors, no sockets, no storage, and no classifier logic.

## Invariants

- Under the three-layer model (per
  `~/primary/skills/component-triad.md` §"Verbs come in three layers"):
  - The wire carries contract-local verbs in verb form (`State`,
    `Record`, `Observe`, `Watch`, `Unwatch`, plus mandatory `Tap` /
    `Untap`).
  - The daemon owns typed Component Commands that lower contract
    operations to executable form.
  - Each Command projects to a payloadless Sema class label via
    `ToSemaOperation` for cross-component observation.
- `State` projects to Sema `Assert` at the daemon.
- `Record` projects to Sema `Assert` at the daemon.
- `Entry` is one top-level statement without client-provided capture time.
  It carries one or more user-created topic strings; topic filters match
  membership in that topic vector.
  Restatement is represented by repeated `Entry` records, not by nesting
  vectors.
- Capture time appears only in daemon-produced provenance as a bare
  `YYYY-MM-DD` date field and a bare `HH:MM:SS` time field.
- `RecordIdentifier` is output-only and minted by `persona-spirit`.
- `Observe`-shaped operations project to Sema `Match`.
- Stream-open variants (domain `Watch` and mandatory `Tap`) project
  to Sema `Subscribe` and carry explicit stream relations.
- Stream-close variants (domain `Unwatch` and mandatory `Untap`)
  project to Sema `Retract`.
- Intent observation is description-first unless the caller asks for
  provenance.
- Intent observations can select records by exact `RecordIdentifier`
  or an inclusive `RecordIdentifierRange` through
  `Observation::RecordIdentifiers`.
- Intent entries are removed through the ordinary `Remove` verb by
  `RecordIdentifier`; this is intent-store maintenance, not owner
  lifecycle policy.
- Mandatory `Tap`/`Untap` observability surface is part of the
  contract per persona-component discipline.
