//! 信箱

pub mod db;

use axum::{routing::get, Router};
use serde::Deserialize;

use axum::extract::{Form, MatchedPath, Path, Query, State};

use axum_session::Session;
use axum_session_sqlx::SessionPgPool;

use maud::{html, Markup, PreEscaped};

use crate::fun::layout::Html;
use crate::fun::user::{get_user_from, is_sudo_role, SessUser};
use crate::fun::widget::*;
use crate::http::types::Page;
use crate::http::{error::AppError, Result, WebContext};

use db::*;

static INBOX_STATUS_MAP: [(i16, &str); 2] = [(0, "新！"), (1, "")];
static OUTBOX_STATUS_MAP: [(i16, &str); 2] = [
    (0, "<span class='text-danger'>未读</span>"),
    (1, "<span class='text-success'>已读</span>"),
];

fn get_status_name<const N: usize>(status_map: [(i16, &str); N], idx: i16) -> Option<&str> {
    status_map.iter().find(|x| x.0 == idx).map(|x| x.1)
}

pub fn router() -> Router<WebContext> {
    Router::new()
        .route("/my/inbox", get(sm_my_inbox))
        .route("/my/inbox/:id/index.html", get(sm_inbox_view))
        .route("/my/inbox/rm/:id", get(sm_rm))
        .route("/my/outbox", get(sm_my_outbox))
        .route("/my/outbox/new", get(sm_new_input).post(sm_new_do))
        .route("/my/outbox/:id/index.html", get(sm_outbox_view))
}

async fn sm_my_inbox(
    State(ctx): State<WebContext>,
    session: Session<SessionPgPool>,
    pagination: Option<Query<Pagination>>,
    path: MatchedPath,
) -> Result<Page> {
    let user = get_user_from(&session).await?;
    let Query(pagination) = pagination.unwrap_or_default();
    let page = pagination.page;

    let new_total = db_new_total(&ctx, user.id).await?;
    let mut tip_new = None;
    if new_total > 0 {
        tip_new = Some(tip(format!("新信件 {} 封", new_total).as_str()));
    }

    let list = List::new(&ctx, ListBy::UserInbox(user.id), page)
        .pager(Some(path.as_str()))
        .i_tip(tip_new);
    let (total, data) = db_list(&list).await?;
    let main = list.show(total, data);

    Ok(Html::new("收信箱", main)
        .path(Some(path.as_str()))
        .my_huxi(&user)
        .sub_nav(Some("我的收件箱"))
        .page(&ctx))
}

async fn sm_my_outbox(
    State(ctx): State<WebContext>,
    session: Session<SessionPgPool>,
    pagination: Option<Query<Pagination>>,
    path: MatchedPath,
) -> Result<Page> {
    let user = get_user_from(&session).await?;
    let Query(pagination) = pagination.unwrap_or_default();
    let page = pagination.page;
    let list = List::new(&ctx, ListBy::UserOutbox(user.id), page)
        .pager(Some(path.as_str()))
        .show_type(ShowType::Outbox);
    let (total, data) = db_list(&list).await?;
    let main = list.show(total, data);
    Ok(Html::new("发信箱", main)
        .path(Some(path.as_str()))
        .my_huxi(&user)
        .sub_nav(Some("我的发件箱"))
        .page(&ctx))
}

#[derive(Deserialize, Default, Debug)]
pub struct SmArgs {
    pub to: Option<String>,
    pub title: Option<String>,
}

async fn sm_new_input(
    State(ctx): State<WebContext>,
    session: Session<SessionPgPool>,
    args: Option<Query<SmArgs>>,
    path: MatchedPath,
) -> Result<Page> {
    let user = get_user_from(&session).await?;
    let Query(args) = args.unwrap_or_default();

    let mut sm: Input = Default::default();
    if let Some(to) = args.to {
        sm.to_user_name = to;
    }
    if let Some(title) = args.title {
        sm.title = title;
    }
    let main = input_form(&ctx, &sm, None, false);
    Ok(Html::new("写信", main)
        .path(Some(path.as_str()))
        .my_huxi(&user)
        .sub_nav(Some("我的信件"))
        .page(&ctx))
}

fn input_form(_ctx: &WebContext, sm: &Input, error: ErrorMessage, _edit: bool) -> Markup {
    html! {
        div class="container" {
            div class="row justify-content-center" {
                div class="col col-md-10 col-xl-10 border p-3 shadow-lg mb-5 bg-body rounded" {
                    div {
                        (error_message(error))
                    }
                    form action="" method="post" {
                        div class="row mb-3 border-bottom" {
                            label for="title" class="col-md-2 col-form-label text-md-end" {"* 收信人："}
                            div class="col-md-5" {
                                (TextInput::new("to_user_name", "to_user_name", true).value(Some(&sm.to_user_name)).show())
                            }
                            div class="col-md-5" {
                            }
                        }
                        div class="row mb-3 border-bottom" {
                            label for="title" class="col-md-2 col-form-label text-md-end" {"* 标题："}
                            div  class="col-md-7" {
                                (TextInput::new("title", "title", true).value(Some(&sm.title)).show())
                            }
                            div class="col-md-3" {
                            }
                        }
                        div class="row mb-3 border-bottom" {
                            label  class="col-md-2 col-form-label text-md-end" {"* 正文："}
                            div  class="col-md-10" {
                                (TextArea::new("body", "body", true).text(Some(&sm.body)).md().show())
                            }
                        }
                        div class="text-center bg-light" {
                            (submit("发送"))
                        }
                    }
                }
            }
        }
    }
}

async fn sm_new_do(
    State(ctx): State<WebContext>,
    session: Session<SessionPgPool>,
    path: MatchedPath,
    Form(input): Form<Input>,
) -> Result<Page> {
    let user = get_user_from(&session).await?;
    let send = send_message(&ctx, user.id, user.name.as_str(), input).await;
    if let Err((e, input2)) = send {
        let main = input_form(&ctx, &input2, Some(e), false);
        return Ok(Html::new("新信件", main)
            .my_huxi(&user)
            .path(Some(path.as_str()))
            .page(&ctx));
    }
    let tip = tip("发送成功");
    let list = List::new(&ctx, ListBy::UserOutbox(user.id), 1)
        .i_tip(Some(tip))
        .pager(Some("/sm/outbox"));
    let (total, data) = db_list(&list).await?;
    let main = list.show(total, data);
    Ok(Html::new("发送成功", main)
        .my_huxi(&user)
        .path(Some(path.as_str()))
        .page(&ctx))
}

pub async fn send_message(
    ctx: &WebContext,
    from_user_id: i32,
    from_user_name: &str,
    mut input: Input,
) -> Result<(), (Vec<String>, Input)> {
    let check = input.check(ctx).await;
    if let Err(e) = check {
        return Err((e, input));
    }
    let to_user_id = check.unwrap();
    let db = db_insert(ctx, from_user_id, from_user_name, &input, to_user_id).await;
    if db.is_err() {
        return Err((vec!["数据库错误".to_string()], input));
    }
    Ok(())
}

async fn check_owner(hu_user_id: i32, session: &Session<SessionPgPool>) -> Result<SessUser> {
    let user = get_user_from(session).await?;
    if hu_user_id == user.id || is_sudo_role(user.role) {
        Ok(user)
    } else {
        Err(AppError::InvalidLogin("/user/error".into()))
    }
}

async fn sm_inbox_view(
    session: Session<SessionPgPool>,
    State(ctx): State<WebContext>,
    Path(id): Path<String>,
    path: MatchedPath,
) -> Result<Page> {
    let user = get_user_from(&session).await?;
    let sm = db_get_one(&ctx, id.as_str()).await?;

    // 检查权限
    let mut allow = false;
    if sm.user_id == user.id {
        allow = true;
    }
    if sm.to_user_id == user.id {
        allow = true;
        // 更新为已经读过
        db_update_status(&ctx, sm.id.to_string()).await?;
    }

    let main = if allow {
        content_html(&sm, true)
    } else {
        tip("权限错误")
    };
    Ok(Html::new(&sm.title, main)
        .path(Some(path.as_str()))
        .my_huxi(&user)
        .sub_nav(Some("我的信件"))
        .page(&ctx))
}

async fn sm_outbox_view(
    session: Session<SessionPgPool>,
    State(ctx): State<WebContext>,
    Path(id): Path<String>,
    path: MatchedPath,
) -> Result<Page> {
    let user = get_user_from(&session).await?;
    let sm = db_get_one(&ctx, id.as_str()).await?;

    // 检查权限
    let mut allow = false;
    if sm.user_id == user.id {
        allow = true;
    }
    if sm.to_user_id == user.id {
        allow = true;
    }

    let main = if allow {
        content_html(&sm, false)
    } else {
        tip("权限错误")
    };
    Ok(Html::new(&sm.title, main)
        .path(Some(path.as_str()))
        .my_huxi(&user)
        .sub_nav(Some("我的信件"))
        .page(&ctx))
}

fn content_html(sm: &Message, reply: bool) -> Markup {
    html! {
        div class="container" {
            div class="row justify-content-center" {
                div class="col col-md-9 p-3 shadow-lg mb-5 bg-body rounded" {
                    div {
                        div {
                            "时间：" (show_date(sm.created_at))
                        }
                        @if reply {
                            "发信人：" (sm.user_name)
                        } @else {
                            "收信人：" (sm.to_user_name)
                        }
                    }
                    div {
                        p {"正文："}
                        hr;
                        div class="md" {
                            (PreEscaped(&sm.html))
                        }
                    }
                    @if reply {
                        hr;
                        @let title = format!("回复：{}", sm.title);
                        a href={"/my/outbox/new?to=" (urlencoding::encode(sm.user_name.as_str()))
                                "&title=" (urlencoding::encode(title.as_str())) "#start"} {"回复"}
                    }
                }
            }
        }
    }
}

async fn sm_rm(
    State(ctx): State<WebContext>,
    session: Session<SessionPgPool>,
    Path(id): Path<String>,
    path: MatchedPath,
) -> Result<Page> {
    let sm = db_get_one(&ctx, &id).await?;
    let user = check_owner(sm.user_id, &session).await?;
    let result_ok = db_rm(&ctx, &id).await?;
    let main = if result_ok {
        tip("删除成功")
    } else {
        tip("错误")
    };
    Ok(Html::new("删除成功", main)
        .my_huxi(&user)
        .path(Some(path.as_str()))
        .page(&ctx))
}

pub enum ShowType {
    Inbox,
    Outbox,
}
pub struct List<'a> {
    ctx: &'a WebContext,
    tip: Option<Markup>,
    filter: ListBy,
    page: u32,
    title_search_key: Option<&'a str>,
    admin: bool,
    sudo: bool,
    size: u8,
    pager: Option<&'a str>,
    show_type: ShowType,
}

impl<'a> List<'a> {
    pub fn new(ctx: &'a WebContext, filter: ListBy, page: u32) -> Self {
        List {
            ctx,
            filter,
            page,
            tip: None,
            admin: true,
            sudo: false,
            size: 5,
            pager: None,
            show_type: ShowType::Inbox,
            title_search_key: None,
        }
    }
    pub fn i_tip(mut self, tip: Option<Markup>) -> Self {
        self.tip = tip;
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
    pub fn size(mut self, size: u8) -> Self {
        self.size = size;
        self
    }
    pub fn pager(mut self, p: Option<&'a str>) -> Self {
        self.pager = p;
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

    pub fn show(self, total: i64, data: Vec<MessageSimple>) -> Markup {
        let box_name = match self.filter {
            ListBy::UserInbox(_) => "inbox",
            ListBy::UserOutbox(_) => "outbox",
        };
        let res = html! {
            @if self.admin {
                div {
                    a class="btn btn-outline-primary shadow" href="/my/outbox/new#start" {"写信"}
                }
            }
            @if let Some(tip_markup) = self.tip {
                (tip_markup)
            }
            div class="container mt-4" {
                @for sm in data {
                    div class="row justify-content-center border m-2 p-2 shadow mb-3 bg-body rounded row-cols-1" {
                        div class="col col-md-6" {
                            a href={"/my/" (box_name) "/" (sm.id) "/index.html#start"} {(sm.title)}
                            @if self.filter.is_inbox() {
                                span class="mx-2 text-danger" {(get_status_name(INBOX_STATUS_MAP, sm.i_status).map_or("", |v| v)) }
                            }
                        }
                        div class="col col-md-3" {
                            @if box_name == "inbox" {
                                "来自：" (sm.user_name)
                            } @else {
                                "收信人：" (sm.to_user_name)
                                    span class="mx-2 text-success" {
                                        (PreEscaped(get_status_name(OUTBOX_STATUS_MAP, sm.i_status).map_or("", |v| v)))
                                    }
                            }
                        }
                        div class="col col-md-2" {
                            (show_date(sm.created_at))
                        }
                        div class="col col-md-1" {
                            @if let ShowType::Inbox = self.show_type {
                                a href={"/my/" (box_name) "/rm/" (sm.id)} {"删除"}
                            }
                        }
                    }
                }
                @if let Some(path) = self.pager {
                    (pager(path, total, self.size, self.page))
                }
            }
        };
        res
    }
}

#[derive(Deserialize, Debug)]
pub struct Search {
    pub page: Option<u32>,
    pub key: Option<String>,
    pub user: Option<i16>, // None: no user, Some: 1: user all, 2: user public, 3: user private
}

impl Default for Search {
    fn default() -> Self {
        Self {
            page: Some(1),
            key: None,
            user: None,
        }
    }
}

async fn _sm_search_my(
    State(ctx): State<WebContext>,
    session: Session<SessionPgPool>,
    Form(_search): Form<Search>,
    args: Option<Query<Search>>,
    path: MatchedPath,
) -> Result<Page> {
    let user = get_user_from(&session).await?;
    let Query(search) = args.unwrap_or_default();
    dbg!(&search);
    let page = search.page.unwrap_or(1);
    let key = search.key.as_ref().unwrap().as_str();
    let list = List::new(&ctx, ListBy::UserInbox(user.id), page)
        .pager(Some(path.as_str()))
        .sudo(is_sudo_role(user.role))
        .title_search_key(Some(key))
        .admin(true);
    let (total, data) = db_list(&list).await?;
    let main = list.show(total, data);
    Ok(Html::new("list my inbox", main)
        .path(Some(path.as_str()))
        .my_huxi(&user)
        .page(&ctx))
}
