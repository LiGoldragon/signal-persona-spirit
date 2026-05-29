# INTENT — signal-persona-spirit

*The psyche's intent for the ordinary peer-callable wire contract
for `persona-spirit`. Synthesised from primary workspace intent
records 656-696 and `reports/designer/345/346` (the schema-driven
crystallisation arc). Verbatim psyche quotes in italics where the
exact wording is load-bearing; surrounding prose is agent-composed.
Companion to `ARCHITECTURE.md` and `AGENTS.md`. Maintenance:
`skills/repo-intent.md` and `skills/intent-manifestation.md`.*

## What this crate is

`signal-persona-spirit` is the **ordinary peer-callable** wire
contract for `persona-spirit`. It carries the vocabulary for
submitting psyche statements, observing psyche state, observing
intent records, and subscribing to those streams. Owner-only
lifecycle/configuration orders live in the sibling crate
`owner-signal-persona-spirit`; runtime actors, sockets, storage,
classifier logic, and mind forwarding live in `persona-spirit`.

## Schemas warrant per channel

Per record 668: *"when the psyche describes a major part of the
system, that description IS a warrant to create a schema for that
part."* The ordinary channel is one such part; this crate carries
its schema. The owner channel is a separate part with its own
crate and its own schema. **One channel = one contract = one
schema.** Per workspace record 668, multi-schema-per-crate is the
new normal in daemons where multiple internal channels coexist;
contract crates like this one stay single-channel because the
crate boundary already enforces the channel boundary.

## Dual wire emission — compatibility approach

The `designer-schema-full-stack-spirit-2026-05-25` branch lands
**dual wire emission**: both `signal_channel!([schema])` (legacy)
and `emit_schema!()` (schema-driven) fire in the same `src/lib.rs`,
producing concrete types at two paths:

- **Legacy** at `signal_persona_spirit::Operation` and siblings
  (crate root).
- **Schema-driven** at `signal_persona_spirit::spirit::Operation`
  and siblings (wrapped in `pub mod spirit { ... }` per the
  `emit_schema!` convention).

Both pass tests. The dual emission IS the **designer-recommended
migration path**: downstream consumers migrate from the legacy
root-level paths to the qualified `::spirit::*` paths
incrementally, then the legacy `signal_channel!([schema])`
invocation removes in a separate breaking commit once all
consumers have flipped. The operator integration cycle decides
when to flip downstream consumers; this crate's job is to keep
both paths live during the migration window.

The dual emission is **not** a permanent compatibility surface; it
is a **migration scaffold**. Once all consumers reach the
qualified `::spirit::*` paths, the legacy invocation retires.

## Description-only discipline

The record shape carries one agent-clarified `Description`, a
`Kind`, a `Magnitude`, daemon-stamped time, and one or more
user-creatable topic strings. Verbatim/context payloads from
earlier shapes are gone. The wire reply to a `Record` operation is
`(RecordAccepted N)` — terse; no echo of the submitted content.
Daemon-stamped timestamps: clients do not supply capture time. Any
new topic word a `Record` uses is registered at the wire layer; no
pre-declared enum. Topic queries match membership in the entry's
topic vector, either as no topic filter, partial one-or-more topic
matching, or full every-topic matching.

## Goals

- Carry the **ordinary peer-callable contract** for psyche-state
  observation, intent-record submission/observation, and
  subscription lifecycle.
- Honour the **single-channel-per-crate** boundary: owner orders
  live in a separate crate.
- Maintain **dual wire emission** through the schema-driven
  migration window: legacy at crate root, schema-driven at
  `::spirit::*`.
- Project the **description-only discipline** — terse
  acknowledgements, daemon-stamped timestamps, and user-creatable
  topic vectors.

## Constraints

- The macro-injected `Tap(ObserverFilter)` /
  `Untap(ObserverSubscriptionToken)` verbs are mandatory on the
  ordinary socket per the persona-component observability
  discipline. Domain-specific `Watch`/`Unwatch` for psyche-state
  and intent-record streams is a separate surface and coexists
  without collision.
- Wire reply shapes stay terse — no verbatim echo of submitted
  content.
- The dual emission must not introduce duplicate symbol conflicts
  at crate root; the `emit_schema!` wrapping in `pub mod spirit
  { ... }` is what keeps the two paths isolated.

## Principles

- **Wire vocabulary uses contract-local verbs.** `State` not
  `Statement`, `Record` not `Entry`-as-a-verb, `Observe` not
  `Observation`. Per the verb-form rule in `intent/naming.nota`
  19:45Z.
- **Three-layer model.** Layer 1 contract operations on the wire
  (this crate). Layer 2 component commands inside the
  `persona-spirit` daemon. Layer 3 payloadless Sema classification
  for observability. *Executable payloads do not live in this
  contract.*
- **Migration by qualification, not by symbol replacement.**
  Downstream consumers reach for `signal_persona_spirit::spirit::
  Operation` when ready; the legacy `signal_persona_spirit::
  Operation` stays valid until all consumers flip.

## Open intent

- When all downstream consumers have flipped to the
  `::spirit::*` paths, the legacy `signal_channel!([schema])`
  invocation retires in a separate breaking commit. Timing is the
  operator's call; the crate stays dual-emission until then.
- Cross-crate schema-import resolution (the same deferral the
  primary explainer documents) — when the resolver lands, the
  persona-spirit daemon's actor schemas can `(Import spirit [...])`
  from this crate directly rather than carry hand-written type
  duplicates.

## See also

- `ARCHITECTURE.md` — three-layer model + wire vocabulary.
- `spirit.schema` — the wire contract source-of-truth.
- `src/lib.rs` — dual `signal_channel!` + `emit_schema!` site.
- Primary workspace: `reports/designer/349-context-maintenance-
  sweep-2026-05-25/1-poc-schema-stack-explainer.md` — full
  explainer of the schema-driven full-stack POC.
- Primary workspace: `repos/persona-spirit/INTENT.md` — the
  daemon-side schema-driven actor architecture.
