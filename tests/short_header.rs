use signal_frame::{
    ExchangeIdentifier, ExchangeLane, LaneSequence, SessionEpoch, ShortHeader,
    short_header_from_length_prefixed,
};
use signal_persona_spirit::{
    CertaintyChange, CertaintySelection, Description, Entry, Frame, FrameBody, Kind, Observation,
    ObservationMode, Operation, OperationKind, RecordIdentifier, RecordQuery,
    RecordedTimeSelection, Reply, RequestUnimplemented, Statement, StatementText, Topic,
    TopicSelection, Topics, UnimplementedReason,
};
use signal_sema::Magnitude;

#[derive(Debug, thiserror::Error)]
enum DispatchError {
    #[error(transparent)]
    Dispatch(#[from] signal_frame::OperationDispatchError),
}

#[derive(Default)]
struct DispatchWitness {
    handled: Vec<OperationKind>,
}

impl signal_persona_spirit::OperationHandler for DispatchWitness {
    type Error = DispatchError;

    async fn handle_state(&mut self, _payload: Statement) -> Result<Reply, Self::Error> {
        self.handled.push(OperationKind::State);
        Ok(unimplemented_reply())
    }

    async fn handle_record(&mut self, _payload: Entry) -> Result<Reply, Self::Error> {
        self.handled.push(OperationKind::Record);
        Ok(unimplemented_reply())
    }

    async fn handle_observe(&mut self, _payload: Observation) -> Result<Reply, Self::Error> {
        self.handled.push(OperationKind::Observe);
        Ok(unimplemented_reply())
    }

    async fn handle_watch(
        &mut self,
        _payload: signal_persona_spirit::Subscription,
    ) -> Result<Reply, Self::Error> {
        self.handled.push(OperationKind::Watch);
        Ok(unimplemented_reply())
    }

    async fn handle_unwatch(
        &mut self,
        _payload: signal_persona_spirit::SubscriptionToken,
    ) -> Result<Reply, Self::Error> {
        self.handled.push(OperationKind::Unwatch);
        Ok(unimplemented_reply())
    }

    async fn handle_remove(&mut self, _payload: RecordIdentifier) -> Result<Reply, Self::Error> {
        self.handled.push(OperationKind::Remove);
        Ok(unimplemented_reply())
    }

    async fn handle_change_certainty(
        &mut self,
        _payload: CertaintyChange,
    ) -> Result<Reply, Self::Error> {
        self.handled.push(OperationKind::ChangeCertainty);
        Ok(unimplemented_reply())
    }

    async fn handle_tap(
        &mut self,
        _payload: signal_persona_spirit::ObserverFilter,
    ) -> Result<Reply, Self::Error> {
        self.handled.push(OperationKind::Tap);
        Ok(unimplemented_reply())
    }

    async fn handle_untap(
        &mut self,
        _payload: signal_persona_spirit::ObserverSubscriptionToken,
    ) -> Result<Reply, Self::Error> {
        self.handled.push(OperationKind::Untap);
        Ok(unimplemented_reply())
    }
}

fn block_on_ready<Output>(future: impl std::future::Future<Output = Output>) -> Output {
    struct NoopWake;

    impl std::task::Wake for NoopWake {
        fn wake(self: std::sync::Arc<Self>) {}
    }

    let waker = std::task::Waker::from(std::sync::Arc::new(NoopWake));
    let mut context = std::task::Context::from_waker(&waker);
    let mut future = Box::pin(future);
    match future.as_mut().poll(&mut context) {
        std::task::Poll::Ready(output) => output,
        std::task::Poll::Pending => panic!("test future unexpectedly yielded"),
    }
}

fn exchange() -> ExchangeIdentifier {
    ExchangeIdentifier::new(
        SessionEpoch::new(1),
        ExchangeLane::Connector,
        LaneSequence::first(),
    )
}

fn entry() -> Entry {
    Entry {
        topics: Topics::single(Topic::new("workspace")),
        kind: Kind::Decision,
        description: Description::new("schema header"),
        certainty: Magnitude::Maximum,
    }
}

fn unimplemented_reply() -> Reply {
    Reply::RequestUnimplemented(RequestUnimplemented {
        reason: UnimplementedReason::NotBuiltYet,
    })
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
fn generated_operation_dispatch_routes_by_short_header() {
    use signal_persona_spirit::OperationDispatch;

    let mut witness = DispatchWitness::default();
    let reply = block_on_ready(witness.dispatch(ShortHeader::new(1), Operation::Record(entry())))
        .expect("dispatch record");

    assert_eq!(witness.handled, vec![OperationKind::Record]);
    assert_eq!(reply, unimplemented_reply());

    let mismatch =
        block_on_ready(witness.dispatch(ShortHeader::new(0), Operation::Record(entry())))
            .expect_err("mismatch rejected");
    assert!(matches!(
        mismatch,
        DispatchError::Dispatch(
            signal_frame::OperationDispatchError::HeaderOperationMismatch {
                expected: 0,
                actual: 1
            }
        )
    ));
}

#[test]
fn nested_query_shape_sets_sub_enum_slots() {
    let frame = Operation::Observe(Observation::Records(RecordQuery {
        topic_selection: TopicSelection::any(),
        kind: None,
        certainty_selection: CertaintySelection::Any,
        recorded_time_selection: RecordedTimeSelection::Any,
        mode: ObservationMode::WithProvenance,
    }))
    .into_frame(exchange());

    assert_eq!(frame.short_header(), header([2, 1, 0, 0, 0, 0, 1, 0]));
}
