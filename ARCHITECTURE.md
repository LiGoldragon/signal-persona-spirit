# signal-persona-spirit — architecture

*Ordinary Signal contract for the psyche-facing Persona spirit surface.*

## Role

`signal-persona-spirit` is the peer-callable contract for
`persona-spirit`. It carries the vocabulary for submitting psyche statements,
observing psyche state, observing intent records, and subscribing to those
streams.

Privileged lifecycle/configuration orders live in
`owner-signal-persona-spirit`. Runtime actors, sockets, storage, classifier
logic, and mind forwarding live in `persona-spirit`.

## MUST IMPLEMENT — three-layer migration

This contract is migrating to the three-layer model affirmed
2026-05-20 per
`primary/reports/designer/246-v4-bundled-fix-deep-design-with-examples.md`
and `primary/reports/designer/248-three-layer-changes-for-operators.md`.

**Layer 1 — Contract Operations on the wire (this crate).** Drop the
`Assert Statement` / `Assert Entry` / `Match *Observation` /
`Subscribe *Subscription` / `Retract *SubscriptionRetraction` shape.
Use contract-local verbs entirely:
- `State` (the psyche stating intent, payload `Quote` or
  `Statement`),
- `Record` (an agent submitting a typed intent entry, payload
  `Entry`),
- `Observe` (the read side — payload is a closed `Observation` enum
  naming `State`, `Records`, `QuestionsPending`, etc.),
- `Watch` / `Unwatch` (domain-specific subscriptions — payload names
  which stream class to open).

Apply the verb-form rule per `intent/naming.nota` 19:45Z:
`State` not `Statement`, `Record` not `Entry`-as-a-verb, `Observe` not
`Observation`.

**Mandatory `Tap`/`Untap` for persona components.** Persona-spirit is
a persona component, so its observable surface is standardized.
The macro-injected `Tap(ObserverFilter)` /
`Untap(SpiritObserverSubscriptionToken)` verbs are mandatory on the
ordinary socket. The domain-specific `Watch`/`Unwatch` for psyche-
state and intent-record streams is a separate surface and coexists
without collision (spirit's domain doesn't use `Tap` as a verb).

**Layer 2 — Component Commands (persona-spirit daemon).** The spirit
daemon owns its typed Command enum (e.g.
`SpiritCommand::AssertStatement(Statement)`,
`SpiritCommand::AssertEntry(Entry)`,
`SpiritCommand::ReadPsycheState`,
`SpiritCommand::ReadIntentRecords`) plus a `CommandExecutor` that
knows the spirit tables.

**Layer 3 — Sema classification (signal-sema).** Each Component
Command projects to a payloadless `SemaOperation` class via
`ToSemaOperation`. Persona-introspect filters cross-component
activity by class.

**Frame layer.** The dependency on `signal-core` shifts to
`signal-frame`.

References:
- `primary/reports/designer/246-v4-bundled-fix-deep-design-with-examples.md`
- `primary/reports/designer/248-three-layer-changes-for-operators.md`
- `primary/skills/component-triad.md` §"Verbs come in three layers"
- `primary/skills/contract-repo.md` §"Public contracts use contract-local operation verbs"

**Note to remover:** when the refactor lands, remove this section and
add a `## Migration history — three-layer model (2026-05-XX)`
paragraph noting the shape change.

## Contract Surface (after migration)

| Operation | Payload | Sema class (Layer 3 projection) |
|---|---|---|
| `State` | `Statement` | `Assert` |
| `Record` | `Entry` | `Assert` |
| `Observe` (State kind) | `Observation::State` | `Match` |
| `Observe` (Records kind) | `Observation::Records` | `Match` |
| `Observe` (QuestionsPending kind) | `Observation::QuestionsPending` | `Match` |
| `Watch` (domain state stream) | `StateSubscription` | `Subscribe` |
| `Unwatch` (domain state stream) | `StateSubscriptionToken` | `Retract` |
| `Watch` (domain records stream) | `RecordsSubscription` | `Subscribe` |
| `Unwatch` (domain records stream) | `RecordsSubscriptionToken` | `Retract` |
| `Tap` (mandatory observability) | `ObserverFilter` | `Subscribe` |
| `Untap` (mandatory observability) | `SpiritObserverSubscriptionToken` | `Retract` |

The wire form carries the contract-local verb only; the Sema class
label is computed at observation publish time inside the daemon.

## Constraints

| Constraint | Witness |
|---|---|
| Every request variant is a contract-local verb in verb form. | `round_trip.rs` asserts each variant's NOTA head. |
| Subscribe-shaped variants declare stream relations. | `signal_channel!` stream blocks bind subscribe/open/event/close. |
| Retract-shaped close variants have typed close acknowledgements. | `StateSubscriptionRetracted` and `RecordSubscriptionRetracted` round-trip through RKYV and NOTA. |
| Intent queries are summary-first unless a richer mode is requested. | `ObservationMode::SummaryOnly` is the explicit query mode used in canonical examples. |
| Every entry is one top-level psyche statement. | `Entry` carries one timestamp and one quote; repeated entries are the restatement signal. |
| Record identifiers are output-only. | `RecordIdentifier` appears in summaries/provenance replies, not in `Entry`. |
| Sema classification is daemon-side projection only; no Sema labels on the wire. | Daemon-side `ToSemaOperation` impl is the witness. |
| This crate contains no runtime. | Source has no Kameo, Tokio, sockets, redb, or sema-engine code. |

## Code Map

```text
src/lib.rs              — request/reply/event records and signal_channel! invocation
examples/canonical.nota — canonical NOTA examples
tests/round_trip.rs     — rkyv frame, NOTA, verb, and stream witnesses
```
