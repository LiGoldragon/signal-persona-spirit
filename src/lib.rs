//! Signal contract for the ordinary `persona-spirit` surface.
//!
//! This crate carries the peer-callable vocabulary for psyche statements,
//! psyche-state observations, intent-record observations, and subscriptions.
//! Runtime actors, sockets, storage, classifier logic, and downstream
//! owner-Mutate forwarding live in `persona-spirit`.

use nota_codec::{Decoder, Encoder, NotaDecode, NotaEncode, NotaEnum, NotaRecord, NotaTransparent};
use rkyv::{Archive, Deserialize as RkyvDeserialize, Serialize as RkyvSerialize};
use signal_frame::{emit_schema, signal_channel};
use signal_sema::{Magnitude, SemaObservation};

pub mod migration;

#[derive(
    Archive, RkyvSerialize, RkyvDeserialize, NotaTransparent, Debug, Clone, PartialEq, Eq, Hash,
)]
pub struct StatementText(String);

impl StatementText {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(
    Archive, RkyvSerialize, RkyvDeserialize, NotaTransparent, Debug, Clone, PartialEq, Eq, Hash,
)]
pub struct Topic(String);

impl Topic {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct Topics(Vec<Topic>);

impl Topics {
    pub fn new(value: Vec<Topic>) -> Self {
        Self(value)
    }

    pub fn single(topic: Topic) -> Self {
        Self(vec![topic])
    }

    pub fn as_slice(&self) -> &[Topic] {
        &self.0
    }

    pub fn contains(&self, topic: &Topic) -> bool {
        self.0.iter().any(|candidate| candidate == topic)
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    fn validate(value: &[Topic]) -> nota_codec::Result<()> {
        if value.is_empty() {
            return Err(nota_codec::Error::Validation {
                type_name: "Topics",
                message: "record must carry at least one topic".to_string(),
            });
        }

        let mut seen = std::collections::BTreeSet::<&str>::new();
        for topic in value {
            if !seen.insert(topic.as_str()) {
                return Err(nota_codec::Error::Validation {
                    type_name: "Topics",
                    message: format!("record repeats topic {}", topic.as_str()),
                });
            }
        }

        Ok(())
    }
}

impl NotaEncode for Topics {
    fn encode(&self, encoder: &mut Encoder) -> nota_codec::Result<()> {
        self.0.encode(encoder)
    }
}

impl NotaDecode for Topics {
    fn decode(decoder: &mut Decoder<'_>) -> nota_codec::Result<Self> {
        let value = Vec::<Topic>::decode(decoder)?;
        Self::validate(&value)?;
        Ok(Self(value))
    }
}

#[derive(
    Archive,
    RkyvSerialize,
    RkyvDeserialize,
    NotaTransparent,
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
)]
pub struct RecordIdentifier(u64);

impl RecordIdentifier {
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    pub const fn value(self) -> u64 {
        self.0
    }
}

#[derive(
    Archive, RkyvSerialize, RkyvDeserialize, NotaTransparent, Debug, Clone, PartialEq, Eq, Hash,
)]
pub struct Description(String);

impl Description {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Date {
    pub year: u16,
    pub month: u8,
    pub day: u8,
}

impl Date {
    pub const fn new(year: u16, month: u8, day: u8) -> Self {
        Self { year, month, day }
    }
}

impl NotaEncode for Date {
    fn encode(&self, encoder: &mut Encoder) -> nota_codec::Result<()> {
        encoder.write_date(self.year, self.month, self.day)
    }
}

impl NotaDecode for Date {
    fn decode(decoder: &mut Decoder<'_>) -> nota_codec::Result<Self> {
        let (year, month, day) = decoder.read_date()?;
        Ok(Self { year, month, day })
    }
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Time {
    pub hour: u8,
    pub minute: u8,
    pub second: u8,
}

impl Time {
    pub const fn new(hour: u8, minute: u8, second: u8) -> Self {
        Self {
            hour,
            minute,
            second,
        }
    }
}

impl NotaEncode for Time {
    fn encode(&self, encoder: &mut Encoder) -> nota_codec::Result<()> {
        encoder.write_time(self.hour, self.minute, self.second)
    }
}

impl NotaDecode for Time {
    fn decode(decoder: &mut Decoder<'_>) -> nota_codec::Result<Self> {
        let (hour, minute, second) = decoder.read_time()?;
        Ok(Self {
            hour,
            minute,
            second,
        })
    }
}

#[derive(
    Archive, RkyvSerialize, RkyvDeserialize, NotaTransparent, Debug, Clone, PartialEq, Eq, Hash,
)]
pub struct FocusArea(String);

impl FocusArea {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct StateSubscriptionToken {
    pub identifier: u64,
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct RecordSubscriptionToken {
    pub identifier: u64,
}

#[derive(
    Archive, RkyvSerialize, RkyvDeserialize, NotaEnum, Debug, Clone, Copy, PartialEq, Eq, Hash,
)]
pub enum Kind {
    Decision,
    Principle,
    Correction,
    Clarification,
    Constraint,
}

#[derive(
    Archive, RkyvSerialize, RkyvDeserialize, NotaEnum, Debug, Clone, Copy, PartialEq, Eq, Hash,
)]
pub enum ObservationMode {
    DescriptionOnly,
    WithProvenance,
}

pub type Certainty = Magnitude;
pub type Mode = ObservationMode;

#[derive(
    Archive, RkyvSerialize, RkyvDeserialize, NotaEnum, Debug, Clone, Copy, PartialEq, Eq, Hash,
)]
pub enum Presence {
    Active,
    Absent,
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct Statement {
    pub text: StatementText,
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    pub topics: Topics,
    pub kind: Kind,
    pub description: Description,
    pub certainty: Magnitude,
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct RecordQuery {
    pub topic: Option<Topic>,
    pub kind: Option<Kind>,
    pub mode: ObservationMode,
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct RecordObservation {
    pub query: RecordQuery,
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct RecordSubscription {
    pub topic: Option<Topic>,
    pub mode: ObservationMode,
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct RecordDescription {
    pub identifier: RecordIdentifier,
    pub topics: Topics,
    pub kind: Kind,
    pub description: Description,
    pub certainty: Magnitude,
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct RecordProvenance {
    pub description: RecordDescription,
    pub date: Date,
    pub time: Time,
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct TopicCount {
    pub topic: Topic,
    pub entries: u64,
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct PresenceView {
    pub presence: Presence,
    pub focus: Option<FocusArea>,
}

#[derive(
    Archive, RkyvSerialize, RkyvDeserialize, NotaTransparent, Debug, Clone, PartialEq, Eq, Hash,
)]
pub struct QuestionIdentifier(String);

impl QuestionIdentifier {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(
    Archive, RkyvSerialize, RkyvDeserialize, NotaTransparent, Debug, Clone, PartialEq, Eq, Hash,
)]
pub struct QuestionText(String);

impl QuestionText {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct QuestionSummary {
    pub identifier: QuestionIdentifier,
    pub question: QuestionText,
}

#[derive(
    Archive, RkyvSerialize, RkyvDeserialize, NotaTransparent, Debug, Clone, Copy, PartialEq, Eq,
)]
pub struct RecordAccepted(RecordIdentifier);

impl RecordAccepted {
    pub const fn new(identifier: RecordIdentifier) -> Self {
        Self(identifier)
    }

    pub const fn identifier(self) -> RecordIdentifier {
        self.0
    }
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct StateObserved {
    pub state: PresenceView,
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct RecordsObserved {
    pub records: Vec<RecordDescription>,
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct RecordProvenancesObserved {
    pub records: Vec<RecordProvenance>,
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct TopicsObserved {
    pub topics: Vec<TopicCount>,
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct QuestionsObserved {
    pub questions: Vec<QuestionSummary>,
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaEnum, Debug, Clone, PartialEq, Eq)]
pub enum Observation {
    State,
    Records(RecordQuery),
    Topics,
    Questions,
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaEnum, Debug, Clone, PartialEq, Eq)]
pub enum Subscription {
    State,
    Records(RecordSubscription),
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaEnum, Debug, Clone, PartialEq, Eq)]
pub enum SubscriptionToken {
    State(StateSubscriptionToken),
    Records(RecordSubscriptionToken),
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaEnum, Debug, Clone, PartialEq, Eq)]
pub enum SubscriptionSnapshot {
    State(PresenceView),
    Records(Vec<RecordDescription>),
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct SubscriptionOpened {
    pub token: SubscriptionToken,
    pub snapshot: SubscriptionSnapshot,
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct SubscriptionRetracted {
    pub token: SubscriptionToken,
}

#[derive(
    Archive, RkyvSerialize, RkyvDeserialize, NotaEnum, Debug, Clone, Copy, PartialEq, Eq, Hash,
)]
pub enum UnimplementedReason {
    NotBuiltYet,
    IntegrationNotLanded,
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct RequestUnimplemented {
    pub reason: UnimplementedReason,
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct StateChanged {
    pub state: PresenceView,
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct RecordCaptured {
    pub record: RecordDescription,
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct OperationReceived {
    pub operation: OperationKind,
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct EffectEmitted {
    pub observation: SemaObservation,
}

signal_channel!([schema]);

// Schema-driven dual emission per psyche 2026-05-26 + intent records
// 709, 710 (the three-language POC). The wire schema (`spirit.schema`)
// IS the WIRE LANGUAGE — the first of three languages. The
// `emit_schema!()` invocation reads the same schema file and emits a
// `pub mod spirit { … }` carrying the schema-derived types alongside
// the legacy `signal_channel!([schema])` emission at crate root.
//
// Downstream consumers reach for one path or the other without a
// forced cutover:
//
//   Legacy:  signal_persona_spirit::Operation
//   Schema:  signal_persona_spirit::spirit::Operation
//
// The schema engine's extended universal-Unknown carrier check
// (`is_universal_unknown_carrier_name`) injects `Unknown(String)` into
// the schema-driven `Reply` enum — the wire-forward-compat floor that
// mirrors the actor RESPONSE floor on the internal-channel side.
emit_schema!();
