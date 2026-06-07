//! Signal contract for the ordinary `persona-spirit` surface.
//!
//! This crate carries the peer-callable vocabulary for psyche statements,
//! psyche-state observations, intent-record observations, and subscriptions.
//! Runtime actors, sockets, storage, classifier logic, and downstream
//! meta-policy forwarding live in `persona-spirit`.

use nota_codec::{
    Decoder, Encoder, NotaDecode, NotaEncode, NotaEnum, NotaRecord, NotaTransparent, Token,
};
use rkyv::{Archive, Deserialize as RkyvDeserialize, Serialize as RkyvSerialize};
use signal_frame::signal_channel;
use signal_sema::{Magnitude, SemaObservation};

pub mod migration;

const RECORD_IDENTIFIER_BYTES: usize = 12;
const RECORD_IDENTIFIER_MINIMUM_CODE_LENGTH: usize = 4;
const RECORD_IDENTIFIER_RADIX: u128 = 36;

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

    pub fn contains_any(&self, topics: &Topics) -> bool {
        topics.as_slice().iter().any(|topic| self.contains(topic))
    }

    pub fn contains_all(&self, topics: &Topics) -> bool {
        topics.as_slice().iter().all(|topic| self.contains(topic))
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
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
)]
pub struct RecordIdentifier([u8; RECORD_IDENTIFIER_BYTES]);

impl RecordIdentifier {
    pub const fn new(value: u64) -> Self {
        let octets = value.to_be_bytes();
        Self([
            0, 0, 0, 0, octets[0], octets[1], octets[2], octets[3], octets[4], octets[5],
            octets[6], octets[7],
        ])
    }

    pub const fn from_bytes(bytes: [u8; RECORD_IDENTIFIER_BYTES]) -> Self {
        Self(bytes)
    }

    pub const fn bytes(self) -> [u8; RECORD_IDENTIFIER_BYTES] {
        self.0
    }

    pub fn value(self) -> u128 {
        u128::from_be_bytes([
            0, 0, 0, 0, self.0[0], self.0[1], self.0[2], self.0[3], self.0[4], self.0[5],
            self.0[6], self.0[7], self.0[8], self.0[9], self.0[10], self.0[11],
        ])
    }

    pub fn code(self) -> String {
        RecordIdentifierCode::from_identifier(self).into_string()
    }

    pub fn from_code(code: &str) -> nota_codec::Result<Self> {
        RecordIdentifierCode::new(code).into_identifier()
    }
}

impl NotaEncode for RecordIdentifier {
    fn encode(&self, encoder: &mut Encoder) -> nota_codec::Result<()> {
        encoder.write_string(&self.code())
    }
}

impl NotaDecode for RecordIdentifier {
    fn decode(decoder: &mut Decoder<'_>) -> nota_codec::Result<Self> {
        Self::from_code(&decoder.read_string()?)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RecordIdentifierCode {
    value: String,
}

impl RecordIdentifierCode {
    fn new(value: impl Into<String>) -> Self {
        Self {
            value: value.into(),
        }
    }

    fn from_identifier(identifier: RecordIdentifier) -> Self {
        let mut value = identifier.value();
        if value == 0 {
            return Self::new("0".repeat(RECORD_IDENTIFIER_MINIMUM_CODE_LENGTH));
        }

        let mut digits = Vec::new();
        while value > 0 {
            let digit = (value % RECORD_IDENTIFIER_RADIX) as u8;
            digits.push(Self::digit_character(digit));
            value /= RECORD_IDENTIFIER_RADIX;
        }
        while digits.len() < RECORD_IDENTIFIER_MINIMUM_CODE_LENGTH {
            digits.push('0');
        }
        digits.reverse();
        Self::new(digits.into_iter().collect::<String>())
    }

    fn into_string(self) -> String {
        self.value
    }

    fn into_identifier(self) -> nota_codec::Result<RecordIdentifier> {
        if self.value.len() < RECORD_IDENTIFIER_MINIMUM_CODE_LENGTH {
            return Err(nota_codec::Error::Validation {
                type_name: "RecordIdentifier",
                message: format!(
                    "record identifier code must be at least {RECORD_IDENTIFIER_MINIMUM_CODE_LENGTH} characters"
                ),
            });
        }

        let mut value = 0_u128;
        for character in self.value.chars() {
            let digit = Self::digit_value(character)?;
            value = value
                .checked_mul(RECORD_IDENTIFIER_RADIX)
                .and_then(|accumulated| accumulated.checked_add(digit))
                .ok_or_else(|| nota_codec::Error::Validation {
                    type_name: "RecordIdentifier",
                    message: "record identifier exceeds 96-bit range".to_string(),
                })?;
        }

        let bytes = value.to_be_bytes();
        if bytes[0..4] != [0, 0, 0, 0] {
            return Err(nota_codec::Error::Validation {
                type_name: "RecordIdentifier",
                message: "record identifier exceeds 96-bit range".to_string(),
            });
        }

        Ok(RecordIdentifier::from_bytes([
            bytes[4], bytes[5], bytes[6], bytes[7], bytes[8], bytes[9], bytes[10], bytes[11],
            bytes[12], bytes[13], bytes[14], bytes[15],
        ]))
    }

    fn digit_character(value: u8) -> char {
        match value {
            0..=9 => (b'0' + value) as char,
            10..=35 => (b'a' + (value - 10)) as char,
            _ => unreachable!("base36 digit outside alphabet"),
        }
    }

    fn digit_value(character: char) -> nota_codec::Result<u128> {
        match character {
            '0'..='9' => Ok((character as u8 - b'0') as u128),
            'a'..='z' => Ok((character as u8 - b'a' + 10) as u128),
            _ => Err(nota_codec::Error::Validation {
                type_name: "RecordIdentifier",
                message: format!(
                    "record identifier code uses unsupported character {character:?}; use lowercase base36"
                ),
            }),
        }
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

#[derive(
    Archive,
    RkyvSerialize,
    RkyvDeserialize,
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
)]
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

#[derive(
    Archive,
    RkyvSerialize,
    RkyvDeserialize,
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
)]
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
    Archive,
    RkyvSerialize,
    RkyvDeserialize,
    NotaRecord,
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
)]
pub struct RecordedTime {
    pub date: Date,
    pub time: Time,
}

impl RecordedTime {
    pub const fn new(date: Date, time: Time) -> Self {
        Self { date, time }
    }
}

#[derive(
    Archive, RkyvSerialize, RkyvDeserialize, NotaRecord, Debug, Clone, Copy, PartialEq, Eq, Hash,
)]
pub struct RecordedTimeRange {
    pub first: RecordedTime,
    pub last: RecordedTime,
}

impl RecordedTimeRange {
    pub const fn new(first: RecordedTime, last: RecordedTime) -> Self {
        Self { first, last }
    }

    pub fn contains(self, recorded_time: RecordedTime) -> bool {
        recorded_time >= self.first && recorded_time <= self.last
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

#[derive(
    Archive, RkyvSerialize, RkyvDeserialize, NotaTransparent, Debug, Clone, PartialEq, Eq, Hash,
)]
pub struct ArchivePath(String);

impl ArchivePath {
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

#[derive(Archive, RkyvSerialize, RkyvDeserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ObservationMode {
    SummaryOnly,
    WithProvenance,
}

impl NotaEncode for ObservationMode {
    fn encode(&self, encoder: &mut Encoder) -> nota_codec::Result<()> {
        match self {
            Self::SummaryOnly => encoder.write_pascal_identifier("SummaryOnly"),
            Self::WithProvenance => encoder.write_pascal_identifier("WithProvenance"),
        }
    }
}

impl NotaDecode for ObservationMode {
    fn decode(decoder: &mut Decoder<'_>) -> nota_codec::Result<Self> {
        match decoder.read_pascal_identifier()?.as_str() {
            "SummaryOnly" | "DescriptionOnly" => Ok(Self::SummaryOnly),
            "WithProvenance" => Ok(Self::WithProvenance),
            other => Err(nota_codec::Error::UnknownVariant {
                enum_name: "ObservationMode",
                got: other.to_string(),
            }),
        }
    }
}

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

pub type Certainty = Magnitude;
pub type Privacy = Magnitude;

#[derive(Archive, RkyvSerialize, RkyvDeserialize, Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    pub topics: Topics,
    pub kind: Kind,
    pub description: Description,
    pub certainty: Certainty,
    pub privacy: Privacy,
}

impl Entry {
    pub fn open(
        topics: Topics,
        kind: Kind,
        description: Description,
        certainty: Certainty,
    ) -> Self {
        Self {
            topics,
            kind,
            description,
            certainty,
            privacy: Magnitude::Zero,
        }
    }
}

impl NotaEncode for Entry {
    fn encode(&self, encoder: &mut Encoder) -> nota_codec::Result<()> {
        encoder.start_record_untagged()?;
        self.topics.encode(encoder)?;
        self.kind.encode(encoder)?;
        self.description.encode(encoder)?;
        self.certainty.encode(encoder)?;
        self.privacy.encode(encoder)?;
        encoder.end_record()
    }
}

impl NotaDecode for Entry {
    fn decode(decoder: &mut Decoder<'_>) -> nota_codec::Result<Self> {
        decoder.expect_positional_record_start("Entry", 5)?;
        let topics = Topics::decode(decoder)?;
        let kind = Kind::decode(decoder)?;
        let description = Description::decode(decoder)?;
        let certainty = Certainty::decode(decoder)?;
        let privacy = if decoder.peek_is_record_end()? {
            Magnitude::Zero
        } else {
            Magnitude::decode(decoder)?
        };
        decoder.expect_record_end()?;
        Ok(Self {
            topics,
            kind,
            description,
            certainty,
            privacy,
        })
    }
}

#[derive(
    Archive, RkyvSerialize, RkyvDeserialize, NotaRecord, Debug, Clone, Copy, PartialEq, Eq,
)]
pub struct CertaintyChange {
    pub identifier: RecordIdentifier,
    pub certainty: Certainty,
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct RecordChange {
    pub record_identifier: RecordIdentifier,
    pub entry: Entry,
}

impl RecordChange {
    pub const fn identifier(&self) -> RecordIdentifier {
        self.record_identifier
    }
}

#[derive(
    Archive, RkyvSerialize, RkyvDeserialize, NotaEnum, Debug, Clone, Copy, PartialEq, Eq, Hash,
)]
pub enum MatchKind {
    Any,
    Partial,
    Full,
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, Debug, Clone, PartialEq, Eq)]
pub struct TopicSelection {
    pub match_kind: MatchKind,
    pub topics: Vec<Topic>,
}

impl TopicSelection {
    pub fn any() -> Self {
        Self {
            match_kind: MatchKind::Any,
            topics: Vec::new(),
        }
    }

    pub fn partial(topics: Vec<Topic>) -> Self {
        Self {
            match_kind: MatchKind::Partial,
            topics,
        }
    }

    pub fn full(topics: Vec<Topic>) -> Self {
        Self {
            match_kind: MatchKind::Full,
            topics,
        }
    }

    pub fn matches(&self, topics: &Topics) -> bool {
        match self.match_kind {
            MatchKind::Any => true,
            MatchKind::Partial => self.topics.iter().any(|topic| topics.contains(topic)),
            MatchKind::Full => {
                !self.topics.is_empty() && self.topics.iter().all(|topic| topics.contains(topic))
            }
        }
    }

    fn validate(&self) -> nota_codec::Result<()> {
        match self.match_kind {
            MatchKind::Any if self.topics.is_empty() => Ok(()),
            MatchKind::Any => Err(nota_codec::Error::Validation {
                type_name: "TopicSelection",
                message: "Any topic selection must not carry topics".to_string(),
            }),
            MatchKind::Partial | MatchKind::Full if self.topics.is_empty() => {
                Err(nota_codec::Error::Validation {
                    type_name: "TopicSelection",
                    message: "Partial and Full topic selections must carry at least one topic"
                        .to_string(),
                })
            }
            MatchKind::Partial | MatchKind::Full => {
                let mut seen = std::collections::BTreeSet::<&str>::new();
                for topic in &self.topics {
                    if !seen.insert(topic.as_str()) {
                        return Err(nota_codec::Error::Validation {
                            type_name: "TopicSelection",
                            message: format!("topic selection repeats topic {}", topic.as_str()),
                        });
                    }
                }
                Ok(())
            }
        }
    }
}

impl NotaEncode for TopicSelection {
    fn encode(&self, encoder: &mut Encoder) -> nota_codec::Result<()> {
        self.validate()?;
        encoder.start_record_untagged()?;
        self.match_kind.encode(encoder)?;
        self.topics.encode(encoder)?;
        encoder.end_record()
    }
}

impl NotaDecode for TopicSelection {
    fn decode(decoder: &mut Decoder<'_>) -> nota_codec::Result<Self> {
        decoder.expect_positional_record_start("TopicSelection", 2)?;
        let match_kind = MatchKind::decode(decoder)?;
        let topics = Vec::<Topic>::decode(decoder)?;
        decoder.expect_record_end()?;
        let selection = Self { match_kind, topics };
        selection.validate()?;
        Ok(selection)
    }
}

#[derive(
    Archive, RkyvSerialize, RkyvDeserialize, NotaEnum, Debug, Clone, Copy, PartialEq, Eq, Hash,
)]
pub enum CertaintySelection {
    Any,
    Exact(Certainty),
    AtMost(Certainty),
    AtLeast(Certainty),
}

impl CertaintySelection {
    pub const fn removal_candidates() -> Self {
        Self::Exact(Magnitude::Zero)
    }

    pub fn matches(self, certainty: Certainty) -> bool {
        match self {
            Self::Any => true,
            Self::Exact(expected) => certainty == expected,
            Self::AtMost(maximum) => certainty <= maximum,
            Self::AtLeast(minimum) => certainty >= minimum,
        }
    }
}

#[derive(
    Archive, RkyvSerialize, RkyvDeserialize, NotaEnum, Debug, Clone, Copy, PartialEq, Eq, Hash,
)]
pub enum PrivacySelection {
    Any,
    Exact(Privacy),
    AtMost(Privacy),
    AtLeast(Privacy),
}

impl PrivacySelection {
    pub const fn default_observation_privacy() -> Self {
        Self::Exact(Magnitude::Zero)
    }

    pub fn matches(self, privacy: Privacy) -> bool {
        match self {
            Self::Any => true,
            Self::Exact(expected) => privacy == expected,
            Self::AtMost(maximum) => privacy <= maximum,
            Self::AtLeast(minimum) => privacy >= minimum,
        }
    }
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaEnum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecordedTimeSelection {
    Any,
    Between(RecordedTimeRange),
    Since(RecordedTime),
    Until(RecordedTime),
    Recent,
    Shallow,
    Deep,
    VeryDeep,
}

impl RecordedTimeSelection {
    pub const fn any() -> Self {
        Self::Any
    }

    pub const fn recent() -> Self {
        Self::Recent
    }

    pub fn matches(self, recorded_time: RecordedTime) -> bool {
        match self {
            Self::Any | Self::Recent | Self::Shallow | Self::Deep | Self::VeryDeep => true,
            Self::Between(range) => range.contains(recorded_time),
            Self::Since(first) => recorded_time >= first,
            Self::Until(last) => recorded_time <= last,
        }
    }
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, Debug, Clone, PartialEq, Eq)]
pub struct RecordQuery {
    pub topic_selection: TopicSelection,
    pub kind: Option<Kind>,
    pub certainty_selection: CertaintySelection,
    pub recorded_time_selection: RecordedTimeSelection,
    pub privacy_selection: PrivacySelection,
    pub mode: ObservationMode,
}

impl RecordQuery {
    pub fn removal_candidates(mode: ObservationMode) -> Self {
        Self {
            topic_selection: TopicSelection::any(),
            kind: None,
            certainty_selection: CertaintySelection::removal_candidates(),
            recorded_time_selection: RecordedTimeSelection::Any,
            privacy_selection: PrivacySelection::default_observation_privacy(),
            mode,
        }
    }
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, Debug, Clone, PartialEq, Eq)]
pub struct PublicRecordQuery {
    pub topic_selection: TopicSelection,
    pub kind: Option<Kind>,
    pub certainty_selection: CertaintySelection,
    pub recorded_time_selection: RecordedTimeSelection,
    pub mode: ObservationMode,
}

impl PublicRecordQuery {
    pub fn new(
        topic_selection: TopicSelection,
        kind: Option<Kind>,
        certainty_selection: CertaintySelection,
        recorded_time_selection: RecordedTimeSelection,
        mode: ObservationMode,
    ) -> Self {
        Self {
            topic_selection,
            kind,
            certainty_selection,
            recorded_time_selection,
            mode,
        }
    }

    pub fn any(mode: ObservationMode) -> Self {
        Self::new(
            TopicSelection::any(),
            None,
            CertaintySelection::Any,
            RecordedTimeSelection::Any,
            mode,
        )
    }

    pub fn removal_candidates(mode: ObservationMode) -> Self {
        Self::new(
            TopicSelection::any(),
            None,
            CertaintySelection::removal_candidates(),
            RecordedTimeSelection::Any,
            mode,
        )
    }

    pub fn into_record_query(self) -> RecordQuery {
        RecordQuery {
            topic_selection: self.topic_selection,
            kind: self.kind,
            certainty_selection: self.certainty_selection,
            recorded_time_selection: self.recorded_time_selection,
            privacy_selection: PrivacySelection::default_observation_privacy(),
            mode: self.mode,
        }
    }
}

impl From<PublicRecordQuery> for RecordQuery {
    fn from(query: PublicRecordQuery) -> Self {
        query.into_record_query()
    }
}

impl NotaEncode for PublicRecordQuery {
    fn encode(&self, encoder: &mut Encoder) -> nota_codec::Result<()> {
        encoder.start_record_untagged()?;
        self.topic_selection.encode(encoder)?;
        self.kind.encode(encoder)?;
        self.certainty_selection.encode(encoder)?;
        self.recorded_time_selection.encode(encoder)?;
        self.mode.encode(encoder)?;
        encoder.end_record()
    }
}

impl NotaDecode for PublicRecordQuery {
    fn decode(decoder: &mut Decoder<'_>) -> nota_codec::Result<Self> {
        decoder.expect_positional_record_start("PublicRecordQuery", 5)?;
        let topic_selection = TopicSelection::decode(decoder)?;
        let kind = Option::<Kind>::decode(decoder)?;
        let next = decoder.peek_token()?;
        let (certainty_selection, recorded_time_selection, mode) = match next {
            Some(Token::Ident(name))
                if name == "SummaryOnly"
                    || name == "WithProvenance"
                    || name == "DescriptionOnly" =>
            {
                (
                    CertaintySelection::Any,
                    RecordedTimeSelection::Any,
                    ObservationMode::decode(decoder)?,
                )
            }
            _ => {
                let certainty_selection = CertaintySelection::decode(decoder)?;
                let next = decoder.peek_token()?;
                let (recorded_time_selection, mode) = match next {
                    Some(Token::Ident(name))
                        if name == "SummaryOnly"
                            || name == "WithProvenance"
                            || name == "DescriptionOnly" =>
                    {
                        (
                            RecordedTimeSelection::Any,
                            ObservationMode::decode(decoder)?,
                        )
                    }
                    _ => {
                        let recorded_time_selection = RecordedTimeSelection::decode(decoder)?;
                        let next = decoder.peek_token()?;
                        let mode = match next {
                            Some(Token::Ident(name))
                                if name == "SummaryOnly"
                                    || name == "WithProvenance"
                                    || name == "DescriptionOnly" =>
                            {
                                ObservationMode::decode(decoder)?
                            }
                            _ => {
                                let privacy_selection = PrivacySelection::decode(decoder)?;
                                if privacy_selection
                                    != PrivacySelection::default_observation_privacy()
                                {
                                    return Err(nota_codec::Error::Validation {
                                        type_name: "PublicRecordQuery",
                                        message:
                                            "public record queries cannot carry elevated privacy"
                                                .to_string(),
                                    });
                                }
                                ObservationMode::decode(decoder)?
                            }
                        };
                        (recorded_time_selection, mode)
                    }
                };
                (certainty_selection, recorded_time_selection, mode)
            }
        };
        decoder.expect_record_end()?;
        Ok(Self {
            topic_selection,
            kind,
            certainty_selection,
            recorded_time_selection,
            mode,
        })
    }
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct PrivacyScopedRecordQuery {
    pub privacy_selection: PrivacySelection,
    pub public_record_query: PublicRecordQuery,
}

impl PrivacyScopedRecordQuery {
    pub fn new(privacy_selection: PrivacySelection, query: PublicRecordQuery) -> Self {
        Self {
            privacy_selection,
            public_record_query: query,
        }
    }

    pub fn at_most(privacy: Privacy, query: PublicRecordQuery) -> Self {
        Self::new(PrivacySelection::AtMost(privacy), query)
    }

    pub fn into_record_query(self) -> RecordQuery {
        let mut query = self.public_record_query.into_record_query();
        query.privacy_selection = self.privacy_selection;
        query
    }
}

impl NotaEncode for RecordQuery {
    fn encode(&self, encoder: &mut Encoder) -> nota_codec::Result<()> {
        encoder.start_record_untagged()?;
        self.topic_selection.encode(encoder)?;
        self.kind.encode(encoder)?;
        self.certainty_selection.encode(encoder)?;
        self.recorded_time_selection.encode(encoder)?;
        self.privacy_selection.encode(encoder)?;
        self.mode.encode(encoder)?;
        encoder.end_record()
    }
}

impl NotaDecode for RecordQuery {
    fn decode(decoder: &mut Decoder<'_>) -> nota_codec::Result<Self> {
        decoder.expect_positional_record_start("RecordQuery", 6)?;
        let topic_selection = TopicSelection::decode(decoder)?;
        let kind = Option::<Kind>::decode(decoder)?;
        let next = decoder.peek_token()?;
        let (certainty_selection, recorded_time_selection, privacy_selection, mode) = match next {
            Some(Token::Ident(name))
                if name == "SummaryOnly"
                    || name == "WithProvenance"
                    || name == "DescriptionOnly" =>
            {
                (
                    CertaintySelection::Any,
                    RecordedTimeSelection::Any,
                    PrivacySelection::default_observation_privacy(),
                    ObservationMode::decode(decoder)?,
                )
            }
            _ => {
                let certainty_selection = CertaintySelection::decode(decoder)?;
                let next = decoder.peek_token()?;
                let (recorded_time_selection, privacy_selection, mode) = match next {
                    Some(Token::Ident(name))
                        if name == "SummaryOnly"
                            || name == "WithProvenance"
                            || name == "DescriptionOnly" =>
                    {
                        (
                            RecordedTimeSelection::Any,
                            PrivacySelection::default_observation_privacy(),
                            ObservationMode::decode(decoder)?,
                        )
                    }
                    _ => {
                        let recorded_time_selection = RecordedTimeSelection::decode(decoder)?;
                        let next = decoder.peek_token()?;
                        let (privacy_selection, mode) = match next {
                            Some(Token::Ident(name))
                                if name == "SummaryOnly"
                                    || name == "WithProvenance"
                                    || name == "DescriptionOnly" =>
                            {
                                (
                                    PrivacySelection::default_observation_privacy(),
                                    ObservationMode::decode(decoder)?,
                                )
                            }
                            _ => (
                                PrivacySelection::decode(decoder)?,
                                ObservationMode::decode(decoder)?,
                            ),
                        };
                        (recorded_time_selection, privacy_selection, mode)
                    }
                };
                (
                    certainty_selection,
                    recorded_time_selection,
                    privacy_selection,
                    mode,
                )
            }
        };
        decoder.expect_record_end()?;
        Ok(Self {
            topic_selection,
            kind,
            certainty_selection,
            recorded_time_selection,
            privacy_selection,
            mode,
        })
    }
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaEnum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecordIdentifierSelection {
    Exact(RecordIdentifier),
}

impl RecordIdentifierSelection {
    pub fn contains(self, identifier: RecordIdentifier) -> bool {
        match self {
            Self::Exact(expected) => identifier == expected,
        }
    }
}

#[derive(
    Archive, RkyvSerialize, RkyvDeserialize, NotaRecord, Debug, Clone, Copy, PartialEq, Eq,
)]
pub struct RecordIdentifierQuery {
    pub record_identifier_selection: RecordIdentifierSelection,
    pub mode: ObservationMode,
}

impl RecordIdentifierQuery {
    pub const fn new(
        record_identifier_selection: RecordIdentifierSelection,
        mode: ObservationMode,
    ) -> Self {
        Self {
            record_identifier_selection,
            mode,
        }
    }

    pub fn contains(self, identifier: RecordIdentifier) -> bool {
        self.record_identifier_selection.contains(identifier)
    }
}

#[derive(
    Archive, RkyvSerialize, RkyvDeserialize, NotaRecord, Debug, Clone, Copy, PartialEq, Eq,
)]
pub struct PrivacyScopedRecordIdentifierQuery {
    pub privacy_selection: PrivacySelection,
    pub record_identifier_query: RecordIdentifierQuery,
}

impl PrivacyScopedRecordIdentifierQuery {
    pub const fn new(privacy_selection: PrivacySelection, query: RecordIdentifierQuery) -> Self {
        Self {
            privacy_selection,
            record_identifier_query: query,
        }
    }

    pub const fn at_most(privacy: Privacy, query: RecordIdentifierQuery) -> Self {
        Self::new(PrivacySelection::AtMost(privacy), query)
    }
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct RecordObservation {
    pub query: RecordQuery,
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaEnum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputStream {
    StandardOutput,
    StandardError,
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaEnum, Debug, Clone, PartialEq, Eq)]
pub enum ArchiveDatabaseTarget {
    Default,
    Path(ArchivePath),
}

impl ArchiveDatabaseTarget {
    pub fn path(path: impl Into<String>) -> Self {
        Self::Path(ArchivePath::new(path))
    }
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaEnum, Debug, Clone, PartialEq, Eq)]
pub enum OutputTarget {
    ArchiveDatabase(ArchiveDatabaseTarget),
    Print(OutputStream),
}

impl OutputTarget {
    pub const fn default_archive_database() -> Self {
        Self::ArchiveDatabase(ArchiveDatabaseTarget::Default)
    }

    pub fn archive_database(path: impl Into<String>) -> Self {
        Self::ArchiveDatabase(ArchiveDatabaseTarget::path(path))
    }

    pub const fn print_standard_output() -> Self {
        Self::Print(OutputStream::StandardOutput)
    }

    pub const fn print_standard_error() -> Self {
        Self::Print(OutputStream::StandardError)
    }
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct RemovalCandidateCollection {
    pub record_query: RecordQuery,
    pub output_target: OutputTarget,
}

impl RemovalCandidateCollection {
    pub fn new(record_query: RecordQuery, output_target: OutputTarget) -> Self {
        Self {
            record_query,
            output_target,
        }
    }

    pub fn default_archive_database() -> Self {
        Self::new(
            RecordQuery::removal_candidates(ObservationMode::SummaryOnly),
            OutputTarget::default_archive_database(),
        )
    }

    pub fn archive_database(path: impl Into<String>) -> Self {
        Self::new(
            RecordQuery::removal_candidates(ObservationMode::SummaryOnly),
            OutputTarget::archive_database(path),
        )
    }

    pub fn print_standard_output() -> Self {
        Self::new(
            RecordQuery::removal_candidates(ObservationMode::SummaryOnly),
            OutputTarget::print_standard_output(),
        )
    }

    pub fn print_standard_error() -> Self {
        Self::new(
            RecordQuery::removal_candidates(ObservationMode::SummaryOnly),
            OutputTarget::print_standard_error(),
        )
    }

    pub fn is_exact_zero_candidate_query(&self) -> bool {
        matches!(
            self.record_query.certainty_selection,
            CertaintySelection::Exact(Magnitude::Zero)
        ) && matches!(
            self.record_query.privacy_selection,
            PrivacySelection::Exact(Magnitude::Zero)
        )
    }
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct RecordSubscription {
    pub topic: Option<Topic>,
    pub mode: ObservationMode,
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct PrivacyScopedRecordSubscription {
    pub privacy_selection: PrivacySelection,
    pub record_subscription: RecordSubscription,
}

impl PrivacyScopedRecordSubscription {
    pub fn new(privacy_selection: PrivacySelection, subscription: RecordSubscription) -> Self {
        Self {
            privacy_selection,
            record_subscription: subscription,
        }
    }

    pub fn at_most(privacy: Privacy, subscription: RecordSubscription) -> Self {
        Self::new(PrivacySelection::AtMost(privacy), subscription)
    }
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct RecordSummary {
    pub identifier: RecordIdentifier,
    pub topics: Topics,
    pub kind: Kind,
    pub description: Description,
    pub certainty: Certainty,
    pub privacy: Privacy,
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct RecordProvenance {
    pub summary: RecordSummary,
    pub date: Date,
    pub time: Time,
}

#[derive(
    Archive, RkyvSerialize, RkyvDeserialize, NotaEnum, Debug, Clone, Copy, PartialEq, Eq, Hash,
)]
pub enum RemovalCandidateSkipReason {
    ArchiveFailed,
    RecordChanged,
    RecordAlreadyRemoved,
    NoLongerCandidate,
}

#[derive(
    Archive, RkyvSerialize, RkyvDeserialize, NotaRecord, Debug, Clone, Copy, PartialEq, Eq, Hash,
)]
pub struct SkippedRemovalCandidate {
    pub identifier: RecordIdentifier,
    pub reason: RemovalCandidateSkipReason,
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct RemovalCandidatesCollected {
    pub archived_records: Vec<RecordSummary>,
    pub removed_identifiers: Vec<RecordIdentifier>,
    pub skipped_candidates: Vec<SkippedRemovalCandidate>,
}

impl RemovalCandidatesCollected {
    pub fn new(
        archived_records: Vec<RecordSummary>,
        removed_identifiers: Vec<RecordIdentifier>,
        skipped_candidates: Vec<SkippedRemovalCandidate>,
    ) -> Self {
        Self {
            archived_records,
            removed_identifiers,
            skipped_candidates,
        }
    }

    pub fn archived_records(&self) -> &[RecordSummary] {
        &self.archived_records
    }

    pub fn removed_identifiers(&self) -> &[RecordIdentifier] {
        &self.removed_identifiers
    }

    pub fn skipped_candidates(&self) -> &[SkippedRemovalCandidate] {
        &self.skipped_candidates
    }
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

#[derive(
    Archive, RkyvSerialize, RkyvDeserialize, NotaTransparent, Debug, Clone, Copy, PartialEq, Eq,
)]
pub struct RecordRemoved(RecordIdentifier);

impl RecordRemoved {
    pub const fn new(identifier: RecordIdentifier) -> Self {
        Self(identifier)
    }

    pub const fn identifier(self) -> RecordIdentifier {
        self.0
    }
}

#[derive(
    Archive, RkyvSerialize, RkyvDeserialize, NotaTransparent, Debug, Clone, Copy, PartialEq, Eq,
)]
pub struct RecordMutationApplied(RecordIdentifier);

impl RecordMutationApplied {
    pub const fn new(identifier: RecordIdentifier) -> Self {
        Self(identifier)
    }

    pub const fn identifier(self) -> RecordIdentifier {
        self.0
    }
}

#[derive(
    Archive, RkyvSerialize, RkyvDeserialize, NotaRecord, Debug, Clone, Copy, PartialEq, Eq,
)]
pub struct CertaintyChanged {
    pub identifier: RecordIdentifier,
    pub certainty: Certainty,
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaTransparent, Debug, Clone, PartialEq, Eq)]
pub struct StateObserved(PresenceView);

impl StateObserved {
    pub fn new(state: PresenceView) -> Self {
        Self(state)
    }

    pub fn state(&self) -> &PresenceView {
        &self.0
    }

    pub fn into_state(self) -> PresenceView {
        self.0
    }
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaTransparent, Debug, Clone, PartialEq, Eq)]
pub struct RecordsObserved(Vec<RecordSummary>);

impl RecordsObserved {
    pub fn new(records: Vec<RecordSummary>) -> Self {
        Self(records)
    }

    pub fn records(&self) -> &[RecordSummary] {
        &self.0
    }

    pub fn into_records(self) -> Vec<RecordSummary> {
        self.0
    }
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaTransparent, Debug, Clone, PartialEq, Eq)]
pub struct RecordProvenancesObserved(Vec<RecordProvenance>);

impl RecordProvenancesObserved {
    pub fn new(records: Vec<RecordProvenance>) -> Self {
        Self(records)
    }

    pub fn records(&self) -> &[RecordProvenance] {
        &self.0
    }

    pub fn into_records(self) -> Vec<RecordProvenance> {
        self.0
    }
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaTransparent, Debug, Clone, PartialEq, Eq)]
pub struct TopicsObserved(Vec<TopicCount>);

impl TopicsObserved {
    pub fn new(topics: Vec<TopicCount>) -> Self {
        Self(topics)
    }

    pub fn topics(&self) -> &[TopicCount] {
        &self.0
    }

    pub fn into_topics(self) -> Vec<TopicCount> {
        self.0
    }
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaTransparent, Debug, Clone, PartialEq, Eq)]
pub struct QuestionsObserved(Vec<QuestionSummary>);

impl QuestionsObserved {
    pub fn new(questions: Vec<QuestionSummary>) -> Self {
        Self(questions)
    }

    pub fn questions(&self) -> &[QuestionSummary] {
        &self.0
    }

    pub fn into_questions(self) -> Vec<QuestionSummary> {
        self.0
    }
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaEnum, Debug, Clone, PartialEq, Eq)]
pub enum Observation {
    State,
    Records(PublicRecordQuery),
    PrivateRecords(PrivacyScopedRecordQuery),
    RecordIdentifiers(RecordIdentifierQuery),
    PrivateRecordIdentifiers(PrivacyScopedRecordIdentifierQuery),
    Topics,
    Questions,
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaEnum, Debug, Clone, PartialEq, Eq)]
pub enum Subscription {
    State,
    Records(RecordSubscription),
    PrivateRecords(PrivacyScopedRecordSubscription),
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaEnum, Debug, Clone, PartialEq, Eq)]
pub enum SubscriptionToken {
    State(StateSubscriptionToken),
    Records(RecordSubscriptionToken),
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaEnum, Debug, Clone, PartialEq, Eq)]
pub enum SubscriptionSnapshot {
    State(PresenceView),
    Records(Vec<RecordSummary>),
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
    pub record: RecordSummary,
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct OperationReceived {
    pub operation: OperationKind,
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct EffectEmitted {
    pub observation: SemaObservation,
}

signal_channel! {
    channel Spirit {
        operation State(Statement),
        operation Record(Entry),
        operation Observe(Observation),
        operation Watch(Subscription) opens DomainStream,
        operation Unwatch(SubscriptionToken),
        operation Remove(RecordIdentifier),
        operation ChangeRecord(RecordChange),
        operation ChangeCertainty(CertaintyChange),
        operation CollectRemovalCandidates(RemovalCandidateCollection),
    }
    reply Reply {
        RecordAccepted(RecordAccepted),
        RecordRemoved(RecordRemoved),
        RecordMutationApplied(RecordMutationApplied),
        StateObserved(StateObserved),
        RecordsObserved(RecordsObserved),
        RecordProvenancesObserved(RecordProvenancesObserved),
        TopicsObserved(TopicsObserved),
        QuestionsObserved(QuestionsObserved),
        SubscriptionOpened(SubscriptionOpened),
        SubscriptionRetracted(SubscriptionRetracted),
        RequestUnimplemented(RequestUnimplemented),
        CertaintyChanged(CertaintyChanged),
        RemovalCandidatesCollected(RemovalCandidatesCollected),
    }
    event Event {
        StateChanged(StateChanged) belongs DomainStream,
        RecordCaptured(RecordCaptured) belongs DomainStream,
    }
    stream DomainStream {
        token SubscriptionToken;
        opened SubscriptionOpened;
        event StateChanged;
        event RecordCaptured;
        close Unwatch;
    }
    observable {
        filter default;
        operation_event OperationReceived;
        effect_event EffectEmitted;
    }
}
