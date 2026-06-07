# signal-persona-spirit — architecture

*Ordinary Signal contract for the psyche-facing Persona spirit surface.*

## Role

`signal-persona-spirit` is the peer-callable contract for
`persona-spirit`. It carries the vocabulary for submitting psyche statements,
observing psyche state, observing intent records, and subscribing to those
streams.

Meta-policy lifecycle/configuration orders live in the sibling meta
contract. Runtime actors, sockets, storage, classifier logic, and mind
forwarding live in `persona-spirit`.

## Three-layer model

This contract is on the current three-layer model affirmed
2026-05-20:

```text
contract Operation  ->  component Command  ->  Sema classification
wire vocabulary         daemon executable      payloadless observation
```

**Layer 1 — Contract operations on the wire (this crate).**
The ordinary contract uses contract-local verbs:
- `State` (the psyche stating intent, payload `Statement`),
- `Record` (an agent submitting a typed intent entry without capture time,
  payload `Entry`),
- `Observe` (the read side — payload is a closed `Observation` enum
  naming `State`, `Records`, `Topics`, `QuestionsPending`, etc.),
- `Watch` / `Unwatch` (domain-specific subscriptions — payload names
  which stream class to open).
- `Remove` (intent-store maintenance — payload is the `RecordIdentifier`
  to delete from the daemon-owned store).
- `ChangeCertainty` (intent-store maintenance — payload is a
  `CertaintyChange` naming the record and replacement certainty).
- `ChangeRecord` (intent-store maintenance — payload is a
  `RecordChange` naming the record and replacement entry).
- `CollectRemovalCandidates` (intent-store maintenance — payload is a
  `RemovalCandidateCollection` that selects exact-`Zero` candidates and
  names an output target before daemon-side retraction).

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

**Layer 3 — Sema classification (daemon-side).** Each Component
Command projects to a payloadless Sema class inside the daemon.
The public observer event stays contract-owned:
`EffectEmitted { operation, outcome }`.

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

The generic observable event record is `EffectEmitted`, matching the
current architecture where generic observers see the effect publication
moment through the contract-owned operation/outcome pair rather than
an executable daemon effect record or Sema payload.

## Contract Surface

| Operation | Payload | Sema class (Layer 3 projection) |
|---|---|---|
| `State` | `Statement` | `Assert` |
| `Record` | `Entry` without date/time | `Assert` |
| `Observe` (state kind) | `Observation::State` unit variant | `Match` |
| `Observe` (Records kind) | `Observation::Records` | `Match` |
| `Observe` (RecordIdentifiers kind) | `Observation::RecordIdentifiers` | `Match` |
| `Observe` (Topics kind) | `Observation::Topics` unit variant | `Match` |
| `Observe` (questions kind) | `Observation::Questions` unit variant | `Match` |
| `Watch` (domain state stream) | `Subscription::State` unit variant | `Subscribe` |
| `Unwatch` (domain state stream) | `StateSubscriptionToken` | `Retract` |
| `Watch` (domain records stream) | `RecordsSubscription` | `Subscribe` |
| `Unwatch` (domain records stream) | `RecordsSubscriptionToken` | `Retract` |
| `Remove` | `RecordIdentifier` | `Retract` |
| `ChangeCertainty` | `CertaintyChange` | `Mutate` |
| `ChangeRecord` | `RecordChange` | `Mutate` |
| `CollectRemovalCandidates` | `RemovalCandidateCollection` | `Retract` |
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
| Intent queries return compact summaries unless provenance is requested. | `ObservationMode::SummaryOnly` is the explicit query mode used in canonical examples. |
| Intent record queries support the agent-useful filters needed for intent work. | `PublicRecordQuery` carries `TopicSelection` (`Any`, `Partial`, `Full`), optional `kind`, `CertaintySelection` (`Any`, `Exact`, `AtMost`, `AtLeast`), `RecordedTimeSelection` (`Any`, `Between`, `Since`, `Until`, `Recent`, `Shallow`, `Deep`, `VeryDeep`), and description/provenance mode; it has no privacy field and means exact-`Zero` privacy. `RecordIdentifierQuery` selects one opaque identifier exactly; identifier ranges are intentionally absent because random identifiers do not carry recency or ordinal meaning. `PrivacyScopedRecordQuery` and `PrivacyScopedRecordIdentifierQuery` are explicit elevated read shapes carrying `PrivacySelection` (`Any`, `Exact`, `AtMost`, `AtLeast`). `RecordQuery` remains the full maintenance/internal query shape for candidate collection and daemon projection. |
| Intent entries can be removed explicitly by identifier. | `Remove(RecordIdentifier)` round-trips through RKYV and NOTA and returns `RecordRemoved`; production identifiers are opaque lowercase base36 codes minted by `persona-spirit`, normally rendered at the shortest collision-free four-to-seven-character length while the wire type remains wide enough to decode older long codes. |
| Intent entries can be nominated for removal without deletion. | `ChangeCertainty(CertaintyChange)` round-trips through RKYV and NOTA and returns `CertaintyChanged`; setting certainty to `Zero` makes the record visible to removal-candidate review. |
| Intent entries can be corrected in place without remove-and-recreate. | `ChangeRecord(RecordChange)` round-trips through RKYV and NOTA and returns `RecordMutationApplied`; the daemon replaces the user-authored `Entry` fields under the same `RecordIdentifier` while preserving daemon-owned provenance. |
| Removal-candidate collection is explicit capture-before-retract maintenance. | `CollectRemovalCandidates(RemovalCandidateCollection)` round-trips through RKYV and NOTA, requires exact-`Zero` certainty and exact-`Zero` privacy by contract method, carries `OutputTarget::ArchiveDatabase(Default)` for the daemon-derived archive database, `OutputTarget::ArchiveDatabase(Path(ArchivePath))` for an explicit archive database path, or `OutputTarget::Print(OutputStream)` for client-rendered compact material, and returns `RemovalCandidatesCollected` with compact `RecordSummary` archive material plus removed identifiers and skipped candidates. |
| Historical storage migration shapes stay contract-owned and explicit. | `tests/migration.rs` projects a v0.3.0 `migration::v030::Entry` and `migration::v030::Operation::Record` into the current privacy-aware shape with `privacy = Zero`, proving the daemon can read the prior production row shape without guessing at bytes. |
| Agents can inspect the intent-topic catalog without reading every entry. | `Observation::Topics` returns `TopicsObserved` with one `TopicCount` per topic membership. |
| Every submitted entry is one top-level psyche statement without client-provided capture time. | `Entry` carries one or more topics, kind, description, required `Magnitude` certainty, and required `Magnitude` privacy; repeated entries are the restatement signal. |
| Spirit never accepts client-provided timestamps on `Record` requests. | `record_request_with_client_timestamp_shape_is_rejected` and `record_request_with_parenthesized_client_date_time_shape_is_rejected` fail old timestamp-bearing input shapes. |
| Capture time appears only in daemon-produced provenance. | `RecordProvenance` carries one bare `YYYY-MM-DD` date field and one bare `HH:MM:SS` time field. |
| Record identifiers are output-only. | `RecordIdentifier` appears in descriptions/provenance replies, not in `Entry`; `persona-spirit` mints it from randomness, not from row position. |
| Sema classification is daemon-side projection only; no Sema payloads appear on the wire. | `EffectEmitted` carries contract-owned `operation` and `outcome` fields; daemon-side projection impls are the executable witnesses. |
| This crate contains no runtime. | Source has no Kameo, Tokio, sockets, database engine, or sema-engine code. |

## Code Map

```text
src/lib.rs              — request/reply/event records and explicit signal_channel! declaration
src/migration.rs        — historical contract shapes and projection bridges for store migrations
examples/canonical.nota — canonical NOTA examples
tests/round_trip.rs     — rkyv frame, NOTA, verb, and stream witnesses
tests/migration.rs      — prior-version projection witnesses
```
