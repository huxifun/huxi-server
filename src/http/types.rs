use axum::http::{header, HeaderMap, HeaderValue};
use axum::response::{IntoResponse, Response};
use maud::PreEscaped;
use maud::{Escaper, Render};
use std::fmt;
use std::fmt::Write as _;

pub struct Page(pub PreEscaped<String>);

impl IntoResponse for Page {
    fn into_response(self) -> Response {
        let mut headers = HeaderMap::new();
        headers.insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static("text/html; charset=utf-8"),
        );
        (headers, self.0 .0).into_response()
    }
}

struct Debug<T: fmt::Debug>(T);

impl<T: fmt::Debug> Render for Debug<T> {
    fn render_to(&self, output: &mut String) {
        let mut escaper = Escaper::new(output);
        write!(escaper, "{:?}", self.0).unwrap();
    }
}
