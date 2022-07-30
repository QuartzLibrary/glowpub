use super::{raw_content_page, raw_copyright_page, raw_title_page, Thread, STYLE};

impl Thread {
    pub fn to_single_html_page(&self, for_tts: bool) -> String {
        let front = raw_title_page(&self.post, self.replies.len());
        let content = raw_content_page(&self.content_blocks(for_tts));
        let back = raw_copyright_page(&self.post);

        wrap_html(&self.post.subject, &format!("{front}{content}{back}"))
    }
}

fn wrap_html(subject: &str, content: &str) -> String {
    format!(
        r##"<!DOCTYPE html>
<html lang="en">
    <head>
        <meta charset="utf-8"/>
        <meta name="viewport" content="width=device-width, initial-scale=1.0"/>
        <meta name="theme-color" content="#000000"/>
        <title>{subject}</title>

        <style>
            {STYLE}
        </style>

    </head>
    <body>

        {content}

    </body>
</html>

    "##
    )
}
