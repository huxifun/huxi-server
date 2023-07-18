//! 公用组件

pub mod list;

use chrono::{DateTime, Local, Utc};
use maud::{html, Markup};
use serde::Deserialize;

pub type ErrorMessage = Option<Vec<String>>;

pub fn error_message(messages: ErrorMessage) -> Markup {
    html! {
        @if let Some(list) = messages {
            div class="text-danger" {
                p {"提示：输入信息有错误，请修改"}
                ul {
                    @for i in list {
                        li {
                            (i)
                        }
                    }
                }
            }
        }
    }
}

pub fn radio(id: &str, name: &str, value: &str, checked: bool, label: &str) -> Markup {
    let class = "me-1";
    html! {
        span class="p-3 d-inline-block" {
        input id=(id) type="radio" name=(name) class=(class) value=(value) checked[checked];
        label for=(id) {(label)}
        }
    }
}
pub fn checkbox(id: &str, name: &str, value: &str, checked: bool) -> Markup {
    let class = "";
    html! {
        input id=(id) type="checkbox" name=(name) class=(class) value=(value) checked[checked];
    }
}
#[derive(Default)]
pub struct TextInput<'a> {
    id: &'a str,
    name: &'a str,
    value: Option<&'a String>,
    my_type: &'a str,
    required: bool,
    checked: bool,
    class: &'a str,
    placeholder: &'a str,
}
impl<'a> TextInput<'a> {
    pub fn new(id: &'a str, name: &'a str, required: bool) -> Self {
        TextInput {
            id,
            name,
            required,
            value: None,
            checked: false,
            my_type: "text",
            class: "form-control mb-3",
            placeholder: "",
        }
    }
    pub fn with_type(mut self, tt: &'a str) -> Self {
        self.my_type = tt;
        self
    }
    pub fn placeholder(mut self, ph: &'a str) -> Self {
        self.placeholder = ph;
        self
    }
    pub fn value(mut self, value: Option<&'a String>) -> Self {
        self.value = value;
        self
    }
    pub fn checked(mut self) -> Self {
        self.checked = true;
        self
    }
    pub fn show(&self) -> Markup {
        html!(input class=(self.class) id=(self.id) type=(self.my_type) placeholder=(self.placeholder)
              name=(self.name) value=[self.value] required[self.required] checked[self.checked];
        )
    }
}

#[derive(Debug, Default)]
pub struct TextArea<'a> {
    id: &'a str,
    name: &'a str,
    text: Option<&'a String>,
    required: bool,
    class: &'a str,
    rows: u8,
    md: bool,
}
impl<'a> TextArea<'a> {
    pub fn new(id: &'a str, name: &'a str, required: bool) -> Self {
        TextArea {
            id,
            name,
            required,
            text: None,
            class: "form-control",
            rows: 5,
            md: false,
        }
    }
    pub fn text(mut self, text: Option<&'a String>) -> Self {
        self.text = text;
        self
    }
    pub fn rows(mut self, rows: u8) -> Self {
        self.rows = rows;
        self
    }
    pub fn md(mut self) -> Self {
        self.md = true;
        self
    }
    pub fn show(&self) -> Markup {
        html!(
            textarea class=(self.class) id=(self.id) name=(self.name) rows=(self.rows)
                required[self.required] {(self.text.map_or("", |s| s.as_str()))}
            @if self.md {
                div class="text-end mb-2" {
                    a href="https://crates.io/crates/pulldown-cmark" target="_blank" class="text-muted" {
                        small {
                            em{"支持 Markdown"}
                        }
                    }
                }
            }
        )
    }
}

pub fn submit(text: &str) -> Markup {
    html! {
        button type="submit" class="btn btn-primary m-3" {(text)}
    }
}

pub fn tip(text: &str) -> Markup {
    html! {
        div class="row justify-content-center" {
            div class="col col-md-7  col-xl-5 border p-2 shadow mb-3 bg-body rounded" {
                div class="alert alert-warning m-1" role="alert" {
                  "提示："  (text)
                }
            }
        }
    }
}

pub fn show_date(dt: DateTime<Utc>) -> String {
    dt.with_timezone(&Local).format("%Y-%m-%d").to_string()
}
pub fn show_day(dt: DateTime<Utc>) -> String {
    dt.with_timezone(&Local).format("%m-%d").to_string()
}
pub fn show_time(dt: DateTime<Utc>) -> String {
    dt.with_timezone(&Local)
        .format("%Y-%m-%d %H:%M:%S")
        .to_string()
}

pub fn pager(url: &str, total: i64, size: u8, current_page: u32) -> Markup {
    let pages = f64::ceil(total as f64 / size as f64) as u32;
    let cp = current_page - 1;
    let cn = current_page + 1;
    html!(
        div class="d-flex justify-content-center" aria-label="..."{
            ul class="pagination" {
                @if total >  size as i64 {
                    @if current_page != 1 {
                        li class="page-item" {
                            a class="page-link" href={(url) "?page=" (cp)} aria-label="上一页" {
                                span aria-hidden="true" {"《"}
                            }
                        }
                     }
                    @for page in 1..=pages {
                        @if current_page == page {
                            li class="page-item active" aria-current="page" {
                                span class="page-link" { (page) }
                             }
                        } @else {
                            li class="page-item" {
                                a class="page-link" href={(url) "?page=" (page)} {(page)}
                             }
                        }
                    }

                    @if current_page < pages {
                      li class="page-item" {
                          a class="page-link" href={(url) "?page=" (cn)} aria-label="下一页" {
                              span aria-hidden="true" {"》"}
                          }
                      }
                    }
                }
            }
        }
    )
}

#[derive(Deserialize, Debug)]
pub struct Pagination {
    pub page: u32,
}

impl Default for Pagination {
    fn default() -> Self {
        Self { page: 1 }
    }
}

#[derive(Deserialize, Debug, Default)]
pub struct CatType {
    pub cat: i16,
    pub typ: i16,
}

pub fn redirect_script(url: &str) -> String {
    format!(
        r#"
<div class="text-center m-2"><span id="s"></span></div>
<script type="text/javascript" language="javascript">
    var time=3;
    function Redirect() {{
        window.location = "{}";
    }}
    var i=0;
    function dis() {{
      document.all.s.innerHTML = "" + (time - i) + "";
      i++;
    }}
    setInterval('dis()', 1000);
    setTimeout('Redirect()', time * 1000);
</script>
"#,
        url
    )
}
