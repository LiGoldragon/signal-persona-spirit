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
| `PsycheStatement` | `Assert` | A psyche prompt entered the system. |
| `PsycheStateObservation` | `Match` | Query spirit's current psyche-state summary. |
| `IntentRecordObservation` | `Match` | Query intent records, summary-first by default. |
| `ClarificationQuestionPending` | `Match` | Query open intent-clarification questions. |
| `SubscribePsycheState` | `Subscribe` | Open the psyche-state stream. |
| `PsycheStateSubscriptionRetraction` | `Retract` | Close the psyche-state stream. |
| `SubscribeIntentRecords` | `Subscribe` | Open the intent-record stream. |
| `IntentRecordSubscriptionRetraction` | `Retract` | Close the intent-record stream. |

## Constraints

| Constraint | Witness |
|---|---|
| Every request variant declares a Signal root verb. | `round_trip.rs` checks every request's `signal_verb()`. |
| Subscribe variants declare stream relations. | `signal_channel!` stream blocks bind subscribe/open/event/close. |
| Intent queries are summary-first unless a richer mode is requested. | `IntentObservationMode::SummaryOnly` is the explicit query mode used in canonical examples. |
| This crate contains no runtime. | Source has no Kameo, Tokio, sockets, redb, or sema-engine code. |

## Code Map

```text
src/lib.rs              — request/reply/event records and signal_channel! invocation
examples/canonical.nota — canonical NOTA examples
tests/round_trip.rs     — rkyv frame, NOTA, verb, and stream witnesses
```
