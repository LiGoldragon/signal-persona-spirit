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

## Contract Surface

| Request | Signal verb | Meaning |
|---|---|---|
| `Statement` | `Assert` | A psyche prompt entered the system. |
| `Entry` | `Assert` | An agent submits a typed intent entry for type-checked logging. |
| `StateObservation` | `Match` | Query spirit's current psyche-state summary. |
| `RecordObservation` | `Match` | Query intent records, summary-first by default. |
| `QuestionPending` | `Match` | Query open intent-clarification questions. |
| `SubscribeState` | `Subscribe` | Open the psyche-state stream. |
| `StateSubscriptionRetraction` | `Retract` | Close the psyche-state stream. |
| `SubscribeRecords` | `Subscribe` | Open the intent-record stream. |
| `RecordSubscriptionRetraction` | `Retract` | Close the intent-record stream. |

## Constraints

| Constraint | Witness |
|---|---|
| Every request variant declares a Signal root verb. | `round_trip.rs` checks every request's `signal_verb()`. |
| Subscribe variants declare stream relations. | `signal_channel!` stream blocks bind subscribe/open/event/close. |
| Retract variants have typed close acknowledgements. | `StateSubscriptionRetracted` and `RecordSubscriptionRetracted` round-trip through RKYV and NOTA. |
| Intent queries are summary-first unless a richer mode is requested. | `ObservationMode::SummaryOnly` is the explicit query mode used in canonical examples. |
| Every entry is one top-level psyche statement. | `Entry` carries one timestamp and one quote; repeated entries are the restatement signal. |
| Record identifiers are output-only. | `RecordIdentifier` appears in summaries/provenance replies, not in `Entry`. |
| This crate contains no runtime. | Source has no Kameo, Tokio, sockets, redb, or sema-engine code. |

## Code Map

```text
src/lib.rs              — request/reply/event records and signal_channel! invocation
examples/canonical.nota — canonical NOTA examples
tests/round_trip.rs     — rkyv frame, NOTA, verb, and stream witnesses
```
