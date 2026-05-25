//! Schema-driven dual-emission smoke tests (psyche 2026-05-26 +
//! intent records 709, 710 — first of the three languages: the WIRE
//! LANGUAGE for the public/ordinary socket).
//!
//! These tests prove the `emit_schema!()` invocation in `src/lib.rs`
//! lands a `pub mod spirit { … }` carrying the schema-derived shapes
//! alongside the legacy `signal_channel!([schema])` emission at crate
//! root.

#[test]
fn schema_driven_module_is_reachable() {
    // The mere fact that `signal_persona_spirit::spirit` resolves at
    // compile time IS the structural proof. Reach for representative
    // type names to make the test concrete.
    let _route_count: usize = signal_persona_spirit::spirit::ROUTE_COUNT;
}

#[test]
fn schema_driven_operation_constructs() {
    // The schema-driven `Operation` is a closed enum carrying one
    // variant per top-level route (State / Record / Observe / Watch /
    // Unwatch). The state route's payload IS a `Statement` per the
    // schema's `State [(Statement)]` declaration; the composer emits
    // it as a tuple-shaped wrapper.
    use signal_persona_spirit::spirit::Operation;
    // The mere presence of the type is what matters here; the inner
    // payload construction depends on the composer's emission shape,
    // which is verified by the broader signal-persona-spirit test
    // suite. Type-naming itself proves the module lands.
    fn _accept_operation(_operation: Operation) {}
}

#[test]
fn universal_unknown_lands_on_wire_reply_enum() {
    // The extended universal-Unknown post-pass per psyche 2026-05-26
    // injects `Unknown(String)` into the wire `Reply` enum. The
    // constructor existing IS the structural proof — if the post-pass
    // didn't fire, this line would fail to compile.
    use signal_persona_spirit::spirit::Reply;
    let _reply: Reply = Reply::Unknown("unknown wire operation".to_string());
}
