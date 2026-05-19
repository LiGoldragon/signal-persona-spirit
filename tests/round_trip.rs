use nota_codec::{Decoder, Encoder, NotaDecode, NotaEncode};
use signal_core::{
    ExchangeIdentifier, ExchangeLane, LaneSequence, NonEmpty, Reply, RequestPayload, SessionEpoch,
    SignalVerb, StreamEventIdentifier, SubReply, SubscriptionTokenInner,
};
use signal_persona_spirit::{
    ClarificationQuestionIdentifier, ClarificationQuestionPending, ClarificationQuestionSummary,
    ClarificationQuestionText, ClarificationQuestionsObserved, Entry, Frame, FrameBody,
    IntentCertainty, IntentContext, IntentKind, IntentObservationMode, IntentQuote,
    IntentRecordCaptured, IntentRecordIdentifier, IntentRecordObservation, IntentRecordQuery,
    IntentRecordSubscription, IntentRecordSubscriptionOpened, IntentRecordSubscriptionToken,
    IntentRecordsObserved, IntentSummary, IntentTimestamp, IntentTopic, PsycheFocusArea,
    PsychePresence, PsycheState, PsycheStateChanged, PsycheStateObservation, PsycheStateObserved,
    PsycheStateSubscription, PsycheStateSubscriptionOpened, PsycheStateSubscriptionToken,
    PsycheStatement, PsycheStatementAccepted, PsycheStatementText, SpiritEvent,
    SpiritOperationKind, SpiritReply, SpiritRequest, SpiritRequestUnimplemented,
    SpiritUnimplementedReason, Verbatim,
};

const CANONICAL: &str = include_str!("../examples/canonical.nota");

fn exchange() -> ExchangeIdentifier {
    ExchangeIdentifier::new(
        SessionEpoch::new(1),
        ExchangeLane::Connector,
        LaneSequence::first(),
    )
}

fn summary() -> signal_persona_spirit::IntentRecordSummary {
    signal_persona_spirit::IntentRecordSummary {
        identifier: IntentRecordIdentifier::new("record-one"),
        topic: IntentTopic::new("workspace"),
        kind: IntentKind::Decision,
        summary: IntentSummary::new("summary only"),
        certainty: IntentCertainty::Maximum,
    }
}

fn entry() -> Entry {
    Entry {
        topic: IntentTopic::new("workspace"),
        kind: IntentKind::Decision,
        summary: IntentSummary::new("summary only"),
        context: IntentContext::new("current implementation context"),
        certainty: IntentCertainty::Maximum,
        verbatim: vec![
            Verbatim {
                timestamp: IntentTimestamp::new("2026-05-19T13:08:11Z"),
                quote: IntentQuote::new("first statement"),
            },
            Verbatim {
                timestamp: IntentTimestamp::new("2026-05-19T13:12:00Z"),
                quote: IntentQuote::new("restated statement"),
            },
        ],
    }
}

fn state() -> PsycheState {
    PsycheState {
        presence: PsychePresence::Active,
        focus: Some(PsycheFocusArea::new("implementation")),
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
        SpiritRequest::PsycheStatement(PsycheStatement {
            statement: PsycheStatementText::new("capture this intent"),
        }),
        SpiritRequest::Entry(entry()),
        SpiritRequest::PsycheStateObservation(PsycheStateObservation {}),
        SpiritRequest::IntentRecordObservation(IntentRecordObservation {
            query: IntentRecordQuery {
                topic: None,
                mode: IntentObservationMode::SummaryOnly,
            },
        }),
        SpiritRequest::ClarificationQuestionPending(ClarificationQuestionPending {}),
        SpiritRequest::SubscribePsycheState(PsycheStateSubscription {}),
        SpiritRequest::PsycheStateSubscriptionRetraction(PsycheStateSubscriptionToken {
            identifier: 1,
        }),
        SpiritRequest::SubscribeIntentRecords(IntentRecordSubscription {
            topic: None,
            mode: IntentObservationMode::SummaryOnly,
        }),
        SpiritRequest::IntentRecordSubscriptionRetraction(IntentRecordSubscriptionToken {
            identifier: 2,
        }),
    ];

    for request in requests {
        assert_eq!(round_trip_request(request.clone()), request);
    }
}

#[test]
fn spirit_replies_round_trip() {
    let replies = [
        SpiritReply::PsycheStatementAccepted(PsycheStatementAccepted {
            captured: summary(),
        }),
        SpiritReply::PsycheStateObserved(PsycheStateObserved { state: state() }),
        SpiritReply::IntentRecordsObserved(IntentRecordsObserved {
            records: vec![summary()],
        }),
        SpiritReply::ClarificationQuestionsObserved(ClarificationQuestionsObserved {
            questions: vec![ClarificationQuestionSummary {
                identifier: ClarificationQuestionIdentifier::new("question-one"),
                question: ClarificationQuestionText::new("which intent wins?"),
            }],
        }),
        SpiritReply::PsycheStateSubscriptionOpened(PsycheStateSubscriptionOpened {
            token: PsycheStateSubscriptionToken { identifier: 1 },
            snapshot: state(),
        }),
        SpiritReply::IntentRecordSubscriptionOpened(IntentRecordSubscriptionOpened {
            token: IntentRecordSubscriptionToken { identifier: 2 },
            snapshot: vec![summary()],
        }),
        SpiritReply::SpiritRequestUnimplemented(SpiritRequestUnimplemented {
            operation: SpiritOperationKind::PsycheStatement,
            reason: SpiritUnimplementedReason::NotBuiltYet,
        }),
    ];

    for reply in replies {
        assert_eq!(round_trip_reply(reply.clone()), reply);
    }
}

#[test]
fn spirit_events_round_trip() {
    let events = [
        SpiritEvent::PsycheStateChanged(PsycheStateChanged { state: state() }),
        SpiritEvent::IntentRecordCaptured(IntentRecordCaptured { record: summary() }),
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
            SpiritRequest::PsycheStatement(PsycheStatement {
                statement: PsycheStatementText::new("capture this intent"),
            }),
            SignalVerb::Assert,
        ),
        (SpiritRequest::Entry(entry()), SignalVerb::Assert),
        (
            SpiritRequest::PsycheStateObservation(PsycheStateObservation {}),
            SignalVerb::Match,
        ),
        (
            SpiritRequest::IntentRecordObservation(IntentRecordObservation {
                query: IntentRecordQuery {
                    topic: None,
                    mode: IntentObservationMode::SummaryOnly,
                },
            }),
            SignalVerb::Match,
        ),
        (
            SpiritRequest::ClarificationQuestionPending(ClarificationQuestionPending {}),
            SignalVerb::Match,
        ),
        (
            SpiritRequest::SubscribePsycheState(PsycheStateSubscription {}),
            SignalVerb::Subscribe,
        ),
        (
            SpiritRequest::PsycheStateSubscriptionRetraction(PsycheStateSubscriptionToken {
                identifier: 1,
            }),
            SignalVerb::Retract,
        ),
        (
            SpiritRequest::SubscribeIntentRecords(IntentRecordSubscription {
                topic: None,
                mode: IntentObservationMode::SummaryOnly,
            }),
            SignalVerb::Subscribe,
        ),
        (
            SpiritRequest::IntentRecordSubscriptionRetraction(IntentRecordSubscriptionToken {
                identifier: 2,
            }),
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
        SpiritRequest::PsycheStatement(PsycheStatement {
            statement: PsycheStatementText::new("capture this intent"),
        })
        .operation_kind(),
        SpiritOperationKind::PsycheStatement
    );
    assert_eq!(
        SpiritRequest::Entry(entry()).operation_kind(),
        SpiritOperationKind::Entry
    );
    assert_eq!(
        SpiritRequest::SubscribeIntentRecords(IntentRecordSubscription {
            topic: None,
            mode: IntentObservationMode::SummaryOnly,
        })
        .operation_kind(),
        SpiritOperationKind::SubscribeIntentRecords
    );
}

#[test]
fn spirit_stream_witnesses_are_emitted() {
    assert_eq!(
        SpiritRequest::SubscribePsycheState(PsycheStateSubscription {}).opened_stream(),
        Some(signal_persona_spirit::SpiritStreamKind::PsycheStateStream)
    );
    assert_eq!(
        SpiritEvent::IntentRecordCaptured(IntentRecordCaptured { record: summary() }).stream_kind(),
        signal_persona_spirit::SpiritStreamKind::IntentRecordStream
    );
}

#[test]
fn spirit_canonical_examples_round_trip() {
    round_trip_nota(
        SpiritRequest::PsycheStatement(PsycheStatement {
            statement: PsycheStatementText::new("capture this intent"),
        }),
        "(PsycheStatement \"capture this intent\")",
    );
    round_trip_nota(
        SpiritRequest::Entry(entry()),
        "(Entry workspace Decision \"summary only\" \"current implementation context\" Maximum [(Verbatim \"2026-05-19T13:08:11Z\" \"first statement\") (Verbatim \"2026-05-19T13:12:00Z\" \"restated statement\")])",
    );
    round_trip_nota(
        SpiritRequest::IntentRecordObservation(IntentRecordObservation {
            query: IntentRecordQuery {
                topic: None,
                mode: IntentObservationMode::SummaryOnly,
            },
        }),
        "(IntentRecordObservation (IntentRecordQuery None SummaryOnly))",
    );
    round_trip_nota(
        SpiritReply::PsycheStatementAccepted(PsycheStatementAccepted {
            captured: summary(),
        }),
        "(PsycheStatementAccepted (IntentRecordSummary record-one workspace Decision \"summary only\" Maximum))",
    );
    round_trip_nota(
        SpiritEvent::IntentRecordCaptured(IntentRecordCaptured { record: summary() }),
        "(IntentRecordCaptured (IntentRecordSummary record-one workspace Decision \"summary only\" Maximum))",
    );
}
