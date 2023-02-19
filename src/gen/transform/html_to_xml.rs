use html5ever::{tendril::TendrilSink, tree_builder::TreeSink};
use markup5ever::{LocalName, Namespace, QualName};
use markup5ever_rcdom::{RcDom, SerializableHandle};

/// This function parses the html and re-serializes it as xml/xhtml.
///
/// This is mostly needed because self-closing/void tags in html look like `<br>`,
/// while in xhtml they need to be either `<br/>` or `<br></br>`.
///
/// Browsers will handle all of those fine as they are very permissive,
/// but some e-readers will break if we use plain html.
pub fn html_to_xml(content: &str) -> String {
    serialize_xml(parse_html(content))
}
fn parse_html(content: &str) -> RcDom {
    let qual_name = QualName::new(
        None,
        Namespace::from("http://www.w3.org/1999/xhtml"),
        LocalName::from("body"),
    );

    html5ever::parse_fragment(
        RcDom::default(),
        html5ever::ParseOpts::default(),
        qual_name,
        vec![],
    )
    .one(content)
    .finish()
}
fn serialize_xml(content: RcDom) -> String {
    let document: SerializableHandle = content.document.into();

    let mut bytes = vec![];
    xml5ever::serialize::serialize(
        &mut bytes,
        &document,
        xml5ever::serialize::SerializeOpts {
            traversal_scope: xml5ever::serialize::TraversalScope::ChildrenOnly(None),
        },
    )
    .unwrap();

    // This is serialised as "<html xmlns="http://www.w3.org/1999/xhtml">(content)</html>"
    // but we don't want the wrapping tag.
    let bytes = bytes[43..(bytes.len() - 7)].to_vec();

    String::from_utf8(bytes).unwrap()
}
