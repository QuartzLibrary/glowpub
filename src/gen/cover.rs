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
    let subject: String = textwrap::wrap(subject, 25)
        .iter()
        .enumerate()
        .map(|(i, part)| {
            let i: u32 = i.try_into().unwrap();
            format!(
                r##"<tspan x="{HALF_WIDTH}" y="{}">{part}</tspan>"##,
                FIFTH_HEIGHT + (TITLE_TEXT_SIZE * (i + 1))
            )
        })
        .collect();
    let authors: String = textwrap::wrap(&super::author_names(authors), 25)
        .iter()
        .enumerate()
        .map(|(i, part)| {
            let i: u32 = i.try_into().unwrap();
            format!(
                r##"<tspan x="{HALF_WIDTH}" y="{}">{part}</tspan>"##,
                ALMOST_BOTTOM + (AUTHORS_TEXT_SIZE * (i + 1))
            )
        })
        .collect();

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
    let tree = {
        let svg_options = usvg::Options {
            fontdb: {
                let mut db = fontdb::Database::new();
                db.load_font_data(include_bytes!("Cinzel-VariableFont_wght.ttf").to_vec());
                db.set_serif_family("Cinzel");
                db
            },
            font_family: "Cinzel".to_string(),
            ..Default::default()
        };
        usvg::Tree::from_str(svg, &svg_options.to_ref()).unwrap()
    };

    let pixmap = {
        let mut pixmap = tiny_skia::Pixmap::new(WIDTH, HEIGHT).unwrap();
        resvg::render(
            &tree,
            usvg::FitTo::Original,
            tiny_skia::Transform::default(),
            pixmap.as_mut(),
        )
        .unwrap();
        pixmap
    };

    pixmap.encode_png().unwrap()
}
