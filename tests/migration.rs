use nota_codec::{Decoder, NotaDecode};
use signal_persona_spirit::{Entry, Kind, Operation, migration::V010ToV011, migration::v010};
use signal_sema::Magnitude;
use version_projection::VersionProjection;

#[test]
fn v010_certainty_projects_to_v011_magnitude() {
    assert_eq!(
        Magnitude::from(v010::Certainty::Maximum),
        Magnitude::Maximum
    );
    assert_eq!(Magnitude::from(v010::Certainty::Medium), Magnitude::Medium);
    assert_eq!(
        Magnitude::from(v010::Certainty::Minimum),
        Magnitude::Minimum
    );
}

#[test]
fn v010_record_entry_projects_to_current_entry_shape() {
    let source = v010::Entry {
        topic: v010::Topic::new("workspace"),
        kind: v010::Kind::Decision,
        summary: v010::Summary::new("summary"),
        context: v010::Context::new("context"),
        certainty: v010::Certainty::Maximum,
        quote: v010::Quote::new("quote"),
    };

    let current = <V010ToV011 as VersionProjection<v010::Entry, Entry>>::project(source).unwrap();

    assert_eq!(current.topic.as_str(), "workspace");
    assert_eq!(current.kind, Kind::Decision);
    assert_eq!(current.summary.as_str(), "summary");
    assert_eq!(current.context.as_str(), "context");
    assert_eq!(current.certainty, Magnitude::Maximum);
    assert_eq!(current.quote.as_str(), "quote");
}

#[test]
fn v010_nota_record_projects_to_current_operation() {
    let mut decoder =
        Decoder::new("(Record (workspace Decision [summary] [context] Medium [quote]))");
    let source = v010::Operation::decode(&mut decoder).unwrap();

    let current =
        <V010ToV011 as VersionProjection<v010::Operation, Operation>>::project(source).unwrap();

    let Operation::Record(entry) = current else {
        panic!("expected current Record operation");
    };
    assert_eq!(entry.certainty, Magnitude::Medium);
    assert_eq!(entry.summary.as_str(), "summary");
}
