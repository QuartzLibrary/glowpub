mod flatten_details;
mod html_to_xml;
mod named_entities;
mod sanitize;

pub use flatten_details::flatten_details;
pub use html_to_xml::html_to_xml;
pub use named_entities::decode_named_entities;
pub use sanitize::repair_and_sanitize;
