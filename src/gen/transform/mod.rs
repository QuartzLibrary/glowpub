mod edit_image_urls;
mod flatten_details;
mod html_to_xml;
mod named_entities;
mod sanitize;

pub use edit_image_urls::edit_image_urls;
pub use flatten_details::flatten_details;
pub use html_to_xml::html_to_xml;
pub use named_entities::decode_named_entities;
pub use sanitize::repair_and_sanitize;

/// The following characters (HTML reserved characters) are escaped:
///
/// - `&` => `&amp;`
/// - `<` => `&lt;`
/// - `>` => `&gt;`
/// - `"` => `&quot;`
/// - `'` => `&#x27;`
pub fn escape_html(v: &str) -> String {
    decode_named_entities(html_escape::encode_quoted_attribute(v).to_string())
}
