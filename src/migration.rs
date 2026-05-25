//! Adjacent-version projection witnesses for the Spirit contract.

use signal_sema::Magnitude;
use version_projection::{ProjectionError, VersionProjection};

use crate::{Description, Entry, Kind, Operation, Statement, Topic};

pub mod v010 {
    use nota_codec::{NotaEnum, NotaRecord, NotaTransparent};
    use rkyv::{Archive, Deserialize as RkyvDeserialize, Serialize as RkyvSerialize};

    #[derive(
        Archive, RkyvSerialize, RkyvDeserialize, NotaTransparent, Debug, Clone, PartialEq, Eq, Hash,
    )]
    pub struct Topic(String);

    impl Topic {
        pub fn new(value: impl Into<String>) -> Self {
            Self(value.into())
        }

        pub fn into_current(self) -> crate::Topic {
            crate::Topic::new(self.0)
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

        pub fn into_description(self) -> crate::Description {
            crate::Description::new(self.0)
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
    }

    #[derive(
        Archive, RkyvSerialize, RkyvDeserialize, NotaTransparent, Debug, Clone, PartialEq, Eq, Hash,
    )]
    pub struct Quote(String);

    impl Quote {
        pub fn new(value: impl Into<String>) -> Self {
            Self(value.into())
        }
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

    impl From<Kind> for crate::Kind {
        fn from(value: Kind) -> Self {
            match value {
                Kind::Decision => Self::Decision,
                Kind::Principle => Self::Principle,
                Kind::Correction => Self::Correction,
                Kind::Clarification => Self::Clarification,
                Kind::Constraint => Self::Constraint,
            }
        }
    }

    #[derive(
        Archive, RkyvSerialize, RkyvDeserialize, NotaEnum, Debug, Clone, Copy, PartialEq, Eq, Hash,
    )]
    pub enum Certainty {
        Maximum,
        Medium,
        Minimum,
    }

    #[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaRecord, Debug, Clone, PartialEq, Eq)]
    pub struct Entry {
        pub topic: Topic,
        pub kind: Kind,
        pub summary: Summary,
        pub context: Context,
        pub certainty: Certainty,
        pub quote: Quote,
    }

    #[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaEnum, Debug, Clone, PartialEq, Eq)]
    pub enum Operation {
        Record(Entry),
    }
}

pub struct V010ToV011;

impl From<v010::Certainty> for Magnitude {
    fn from(value: v010::Certainty) -> Self {
        match value {
            v010::Certainty::Maximum => Self::Maximum,
            v010::Certainty::Medium => Self::Medium,
            v010::Certainty::Minimum => Self::Minimum,
        }
    }
}

impl VersionProjection<v010::Entry, Entry> for V010ToV011 {
    type Error = ProjectionError;

    fn project(source: v010::Entry) -> Result<Entry, Self::Error> {
        Ok(Entry {
            topic: source.topic.into_current(),
            kind: source.kind.into(),
            description: source.summary.into_description(),
            certainty: source.certainty.into(),
        })
    }
}

impl VersionProjection<v010::Operation, Operation> for V010ToV011 {
    type Error = ProjectionError;

    fn project(source: v010::Operation) -> Result<Operation, Self::Error> {
        match source {
            v010::Operation::Record(entry) => Ok(Operation::Record(<Self as VersionProjection<
                v010::Entry,
                Entry,
            >>::project(entry)?)),
        }
    }
}

impl VersionProjection<Statement, Statement> for V010ToV011 {
    type Error = std::convert::Infallible;

    fn project(source: Statement) -> Result<Statement, Self::Error> {
        Ok(source)
    }
}

impl VersionProjection<Topic, Topic> for V010ToV011 {
    type Error = std::convert::Infallible;

    fn project(source: Topic) -> Result<Topic, Self::Error> {
        Ok(source)
    }
}

impl VersionProjection<Kind, Kind> for V010ToV011 {
    type Error = std::convert::Infallible;

    fn project(source: Kind) -> Result<Kind, Self::Error> {
        Ok(source)
    }
}

impl VersionProjection<v010::Summary, Description> for V010ToV011 {
    type Error = std::convert::Infallible;

    fn project(source: v010::Summary) -> Result<Description, Self::Error> {
        Ok(source.into_description())
    }
}
