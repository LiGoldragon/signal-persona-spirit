use nota_codec::{Decoder, Encoder, NotaDecode, NotaEncode};
use signal_core::{
    ExchangeIdentifier, ExchangeLane, LaneSequence, NonEmpty, Reply, RequestPayload, SessionEpoch,
    SignalVerb, StreamEventIdentifier, SubReply, SubscriptionTokenInner,
};
use signal_persona_spirit::{
    Certainty, Context, Entry, FocusArea, Frame, FrameBody, Kind, ObservationMode, OperationKind,
    Presence, QuestionIdentifier, QuestionPending, QuestionSummary, QuestionText,
    QuestionsObserved, Quote, RecordAccepted, RecordCaptured, RecordIdentifier, RecordObservation,
    RecordProvenance, RecordProvenancesObserved, RecordQuery, RecordSubscription,
    RecordSubscriptionOpened, RecordSubscriptionRetracted, RecordSubscriptionToken,
    RecordsObserved, RequestUnimplemented, SpiritEvent, SpiritReply, SpiritRequest, State,
    StateChanged, StateObservation, StateObserved, StateSubscription, StateSubscriptionOpened,
    StateSubscriptionRetracted, StateSubscriptionToken, Statement, StatementText, Summary,
    Timestamp, Topic, UnimplementedReason,
};

const CANONICAL: &str = include_str!("../examples/canonical.nota");

fn exchange() -> ExchangeIdentifier {
    ExchangeIdentifier::new(
        SessionEpoch::new(1),
        ExchangeLane::Connector,
        LaneSequence::first(),
    )
}

fn summary() -> signal_persona_spirit::RecordSummary {
    signal_persona_spirit::RecordSummary {
        identifier: RecordIdentifier::new(1),
        topic: Topic::new("workspace"),
        kind: Kind::Decision,
        summary: Summary::new("summary only"),
        certainty: Certainty::Maximum,
    }
}

fn provenance() -> RecordProvenance {
    RecordProvenance {
        summary: summary(),
        context: Context::new("current implementation context"),
        timestamp: Timestamp::new("2026-05-19T13:08:11Z"),
        quote: Quote::new("first statement"),
    }
}

fn entry() -> Entry {
    Entry {
        topic: Topic::new("workspace"),
        kind: Kind::Decision,
        summary: Summary::new("summary only"),
        context: Context::new("current implementation context"),
        certainty: Certainty::Maximum,
        timestamp: Timestamp::new("2026-05-19T13:08:11Z"),
        quote: Quote::new("first statement"),
    }
}

fn state() -> State {
    State {
        presence: Presence::Active,
        focus: Some(FocusArea::new("implementation")),
    }
}

fn round_trip_request(request: SpiritRequest) -> SpiritRequest {
    let expected_verb = request.signal_verb();
    let frame = Frame::new(FrameBody::Request {
        exchange: exchange(),
        request: request.into_request(),
    });
    let bytes = frame.encode_length_prefixed().expect("encode");
    let decoded = Frame::decode_length_prefixed(&bytes).expect("decode");
    match decoded.into_body() {
        FrameBody::Request { request, .. } => {
            let operation = request.operations().head();
            assert_eq!(operation.verb, expected_verb);
            operation.payload.clone()
        }
        other => panic!("expected request operation, got {other:?}"),
    }
}

fn round_trip_reply(reply: SpiritReply) -> SpiritReply {
    let frame = Frame::new(FrameBody::Reply {
        exchange: exchange(),
        reply: Reply::completed(NonEmpty::single(SubReply::Ok {
            verb: SignalVerb::Assert,
            payload: reply,
        })),
    });
    let bytes = frame.encode_length_prefixed().expect("encode");
    let decoded = Frame::decode_length_prefixed(&bytes).expect("decode");
    match decoded.into_body() {
        FrameBody::Reply { reply, .. } => match reply {
            Reply::Accepted { per_operation, .. } => match per_operation.into_head() {
                SubReply::Ok { payload, .. } => payload,
                other => panic!("expected accepted reply payload, got {other:?}"),
            },
            other => panic!("expected accepted reply, got {other:?}"),
        },
        other => panic!("expected reply operation, got {other:?}"),
    }
}

fn round_trip_nota<T>(value: T, expected: &str)
where
    T: NotaEncode + NotaDecode + PartialEq + std::fmt::Debug,
{
    let mut encoder = Encoder::new();
    value.encode(&mut encoder).expect("encode nota text");
    let encoded = encoder.into_string();
    assert_eq!(encoded, expected);

    let mut decoder = Decoder::new(&encoded);
    let recovered = T::decode(&mut decoder).expect("decode nota text");
    assert_eq!(recovered, value);
    assert!(
        CANONICAL.contains(expected),
        "examples/canonical.nota missing line: {expected}"
    );
}

#[test]
fn spirit_requests_round_trip() {
    let requests = [
        SpiritRequest::Statement(Statement {
            statement: StatementText::new("capture this intent"),
        }),
        SpiritRequest::Entry(entry()),
        SpiritRequest::StateObservation(StateObservation {}),
        SpiritRequest::RecordObservation(RecordObservation {
            query: RecordQuery {
                topic: None,
                mode: ObservationMode::SummaryOnly,
            },
        }),
        SpiritRequest::QuestionPending(QuestionPending {}),
        SpiritRequest::SubscribeState(StateSubscription {}),
        SpiritRequest::StateSubscriptionRetraction(StateSubscriptionToken { identifier: 1 }),
        SpiritRequest::SubscribeRecords(RecordSubscription {
            topic: None,
            mode: ObservationMode::SummaryOnly,
        }),
        SpiritRequest::RecordSubscriptionRetraction(RecordSubscriptionToken { identifier: 2 }),
    ];

    for request in requests {
        assert_eq!(round_trip_request(request.clone()), request);
    }
}

#[test]
fn spirit_replies_round_trip() {
    let replies = [
        SpiritReply::RecordAccepted(RecordAccepted {
            captured: summary(),
        }),
        SpiritReply::StateObserved(StateObserved { state: state() }),
        SpiritReply::RecordsObserved(RecordsObserved {
            records: vec![summary()],
        }),
        SpiritReply::RecordProvenancesObserved(RecordProvenancesObserved {
            records: vec![provenance()],
        }),
        SpiritReply::QuestionsObserved(QuestionsObserved {
            questions: vec![QuestionSummary {
                identifier: QuestionIdentifier::new("question-one"),
                question: QuestionText::new("which intent wins?"),
            }],
        }),
        SpiritReply::StateSubscriptionOpened(StateSubscriptionOpened {
            token: StateSubscriptionToken { identifier: 1 },
            snapshot: state(),
        }),
        SpiritReply::RecordSubscriptionOpened(RecordSubscriptionOpened {
            token: RecordSubscriptionToken { identifier: 2 },
            snapshot: vec![summary()],
        }),
        SpiritReply::StateSubscriptionRetracted(StateSubscriptionRetracted {
            token: StateSubscriptionToken { identifier: 1 },
        }),
        SpiritReply::RecordSubscriptionRetracted(RecordSubscriptionRetracted {
            token: RecordSubscriptionToken { identifier: 2 },
        }),
        SpiritReply::RequestUnimplemented(RequestUnimplemented {
            operation: OperationKind::Statement,
            reason: UnimplementedReason::NotBuiltYet,
        }),
    ];

    for reply in replies {
        assert_eq!(round_trip_reply(reply.clone()), reply);
    }
}

#[test]
fn spirit_events_round_trip() {
    let events = [
        SpiritEvent::StateChanged(StateChanged { state: state() }),
        SpiritEvent::RecordCaptured(RecordCaptured { record: summary() }),
    ];

    for event in events {
        let frame = Frame::new(FrameBody::SubscriptionEvent {
            event_identifier: StreamEventIdentifier::new(
                SessionEpoch::new(1),
                ExchangeLane::Acceptor,
                LaneSequence::first(),
            ),
            token: SubscriptionTokenInner::new(1),
            event: event.clone(),
        });
        let bytes = frame.encode_length_prefixed().expect("encode");
        let decoded = Frame::decode_length_prefixed(&bytes).expect("decode");
        match decoded.into_body() {
            FrameBody::SubscriptionEvent { event: decoded, .. } => assert_eq!(decoded, event),
            other => panic!("expected event frame, got {other:?}"),
        }
    }
}

#[test]
fn spirit_request_variants_declare_expected_signal_root_verbs() {
    let cases = [
        (
            SpiritRequest::Statement(Statement {
                statement: StatementText::new("capture this intent"),
            }),
            SignalVerb::Assert,
        ),
        (SpiritRequest::Entry(entry()), SignalVerb::Assert),
        (
            SpiritRequest::StateObservation(StateObservation {}),
            SignalVerb::Match,
        ),
        (
            SpiritRequest::RecordObservation(RecordObservation {
                query: RecordQuery {
                    topic: None,
                    mode: ObservationMode::SummaryOnly,
                },
            }),
            SignalVerb::Match,
        ),
        (
            SpiritRequest::QuestionPending(QuestionPending {}),
            SignalVerb::Match,
        ),
        (
            SpiritRequest::SubscribeState(StateSubscription {}),
            SignalVerb::Subscribe,
        ),
        (
            SpiritRequest::StateSubscriptionRetraction(StateSubscriptionToken { identifier: 1 }),
            SignalVerb::Retract,
        ),
        (
            SpiritRequest::SubscribeRecords(RecordSubscription {
                topic: None,
                mode: ObservationMode::SummaryOnly,
            }),
            SignalVerb::Subscribe,
        ),
        (
            SpiritRequest::RecordSubscriptionRetraction(RecordSubscriptionToken { identifier: 2 }),
            SignalVerb::Retract,
        ),
    ];

    for (request, verb) in cases {
        assert_eq!(request.signal_verb(), verb);
    }
}

#[test]
fn spirit_request_exposes_contract_owned_operation_kind() {
    assert_eq!(
        SpiritRequest::Statement(Statement {
            statement: StatementText::new("capture this intent"),
        })
        .operation_kind(),
        OperationKind::Statement
    );
    assert_eq!(
        SpiritRequest::Entry(entry()).operation_kind(),
        OperationKind::Entry
    );
    assert_eq!(
        SpiritRequest::SubscribeRecords(RecordSubscription {
            topic: None,
            mode: ObservationMode::SummaryOnly,
        })
        .operation_kind(),
        OperationKind::SubscribeRecords
    );
}

#[test]
fn spirit_stream_witnesses_are_emitted() {
    assert_eq!(
        SpiritRequest::SubscribeState(StateSubscription {}).opened_stream(),
        Some(signal_persona_spirit::SpiritStreamKind::StateStream)
    );
    assert_eq!(
        SpiritEvent::RecordCaptured(RecordCaptured { record: summary() }).stream_kind(),
        signal_persona_spirit::SpiritStreamKind::RecordStream
    );
    assert_eq!(
        SpiritRequest::StateSubscriptionRetraction(StateSubscriptionToken { identifier: 1 })
            .closed_stream(),
        Some(signal_persona_spirit::SpiritStreamKind::StateStream)
    );
    assert_eq!(
        SpiritRequest::RecordSubscriptionRetraction(RecordSubscriptionToken { identifier: 2 })
            .closed_stream(),
        Some(signal_persona_spirit::SpiritStreamKind::RecordStream)
    );
}

#[test]
fn spirit_canonical_examples_round_trip() {
    round_trip_nota(
        SpiritRequest::Statement(Statement {
            statement: StatementText::new("capture this intent"),
        }),
        "(Statement (\"capture this intent\"))",
    );
    round_trip_nota(
        SpiritRequest::Entry(entry()),
        "(Entry (workspace Decision \"summary only\" \"current implementation context\" Maximum \"2026-05-19T13:08:11Z\" \"first statement\"))",
    );
    round_trip_nota(
        SpiritRequest::RecordObservation(RecordObservation {
            query: RecordQuery {
                topic: None,
                mode: ObservationMode::SummaryOnly,
            },
        }),
        "(RecordObservation ((None SummaryOnly)))",
    );
    round_trip_nota(
        SpiritReply::RecordAccepted(RecordAccepted {
            captured: summary(),
        }),
        "(RecordAccepted ((1 workspace Decision \"summary only\" Maximum)))",
    );
    round_trip_nota(
        SpiritReply::RecordProvenancesObserved(RecordProvenancesObserved {
            records: vec![provenance()],
        }),
        "(RecordProvenancesObserved ([((1 workspace Decision \"summary only\" Maximum) \"current implementation context\" \"2026-05-19T13:08:11Z\" \"first statement\")]))",
    );
    round_trip_nota(
        SpiritEvent::RecordCaptured(RecordCaptured { record: summary() }),
        "(RecordCaptured ((1 workspace Decision \"summary only\" Maximum)))",
    );
    round_trip_nota(
        SpiritReply::StateSubscriptionRetracted(StateSubscriptionRetracted {
            token: StateSubscriptionToken { identifier: 1 },
        }),
        "(StateSubscriptionRetracted ((1)))",
    );
    round_trip_nota(
        SpiritReply::RecordSubscriptionRetracted(RecordSubscriptionRetracted {
            token: RecordSubscriptionToken { identifier: 2 },
        }),
        "(RecordSubscriptionRetracted ((2)))",
    );
}
