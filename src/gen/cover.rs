use std::fmt::Write;

use crate::types::User;

pub fn image(subject: &str, authors: &[User]) -> Vec<u8> {
    let svg = svg(subject, authors);

    render_svg(&svg)
}

const WIDTH: u32 = 1600;
const HALF_WIDTH: u32 = WIDTH / 2; // 800
const HEIGHT: u32 = 2650;
const FIFTH_HEIGHT: u32 = HEIGHT / 5; // 530
const ALMOST_BOTTOM: u32 = HEIGHT * 8 / 10; // 2120

const TITLE_TEXT_SIZE: u32 = 100;
const AUTHORS_TEXT_SIZE: u32 = 80;
const STYLE: &str = r##"margin="20px" max-width="100%" text-anchor="middle" font-family="serif""##;

fn svg(subject: &str, authors: &[User]) -> String {
    let subject = super::transform::escape_html(subject);
    let subject: String = textwrap::wrap(&subject, 25).iter().enumerate().fold(
        String::new(),
        |mut subject, (i, part)| {
            let i: u32 = i.try_into().unwrap();
            write!(
                subject,
                r##"<tspan x="{HALF_WIDTH}" y="{}">{part}</tspan>"##,
                FIFTH_HEIGHT + (TITLE_TEXT_SIZE * (i + 1))
            )
            .unwrap();
            subject
        },
    );
    let authors: String = textwrap::wrap(&super::author_names(authors), 25)
        .iter()
        .enumerate()
        .fold(String::new(), |mut authors, (i, part)| {
            let i: u32 = i.try_into().unwrap();
            write!(
                authors,
                r##"<tspan x="{HALF_WIDTH}" y="{}">{part}</tspan>"##,
                ALMOST_BOTTOM + (AUTHORS_TEXT_SIZE * (i + 1))
            )
            .unwrap();
            authors
        });

    format!(
        r##"<svg viewBox="0 0 {WIDTH} {HEIGHT}" xmlns="http://www.w3.org/2000/svg">
        <style>svg {{ background-color: #F5F5DF; }}</style>
        <rect width="100%" height="100%" fill="#F5F5DF"/>
        <text x="{HALF_WIDTH}" y="{FIFTH_HEIGHT}" font-size="{TITLE_TEXT_SIZE}px" {STYLE} class="title">
            {subject}
        </text>
        <text x="{HALF_WIDTH}" y="{ALMOST_BOTTOM}" font-size="{AUTHORS_TEXT_SIZE}px" {STYLE} class="authors">
            {authors}
        </text>
      </svg>
      "##
    )
}

fn render_svg(svg: &str) -> Vec<u8> {
    let db = {
        let mut db = fontdb::Database::new();
        for font in FONTS {
            db.load_font_data(font.to_vec());
        }
        db.set_serif_family("Cinzel");
        db
    };

    let tree = usvg::Tree::from_str(
        svg,
        &usvg::Options {
            font_family: "Cinzel".to_string(),
            ..Default::default()
        },
        &db,
    )
    .unwrap();

    let pixmap = {
        let mut pixmap = tiny_skia::Pixmap::new(WIDTH, HEIGHT).unwrap();
        resvg::render(&tree, tiny_skia::Transform::default(), &mut pixmap.as_mut());
        pixmap
    };

    pixmap.encode_png().unwrap()
}

const FONTS: &[&[u8]] = &[include_bytes!("../../fonts/Cinzel-VariableFont_wght.ttf")];
