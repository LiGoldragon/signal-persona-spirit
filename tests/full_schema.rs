use signal_frame::{LogVariant, ShortHeader};

signal_frame::emit_schema!("spirit.schema");

#[test]
fn schema_generated_spirit_signal_matches_current_record_header() {
    let entry = spirit::Entry {
        topics: spirit::Topics(vec![
            spirit::Topic("schema".to_string()),
            spirit::Topic("spirit".to_string()),
        ]),
        kind: spirit::Kind::Decision,
        description: spirit::Description("schema carries spirit v0.3".to_string()),
        certainty: spirit::Magnitude,
    };
    let operation = spirit::Operation::Record(spirit::RecordEndpoint::Entry(entry));

    assert_eq!(
        operation.log_variant(),
        u64::from_le_bytes([1, 0, 0, 0, 0, 0, 0, 0])
    );

    let route = spirit::route_for_short_header(
        spirit::Leg::Ordinary,
        ShortHeader::new(operation.log_variant()),
    )
    .expect("ordinary record route");

    assert_eq!(route.root, "Record");
    assert_eq!(route.endpoint, "Entry");
    assert_eq!(route.body, spirit::RouteBodyDescriptor::Type("Entry"));
}

#[test]
fn schema_generated_spirit_sema_language_routes_command_effect_and_response() {
    assert!(
        spirit::ROUTES
            .iter()
            .any(|route| route.leg == spirit::Leg::Sema
                && route.root == "Project"
                && route.endpoint == "AssertEntry")
    );
    assert!(
        spirit::ROUTES
            .iter()
            .any(|route| route.leg == spirit::Leg::Sema
                && route.root == "Emit"
                && route.endpoint == "EntryAsserted")
    );
    assert!(
        spirit::ROUTES
            .iter()
            .any(|route| route.leg == spirit::Leg::Sema
                && route.root == "Respond"
                && route.endpoint == "RecordAccepted")
    );

    let command = spirit::SemaTurn::Project(spirit::ProjectEndpoint::AssertEntry(spirit::Entry {
        topics: spirit::Topics(vec![spirit::Topic("schema".to_string())]),
        kind: spirit::Kind::Principle,
        description: spirit::Description("schema projects command".to_string()),
        certainty: spirit::Magnitude,
    }));

    let route =
        spirit::route_for_short_header(spirit::Leg::Sema, ShortHeader::new(command.log_variant()))
            .expect("sema command route");

    assert_eq!(route.root, "Project");
    assert_eq!(route.endpoint, "AssertEntry");
}

#[test]
fn schema_generated_effect_table_maps_spirit_command_to_fan_out_reply() {
    assert_eq!(
        spirit::AuthoredEffectTable::effect_for_action("AssertEntry"),
        Some("EntryAsserted")
    );

    let fan_out = spirit::AuthoredEffectTable::fan_out_for_effect("EntryAsserted")
        .expect("entry effect fan-out");

    assert_eq!(
        fan_out.outputs,
        vec![spirit::AuthoredFanOutOutput::Reply {
            variant: "RecordAccepted"
        }]
    );
    assert_eq!(
        spirit::AuthoredEffectTable::effect_for_action("UnknownCommand"),
        None
    );
}
