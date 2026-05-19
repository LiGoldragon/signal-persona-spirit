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

- `PsycheStatement` is an `Assert`.
- `Entry` is an `Assert` for agents submitting already typed intent.
- `Entry.verbatim` is a `Vec<Verbatim>` so restatements are preserved
  as timestamped psyche statements.
- Query variants are `Match`.
- Stream-open variants are `Subscribe` and carry explicit stream relations.
- Stream-close variants are `Retract`.
- Intent observation is summary-first unless the caller asks for provenance.
