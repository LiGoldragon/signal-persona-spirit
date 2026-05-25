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

## Three-layer model

This contract is on the current three-layer model affirmed
2026-05-20:

```text
contract Operation  ->  component Command  ->  Sema classification
wire vocabulary         daemon executable      payloadless observation
```

**Layer 1 — Contract operations on the wire (this crate).**
The ordinary contract uses contract-local verbs:
- `State` (the psyche stating intent, payload `Quote` or
  `Statement`),
- `Record` (an agent submitting a typed intent entry without capture time,
  payload `Entry`),
- `Observe` (the read side — payload is a closed `Observation` enum
  naming `State`, `Records`, `Topics`, `QuestionsPending`, etc.),
- `Watch` / `Unwatch` (domain-specific subscriptions — payload names
  which stream class to open).

Apply the verb-form rule per `intent/naming.nota` 19:45Z:
`State` not `Statement`, `Record` not `Entry`-as-a-verb, `Observe` not
`Observation`.

**Mandatory `Tap`/`Untap` for persona components.** Persona-spirit is
a persona component, so its observable surface is standardized.
The macro-injected `Tap(ObserverFilter)` /
`Untap(ObserverSubscriptionToken)` verbs are mandatory on the
ordinary socket. The domain-specific `Watch`/`Unwatch` for psyche-
state and intent-record streams is a separate surface and coexists
without collision (spirit's domain doesn't use `Tap` as a verb).

**Layer 2 — Component Commands (persona-spirit daemon).** The spirit
daemon owns its typed Command enum plus a `CommandExecutor` that knows
the spirit tables. Executable payloads do not live in this contract.

**Layer 3 — Sema classification (signal-sema).** Each Component
Command projects to a payloadless `SemaOperation` class via
`ToSemaOperation`; each Component Effect projects to a payloadless
`SemaOutcome` class via `ToSemaOutcome`. Persona-introspect filters
cross-component activity through `SemaObservation`.

**Frame layer.** Frame mechanics come from `signal-frame`.

References:
- `primary/reports/designer/246-v4-bundled-fix-deep-design-with-examples.md`
- `primary/reports/designer/248-three-layer-changes-for-operators.md`
- `primary/skills/component-triad.md` §"Verbs come in three layers"
- `primary/skills/contract-repo.md` §"Public contracts use contract-local operation verbs"

## Migration history — three-layer model (2026-05-20)

The old shape coupled the wire vocabulary to Sema roots
(`Assert Statement`, `Match *Observation`, `Subscribe *Subscription`,
`Retract *SubscriptionRetraction`). That shape is retired. The wire now
uses the contract-local verbs listed below, while Sema appears only as
daemon-side payloadless classification.

The generic observable classification event record is now
`EffectEmitted`, matching the current architecture where generic
observers see the effect publication moment carrying payloadless
Sema observations rather than executable effect records.

## Contract Surface

| Operation | Payload | Sema class (Layer 3 projection) |
|---|---|---|
| `State` | `Statement` | `Assert` |
| `Record` | `Entry` without date/time | `Assert` |
| `Observe` (state kind) | `Observation::State` unit variant | `Match` |
| `Observe` (Records kind) | `Observation::Records` | `Match` |
| `Observe` (Topics kind) | `Observation::Topics` unit variant | `Match` |
| `Observe` (questions kind) | `Observation::Questions` unit variant | `Match` |
| `Watch` (domain state stream) | `Subscription::State` unit variant | `Subscribe` |
| `Unwatch` (domain state stream) | `StateSubscriptionToken` | `Retract` |
| `Watch` (domain records stream) | `RecordsSubscription` | `Subscribe` |
| `Unwatch` (domain records stream) | `RecordsSubscriptionToken` | `Retract` |
| `Tap` (mandatory observability) | `ObserverFilter` | `Subscribe` |
| `Untap` (mandatory observability) | `ObserverSubscriptionToken` | `Retract` |

The wire form carries the contract-local verb only; the Sema class
label is computed at observation publish time inside the daemon.

## Constraints

| Constraint | Witness |
|---|---|
| Every request variant is a contract-local verb in verb form. | `round_trip.rs` asserts each variant's NOTA head. |
| Subscribe-shaped variants declare stream relations. | `signal_channel!` stream blocks bind subscribe/open/event/close. |
| Retract-shaped close variants have typed close acknowledgements. | `SubscriptionRetracted` carries the typed `SubscriptionToken` sum and round-trips through RKYV and NOTA. |
| Intent queries are summary-first unless a richer mode is requested. | `ObservationMode::SummaryOnly` is the explicit query mode used in canonical examples. |
| Intent record queries support the agent-useful filters needed for intent work. | `RecordQuery` carries optional `topic` and optional `kind` filters, with summary-only and provenance modes. |
| Agents can inspect the intent-topic catalog without reading every entry. | `Observation::Topics` returns `TopicsObserved` with one `TopicCount` per topic. |
| Every submitted entry is one top-level psyche statement without client-provided capture time. | `Entry` carries topic, kind, summary, context, certainty, and quote; repeated entries are the restatement signal. |
| Spirit never accepts client-provided timestamps on `Record` requests. | `record_request_with_client_timestamp_shape_is_rejected` and `record_request_with_parenthesized_client_date_time_shape_is_rejected` fail old timestamp-bearing input shapes. |
| Capture time appears only in daemon-produced provenance. | `RecordProvenance` carries one bare `YYYY-MM-DD` date field and one bare `HH:MM:SS` time field. |
| Record identifiers are output-only. | `RecordIdentifier` appears in summaries/provenance replies, not in `Entry`. |
| Sema classification is daemon-side projection only; no executable Sema payloads appear on the wire. | `EffectEmitted` carries payloadless `SemaObservation` and daemon-side `ToSemaOperation` / `ToSemaOutcome` impls are the executable witnesses. |
| This crate contains no runtime. | Source has no Kameo, Tokio, sockets, redb, or sema-engine code. |

## Code Map

```text
src/lib.rs              — request/reply/event records and signal_channel! invocation
examples/canonical.nota — canonical NOTA examples
tests/round_trip.rs     — rkyv frame, NOTA, verb, and stream witnesses
```

## Pending schema-engine upgrade

**Status:** scheduled for migration to schema-language-based contract per
`primary/reports/designer/326-v13-spirit-complete-schema-vision.md` +
`primary/reports/designer/324-migration-mvp-spirit-handover-re-specification.md`.
The reader model is multi-pass NOTA-first per spirit record 549; macro
application iterates to a fixed point per record 569.

**Target:** this component's hand-written `signal_channel!` invocation +
Layer 2 Command/Effect + storage types convert to a single
`spirit/spirit.schema` file. The brilliant macro library (`primary-ezqx.1`)
reads the schema + emits all the wire types + ShortHeader projection +
dispatcher + VersionProjection + storage descriptors.

**Sequence:** Spirit is the MVP pilot landing first via `primary-ezqx.1`.
This ordinary contract is the first to be reborn from the `.schema`
source. The existing tables in `## Contract Surface` and `## Constraints`
become outputs of the macro rather than hand-maintained surfaces.

**Per-component concerns:** Repo still carries the `persona-` prefix per
the /318 pilot block; the rename to bare `signal-spirit` lands after the
schema-engine cutover proves the macro library shape. The current
three-layer-migration documentation in `## Three-layer model` describes
contract-local verbs the macro must continue to emit (`State`, `Record`,
`Observe`, `Watch`/`Unwatch`, mandatory `Tap`/`Untap`); the schema source
encodes these as the verb roots and the macro lowers them to the wire
records.

**Variant slot policy.** Per spirit record 562 the spirit schema's enums
place data-carrying variants in slots 0-6 and unit variants after. New
unit variants land after the existing units; the wire format is
backward-compatible by construction. Per record 564 the workspace's
common identifiers (record, forward, send, similar) are stored as
enum-encoded composite names — the namespace is composable across
components and enables multilingual labels because the wire form is the
discriminator, not the string.

**References:**
- `primary/reports/designer/326-v13-spirit-complete-schema-vision.md` —
  uniform header form + schema-language design
- `primary/reports/designer/333-upgrade-mechanism-full-design-explained.md`
  + `333-v2` — upgrade mechanism design + corrections (wire-compat gap
  affects this contract's cross-version frame exchange)
- `primary/reports/designer/334-v2-multi-pass-nota-first-schema-reader.md`
  — multi-pass reader model (record 549)
- `primary/reports/operator/174-schema-import-header-design-critique-2026-05-24.md`
  — header/body/feature separation + lowering rules
