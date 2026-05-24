use signal_frame::{
    ExchangeIdentifier, ExchangeLane, LaneSequence, SessionEpoch, ShortHeader,
    short_header_from_length_prefixed,
};
use signal_persona_spirit::{
    Context, Entry, Frame, FrameBody, Kind, Observation, ObservationMode, Operation, OperationKind,
    Quote, RecordQuery, Statement, StatementText, Summary, Topic,
};
use signal_sema::Magnitude;

fn exchange() -> ExchangeIdentifier {
    ExchangeIdentifier::new(
        SessionEpoch::new(1),
        ExchangeLane::Connector,
        LaneSequence::first(),
    )
}

fn entry() -> Entry {
    Entry {
        topic: Topic::new("workspace"),
        kind: Kind::Decision,
        summary: Summary::new("schema header"),
        context: Context::new("schema-derived short header"),
        certainty: Magnitude::Maximum,
        quote: Quote::new("header witness"),
    }
}

fn header(bytes: [u8; 8]) -> ShortHeader {
    ShortHeader::from_le_bytes(bytes)
}

#[test]
fn record_request_short_header_is_schema_derived_and_peekable() {
    let expected = header([1, 0, 6, 0, 0, 0, 0, 0]);
    let frame = Operation::Record(entry()).into_frame(exchange());

    assert_eq!(frame.short_header(), expected);

    let bytes = frame.encode_length_prefixed().expect("encode");
    assert_eq!(short_header_from_length_prefixed(&bytes).unwrap(), expected);

    let decoded = Frame::decode_length_prefixed(&bytes).expect("decode");
    assert_eq!(decoded.short_header(), expected);
    match decoded.into_body() {
        FrameBody::Request { request, .. } => {
            assert_eq!(request.payloads().head().kind(), OperationKind::Record);
        }
        other => panic!("expected request frame, got {other:?}"),
    }
}

#[test]
fn receive_side_triage_matches_header_root_before_body_decode() {
    let statement = Statement {
        text: StatementText::new("capture this intent"),
    };
    let state_frame = Operation::State(statement).into_frame(exchange());
    let record_frame = Operation::Record(entry()).into_frame(exchange());

    assert_eq!(
        Operation::kind_from_short_header(state_frame.short_header()),
        Some(OperationKind::State)
    );
    assert_eq!(
        Operation::kind_from_short_header(record_frame.short_header()),
        Some(OperationKind::Record)
    );
    assert_eq!(
        Operation::kind_from_short_header(header([99, 0, 0, 0, 0, 0, 0, 0])),
        None
    );
}

#[test]
fn nested_query_shape_sets_sub_enum_slots() {
    let frame = Operation::Observe(Observation::Records(RecordQuery {
        topic: None,
        kind: None,
        mode: ObservationMode::WithProvenance,
    }))
    .into_frame(exchange());

    assert_eq!(frame.short_header(), header([2, 1, 1, 0, 0, 0, 0, 0]));
}
