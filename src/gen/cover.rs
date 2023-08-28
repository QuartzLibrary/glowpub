use usvg::TreeParsing;
use usvg_text_layout::TreeTextToPath;

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
    let subject = super::transform::encode_html(subject);
    let subject: String = textwrap::wrap(&subject, 25)
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
    let mut tree = usvg::Tree::from_str(
        svg,
        &usvg::Options {
            font_family: "Cinzel".to_string(),
            ..Default::default()
        },
    )
    .unwrap();

    let db = {
        let mut db = fontdb::Database::new();
        for font in FONTS {
            db.load_font_data(font.to_vec());
        }
        db.set_serif_family("Cinzel");
        db
    };
    tree.convert_text(&db);
    let tree = resvg::Tree::from_usvg(&tree);

    let pixmap = {
        let mut pixmap = tiny_skia::Pixmap::new(WIDTH, HEIGHT).unwrap();
        tree.render(tiny_skia::Transform::default(), &mut pixmap.as_mut());
        pixmap
    };

    pixmap.encode_png().unwrap()
}

const FONTS: &[&[u8]] = &[
    include_bytes!("../../fonts/Cinzel-VariableFont_wght.ttf"),
    include_bytes!("../../fonts/NotoNaskhArabic-VariableFont_wght.ttf"),
    include_bytes!("../../fonts/NotoNastaliqUrdu-VariableFont_wght.ttf"),
    include_bytes!("../../fonts/NotoRashiHebrew-VariableFont_wght.ttf"),
    include_bytes!("../../fonts/NotoSerif-Italic-VariableFont_wdth,wght.ttf"),
    include_bytes!("../../fonts/NotoSerif-VariableFont_wdth,wght.ttf"),
    include_bytes!("../../fonts/NotoSerifAhom-Regular.ttf"),
    include_bytes!("../../fonts/NotoSerifArmenian-VariableFont_wdth,wght.ttf"),
    include_bytes!("../../fonts/NotoSerifBalinese-Regular.ttf"),
    include_bytes!("../../fonts/NotoSerifBengali-VariableFont_wdth,wght.ttf"),
    include_bytes!("../../fonts/NotoSerifDevanagari-VariableFont_wdth,wght.ttf"),
    include_bytes!("../../fonts/NotoSerifDisplay-Italic-VariableFont_wdth,wght.ttf"),
    include_bytes!("../../fonts/NotoSerifDisplay-VariableFont_wdth,wght.ttf"),
    include_bytes!("../../fonts/NotoSerifDogra-Regular.ttf"),
    include_bytes!("../../fonts/NotoSerifEthiopic-VariableFont_wdth,wght.ttf"),
    include_bytes!("../../fonts/NotoSerifGeorgian-VariableFont_wdth,wght.ttf"),
    include_bytes!("../../fonts/NotoSerifGrantha-Regular.ttf"),
    include_bytes!("../../fonts/NotoSerifGujarati-VariableFont_wght.ttf"),
    include_bytes!("../../fonts/NotoSerifGurmukhi-VariableFont_wght.ttf"),
    include_bytes!("../../fonts/NotoSerifHebrew-VariableFont_wdth,wght.ttf"),
    include_bytes!("../../fonts/NotoSerifHK-VariableFont_wght.ttf"),
    include_bytes!("../../fonts/NotoSerifJP-Black.otf"),
    include_bytes!("../../fonts/NotoSerifJP-Bold.otf"),
    include_bytes!("../../fonts/NotoSerifJP-ExtraLight.otf"),
    include_bytes!("../../fonts/NotoSerifJP-Light.otf"),
    include_bytes!("../../fonts/NotoSerifJP-Medium.otf"),
    include_bytes!("../../fonts/NotoSerifJP-Regular.otf"),
    include_bytes!("../../fonts/NotoSerifJP-SemiBold.otf"),
    include_bytes!("../../fonts/NotoSerifKannada-VariableFont_wght.ttf"),
    include_bytes!("../../fonts/NotoSerifKhitanSmallScript-Regular.ttf"),
    include_bytes!("../../fonts/NotoSerifKhmer-VariableFont_wdth,wght.ttf"),
    include_bytes!("../../fonts/NotoSerifKhojki-VariableFont_wght.ttf"),
    include_bytes!("../../fonts/NotoSerifKR-Black.otf"),
    include_bytes!("../../fonts/NotoSerifKR-Bold.otf"),
    include_bytes!("../../fonts/NotoSerifKR-ExtraLight.otf"),
    include_bytes!("../../fonts/NotoSerifKR-Light.otf"),
    include_bytes!("../../fonts/NotoSerifKR-Medium.otf"),
    include_bytes!("../../fonts/NotoSerifKR-Regular.otf"),
    include_bytes!("../../fonts/NotoSerifKR-SemiBold.otf"),
    include_bytes!("../../fonts/NotoSerifLao-VariableFont_wdth,wght.ttf"),
    include_bytes!("../../fonts/NotoSerifMakasar-Regular.ttf"),
    include_bytes!("../../fonts/NotoSerifMalayalam-VariableFont_wght.ttf"),
    include_bytes!("../../fonts/NotoSerifMyanmar-Black.ttf"),
    include_bytes!("../../fonts/NotoSerifMyanmar-Bold.ttf"),
    include_bytes!("../../fonts/NotoSerifMyanmar-ExtraBold.ttf"),
    include_bytes!("../../fonts/NotoSerifMyanmar-ExtraLight.ttf"),
    include_bytes!("../../fonts/NotoSerifMyanmar-Light.ttf"),
    include_bytes!("../../fonts/NotoSerifMyanmar-Medium.ttf"),
    include_bytes!("../../fonts/NotoSerifMyanmar-Regular.ttf"),
    include_bytes!("../../fonts/NotoSerifMyanmar-SemiBold.ttf"),
    include_bytes!("../../fonts/NotoSerifMyanmar-Thin.ttf"),
    include_bytes!("../../fonts/NotoSerifNPHmong-VariableFont_wght.ttf"),
    include_bytes!("../../fonts/NotoSerifOriya-VariableFont_wght.ttf"),
    include_bytes!("../../fonts/NotoSerifOttomanSiyaq-Regular.ttf"),
    include_bytes!("../../fonts/NotoSerifSC-Black.otf"),
    include_bytes!("../../fonts/NotoSerifSC-Bold.otf"),
    include_bytes!("../../fonts/NotoSerifSC-ExtraLight.otf"),
    include_bytes!("../../fonts/NotoSerifSC-Light.otf"),
    include_bytes!("../../fonts/NotoSerifSC-Medium.otf"),
    include_bytes!("../../fonts/NotoSerifSC-Regular.otf"),
    include_bytes!("../../fonts/NotoSerifSC-SemiBold.otf"),
    include_bytes!("../../fonts/NotoSerifSinhala-VariableFont_wdth,wght.ttf"),
    include_bytes!("../../fonts/NotoSerifTamil-Italic-VariableFont_wdth,wght.ttf"),
    include_bytes!("../../fonts/NotoSerifTamil-VariableFont_wdth,wght.ttf"),
    include_bytes!("../../fonts/NotoSerifTangut-Regular.ttf"),
    include_bytes!("../../fonts/NotoSerifTC-Black.otf"),
    include_bytes!("../../fonts/NotoSerifTC-Bold.otf"),
    include_bytes!("../../fonts/NotoSerifTC-ExtraLight.otf"),
    include_bytes!("../../fonts/NotoSerifTC-Light.otf"),
    include_bytes!("../../fonts/NotoSerifTC-Medium.otf"),
    include_bytes!("../../fonts/NotoSerifTC-Regular.otf"),
    include_bytes!("../../fonts/NotoSerifTC-SemiBold.otf"),
    include_bytes!("../../fonts/NotoSerifTelugu-VariableFont_wght.ttf"),
    include_bytes!("../../fonts/NotoSerifThai-VariableFont_wdth,wght.ttf"),
    include_bytes!("../../fonts/NotoSerifTibetan-VariableFont_wght.ttf"),
    include_bytes!("../../fonts/NotoSerifToto-VariableFont_wght.ttf"),
    include_bytes!("../../fonts/NotoSerifVithkuqi-VariableFont_wght.ttf"),
    include_bytes!("../../fonts/NotoSerifYezidi-VariableFont_wght.ttf"),
];
