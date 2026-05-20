use nota_codec::{Decoder, Encoder, NotaDecode, NotaEncode};
use signal_frame::{
    ExchangeIdentifier, ExchangeLane, LaneSequence, NonEmpty, Reply, RequestPayload, SessionEpoch,
    StreamEventIdentifier, StreamingFrameBody, SubReply, SubscriptionTokenInner,
};
use signal_persona_spirit::{
    Certainty, Context, Entry, FocusArea, Frame, FrameBody, Kind, Observation, ObservationMode,
    OperationKind, OperationReceived, Presence, QuestionIdentifier, QuestionPending,
    QuestionSummary, QuestionText, QuestionsObserved, Quote, RecordAccepted, RecordCaptured,
    RecordIdentifier, RecordProvenance, RecordProvenancesObserved, RecordQuery, RecordSubscription,
    RecordSubscriptionOpened, RecordSubscriptionRetracted, RecordSubscriptionToken,
    RecordsObserved, RequestUnimplemented, SemaEffectEmitted, SpiritEvent, SpiritObserverFilter,
    SpiritObserverFilterMatch, SpiritObserverSubscriptionToken, SpiritReply, SpiritRequest, State,
    StateChanged, StateObservation, StateObserved, StateSubscription, StateSubscriptionOpened,
    StateSubscriptionRetracted, StateSubscriptionToken, Statement, StatementText, Subscription,
    SubscriptionSnapshot, SubscriptionToken, Summary, Timestamp, Topic, UnimplementedReason,
};
use signal_sema::{SemaObservation, SemaOperation, SemaOutcome};

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
    let frame = Frame::new(FrameBody::Request {
        exchange: exchange(),
        request: request.clone().into_request(),
    });
    let bytes = frame.encode_length_prefixed().expect("encode");
    let decoded = Frame::decode_length_prefixed(&bytes).expect("decode");
    match decoded.into_body() {
        FrameBody::Request { request, .. } => request.payloads().head().clone(),
        other => panic!("expected request operation, got {other:?}"),
    }
}

fn round_trip_reply(reply: SpiritReply) -> SpiritReply {
    let frame = Frame::new(FrameBody::Reply {
        exchange: exchange(),
        reply: Reply::committed(NonEmpty::single(SubReply::Ok(reply))),
    });
    let bytes = frame.encode_length_prefixed().expect("encode");
    let decoded = Frame::decode_length_prefixed(&bytes).expect("decode");
    match decoded.into_body() {
        FrameBody::Reply { reply, .. } => match reply {
            Reply::Accepted { per_operation, .. } => match per_operation.into_head() {
                SubReply::Ok(payload) => payload,
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
        SpiritRequest::State(Statement {
            statement: StatementText::new("capture this intent"),
        }),
        SpiritRequest::Record(entry()),
        SpiritRequest::Observe(Observation::State(StateObservation {})),
        SpiritRequest::Observe(Observation::Records(RecordQuery {
            topic: None,
            mode: ObservationMode::SummaryOnly,
        })),
        SpiritRequest::Observe(Observation::Questions(QuestionPending {})),
        SpiritRequest::Watch(Subscription::State(StateSubscription {})),
        SpiritRequest::Watch(Subscription::Records(RecordSubscription {
            topic: None,
            mode: ObservationMode::SummaryOnly,
        })),
        SpiritRequest::Unwatch(SubscriptionToken::State(StateSubscriptionToken {
            identifier: 1,
        })),
        SpiritRequest::Unwatch(SubscriptionToken::Records(RecordSubscriptionToken {
            identifier: 2,
        })),
        SpiritRequest::Tap(SpiritObserverFilter::OperationsOnly),
        SpiritRequest::Untap(SpiritObserverSubscriptionToken::new(
            SubscriptionTokenInner::new(3),
        )),
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
        SpiritReply::SubscriptionOpened(signal_persona_spirit::SubscriptionOpened {
            token: SubscriptionToken::State(StateSubscriptionToken { identifier: 1 }),
            snapshot: SubscriptionSnapshot::State(state()),
        }),
        SpiritReply::StateSubscriptionRetracted(StateSubscriptionRetracted {
            token: StateSubscriptionToken { identifier: 1 },
        }),
        SpiritReply::RecordSubscriptionRetracted(RecordSubscriptionRetracted {
            token: RecordSubscriptionToken { identifier: 2 },
        }),
        SpiritReply::SubscriptionRetracted(signal_persona_spirit::SubscriptionRetracted {
            token: SubscriptionToken::Records(RecordSubscriptionToken { identifier: 2 }),
        }),
        SpiritReply::RequestUnimplemented(RequestUnimplemented {
            operation: OperationKind::State,
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
        SpiritEvent::OperationReceived(OperationReceived {
            operation: OperationKind::Record,
        }),
        SpiritEvent::SemaEffectEmitted(SemaEffectEmitted {
            observation: SemaObservation::new(SemaOperation::Assert, SemaOutcome::Asserted),
        }),
    ];

    for event in events {
        let frame = Frame::new(StreamingFrameBody::SubscriptionEvent {
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
fn spirit_request_exposes_contract_owned_operation_kind() {
    assert_eq!(
        SpiritRequest::State(Statement {
            statement: StatementText::new("capture this intent"),
        })
        .operation_kind(),
        OperationKind::State
    );
    assert_eq!(
        SpiritRequest::Record(entry()).operation_kind(),
        OperationKind::Record
    );
    assert_eq!(
        SpiritRequest::Watch(Subscription::Records(RecordSubscription {
            topic: None,
            mode: ObservationMode::SummaryOnly,
        }))
        .operation_kind(),
        OperationKind::Watch
    );
}

#[test]
fn spirit_stream_witnesses_are_emitted() {
    assert_eq!(
        SpiritRequest::Watch(Subscription::State(StateSubscription {})).opened_stream(),
        Some(signal_persona_spirit::SpiritStreamKind::DomainStream)
    );
    assert_eq!(
        SpiritEvent::RecordCaptured(RecordCaptured { record: summary() }).stream_kind(),
        signal_persona_spirit::SpiritStreamKind::DomainStream
    );
    assert_eq!(
        SpiritRequest::Unwatch(SubscriptionToken::State(StateSubscriptionToken {
            identifier: 1
        }))
        .closed_stream(),
        Some(signal_persona_spirit::SpiritStreamKind::DomainStream)
    );
    assert_eq!(
        SpiritRequest::Tap(SpiritObserverFilter::All).opened_stream(),
        Some(signal_persona_spirit::SpiritStreamKind::ObserverStream)
    );
}

#[test]
fn spirit_observer_filter_routes_operation_and_effect_events() {
    let operation = OperationReceived {
        operation: OperationKind::Record,
    };
    let effect = SemaEffectEmitted {
        observation: SemaObservation::new(SemaOperation::Assert, SemaOutcome::Asserted),
    };

    assert!(SpiritObserverFilter::All.matches_operation_received(&operation));
    assert!(SpiritObserverFilter::All.matches_effect_emitted(&effect));
    assert!(SpiritObserverFilter::OperationsOnly.matches_operation_received(&operation));
    assert!(!SpiritObserverFilter::OperationsOnly.matches_effect_emitted(&effect));
    assert!(!SpiritObserverFilter::EffectsOnly.matches_operation_received(&operation));
    assert!(SpiritObserverFilter::EffectsOnly.matches_effect_emitted(&effect));
}

#[test]
fn spirit_canonical_examples_round_trip() {
    round_trip_nota(
        SpiritRequest::State(Statement {
            statement: StatementText::new("capture this intent"),
        }),
        "(State (\"capture this intent\"))",
    );
    round_trip_nota(
        SpiritRequest::Record(entry()),
        "(Record (workspace Decision \"summary only\" \"current implementation context\" Maximum \"2026-05-19T13:08:11Z\" \"first statement\"))",
    );
    round_trip_nota(
        SpiritRequest::Observe(Observation::Records(RecordQuery {
            topic: None,
            mode: ObservationMode::SummaryOnly,
        })),
        "(Observe (Records (None SummaryOnly)))",
    );
    round_trip_nota(
        SpiritRequest::Watch(Subscription::Records(RecordSubscription {
            topic: None,
            mode: ObservationMode::SummaryOnly,
        })),
        "(Watch (Records (None SummaryOnly)))",
    );
    round_trip_nota(
        SpiritRequest::Unwatch(SubscriptionToken::Records(RecordSubscriptionToken {
            identifier: 2,
        })),
        "(Unwatch (Records (2)))",
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
        SpiritEvent::SemaEffectEmitted(SemaEffectEmitted {
            observation: SemaObservation::new(SemaOperation::Assert, SemaOutcome::Asserted),
        }),
        "(SemaEffectEmitted ((Assert Asserted)))",
    );
}
