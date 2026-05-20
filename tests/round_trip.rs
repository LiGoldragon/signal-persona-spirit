use nota_codec::{Decoder, Encoder, NotaDecode, NotaEncode};
use signal_frame::{
    ExchangeIdentifier, ExchangeLane, LaneSequence, NonEmpty, Reply as FrameReply, RequestPayload,
    SessionEpoch, StreamEventIdentifier, StreamingFrameBody, SubReply, SubscriptionTokenInner,
};
use signal_persona_spirit::{
    Certainty, Context, Date, EffectEmitted, Entry, Event, FocusArea, Frame, FrameBody, Kind,
    Observation, ObservationMode, ObserverFilter, ObserverFilterMatch, ObserverSubscriptionToken,
    Operation, OperationKind, OperationReceived, Presence, QuestionIdentifier, QuestionSummary,
    QuestionText, QuestionsObserved, Quote, RecordAccepted, RecordCaptured, RecordIdentifier,
    RecordProvenance, RecordProvenancesObserved, RecordQuery, RecordSubscription,
    RecordSubscriptionToken, RecordsObserved, Reply, RequestUnimplemented, State, StateChanged,
    StateObserved, StateSubscriptionToken, Statement, StatementText, Subscription,
    SubscriptionOpened, SubscriptionRetracted, SubscriptionSnapshot, SubscriptionToken, Summary,
    Time, Topic, UnimplementedReason,
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
        date: Date::new(2026, 5, 20),
        time: Time::new(14, 30, 0),
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
        quote: Quote::new("first statement"),
    }
}

fn state() -> State {
    State {
        presence: Presence::Active,
        focus: Some(FocusArea::new("implementation")),
    }
}

fn round_trip_request(request: Operation) -> Operation {
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

fn round_trip_reply(reply: Reply) -> Reply {
    let frame = Frame::new(FrameBody::Reply {
        exchange: exchange(),
        reply: FrameReply::committed(NonEmpty::single(SubReply::Ok(reply))),
    });
    let bytes = frame.encode_length_prefixed().expect("encode");
    let decoded = Frame::decode_length_prefixed(&bytes).expect("decode");
    match decoded.into_body() {
        FrameBody::Reply { reply, .. } => match reply {
            FrameReply::Accepted { per_operation, .. } => match per_operation.into_head() {
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
        Operation::State(Statement {
            text: StatementText::new("capture this intent"),
        }),
        Operation::Record(entry()),
        Operation::Observe(Observation::State),
        Operation::Observe(Observation::Records(RecordQuery {
            topic: None,
            mode: ObservationMode::SummaryOnly,
        })),
        Operation::Observe(Observation::Questions),
        Operation::Watch(Subscription::State),
        Operation::Watch(Subscription::Records(RecordSubscription {
            topic: None,
            mode: ObservationMode::SummaryOnly,
        })),
        Operation::Unwatch(SubscriptionToken::State(StateSubscriptionToken {
            identifier: 1,
        })),
        Operation::Unwatch(SubscriptionToken::Records(RecordSubscriptionToken {
            identifier: 2,
        })),
        Operation::Tap(ObserverFilter::OperationsOnly),
        Operation::Untap(ObserverSubscriptionToken::new(SubscriptionTokenInner::new(
            3,
        ))),
    ];

    for request in requests {
        assert_eq!(round_trip_request(request.clone()), request);
    }
}

#[test]
fn spirit_replies_round_trip() {
    let replies = [
        Reply::RecordAccepted(RecordAccepted {
            captured: summary(),
        }),
        Reply::StateObserved(StateObserved { state: state() }),
        Reply::RecordsObserved(RecordsObserved {
            records: vec![summary()],
        }),
        Reply::RecordProvenancesObserved(RecordProvenancesObserved {
            records: vec![provenance()],
        }),
        Reply::QuestionsObserved(QuestionsObserved {
            questions: vec![QuestionSummary {
                identifier: QuestionIdentifier::new("question-one"),
                question: QuestionText::new("which intent wins?"),
            }],
        }),
        Reply::SubscriptionOpened(SubscriptionOpened {
            token: SubscriptionToken::State(StateSubscriptionToken { identifier: 1 }),
            snapshot: SubscriptionSnapshot::State(state()),
        }),
        Reply::SubscriptionOpened(SubscriptionOpened {
            token: SubscriptionToken::Records(RecordSubscriptionToken { identifier: 2 }),
            snapshot: SubscriptionSnapshot::Records(vec![summary()]),
        }),
        Reply::SubscriptionRetracted(SubscriptionRetracted {
            token: SubscriptionToken::State(StateSubscriptionToken { identifier: 1 }),
        }),
        Reply::SubscriptionRetracted(SubscriptionRetracted {
            token: SubscriptionToken::Records(RecordSubscriptionToken { identifier: 2 }),
        }),
        Reply::RequestUnimplemented(RequestUnimplemented {
            reason: UnimplementedReason::NotBuiltYet,
        }),
    ];

    for reply in replies {
        assert_eq!(round_trip_reply(reply.clone()), reply);
    }
}

#[test]
fn spirit_reply_payloads_convert_through_macro_generated_from_impls() {
    let reply: Reply = RecordAccepted {
        captured: summary(),
    }
    .into();

    assert_eq!(
        reply,
        Reply::RecordAccepted(RecordAccepted {
            captured: summary(),
        })
    );
}

#[test]
fn spirit_events_round_trip() {
    let events = [
        Event::StateChanged(StateChanged { state: state() }),
        Event::RecordCaptured(RecordCaptured { record: summary() }),
        Event::OperationReceived(OperationReceived {
            operation: OperationKind::Record,
        }),
        Event::EffectEmitted(EffectEmitted {
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
fn spirit_request_exposes_contract_owned_kind() {
    assert_eq!(
        Operation::State(Statement {
            text: StatementText::new("capture this intent"),
        })
        .kind(),
        OperationKind::State
    );
    assert_eq!(Operation::Record(entry()).kind(), OperationKind::Record);
    assert_eq!(
        Operation::Watch(Subscription::Records(RecordSubscription {
            topic: None,
            mode: ObservationMode::SummaryOnly,
        }))
        .kind(),
        OperationKind::Watch
    );
}

#[test]
fn spirit_stream_witnesses_are_emitted() {
    assert_eq!(
        Operation::Watch(Subscription::State).opened_stream(),
        Some(signal_persona_spirit::StreamKind::DomainStream)
    );
    assert_eq!(
        Event::RecordCaptured(RecordCaptured { record: summary() }).stream_kind(),
        signal_persona_spirit::StreamKind::DomainStream
    );
    assert_eq!(
        Operation::Unwatch(SubscriptionToken::State(StateSubscriptionToken {
            identifier: 1
        }))
        .closed_stream(),
        Some(signal_persona_spirit::StreamKind::DomainStream)
    );
    assert_eq!(
        Operation::Tap(ObserverFilter::All).opened_stream(),
        Some(signal_persona_spirit::StreamKind::ObserverStream)
    );
}

#[test]
fn spirit_observer_filter_routes_operation_and_effect_events() {
    let operation = OperationReceived {
        operation: OperationKind::Record,
    };
    let effect = EffectEmitted {
        observation: SemaObservation::new(SemaOperation::Assert, SemaOutcome::Asserted),
    };

    assert!(ObserverFilter::All.matches_operation_received(&operation));
    assert!(ObserverFilter::All.matches_effect_emitted(&effect));
    assert!(ObserverFilter::OperationsOnly.matches_operation_received(&operation));
    assert!(!ObserverFilter::OperationsOnly.matches_effect_emitted(&effect));
    assert!(!ObserverFilter::EffectsOnly.matches_operation_received(&operation));
    assert!(ObserverFilter::EffectsOnly.matches_effect_emitted(&effect));
}

#[test]
fn spirit_canonical_examples_round_trip() {
    round_trip_nota(
        Operation::State(Statement {
            text: StatementText::new("capture this intent"),
        }),
        "(State (\"capture this intent\"))",
    );
    round_trip_nota(
        Operation::Record(entry()),
        "(Record (workspace Decision \"summary only\" \"current implementation context\" Maximum \"first statement\"))",
    );
    round_trip_nota(Operation::Observe(Observation::State), "(Observe State)");
    round_trip_nota(
        Operation::Observe(Observation::Records(RecordQuery {
            topic: None,
            mode: ObservationMode::SummaryOnly,
        })),
        "(Observe (Records (None SummaryOnly)))",
    );
    round_trip_nota(
        Operation::Observe(Observation::Questions),
        "(Observe Questions)",
    );
    round_trip_nota(Operation::Watch(Subscription::State), "(Watch State)");
    round_trip_nota(
        Operation::Watch(Subscription::Records(RecordSubscription {
            topic: None,
            mode: ObservationMode::SummaryOnly,
        })),
        "(Watch (Records (None SummaryOnly)))",
    );
    round_trip_nota(
        Operation::Unwatch(SubscriptionToken::Records(RecordSubscriptionToken {
            identifier: 2,
        })),
        "(Unwatch (Records (2)))",
    );
    round_trip_nota(
        Reply::RecordAccepted(RecordAccepted {
            captured: summary(),
        }),
        "(RecordAccepted ((1 workspace Decision \"summary only\" Maximum)))",
    );
    round_trip_nota(
        Reply::RecordProvenancesObserved(RecordProvenancesObserved {
            records: vec![provenance()],
        }),
        "(RecordProvenancesObserved ([((1 workspace Decision \"summary only\" Maximum) \"current implementation context\" 2026-05-20 14:30:00 \"first statement\")]))",
    );
    round_trip_nota(
        Event::RecordCaptured(RecordCaptured { record: summary() }),
        "(RecordCaptured ((1 workspace Decision \"summary only\" Maximum)))",
    );
    round_trip_nota(
        Event::EffectEmitted(EffectEmitted {
            observation: SemaObservation::new(SemaOperation::Assert, SemaOutcome::Asserted),
        }),
        "(EffectEmitted ((Assert Asserted)))",
    );
}

#[test]
fn record_request_with_client_timestamp_shape_is_rejected() {
    let mut decoder = Decoder::new(
        "(Record (workspace Decision \"summary only\" \"current implementation context\" Maximum 1779000000 \"first statement\"))",
    );
    Operation::decode(&mut decoder).expect_err("client timestamp must not decode");
}

#[test]
fn record_request_with_parenthesized_client_date_time_shape_is_rejected() {
    let mut decoder = Decoder::new(
        "(Record (workspace Decision \"summary only\" \"current implementation context\" Maximum (2026 5 20) (14 30 0) \"first statement\"))",
    );
    Operation::decode(&mut decoder).expect_err("parenthesized client date/time must not decode");
}
