use crate::config::CategoryType;
use crate::fun::book::db::BookSimple;
use crate::fun::hu::db::HuSimple;
use crate::fun::widget::*;
use crate::fun::xi::db::XiSimple;
use crate::http::WebContext;
use maud::{html, Markup, PreEscaped};

pub const PUBLIC_STATUS_HTML: [(i16, &str); 2] = [
    (0, "<span class='text-danger'>草稿</span>"),
    (1, "<span class='text-success'>已公布</span>"),
];

pub fn get_status_name<const N: usize>(status_map: [(i16, &str); N], idx: i16) -> Option<&str> {
    status_map.iter().find(|x| x.0 == idx).map(|x| x.1)
}

pub fn get_igood_status<'a>(i_good: i16, good: i16) -> &'a str {
    if i_good == 1 && good == 0 {
        return "申请推荐";
    }
    if good == 1 {
        return "已推荐";
    }
    "未推荐"
}
pub fn get_good_status<'a>(good: i16) -> &'a str {
    if good == 1 {
        return "<span class='badge bg-secondary ms-2'>推荐</span>";
    }
    ""
}

pub enum DbList {
    Book(Vec<BookSimple>),
    Hu(Vec<HuSimple>),
    Xi(Vec<XiSimple>),
}

#[derive(Clone)]
pub enum ListBy {
    All,
    UserId(i32),
    AllPublic,
}

pub enum ShowType {
    Normal,
    Good,
}
pub struct List<'a> {
    pub ctx: &'a WebContext,
    pub tip: Option<Markup>,
    pub filter: ListBy,
    pub page: u32,
    pub i_type: Option<u8>,
    pub cat: Option<u8>,
    pub web_search_key: Option<&'a str>,
    pub title_search_key: Option<&'a str>,
    pub admin: bool,
    pub sudo: bool,
    pub size: u8,
    pub good: Option<u8>,
    pub pager: Option<&'a str>,
    pub show_search_bar: bool,
    pub show_type: ShowType,
    pub show_cat_type_name: bool,
}

impl<'a> List<'a> {
    pub fn new(ctx: &'a WebContext, filter: ListBy, page: u32) -> Self {
        List {
            ctx,
            filter,
            page,
            tip: None,
            i_type: None,
            cat: None,
            admin: true,
            sudo: false,
            size: 20,
            good: None,
            pager: None,
            show_type: ShowType::Normal,
            web_search_key: None,
            title_search_key: None,
            show_cat_type_name: false,
            show_search_bar: true,
        }
    }
    pub fn i_tip(mut self, tip: Option<Markup>) -> Self {
        self.tip = tip;
        self
    }
    pub fn i_type(mut self, tt: Option<u8>) -> Self {
        self.i_type = tt;
        self
    }
    pub fn cat(mut self, cat: Option<u8>) -> Self {
        self.cat = cat;
        self
    }
    pub fn admin(mut self, admin: bool) -> Self {
        self.admin = admin;
        self
    }
    pub fn sudo(mut self, sudo: bool) -> Self {
        self.sudo = sudo;
        self
    }
    pub fn search(mut self, show: bool) -> Self {
        self.show_search_bar = show;
        self
    }
    pub fn show_cat_type_name(mut self) -> Self {
        self.show_cat_type_name = true;
        self
    }
    pub fn good(mut self, good: Option<u8>) -> Self {
        self.good = good;
        self.show_type = ShowType::Good;
        self
    }
    pub fn size(mut self, size: u8) -> Self {
        self.size = size;
        self
    }
    pub fn pager(mut self, p: Option<&'a str>) -> Self {
        self.pager = p;
        self
    }
    pub fn web_search_key(mut self, k: Option<&'a str>) -> Self {
        self.web_search_key = k;
        self
    }
    pub fn title_search_key(mut self, k: Option<&'a str>) -> Self {
        self.title_search_key = k;
        self
    }
    pub fn show_type(mut self, show: ShowType) -> Self {
        self.show_type = show;
        self
    }
    pub fn show(self, total: i64, db: DbList) -> Markup {
        match db {
            DbList::Book(data) => self.list_book(total, data),
            DbList::Hu(data) => self.list_hu(total, data),
            DbList::Xi(data) => self.list_xi(total, data),
        }
    }
    pub fn list_book(self, total: i64, data: Vec<BookSimple>) -> Markup {
        let cat_url = if self.admin {
            "/my/book/cat/"
        } else {
            "/book/cat/"
        };
        match self.show_type {
            ShowType::Normal => {
                html! {
                    @if let Some(tip_markup) = self.tip {
                        (tip_markup)
                    }
                    @if self.show_search_bar {
                        (search_bar("book", self.cat.unwrap_or(0), self.i_type.unwrap_or(0), self.admin))
                    }
                    div class="container" {
                        @for book in data {
                            div class="row justify-content-center border m-2 p-2 shadow p-3 mb-3 bg-body rounded row-cols-1" {
                                div class="col col-md-3 text-center" {
                                    @let file = book.file.unwrap_or_default();
                                    a href={"/book/view/" (book.book_id) "/index.html#start"} {
                                        img class="shadow p-2 mb-5 bg-body rounded w-75" src={(&self.ctx.config.book.public_url) "/s-" (&file)};
                                    }
                                }
                                div class="col col-md-9 p-1" {
                                    div class="row" {
                                        div class="col" {
                                            a class="fs-5 fw-bold" href={"/book/view/" (book.book_id) "/index.html#start"} {(book.title)}
                                            (PreEscaped(get_good_status(book.good)))
                                        }
                                    }
                                    div class="row p-2" {
                                        div class="col" {
                                            span class="mx-2" {
                                                (show_date(book.created_at))
                                            }
                                            @if self.show_cat_type_name {
                                                @if let Some((cat_path, cat_name)) = self.ctx.config.book.category.path_name(book.i_category as u8) {
                                                    mark class="me-2" {
                                                        a href={(cat_url) (cat_path) } {(cat_name)}
                                                    }
                                                    @if let Some((type_path, type_name)) = self.ctx.config.book.content_type.path_name(book.i_type as u8) {
                                                        mark class="me-2" {
                                                            a href={(cat_url) (cat_path) "/" (type_path)} {(type_name)}
                                                        }
                                                    }
                                                }
                                            }

                                            @if self.admin || self.sudo {
                                                span class="float-end" {
                                                    (PreEscaped(get_status_name(PUBLIC_STATUS_HTML, book.i_public).map_or("", |v| v)))
                                                        span class="mx-2" {
                                                            (get_igood_status(book.i_good, book.good))
                                                        }
                                                    br;
                                                    a href={"/my/book/edit/" (book.book_id) "#start"} class="mx-3" {"编辑"}
                                                    @let cfm = format!("javascript:if(confirm('确实要删除吗?'))location='/my/book/rm/{}'", book.book_id);
                                                    a class="mx-3" href=(cfm) {"删除"}
                                                    @if self.sudo {
                                                        @if book.good == 1 {
                                                            a href={"/my/book/good/cancel/" (book.book_id)} class="ms-2" {"取消推荐"}
                                                        } @else {
                                                            a href={"/my/book/good/" (book.book_id)} class="ms-2" {"推荐"}
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    div class="row" {
                                        div class="col" {
                                            @if let Some(ref html) = book.brief_html {
                                                @if !html.is_empty() {
                                                    div class="border m-2 p-2" {
                                                        (PreEscaped(html))
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    div class="row" {
                                        div class="col" {
                                            @if let Some(ref url) = book.url {
                                                @if !url.is_empty() {
                                                    div class="m-2 p-2 text-end" {
                                                        a class="fs-5" href={(url)} {"阅读"}
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        @if let Some(path) = self.pager {
                            (pager(path, total, self.size, self.page))
                        }
                    }
                }
            }

            ShowType::Good => {
                html!(
                    div class="" {
                        h5 class="text-center mb-2 p-2 border-bottom border-secondary border-2" {"推 荐"}
                        @for book in data {
                            div class="border m-2 p-2 text-center" {
                                div {
                                    @let file = book.file.unwrap_or_default();
                                    a href={"/book/view/" (book.book_id) "/index.html#start"} {
                                        img class="shadow p-2 mb-2 bg-body rounded" src={(&self.ctx.config.book.public_url) "/s-" (&file)};
                                    }
                                }
                                div {
                                    a class="fs-5 fw-bold" href={"/book/view/" (book.book_id) "/index.html#start"} {(book.title)}
                                }
                                div {
                                    @if self.admin || self.sudo {
                                        a class="mx-2" href={"/my/book/edit/" (book.book_id)} {"编辑"}
                                    }
                                    @if self.sudo {
                                        a class="mx-2" href={"/my/book/good/cancel/" (book.book_id)} {"取消推荐"}
                                    }
                                }
                            }
                        }
                    }
                )
            }
        }
    }

    fn list_hu(self, total: i64, data: Vec<HuSimple>) -> Markup {
        let cat_url = if self.admin {
            "/my/hu/cat/"
        } else {
            "/hu/cat/"
        };
        match self.show_type {
            ShowType::Normal => {
                html! {
                    @if let Some(tip_markup) = self.tip {
                        (tip_markup)
                    }
                    @if self.show_search_bar {
                        (search_bar("hu", self.cat.unwrap_or(0), self.i_type.unwrap_or(0), self.admin))
                    }
                    div class="container hu" {
                        @for hu in data {
                            div class="row border p-1 shadow mb-2 bg-body rounded row-cols-1" {
                                div class="col col-md-5" {
                                    a class="fw-semibold fs-5" href={"/hu/view/" (hu.hu_id) "/index.html#start"} {(hu.title)}
                                    (PreEscaped(get_good_status(hu.good)))
                                }
                                div class="col col-md-2" {
                                    span class="mx-2" {
                                        (show_date(hu.created_at))
                                    }
                                    span class="" {
                                        (hu.user_name)
                                    }
                                }
                                div class="col col-md-3" {
                                    @if self.show_cat_type_name {
                                        @if let Some((cat_path, cat_name)) = self.ctx.config.hu.category.path_name(hu.i_category as u8) {
                                            mark class="me-2" {
                                                a href={(cat_url) (cat_path) } {(cat_name)}
                                            }
                                            @if let Some((type_path, type_name)) = self.ctx.config.hu.content_type.path_name(hu.i_type as u8) {
                                                mark class="me-2" {
                                                    a href={(cat_url) (cat_path) "/" (type_path)} {(type_name)}
                                                }
                                            }
                                        }
                                    }

                                }
                                @if self.admin || self.sudo {
                                    div class="col col-md-2" {
                                        (PreEscaped(get_status_name(PUBLIC_STATUS_HTML, hu.i_public).map_or("", |v| v)))
                                        span class="mx-2" {
                                            (get_igood_status(hu.i_good, hu.good))
                                        }
                                        br;
                                        a href={"/my/hu/edit/" (hu.hu_id) "#start"} class="ms-3" {"编辑"}
                                        @let cfm = format!("javascript:if(confirm('确实要删除吗?'))location='/my/hu/rm/{}'", hu.hu_id);
                                        a class="mx-3" href=(cfm) {"删除"}
                                        @if self.sudo {
                                            @if hu.good == 1 {
                                                a href={"/my/hu/good/cancel/" (hu.hu_id)} class="ms-2" {"不推"}
                                            } @else {
                                                a href={"/my/hu/good/" (hu.hu_id)} class="ms-2" {"推荐"}
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        @if let Some(path) = self.pager {
                            (pager(path, total, self.size, self.page))
                        }
                    }
                }
            }

            ShowType::Good => {
                html!(
                    div class="" {
                        h5 class="text-center mb-2 p-2 border-bottom border-secondary border-2" {"推 荐"}
                        @for hu in data {
                            div {
                                div {
                                    a href={"/hu/view/" (hu.hu_id) "/index.html"} {(hu.title)}
                                }
                                div {
                                    @if self.admin || self.sudo {
                                        a class="mx-2" href={"/my/hu/edit/" (hu.hu_id)} {"编辑"}
                                    }
                                    @if self.sudo {
                                        a class="mx-2" href={"/my/hu/good/cancel/" (hu.hu_id)} {"取消推荐"}
                                    }
                                }
                                //(show_date(hu.created_at))
                                @if let Some(text) = hu.brief {
                                    div {
                                        (text)
                                    }
                                }
                                @if let Some(ref url) = hu.url {
                                    div {
                                        a href={(url)} target="_blank" {"链接"}
                                    }
                                }
                            }
                        }
                    }
                )
            }
        }
    }

    pub fn list_xi(self, total: i64, data: Vec<XiSimple>) -> Markup {
        let cat_url = if self.admin {
            "/my/xi/cat/"
        } else {
            "/xi/cat/"
        };
        match self.show_type {
            ShowType::Normal => {
                html! {
                    @if let Some(tip_markup) = self.tip {
                        (tip_markup)
                    }
                    @if self.show_search_bar {
                        (search_bar("xi", self.cat.unwrap_or(0), self.i_type.unwrap_or(0), self.admin))
                    }
                    div class="container weibo" {
                        @for xi in data {
                            div class="row justify-content-center"{
                                div class="col col-md-10 border  border-success  m-2 shadow p-3 mb-5 bg-body rounded"{
                                    div class="bg-light border-bottom border-secondary border-2 p-2" {
                                        div class="" {
                                            a class="fs-5 me-3" href={"/xi/view/" (xi.xi_id) "/index.html#start"} {(xi.title)}
                                            (PreEscaped(get_good_status(xi.good)))
                                        }
                                      div class="container" {
                                        div class="row row-cols-1 row-cols-md-4 justify-content-end" {
                                            div class="col col-md-3" {
                                                span class="me-3" {
                                                    (show_date(xi.created_at))
                                                }
                                                span class="" {
                                                    (xi.user_name)
                                                }
                                            }
                                            div class="col col-md-3" {
                                                @if let Some((cat_path, cat_name)) = self.ctx.config.xi.category.path_name(xi.i_category as u8) {
                                                    mark class="me-2 text-nowrap" {
                                                        a href={(cat_url) (cat_path) } {(cat_name)}
                                                    }
                                                    @if let Some((type_path, type_name)) = self.ctx.config.xi.content_type.path_name(xi.i_type as u8) {
                                                        mark class="text-nowrap" {
                                                            a href={(cat_url) (cat_path) "/" (type_path)} {(type_name)}
                                                        }
                                                    }
                                                }
                                            }
                                            @if self.admin || self.sudo {
                                                div class="col col-md-3" {
                                                    (PreEscaped(get_status_name(PUBLIC_STATUS_HTML, xi.i_public).map_or("", |v| v)))
                                                        span class="mx-2" {
                                                            (get_igood_status(xi.i_good, xi.good))
                                                        }
                                                }
                                                div class="col col-md-3" {
                                                    span class="float-none float-md-end text-nowrap" {
                                                        a href={"/my/xi/edit/" (xi.xi_id) "#start"} class="mx-2" {"编辑"}
                                                        @let cfm = format!("javascript:if(confirm('确实要删除吗?'))location='/my/xi/rm/{}'", xi.xi_id);
                                                        a href=(cfm) class="mx-2" {"删除"}
                                                        @if self.sudo {
                                                            @if xi.good == 1 {
                                                                a href={"/my/xi/good/cancel/" (xi.xi_id)} class="ms-2" {"取消推荐"}
                                                            } @else {
                                                                a href={"/my/xi/good/" (xi.xi_id)} class="ms-2" {"推荐"}
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                      }
                                    }
                                    div class="md p-2" {
                                        (PreEscaped(&xi.html))
                                        @if let Some(ref url) = xi.url {
                                            div {
                                                a href={(url)} target="_blank" class="m-2" {"》链接"}
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        @if let Some(path) = self.pager {
                            (pager(path, total, self.size, self.page))
                        }
                    }
                }
            }

            ShowType::Good => {
                html!(
                    div class="" {
                        h5 class="text-center mb-2 p-2 border-bottom border-secondary border-2" {"推 荐"}
                        @for xi in data {
                            div class="border border-info m-2 p-2 mb-4" {
                                div {
                                    a href={"/xi/view/" (xi.xi_id) "/index.html"} {(xi.title)}
                                }
                                div class="px-2" {
                                    (show_date(xi.created_at))
                                }
                                div {
                                    @if self.admin || self.sudo {
                                        a class="px-2" href={"/my/xi/edit/" (xi.xi_id)} {"编辑"}
                                    }
                                    @if self.sudo {
                                        a class="px-2" href={"/my/xi/good/cancel/" (xi.xi_id)} {"取消推荐"}
                                    }
                                }
                                @if let Some(ref url) = xi.url {
                                    div {
                                        a href={(url)} target="_blank" {"链接"}
                                    }
                                }
                            }
                        }
                    }
                )
            }
        }
    }
}

fn search_bar(ty: &str, cat: u8, typ: u8, my: bool) -> Markup {
    let pre = if my { "/my/" } else { "/" };
    html!(
        form action={(pre) (ty) "/search"} method="get" {
            div class="container mb-3" {
                div class="row" {
                    div class="col text-end" {
                        input name="key" required class="w-sm-75";
                        button class="btn btn-primary m-1" {"查找"}
                        a class="btn btn-outline-primary shadow m-1"
                            href={"/my/" (ty) "/add?cat=" (cat)
                                  "&typ=" (typ) "#start" }
                        {"新建"}
                    }
                }
            }
        }
    )
}
