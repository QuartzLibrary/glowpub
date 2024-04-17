use std::{borrow::Cow, sync::OnceLock};

use ammonia::{Builder, UrlRelative};
use lightningcss::{
    properties::{Property, PropertyId},
    stylesheet::ParserOptions,
};

pub fn repair_and_sanitize(content: &str) -> String {
    let document = cleaner().clean(content);
    document.to_string()
}

fn cleaner() -> &'static ammonia::Builder<'static> {
    fn new_cleaner() -> ammonia::Builder<'static> {
        let mut builder = Builder::default();
        builder
            .url_relative(UrlRelative::RewriteWithBase(
                "https://glowfic.com/".try_into().unwrap(),
            ))
            .strip_comments(false)
            .add_generic_attributes(ALLOWED_ATTRIBUTES)
            .attribute_filter(allowed_attribute);
        builder
    }
    static CLEANER: OnceLock<ammonia::Builder> = OnceLock::new();
    CLEANER.get_or_init(new_cleaner)
}

fn allowed_attribute<'u>(element: &str, attribute: &str, value: &'u str) -> Option<Cow<'u, str>> {
    match (element, attribute, value) {
        (_, "style", _) => style::allowed(value),

        (_, "title" | "lang" | "aria-label", _) => Some(Cow::Borrowed(value)),

        (_, "id" | "name", _) => None,
        (_, "class", class) if CLASSES.contains(&class) => Some(Cow::Borrowed(value)),
        (_, "class", _) => {
            log::info!("Removing unrecognised class: <{element} {attribute}=\"{value}\">");
            None
        }

        ("a", "href", _) => Some(Cow::Borrowed(value)),
        ("a", "rel", "noopener noreferrer") => Some(Cow::Borrowed(value)),
        ("a", "target", "_blank" | "_self" | "_parent" | "_top") => Some(Cow::Borrowed(value)),

        ("img", "src", _) => Some(Cow::Borrowed(value)),
        ("img", "alt", _) => Some(Cow::Borrowed(value)),

        ("img" | "table", "width" | "height", _)
            if Property::parse_string(PropertyId::Width, value, ParserOptions::default())
                .is_ok() =>
        {
            Some(Cow::Borrowed(value))
        }
        ("img" | "table", "border", _)
            if Property::parse_string(PropertyId::Border, value, ParserOptions::default())
                .is_ok() =>
        {
            Some(Cow::Borrowed(value))
        }

        ("ol", "start", _) if value.parse::<u128>().is_ok() => Some(Cow::Borrowed(value)),
        ("td" | "th", "colspan", _) if value.parse::<u128>().is_ok() => Some(Cow::Borrowed(value)),

        (_, "dir", "ltr" | "rtl") => Some(Cow::Borrowed(value)),

        ("details", "open", "" | "open") => Some(Cow::Borrowed(value)),
        ("details", "open", "closed") => None, // Any value means open

        // Deprecated
        (_, "align", "left" | "right" | "center" | "justify") => Some(Cow::Borrowed(value)),

        // Internal
        (
            _,
            "post-id" | "icon-id" | "board-id" | "author-id" | "character-id" | "reply-id",
            value,
        ) if value.parse::<u128>().is_ok() => Some(Cow::Borrowed(value)),
        (_, "author-ids" | "author-name" | "character-name", value) => Some(Cow::Borrowed(value)),

        (_, _, _) => {
            log::info!("Removing unrecognised attribute: <{element} {attribute}=\"{value}\">");
            None
        }
    }
}
mod style {
    use std::borrow::Cow;

    use cssparser::{Parser, ParserInput, ToCss};
    use lightningcss::{
        declaration::DeclarationBlock,
        printer::Printer,
        properties::Property,
        stylesheet::{ParserOptions, PrinterOptions},
    };

    pub(super) fn allowed(value: &str) -> Option<Cow<'_, str>> {
        let Ok(declarations) = parse_declaration_block(value) else {
            log::info!("Removing style attribute that has failed to parse: style=\"{value}\"");
            return None;
        };

        let new = clean_declarations(declarations.clone());
        if new == declarations {
            return Some(Cow::Borrowed(value));
        }

        let new_value = serialize_declaration_block(&new);
        log::info!("Removing unrecognised or forbidden properties from style attribute: \nstyle=\"{value}\"\n-> style=\"{new_value}\"");
        if new_value.is_empty() {
            None
        } else {
            Some(Cow::Owned(new_value))
        }
    }
    fn clean_declarations(mut declarations: DeclarationBlock<'_>) -> DeclarationBlock<'_> {
        declarations.important_declarations.retain(allowed_property);
        declarations.declarations.retain(allowed_property);
        declarations
    }
    fn allowed_property(property: &Property<'_>) -> bool {
        match property {
            // All of these could cause network calls
            Property::Background(_)
            | Property::BackgroundImage(_)
            | Property::BorderImage(_, _)
            | Property::BorderImageSource(_)
            | Property::FontFamily(_)
            | Property::ListStyle(_)
            | Property::ListStyleType(_)
            | Property::ListStyleImage(_)
            | Property::Mask(_, _)
            | Property::MaskImage(_, _)
            | Property::MaskBorder(_)
            | Property::MaskBorderSource(_)
            | Property::WebKitMaskBoxImage(_, _)
            | Property::WebKitMaskBoxImageSource(_, _)
            | Property::ClipPath(_, _)
            | Property::Filter(_, _)
            | Property::BackdropFilter(_, _)
            | Property::Cursor(_) => false,

            Property::Display(_)
            | Property::Position(_)
            | Property::Overflow(_)
            | Property::OverflowWrap(_)
            | Property::BoxSizing(_, _)
            | Property::Transform(_, _)
            | Property::Width(_)
            | Property::MinWidth(_)
            | Property::MaxWidth(_)
            | Property::Height(_)
            | Property::MinHeight(_)
            | Property::MaxHeight(_)
            | Property::Margin(_)
            | Property::MarginTop(_)
            | Property::MarginRight(_)
            | Property::MarginBottom(_)
            | Property::MarginLeft(_)
            | Property::MarginInline(_)
            | Property::MarginInlineStart(_)
            | Property::MarginInlineEnd(_)
            | Property::ScrollMarginInline(_)
            | Property::ScrollMarginInlineStart(_)
            | Property::ScrollMarginInlineEnd(_)
            | Property::Padding(_)
            | Property::PaddingTop(_)
            | Property::PaddingRight(_)
            | Property::PaddingBottom(_)
            | Property::PaddingLeft(_)
            | Property::Border(_)
            | Property::BorderTop(_)
            | Property::BorderRight(_)
            | Property::BorderBottom(_)
            | Property::BorderLeft(_)
            | Property::BorderRadius(_, _)
            | Property::Outline(_)
            | Property::VerticalAlign(_)
            | Property::Color(_)
            | Property::BackgroundColor(_)
            | Property::BackgroundPosition(_)
            | Property::BackgroundPositionX(_)
            | Property::BackgroundPositionY(_)
            | Property::BackgroundRepeat(_)
            | Property::FontSize(_)
            | Property::FontWeight(_)
            | Property::FontStyle(_)
            | Property::FontVariantCaps(_)
            | Property::FontStretch(_)
            | Property::LineHeight(_)
            | Property::TextDecoration(_, _)
            | Property::TextDecorationLine(_, _)
            | Property::TextDecorationColor(_, _)
            | Property::TextDecorationStyle(_, _)
            | Property::TextDecorationThickness(_)
            | Property::TextAlign(_)
            | Property::TextIndent(_)
            | Property::TextTransform(_)
            | Property::TextShadow(_)
            | Property::TextSizeAdjust(_, _)
            | Property::LetterSpacing(_)
            | Property::WordWrap(_)
            | Property::WordSpacing(_)
            | Property::WhiteSpace(_)
            | Property::Caret(_)
            | Property::CaretColor(_)
            | Property::ListStylePosition(_) => true,

            Property::Unparsed(_) | Property::Custom(_) => allowed_property_manual(property),

            _ => false,
        }
    }
    fn allowed_property_manual(property: &Property<'_>) -> bool {
        let (prefix, name, value) = property_prefix_name_and_value(property);
        match (&*prefix, &*name, &*value) {
            (_, "clear", "none" | "left" | "right" | "both" | "inline-star" | "inline-end")
            | (_, "float", "none" | "left" | "right" | "inline-start" | "inline-end")
            | (
                _,
                "break-before",
                "auto" | "avoid" | "always" | "all" | "avoid-page" | "page" | "left" | "right"
                | "recto" | "verso" | "avoid-column" | "column" | "avoid-region" | "region",
            )
            | (_, "font-kerning", "auto" | "normal" | "none") => true,
            (_, "orphans" | "widows", _) if value.parse::<u128>().is_ok() => true,
            _ => false,
        }
    }
    pub fn property_prefix_name_and_value(property: &Property) -> (String, String, String) {
        let vendor_prefix = property.property_id().prefix().to_css_string();
        let name = property.property_id().name().to_owned();
        let value = {
            let value = property
                .to_css_string(false, PrinterOptions::default())
                .unwrap();
            let value = value
                .strip_prefix(&vendor_prefix)
                .unwrap()
                .strip_prefix(&name)
                .unwrap()
                .strip_prefix(": ")
                .unwrap()
                .to_owned();
            value
        };
        (vendor_prefix, name, value)
    }
    fn parse_declaration_block(
        value: &str,
    ) -> Result<DeclarationBlock<'_>, cssparser::ParseError<'_, lightningcss::error::ParserError<'_>>>
    {
        let mut input = ParserInput::new(value);
        let mut parser = Parser::new(&mut input);
        DeclarationBlock::parse(&mut parser, &ParserOptions::default())
    }
    fn serialize_declaration_block(declarations: &DeclarationBlock<'_>) -> String {
        let mut css = String::new();
        let mut printer = Printer::new(&mut css, PrinterOptions::default());
        for property in &declarations.important_declarations {
            property.to_css(&mut printer, true).unwrap();
            printer.delim(';', false).unwrap();
        }
        for property in &declarations.declarations {
            property.to_css(&mut printer, false).unwrap();
            printer.delim(';', false).unwrap();
        }
        css.trim().to_owned()
    }
}

/// A list of all the attirbutes I have seen in glowfic content.
/// Unusual ones that are unlikely to be useful are commented out.
const ALLOWED_ATTRIBUTES: &[&str] = &[
    "align",
    "alt",
    "aria-label",
    "border",
    "boundary",
    // "cellspacing",
    "class",
    "colspan",
    // "data-confirm",
    // "data-darkreader-inline-bgcolor",
    // "data-darkreader-inline-bgimage",
    // "data-darkreader-inline-border-bottom",
    // "data-darkreader-inline-border-left",
    // "data-darkreader-inline-border-right",
    // "data-darkreader-inline-border-top",
    // "data-darkreader-inline-color",
    // "data-mce-bogus",
    // "data-mce-style",
    // "data-method",
    // "DefLockedState",
    // "DefPriority",
    // "DefQFormat",
    // "DefSemiHidden",
    // "DefUnhideWhenUsed",
    "dir",
    "height",
    "href",
    "id",
    "lang",
    // "LatentStyleCount",
    // "Locked",
    "name",
    // "Name",
    "open",
    // "Priority",
    // "QFormat",
    "role",
    // "SemiHidden",
    "src",
    "start",
    "style",
    "target",
    "title",
    // "UnhideWhenUsed",
    "width",
    // "rel", // Handled separately

    // Internal
    "post-id",
    "author-ids",
    "icon-id",
    "board-id",
    "author-id",
    "author-name",
    "character-id",
    "character-name",
    "reply-id",
];

/// Keep in sync with `book.css`
const CLASSES: &[&str] = &[
    "title-page",
    "copyright-page",
    "description",
    "content",
    "content-block",
    "character",
    "character",
    "icon",
    "icon-caption",
];
