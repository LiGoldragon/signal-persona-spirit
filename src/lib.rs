//! Signal contract for the ordinary `persona-spirit` surface.
//!
//! This crate carries the peer-callable vocabulary for psyche statements,
//! psyche-state observations, intent-record observations, and subscriptions.
//! Runtime actors, sockets, storage, classifier logic, and downstream
//! owner-Mutate forwarding live in `persona-spirit`.

use nota_codec::{NotaEnum, NotaRecord, NotaTransparent};
use rkyv::{Archive, Deserialize as RkyvDeserialize, Serialize as RkyvSerialize};
use signal_core::signal_channel;

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
pub struct Summary(String);

impl Summary {
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
pub struct Quote(String);

impl Quote {
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
pub struct Context(String);

impl Context {
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
pub struct Timestamp(String);

impl Timestamp {
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
pub enum Certainty {
    Maximum,
    Medium,
    Minimum,
}

#[derive(
    Archive, RkyvSerialize, RkyvDeserialize, NotaEnum, Debug, Clone, Copy, PartialEq, Eq, Hash,
)]
pub enum ObservationMode {
    SummaryOnly,
    WithProvenance,
}

#[derive(
    Archive, RkyvSerialize, RkyvDeserialize, NotaEnum, Debug, Clone, Copy, PartialEq, Eq, Hash,
)]
pub enum Presence {
    Active,
    Absent,
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct Statement {
    pub statement: StatementText,
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    pub topic: Topic,
    pub kind: Kind,
    pub summary: Summary,
    pub context: Context,
    pub certainty: Certainty,
    pub timestamp: Timestamp,
    pub quote: Quote,
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct StateObservation {}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct RecordQuery {
    pub topic: Option<Topic>,
    pub mode: ObservationMode,
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct RecordObservation {
    pub query: RecordQuery,
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct QuestionPending {}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct StateSubscription {}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct RecordSubscription {
    pub topic: Option<Topic>,
    pub mode: ObservationMode,
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct RecordSummary {
    pub identifier: RecordIdentifier,
    pub topic: Topic,
    pub kind: Kind,
    pub summary: Summary,
    pub certainty: Certainty,
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct RecordProvenance {
    pub summary: RecordSummary,
    pub context: Context,
    pub timestamp: Timestamp,
    pub quote: Quote,
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct State {
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

#[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct RecordAccepted {
    pub captured: RecordSummary,
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct StateObserved {
    pub state: State,
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct RecordsObserved {
    pub records: Vec<RecordSummary>,
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct RecordProvenancesObserved {
    pub records: Vec<RecordProvenance>,
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct QuestionsObserved {
    pub questions: Vec<QuestionSummary>,
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct StateSubscriptionOpened {
    pub token: StateSubscriptionToken,
    pub snapshot: State,
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct RecordSubscriptionOpened {
    pub token: RecordSubscriptionToken,
    pub snapshot: Vec<RecordSummary>,
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct StateSubscriptionRetracted {
    pub token: StateSubscriptionToken,
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct RecordSubscriptionRetracted {
    pub token: RecordSubscriptionToken,
}

#[derive(
    Archive, RkyvSerialize, RkyvDeserialize, NotaEnum, Debug, Clone, Copy, PartialEq, Eq, Hash,
)]
pub enum OperationKind {
    Statement,
    Entry,
    StateObservation,
    RecordObservation,
    QuestionPending,
    SubscribeState,
    StateSubscriptionRetraction,
    SubscribeRecords,
    RecordSubscriptionRetraction,
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
    pub operation: OperationKind,
    pub reason: UnimplementedReason,
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct StateChanged {
    pub state: State,
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct RecordCaptured {
    pub record: RecordSummary,
}

signal_channel! {
    channel Spirit {
        request SpiritRequest {
            Assert Statement(Statement),
            Assert Entry(Entry),
            Match StateObservation(StateObservation),
            Match RecordObservation(RecordObservation),
            Match QuestionPending(QuestionPending),
            Subscribe SubscribeState(StateSubscription) opens StateStream,
            Retract StateSubscriptionRetraction(StateSubscriptionToken),
            Subscribe SubscribeRecords(RecordSubscription) opens RecordStream,
            Retract RecordSubscriptionRetraction(RecordSubscriptionToken),
        }
        reply SpiritReply {
            RecordAccepted(RecordAccepted),
            StateObserved(StateObserved),
            RecordsObserved(RecordsObserved),
            RecordProvenancesObserved(RecordProvenancesObserved),
            QuestionsObserved(QuestionsObserved),
            StateSubscriptionOpened(StateSubscriptionOpened),
            RecordSubscriptionOpened(RecordSubscriptionOpened),
            StateSubscriptionRetracted(StateSubscriptionRetracted),
            RecordSubscriptionRetracted(RecordSubscriptionRetracted),
            RequestUnimplemented(RequestUnimplemented),
        }
        event SpiritEvent {
            StateChanged(StateChanged) belongs StateStream,
            RecordCaptured(RecordCaptured) belongs RecordStream,
        }
        stream StateStream {
            token StateSubscriptionToken;
            opened StateSubscriptionOpened;
            event StateChanged;
            close StateSubscriptionRetraction;
        }
        stream RecordStream {
            token RecordSubscriptionToken;
            opened RecordSubscriptionOpened;
            event RecordCaptured;
            close RecordSubscriptionRetraction;
        }
    }
}

pub type Frame = SpiritFrame;
pub type FrameBody = SpiritFrameBody;
pub type ChannelRequest = SpiritChannelRequest;
pub type ChannelReply = SpiritChannelReply;
pub type RequestBuilder = SpiritRequestBuilder;

impl SpiritRequest {
    pub fn operation_kind(&self) -> OperationKind {
        match self {
            Self::Statement(_) => OperationKind::Statement,
            Self::Entry(_) => OperationKind::Entry,
            Self::StateObservation(_) => OperationKind::StateObservation,
            Self::RecordObservation(_) => OperationKind::RecordObservation,
            Self::QuestionPending(_) => OperationKind::QuestionPending,
            Self::SubscribeState(_) => OperationKind::SubscribeState,
            Self::StateSubscriptionRetraction(_) => OperationKind::StateSubscriptionRetraction,
            Self::SubscribeRecords(_) => OperationKind::SubscribeRecords,
            Self::RecordSubscriptionRetraction(_) => OperationKind::RecordSubscriptionRetraction,
        }
    }
}

impl From<RecordAccepted> for SpiritReply {
    fn from(payload: RecordAccepted) -> Self {
        Self::RecordAccepted(payload)
    }
}

impl From<StateObserved> for SpiritReply {
    fn from(payload: StateObserved) -> Self {
        Self::StateObserved(payload)
    }
}

impl From<RecordsObserved> for SpiritReply {
    fn from(payload: RecordsObserved) -> Self {
        Self::RecordsObserved(payload)
    }
}

impl From<RecordProvenancesObserved> for SpiritReply {
    fn from(payload: RecordProvenancesObserved) -> Self {
        Self::RecordProvenancesObserved(payload)
    }
}

impl From<QuestionsObserved> for SpiritReply {
    fn from(payload: QuestionsObserved) -> Self {
        Self::QuestionsObserved(payload)
    }
}

impl From<StateSubscriptionOpened> for SpiritReply {
    fn from(payload: StateSubscriptionOpened) -> Self {
        Self::StateSubscriptionOpened(payload)
    }
}

impl From<RecordSubscriptionOpened> for SpiritReply {
    fn from(payload: RecordSubscriptionOpened) -> Self {
        Self::RecordSubscriptionOpened(payload)
    }
}

impl From<StateSubscriptionRetracted> for SpiritReply {
    fn from(payload: StateSubscriptionRetracted) -> Self {
        Self::StateSubscriptionRetracted(payload)
    }
}

impl From<RecordSubscriptionRetracted> for SpiritReply {
    fn from(payload: RecordSubscriptionRetracted) -> Self {
        Self::RecordSubscriptionRetracted(payload)
    }
}

impl From<RequestUnimplemented> for SpiritReply {
    fn from(payload: RequestUnimplemented) -> Self {
        Self::RequestUnimplemented(payload)
    }
}
