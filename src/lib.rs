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
pub struct PsycheStatementText(String);

impl PsycheStatementText {
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
pub struct IntentTopic(String);

impl IntentTopic {
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
pub struct IntentRecordIdentifier(String);

impl IntentRecordIdentifier {
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
pub struct IntentSummary(String);

impl IntentSummary {
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
pub struct IntentQuote(String);

impl IntentQuote {
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
pub struct IntentContext(String);

impl IntentContext {
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
pub struct IntentTimestamp(String);

impl IntentTimestamp {
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
pub struct PsycheFocusArea(String);

impl PsycheFocusArea {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct PsycheStateSubscriptionToken {
    pub identifier: u64,
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct IntentRecordSubscriptionToken {
    pub identifier: u64,
}

#[derive(
    Archive, RkyvSerialize, RkyvDeserialize, NotaEnum, Debug, Clone, Copy, PartialEq, Eq, Hash,
)]
pub enum IntentKind {
    Decision,
    Principle,
    Correction,
    Clarification,
    Constraint,
}

#[derive(
    Archive, RkyvSerialize, RkyvDeserialize, NotaEnum, Debug, Clone, Copy, PartialEq, Eq, Hash,
)]
pub enum IntentCertainty {
    Maximum,
    Medium,
    Minimum,
}

#[derive(
    Archive, RkyvSerialize, RkyvDeserialize, NotaEnum, Debug, Clone, Copy, PartialEq, Eq, Hash,
)]
pub enum IntentObservationMode {
    SummaryOnly,
    WithProvenance,
}

#[derive(
    Archive, RkyvSerialize, RkyvDeserialize, NotaEnum, Debug, Clone, Copy, PartialEq, Eq, Hash,
)]
pub enum PsychePresence {
    Active,
    Absent,
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct PsycheStatement {
    pub statement: PsycheStatementText,
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct PsycheStateObservation {}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct IntentRecordQuery {
    pub topic: Option<IntentTopic>,
    pub mode: IntentObservationMode,
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct IntentRecordObservation {
    pub query: IntentRecordQuery,
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct ClarificationQuestionPending {}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct PsycheStateSubscription {}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct IntentRecordSubscription {
    pub topic: Option<IntentTopic>,
    pub mode: IntentObservationMode,
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct IntentRecordSummary {
    pub identifier: IntentRecordIdentifier,
    pub topic: IntentTopic,
    pub kind: IntentKind,
    pub summary: IntentSummary,
    pub certainty: IntentCertainty,
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct IntentRecordProvenance {
    pub summary: IntentRecordSummary,
    pub quote: IntentQuote,
    pub context: IntentContext,
    pub timestamp: IntentTimestamp,
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct PsycheState {
    pub presence: PsychePresence,
    pub focus: Option<PsycheFocusArea>,
}

#[derive(
    Archive, RkyvSerialize, RkyvDeserialize, NotaTransparent, Debug, Clone, PartialEq, Eq, Hash,
)]
pub struct ClarificationQuestionIdentifier(String);

impl ClarificationQuestionIdentifier {
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
pub struct ClarificationQuestionText(String);

impl ClarificationQuestionText {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct ClarificationQuestionSummary {
    pub identifier: ClarificationQuestionIdentifier,
    pub question: ClarificationQuestionText,
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct PsycheStatementAccepted {
    pub captured: IntentRecordSummary,
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct PsycheStateObserved {
    pub state: PsycheState,
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct IntentRecordsObserved {
    pub records: Vec<IntentRecordSummary>,
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct ClarificationQuestionsObserved {
    pub questions: Vec<ClarificationQuestionSummary>,
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct PsycheStateSubscriptionOpened {
    pub token: PsycheStateSubscriptionToken,
    pub snapshot: PsycheState,
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct IntentRecordSubscriptionOpened {
    pub token: IntentRecordSubscriptionToken,
    pub snapshot: Vec<IntentRecordSummary>,
}

#[derive(
    Archive, RkyvSerialize, RkyvDeserialize, NotaEnum, Debug, Clone, Copy, PartialEq, Eq, Hash,
)]
pub enum SpiritOperationKind {
    PsycheStatement,
    PsycheStateObservation,
    IntentRecordObservation,
    ClarificationQuestionPending,
    SubscribePsycheState,
    PsycheStateSubscriptionRetraction,
    SubscribeIntentRecords,
    IntentRecordSubscriptionRetraction,
}

#[derive(
    Archive, RkyvSerialize, RkyvDeserialize, NotaEnum, Debug, Clone, Copy, PartialEq, Eq, Hash,
)]
pub enum SpiritUnimplementedReason {
    NotBuiltYet,
    IntegrationNotLanded,
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct SpiritRequestUnimplemented {
    pub operation: SpiritOperationKind,
    pub reason: SpiritUnimplementedReason,
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct PsycheStateChanged {
    pub state: PsycheState,
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct IntentRecordCaptured {
    pub record: IntentRecordSummary,
}

signal_channel! {
    channel Spirit {
        request SpiritRequest {
            Assert PsycheStatement(PsycheStatement),
            Match PsycheStateObservation(PsycheStateObservation),
            Match IntentRecordObservation(IntentRecordObservation),
            Match ClarificationQuestionPending(ClarificationQuestionPending),
            Subscribe SubscribePsycheState(PsycheStateSubscription) opens PsycheStateStream,
            Retract PsycheStateSubscriptionRetraction(PsycheStateSubscriptionToken),
            Subscribe SubscribeIntentRecords(IntentRecordSubscription) opens IntentRecordStream,
            Retract IntentRecordSubscriptionRetraction(IntentRecordSubscriptionToken),
        }
        reply SpiritReply {
            PsycheStatementAccepted(PsycheStatementAccepted),
            PsycheStateObserved(PsycheStateObserved),
            IntentRecordsObserved(IntentRecordsObserved),
            ClarificationQuestionsObserved(ClarificationQuestionsObserved),
            PsycheStateSubscriptionOpened(PsycheStateSubscriptionOpened),
            IntentRecordSubscriptionOpened(IntentRecordSubscriptionOpened),
            SpiritRequestUnimplemented(SpiritRequestUnimplemented),
        }
        event SpiritEvent {
            PsycheStateChanged(PsycheStateChanged) belongs PsycheStateStream,
            IntentRecordCaptured(IntentRecordCaptured) belongs IntentRecordStream,
        }
        stream PsycheStateStream {
            token PsycheStateSubscriptionToken;
            opened PsycheStateSubscriptionOpened;
            event PsycheStateChanged;
            close PsycheStateSubscriptionRetraction;
        }
        stream IntentRecordStream {
            token IntentRecordSubscriptionToken;
            opened IntentRecordSubscriptionOpened;
            event IntentRecordCaptured;
            close IntentRecordSubscriptionRetraction;
        }
    }
}

pub type Frame = SpiritFrame;
pub type FrameBody = SpiritFrameBody;
pub type ChannelRequest = SpiritChannelRequest;
pub type ChannelReply = SpiritChannelReply;
pub type RequestBuilder = SpiritRequestBuilder;

impl SpiritRequest {
    pub fn operation_kind(&self) -> SpiritOperationKind {
        match self {
            Self::PsycheStatement(_) => SpiritOperationKind::PsycheStatement,
            Self::PsycheStateObservation(_) => SpiritOperationKind::PsycheStateObservation,
            Self::IntentRecordObservation(_) => SpiritOperationKind::IntentRecordObservation,
            Self::ClarificationQuestionPending(_) => {
                SpiritOperationKind::ClarificationQuestionPending
            }
            Self::SubscribePsycheState(_) => SpiritOperationKind::SubscribePsycheState,
            Self::PsycheStateSubscriptionRetraction(_) => {
                SpiritOperationKind::PsycheStateSubscriptionRetraction
            }
            Self::SubscribeIntentRecords(_) => SpiritOperationKind::SubscribeIntentRecords,
            Self::IntentRecordSubscriptionRetraction(_) => {
                SpiritOperationKind::IntentRecordSubscriptionRetraction
            }
        }
    }
}

impl From<PsycheStatementAccepted> for SpiritReply {
    fn from(payload: PsycheStatementAccepted) -> Self {
        Self::PsycheStatementAccepted(payload)
    }
}

impl From<PsycheStateObserved> for SpiritReply {
    fn from(payload: PsycheStateObserved) -> Self {
        Self::PsycheStateObserved(payload)
    }
}

impl From<IntentRecordsObserved> for SpiritReply {
    fn from(payload: IntentRecordsObserved) -> Self {
        Self::IntentRecordsObserved(payload)
    }
}

impl From<ClarificationQuestionsObserved> for SpiritReply {
    fn from(payload: ClarificationQuestionsObserved) -> Self {
        Self::ClarificationQuestionsObserved(payload)
    }
}

impl From<PsycheStateSubscriptionOpened> for SpiritReply {
    fn from(payload: PsycheStateSubscriptionOpened) -> Self {
        Self::PsycheStateSubscriptionOpened(payload)
    }
}

impl From<IntentRecordSubscriptionOpened> for SpiritReply {
    fn from(payload: IntentRecordSubscriptionOpened) -> Self {
        Self::IntentRecordSubscriptionOpened(payload)
    }
}

impl From<SpiritRequestUnimplemented> for SpiritReply {
    fn from(payload: SpiritRequestUnimplemented) -> Self {
        Self::SpiritRequestUnimplemented(payload)
    }
}
