use ammonia;
use maud::{Markup, PreEscaped, Render};
use pulldown_cmark::{html, Options, Parser};

struct Markdown<T: AsRef<str>>(T);

impl<T: AsRef<str>> Render for Markdown<T> {
    fn render(&self) -> Markup {
        // Generate raw HTML
        let mut unsafe_html = String::new();
        let parser = Parser::new_ext(self.0.as_ref(), Options::all());
        html::push_html(&mut unsafe_html, parser);
        // Sanitize it with ammonia
        let safe_html = ammonia::clean(&unsafe_html);
        PreEscaped(safe_html)
    }
}

fn get_parser(md: &str) -> Parser {
    Parser::new_ext(md, Options::all())
}
pub fn to_html(md: &str) -> String {
    let mut out_html = String::new();
    html::push_html(&mut out_html, get_parser(md));
    out_html
}
