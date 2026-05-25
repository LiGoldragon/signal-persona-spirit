use nota_codec::{Decoder, Encoder, NotaDecode, NotaEncode};
use signal_frame::{
    ExchangeIdentifier, ExchangeLane, LaneSequence, NonEmpty, Reply as FrameReply, RequestPayload,
    SessionEpoch, StreamEventIdentifier, StreamingFrameBody, SubReply, SubscriptionTokenInner,
};
use signal_persona_spirit::{
    Date, Description, EffectEmitted, Entry, Event, FocusArea, Frame, FrameBody, Kind, Observation,
    ObservationMode, ObserverFilter, ObserverFilterMatch, ObserverSubscriptionToken, Operation,
    OperationKind, OperationReceived, Presence, PresenceView, QuestionIdentifier, QuestionSummary,
    QuestionText, QuestionsObserved, RecordAccepted, RecordCaptured, RecordIdentifier,
    RecordProvenance, RecordProvenancesObserved, RecordQuery, RecordSubscription,
    RecordSubscriptionToken, RecordsObserved, Reply, RequestUnimplemented, StateChanged,
    StateObserved, StateSubscriptionToken, Statement, StatementText, Subscription,
    SubscriptionOpened, SubscriptionRetracted, SubscriptionSnapshot, SubscriptionToken, Time,
    Topic, TopicCount, TopicsObserved, UnimplementedReason,
};
use signal_sema::{Magnitude, SemaObservation, SemaOperation, SemaOutcome};

const CANONICAL: &str = include_str!("../examples/canonical.nota");

fn exchange() -> ExchangeIdentifier {
    ExchangeIdentifier::new(
        SessionEpoch::new(1),
        ExchangeLane::Connector,
        LaneSequence::first(),
    )
}

fn description() -> signal_persona_spirit::RecordDescription {
    signal_persona_spirit::RecordDescription {
        identifier: RecordIdentifier::new(1),
        topic: Topic::new("workspace"),
        kind: Kind::Decision,
        description: Description::new("description only"),
        certainty: Magnitude::Maximum,
    }
}

fn provenance() -> RecordProvenance {
    RecordProvenance {
        description: description(),
        date: Date::new(2026, 5, 20),
        time: Time::new(14, 30, 0),
    }
}

fn entry() -> Entry {
    Entry {
        topic: Topic::new("workspace"),
        kind: Kind::Decision,
        description: Description::new("description only"),
        certainty: Magnitude::Maximum,
    }
}

fn state() -> PresenceView {
    PresenceView {
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
            kind: None,
            mode: ObservationMode::DescriptionOnly,
        })),
        Operation::Observe(Observation::Topics),
        Operation::Observe(Observation::Questions),
        Operation::Watch(Subscription::State),
        Operation::Watch(Subscription::Records(RecordSubscription {
            topic: None,
            mode: ObservationMode::DescriptionOnly,
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
        Reply::RecordAccepted(RecordAccepted::new(RecordIdentifier::new(1))),
        Reply::StateObserved(StateObserved { state: state() }),
        Reply::RecordsObserved(RecordsObserved {
            records: vec![description()],
        }),
        Reply::RecordProvenancesObserved(RecordProvenancesObserved {
            records: vec![provenance()],
        }),
        Reply::TopicsObserved(TopicsObserved {
            topics: vec![TopicCount {
                topic: Topic::new("workspace"),
                entries: 2,
            }],
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
            snapshot: SubscriptionSnapshot::Records(vec![description()]),
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
    let reply: Reply = RecordAccepted::new(RecordIdentifier::new(1)).into();

    assert_eq!(
        reply,
        Reply::RecordAccepted(RecordAccepted::new(RecordIdentifier::new(1)))
    );
}

#[test]
fn spirit_events_round_trip() {
    let events = [
        Event::StateChanged(StateChanged { state: state() }),
        Event::RecordCaptured(RecordCaptured {
            record: description(),
        }),
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
            mode: ObservationMode::DescriptionOnly,
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
        Event::RecordCaptured(RecordCaptured {
            record: description()
        })
        .stream_kind(),
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
        "(State ([capture this intent]))",
    );
    round_trip_nota(
        Operation::Record(entry()),
        "(Record (workspace Decision [description only] Maximum))",
    );
    let mut high_entry = entry();
    high_entry.description = Description::new("high description");
    high_entry.certainty = Magnitude::High;
    round_trip_nota(
        Operation::Record(high_entry),
        "(Record (workspace Decision [high description] High))",
    );
    round_trip_nota(Operation::Observe(Observation::State), "(Observe State)");
    round_trip_nota(
        Operation::Observe(Observation::Records(RecordQuery {
            topic: None,
            kind: None,
            mode: ObservationMode::DescriptionOnly,
        })),
        "(Observe (Records (None None DescriptionOnly)))",
    );
    round_trip_nota(
        Operation::Observe(Observation::Records(RecordQuery {
            topic: Some(Topic::new("workspace")),
            kind: Some(Kind::Decision),
            mode: ObservationMode::DescriptionOnly,
        })),
        "(Observe (Records ((Some workspace) (Some Decision) DescriptionOnly)))",
    );
    round_trip_nota(Operation::Observe(Observation::Topics), "(Observe Topics)");
    round_trip_nota(
        Operation::Observe(Observation::Questions),
        "(Observe Questions)",
    );
    round_trip_nota(Operation::Watch(Subscription::State), "(Watch State)");
    round_trip_nota(
        Operation::Watch(Subscription::Records(RecordSubscription {
            topic: None,
            mode: ObservationMode::DescriptionOnly,
        })),
        "(Watch (Records (None DescriptionOnly)))",
    );
    round_trip_nota(
        Operation::Unwatch(SubscriptionToken::Records(RecordSubscriptionToken {
            identifier: 2,
        })),
        "(Unwatch (Records (2)))",
    );
    round_trip_nota(
        Reply::RecordAccepted(RecordAccepted::new(RecordIdentifier::new(1))),
        "(RecordAccepted 1)",
    );
    round_trip_nota(
        Reply::RecordProvenancesObserved(RecordProvenancesObserved {
            records: vec![provenance()],
        }),
        "(RecordProvenancesObserved ([((1 workspace Decision [description only] Maximum) 2026-05-20 14:30:00)]))",
    );
    round_trip_nota(
        Reply::TopicsObserved(TopicsObserved {
            topics: vec![TopicCount {
                topic: Topic::new("workspace"),
                entries: 2,
            }],
        }),
        "(TopicsObserved ([(workspace 2)]))",
    );
    round_trip_nota(
        Event::RecordCaptured(RecordCaptured {
            record: description(),
        }),
        "(RecordCaptured ((1 workspace Decision [description only] Maximum)))",
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
    let mut decoder =
        Decoder::new("(Record (workspace Decision [description only] Maximum 1779000000))");
    Operation::decode(&mut decoder).expect_err("client timestamp must not decode");
}

#[test]
fn record_request_with_parenthesized_client_date_time_shape_is_rejected() {
    let mut decoder = Decoder::new(
        "(Record (workspace Decision [description only] Maximum (2026 5 20) (14 30 0)))",
    );
    Operation::decode(&mut decoder).expect_err("parenthesized client date/time must not decode");
}
