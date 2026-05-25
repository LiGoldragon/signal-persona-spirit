//! Verify the `emit_schema!()` invocation in src/lib.rs lands a
//! reachable `signal_persona_spirit::spirit` module alongside the
//! legacy `signal_channel!([schema])` emission per /345 §4 + /346
//! §10.
//!
//! Downstream consumers will migrate from
//! `signal_persona_spirit::Operation` (legacy ChannelSpec emission)
//! to `signal_persona_spirit::spirit::Operation` (emit_schema
//! emission) over the operator integration cycle. Until that flip,
//! both paths resolve.

#[test]
fn schema_driven_module_emits_operation_variants() {
    // Construction of an Operation variant proves the schema-driven
    // module exists and the variants are reachable through it.
    use signal_persona_spirit::spirit::{Operation, StateEndpoint, Statement, StatementText};
    // The schema declares `Statement [StatementText]` and the
    // composer emits a newtype because there's exactly one field
    // with an inferred name.
    let _: Operation = Operation::State(StateEndpoint::Statement(Statement(StatementText(
        String::new(),
    ))));
}

#[test]
fn schema_driven_module_emits_routes_constants() {
    // ROUTES is the closed table emitted from the schema's headers.
    // ROUTE_COUNT lets downstream consumers reason about the
    // top-level operation surface size.
    use signal_persona_spirit::spirit::{ROUTE_COUNT, ROUTES};
    assert_eq!(ROUTES.len(), ROUTE_COUNT);
    // spirit.schema declares 5 operation roots. The const_assert is
    // there to verify the constant the macro emitted.
    const _: () = assert!(ROUTE_COUNT >= 5);
}

#[test]
fn schema_driven_module_emits_extended_header() {
    // The ExtendedHeader is the prefix-preserving 256-byte form
    // per record 657 + /341.
    use signal_persona_spirit::spirit::{EXTENDED_HEADER_BYTE_COUNT, ExtendedHeader};
    assert_eq!(EXTENDED_HEADER_BYTE_COUNT, 256);
    let header = ExtendedHeader::empty();
    assert_eq!(header.as_bytes().len(), 256);
}

#[test]
fn schema_driven_module_emits_effect_table_scaffold() {
    // Per operator/185 the EffectTable scaffold is emitted alongside
    // the Operation enum --- runtime contact point for the schema's
    // effect-side. Even without an authored EffectTable feature,
    // the route-derived legacy effect-table still emits.
    use signal_persona_spirit::spirit::EffectTable;
    let _ = EffectTable;
}
