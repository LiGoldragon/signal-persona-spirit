use signal_persona_spirit::{Description, Entry, Kind, Topic, Topics};
use signal_sema::Magnitude;

fn entry() -> Entry {
    Entry {
        topics: Topics::single(Topic::new("workspace")),
        kind: Kind::Decision,
        description: Description::new("schema box description"),
        certainty: Magnitude::High,
    }
}

#[test]
fn entry_uses_schema_derived_text_box_form() {
    let text = nota_box::encode_text(&entry()).unwrap();

    assert_eq!(
        text,
        "(Entry Decision High) [workspace] [schema box description]"
    );

    let decoded: Entry = nota_box::decode_text(&text).unwrap();
    assert_eq!(decoded, entry());
}

#[test]
fn entry_uses_schema_derived_binary_box_form_with_peekable_boxes() {
    let bytes = nota_box::encode_binary(&entry()).unwrap();
    let root_length = nota_box::root_text_length(&bytes).unwrap();

    assert_eq!(root_length, "(Entry Decision High)".len());
    assert_eq!(
        nota_box::peek_box(&bytes, root_length, 1).unwrap(),
        b"[schema box description]"
    );

    let decoded: Entry = nota_box::decode_binary(&bytes).unwrap();
    assert_eq!(decoded, entry());
}
