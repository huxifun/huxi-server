//! 文章

pub mod db;

use axum::extract::{Form, MatchedPath, Path, Query, State};
use axum::{routing::get, Router};
use axum_session::Session;
use axum_session_sqlx::SessionPgPool;

use maud::{html, Markup, PreEscaped};
use serde::Deserialize;

use crate::config::CategoryType;
use crate::fun::comment;
use crate::fun::layout::{split, vsplit, Html};
use crate::fun::user::{check_sudo, get_user_from, is_sudo_role, SessUser};
use crate::fun::widget::list::*;
use crate::fun::widget::*;
use crate::http::types::Page;
use crate::http::{error::AppError, Result, WebContext};

use db::*;

static PUBLIC_STATUS: [(i16, &str, &str); 2] = [(0, "craft", "草稿"), (1, "published", "公布")];

pub fn router() -> Router<WebContext> {
    Router::new()
        .route("/hu", get(hu_pub))
        .route("/hu/cat/:cat", get(hu_pub_cat))
        .route("/hu/cat/:cat/:tid", get(hu_pub_cat_type))
        .route("/hu/view/:id/index.html", get(hu_view))
        .route("/hu/search", get(hu_search))
        .route("/my/hu", get(hu_my))
        .route("/my/hu/add", get(hu_add_input).post(hu_add_do))
        .route("/my/hu/edit/:id", get(hu_edit_input).post(hu_edit_do))
        .route("/my/hu/rm/:id", get(hu_rm))
        .route("/my/hu/good/:id", get(hu_good))
        .route("/my/hu/good/cancel/:id", get(hu_good_cancel))
        .route("/my/hu/cat/:cat", get(hu_my_cat))
        .route("/my/hu/cat2/:cat", get(hu_my_cat2))
        .route("/my/hu/cat/:cat/:tid", get(hu_my_cat_type))
        .route("/my/hu/search", get(hu_search_my))
}

/// 用户登录以后管理
async fn hu_my(
    State(ctx): State<WebContext>,
    session: Session<SessionPgPool>,
    pagination: Option<Query<Pagination>>,
    path: MatchedPath,
) -> Result<Page> {
    let user = get_user_from(&session).await?;
    let Query(pagination) = pagination.unwrap_or_default();
    let page = pagination.page;
    let list = List::new(&ctx, ListBy::UserId(user.id), page)
        .pager(Some(path.as_str()))
        .show_cat_type_name()
        .sudo(is_sudo_role(user.role));
    let (total, data) = db_list(&list).await?;
    let main = list.show(total, data);
    Ok(Html::new("我的文章", main)
        .path(Some(path.as_str()))
        .my_huxi(&user)
        .page(&ctx))
}

async fn hu_pub(
    State(ctx): State<WebContext>,
    session: Session<SessionPgPool>,
    pagination: Option<Query<Pagination>>,
    path: MatchedPath,
) -> Result<Page> {
    let Query(pagination) = pagination.unwrap_or_default();
    let page = pagination.page;
    let sudo = check_sudo(&session).await;
    let list = List::new(&ctx, ListBy::AllPublic, page)
        .pager(Some(path.as_str()))
        .admin(false)
        .sudo(sudo)
        .show_cat_type_name();
    let (total, data) = db_list(&list).await?;
    let left = list.show(total, data);
    let right = list_category_name(&ctx, false);
    let main = split(left, right);
    Ok(Html::new("文章", main).path(Some(path.as_str())).page(&ctx))
}

async fn hu_add_input(
    State(ctx): State<WebContext>,
    cat_type: Option<Query<CatType>>,
    session: Session<SessionPgPool>,
    path: MatchedPath,
) -> Result<Page> {
    let user = get_user_from(&session).await?;
    let Query(cat_type) = cat_type.unwrap_or_default();
    let hu: Input = Input {
        i_category: cat_type.cat,
        i_type: cat_type.typ,
        ..Default::default()
    };

    let main = input_form(&ctx, &hu, None, false);
    Ok(Html::new("新文章", main)
        .path(Some(path.as_str()))
        .my_huxi(&user)
        .page(&ctx))
}

/// 字段：body2 补充内容，用于登录用户查看，或将来进行收费
fn input_form(ctx: &WebContext, hu: &Input, error: ErrorMessage, edit: bool) -> Markup {
    let title = if edit { "修改" } else { "新建" };
    html! {
        div {
            div class="container" {
                div class="row justify-content-center" {
                    div class="col col-md-11 col-xl-11 border p-3 shadow-lg mb-5 bg-body rounded" {
                        // <form>
                        form action="" method="post" {
                            // div class="w-100 text-center mb-5" {
                            //     h2 {(title)}
                            // }
                            div {
                                (error_message(error))
                            }
                            div class="row mb-3 border-bottom" {
                                label for="title" class="col-md-2 col-form-label text-md-end" {"* 标题："}
                                div class="col-md-7" {
                                    (TextInput::new("title", "title", true).value(Some(&hu.title)).show())
                                }
                                div class="col-md-3" {
                                }
                            }

                            div class="row mb-3 border-bottom" {
                                label class="col-md-2 col-form-label text-md-end" {"分类："}
                                div class="col-md-10" {
                                    @for (_i, c) in ctx.config.hu.category.iter().enumerate() {
                                        (radio(&c.1, "i_category", &c.0.to_string(), hu.i_category == c.0 as i16, &c.2))
                                    }
                                }
                            }
                            div class="row mb-3 border-bottom" {
                                label class="col-md-2 col-form-label text-md-end" {"类型："}
                                div class="col-md-10" {
                                    @for (_i, t) in ctx.config.hu.content_type.iter().enumerate() {
                                        (radio(&t.1, "i_type", &t.0.to_string(), hu.i_type == t.0 as i16, &t.2))
                                    }
                                }
                            }
                            div class="row mb-3 border-bottom" {
                                label class="col-md-2 col-form-label text-md-end" {"简介："}
                                div class="col-md-10" {
                                    (TextArea::new("brief", "brief", false).text(hu.brief.as_ref()).md().show())
                                }
                            }
                            div class="row mb-3 border-bottom" {
                                label class="col-md-2 col-form-label text-md-end" {"* 正文："}
                                div class="col-md-10" {
                                    (TextArea::new("body", "body", true).text(Some(&hu.body)).rows(10).md().show())
                                }
                                // script {
                                //     "const easyMDE = new EasyMDE({element: document.getElementById('body')});"
                                // }
                            }
                            div class="row mb-3 border-bottom" {
                                label class="col-md-2 col-form-label text-md-end" {"补充："}
                                div class="col-md-10" {
                                    (TextArea::new("body2", "body2", false).text(hu.body2.as_ref()).md().show())
                                }
                            }
                            div class="row mb-3 border-bottom" {
                                label class="col-md-2 col-form-label text-md-end" for="tags" {"Tags："}
                                div class="col-md-7" {
                                    (TextInput::new("tags", "tags", false).value(hu.tags.as_ref()).show())
                                }
                                div class="col-md-3" {
                                }
                            }
                            div class="row mb-3 border-bottom" {
                                label class="col-md-2 col-form-label text-md-end" for="url" {"网址："}
                                div class="col-md-7" {
                                    (TextInput::new("url", "url", false).value(hu.url.as_ref()).show())
                                }
                                div class="col-md-3" {
                                }
                            }
                            @if edit {
                                div class="row mb-3 border-bottom" {
                                    label class="col-md-2 col-form-label text-md-end" {"更新日志："}
                                    div class="col-md-10" {
                                        (TextArea::new("log", "log", false).text(hu.log.as_ref()).show())
                                    }
                                }
                            }
                            div class="row mb-3 border-bottom" {
                                label class="col-md-2 col-form-label text-md-end" for="i_good" {"推荐："}
                                div class="col-md-10" {
                                    (checkbox("i_good", "i_good", "1", hu.i_good.map_or(false, |v| v!=0 )))
                                        label for="i_good" class="mx-2" {"申请推荐"}
                                }
                            }
                            div class="row mb-3 border-bottom" {
                                label class="col-md-2 col-form-label text-md-end" {"状态："}
                                div class="col-md-10" {
                                    @for (_i, p) in PUBLIC_STATUS.iter().enumerate() {
                                        (radio(p.1, "i_public", &p.0.to_string(), hu.i_public == p.0, p.2))
                                    }
                                }
                            }
                            div class="text-center bg-light" {
                                (submit(title))
                                    a class="btn btn-primary mx-3" href="javascript:window.history.back()" {"取消"}
                                a class="btn btn-primary" href="/my/hu#start" {"列表"}
                            }

                        }
                        // </form>
                    }
                }
            }
        }
    }
}

async fn hu_add_do(
    State(ctx): State<WebContext>,
    session: Session<SessionPgPool>,
    path: MatchedPath,
    Form(mut input): Form<Input>,
) -> Result<Page> {
    let user = get_user_from(&session).await?;
    let check = input.check();
    if let Err(e) = check {
        let main = input_form(&ctx, &input, Some(e), false);
        return Ok(Html::new("新文章", main)
            .my_huxi(&user)
            .path(Some(path.as_str()))
            .page(&ctx));
    }
    let id = db_insert(&ctx, &user, input).await?;
    let main = html!(
        (tip("新就文章添加成功"))
        div class="text-center" {
            a href={"/hu/view/" (id) "/index.html#start"} class="m-2" {"继续查看"}
            a href="/my/hu" class="m-2" {"显示列表"}
        }
        (PreEscaped(redirect_script("/my/hu")))
    );
    Ok(Html::new("添加文章成功", main)
        .my_huxi(&user)
        .path(Some(path.as_str()))
        .page(&ctx))
}

async fn hu_edit_input(
    session: Session<SessionPgPool>,
    State(ctx): State<WebContext>,
    Path(id): Path<i32>,
    path: MatchedPath,
) -> Result<Page> {
    let hu: Hu = db_get_one(&ctx, id).await?;
    let user = check_owner(hu.user_id, &session).await?;
    let main = input_form(&ctx, &hu.to_edit(), None, true);
    Ok(Html::new("修改文章", main)
        .my_huxi(&user)
        .path(Some(path.as_str()))
        .page(&ctx))
}

async fn check_owner(hu_user_id: i32, session: &Session<SessionPgPool>) -> Result<SessUser> {
    let user = get_user_from(session).await?;
    if hu_user_id == user.id || is_sudo_role(user.role) {
        Ok(user)
    } else {
        Err(AppError::InvalidLogin("/user/error".into()))
    }
}

async fn hu_edit_do(
    State(ctx): State<WebContext>,
    session: Session<SessionPgPool>,
    Path(id): Path<i32>,
    path: MatchedPath,
    Form(mut input): Form<Input>,
) -> Result<Page> {
    let hu = db_get_one(&ctx, id).await?;
    let user = check_owner(hu.user_id, &session).await?;
    let check = input.check();
    if let Err(e) = check {
        let main = input_form(&ctx, &input, Some(e), true);
        return Ok(Html::new("修改文章", main)
            .my_huxi(&user)
            .path(Some(path.as_str()))
            .page(&ctx));
    }

    let update = db_update(&ctx, id, &input).await?;

    if !update {
        let main = tip("数据错误");
        return Ok(Html::new("数据错误", main)
            .my_huxi(&user)
            .path(Some(path.as_str()))
            .page(&ctx));
    }
    let main = html!(
        (tip("修改文章成功"))
        div class="text-center" {
            a href={"/hu/view/" (id) "/index.html#start"} class="m-2" {"继续查看"}
            a href="/my/hu" class="m-2" {"显示列表"}
        }
        (PreEscaped(redirect_script("/my/hu")))
    );
    Ok(Html::new("修改文章", main)
        .my_huxi(&user)
        .path(Some(path.as_str()))
        .page(&ctx))
}

fn view_url(id: i32) -> String {
    format!("/hu/view/{}/index.html", id)
}

async fn hu_view(
    session: Session<SessionPgPool>,
    State(ctx): State<WebContext>,
    Path(id): Path<i32>,
    path: MatchedPath,
) -> Result<Page> {
    let hu = db_get_one(&ctx, id).await?;

    // 检查权限
    let mut allow = false;
    let mut login = false;
    let mut admin = false;
    let mut sudo = false;
    let mut owner: Option<SessUser> = None;
    if hu.i_public > 0 {
        allow = true;
    }

    let result = get_user_from(&session).await;
    if let Ok(user) = result {
        login = true;
        if is_sudo_role(user.role) {
            sudo = true;
            allow = true;
            admin = true;
        }
        if hu.user_id == user.id {
            allow = true;
            admin = true;
            owner = Some(user);
        }
    }

    let left = if allow {
        let url = view_url(id);
        let _ = db_update_click(&ctx, id).await;
        let cms = comment::list_comment(&ctx, hu.id, url.as_str(), login, admin, "hu").await?;
        content_html(&hu, cms, &ctx, admin, sudo)
    } else {
        tip("权限错误")
    };
    let main = if owner.is_some() {
        left
    } else {
        let list = List::new(&ctx, ListBy::AllPublic, 1)
            .cat(Some(hu.i_category as u8))
            .admin(admin)
            .sudo(sudo)
            .good(Some(1));
        let (total, data) = db_list(&list).await?;
        let cats = list_category_name(&ctx, false);
        let right = vsplit(list.show(total, data), cats);
        split(left, right)
    };

    let mut html = Html::new(&hu.title, main)
        .path(Some(path.as_str()))
        .description(hu.brief)
        .highlight();
    if owner.is_some() {
        html = html
            .my_huxi(owner.as_ref().unwrap())
            .sub_nav(Some("我的文章"));
    }
    Ok(html.page(&ctx))
}

fn content_html(hu: &Hu, comment: Markup, ctx: &WebContext, admin: bool, sudo: bool) -> Markup {
    html! {
        div class="container" {
            div class="row justify-content-center" {
                div class="col col-md-11 p-3 shadow-lg mb-5 bg-body rounded" {
                    div class="text-end" {
                        (PreEscaped(get_good_status(hu.good)))
                    }
                    div class="container text-muted"{
                        div class="row row-cols-1  row-cols-md-4" {
                            div class="col mx-2"{
                                "作者：" (hu.user_name)
                            }
                            div class="col mx-2" {
                                "日期：" (show_date(hu.created_at))
                            }
                            @if hu.updated_at.is_some() {
                                div class="col mx-2" {
                                    "更新："
                                        (show_date(hu.updated_at.unwrap()))
                                }
                            }
                        }
                        div class="row row-cols-1  row-cols-md-4" {
                            @if let Some((cat_path, cat_name)) = ctx.config.hu.category.path_name(hu.i_category as u8) {
                                div class="col mx-2" {
                                    "分类："
                                        mark class="me-2" {
                                            a href={"/hu/cat/" (cat_path) } {(cat_name)}
                                        }
                                }
                                @if let Some((type_path, type_name)) = ctx.config.hu.content_type.path_name(hu.i_type as u8) {
                                    div class="col mx-2" {
                                        "类型："
                                            mark class="me-2" {
                                                a href={"/hu/cat/" (cat_path) "/" (type_path)} {(type_name)}
                                            }
                                    }
                                }
                            }
                            div class="col mx-2" {
                                "点击：" (hu.click)
                            }
                        }
                        div class="row row-cols-1  row-cols-md-4" {
                            @if let Some(ref tags) = hu.tags {
                                div class="col mx-2" {
                                    "Tags: " (tags)
                                }
                            }
                            @if admin {
                                div class="col mx-2" {
                                    "状态："(PreEscaped(get_status_name(PUBLIC_STATUS_HTML, hu.i_public).map_or("", |v| v)))
                                }
                                div class="col mx-2" {
                                    @let url = format!("/my/hu/edit/{}#start", hu.id);
                                    "管理：" a href={(url)} {"编辑"}
                                }
                                div class="col mx-2" {
                                    "推荐：" (get_igood_status(hu.i_good, hu.good))
                                        @if sudo {
                                            @if hu.good == 1 {
                                                a href={"/my/hu/good/cancel/" (hu.id)} class="ms-2" {"取消推荐"}
                                            } @else {
                                                a href={"/my/hu/good/" (hu.id)} class="ms-2" {"推荐"}
                                            }
                                        }
                                }
                            }
                        }
                    }
                    hr;
                    div class="md border border-4" {
                        @if hu.log.is_some() {
                            div class="border bg-light m-2 p-2" {
                                "更新日志："
                                    (PreEscaped(&hu.log_html.as_ref().unwrap()))
                            }
                        }
                        @if let Some(ref html) = hu.brief_html {
                            @if !html.is_empty() {
                                (PreEscaped(html))
                            }
                        }
                        div {
                            (PreEscaped(&hu.html.as_ref().unwrap()))
                        }
                        @if let Some(ref html) = hu.html2 {
                            @if !html.is_empty() {
                                (PreEscaped(html))
                            }
                        }
                    }

                    @if let Some(ref url) = hu.url {
                        div {
                            a href={(url)} target="_blank" {"链接"}
                        }
                    }
                    hr;
                    (comment)
                }
            }
        }
    }
}

async fn hu_rm(
    State(ctx): State<WebContext>,
    session: Session<SessionPgPool>,
    Path(id): Path<i32>,
    path: MatchedPath,
) -> Result<Page> {
    let hu = db_get_one(&ctx, id).await?;
    let user = check_owner(hu.user_id, &session).await?;
    let result_ok = db_rm(&ctx, id).await?;
    let main = if result_ok {
        tip("删除成功")
    } else {
        tip("数据错误")
    };
    Ok(Html::new("删除成功", main)
        .my_huxi(&user)
        .path(Some(path.as_str()))
        .page(&ctx))
}

// 推荐
async fn hu_good(
    State(ctx): State<WebContext>,
    session: Session<SessionPgPool>,
    Path(id): Path<i32>,
    path: MatchedPath,
) -> Result<Page> {
    let hu = db_get_one(&ctx, id).await?;
    let user = check_owner(hu.user_id, &session).await?;

    let mut main = tip("错误");
    if is_sudo_role(user.role) {
        let result_ok = db_good(&ctx, id, 1).await?;
        if result_ok {
            main = tip("推荐成功");
        }
    }
    Ok(Html::new("推荐成功", main)
        .path(Some(path.as_str()))
        .my_huxi(&user)
        .page(&ctx))
}
async fn hu_good_cancel(
    State(ctx): State<WebContext>,
    session: Session<SessionPgPool>,
    Path(id): Path<i32>,
    path: MatchedPath,
) -> Result<Page> {
    let hu = db_get_one(&ctx, id).await?;
    let user = check_owner(hu.user_id, &session).await?;

    let mut main = tip("错误");
    if is_sudo_role(user.role) {
        let result_ok = db_good(&ctx, id, 0).await?;
        if result_ok {
            main = tip("取消推荐成功");
        }
    }
    Ok(Html::new("取消推荐成功", main)
        .path(Some(path.as_str()))
        .my_huxi(&user)
        .page(&ctx))
}
async fn hu_my_cat(
    State(ctx): State<WebContext>,
    session: Session<SessionPgPool>,
    pagination: Option<Query<Pagination>>,
    Path(cat): Path<String>,
    path: MatchedPath,
) -> Result<Page> {
    let user = get_user_from(&session).await?;
    let Query(pagination) = pagination.unwrap_or_default();
    let page = pagination.page;

    let cat_id = ctx.config.hu.category.id(cat.as_str());
    let cat_name = ctx
        .config
        .hu
        .category
        .name(cat.as_str())
        .ok_or_else(|| AppError::InvalidArg("cat name error".to_string()))?;

    let list = List::new(&ctx, ListBy::UserId(user.id), page)
        .cat(cat_id)
        .pager(Some(path.as_str()))
        .show_cat_type_name()
        .admin(true)
        .sudo(is_sudo_role(user.role));
    let (total, data) = db_list(&list).await?;
    let main = list.show(total, data);
    let title = format!("我的文章: {} ", cat_name);
    Ok(Html::new(title.as_str(), main)
        .path(Some(path.as_str()))
        .my_huxi(&user)
        .highlight()
        .page(&ctx))
}

async fn hu_my_cat2(
    State(ctx): State<WebContext>,
    session: Session<SessionPgPool>,
    pagination: Option<Query<Pagination>>,
    Path(cat): Path<String>,
    path: MatchedPath,
) -> Result<Page> {
    let user = get_user_from(&session).await?;
    let Query(pagination) = pagination.unwrap_or_default();
    let page = pagination.page;
    let sudo = check_sudo(&session).await;

    let cat_id = ctx.config.hu.category.id(cat.as_str());
    let cid = cat.as_str();
    let cat_name = ctx
        .config
        .hu
        .category
        .name(cat.as_str())
        .ok_or_else(|| AppError::InvalidArg("cat name error".to_string()))?;
    let main = html!(
        div class="" {
            @for ty in &ctx.config.hu.content_type {
                @let list = List::new(&ctx, ListBy::UserId(user.id), page)
                    .cat(cat_id)
                    .admin(true)
                    .size(5)
                    .search(false)
                    .sudo(sudo)
                    .show_cat_type_name()
                    .i_type(Some(ty.0));
                @let (total, data) = db_list(&list).await?;
                div class="border m-3 shadow p-3 mb-5 bg-body rounded" {
                    div class="bg-light p-2"{
                        a href={"/my/hu/cat/" (cid) "/" (ty.1)} {(ty.2)}
                    }
                    div class="p-2 border border-info" {
                        @if total > 0 {
                            (list.show(total, data))
                            div class="text-end" {
                                a href={"/my/hu/cat/" (cid) "/" (ty.1)} {"》》 更多 " (ty.2)}
                            }
                        } @else {
                            p {"当前类型暂无数据"}
                        }
                    }
                }
            }
        }
    );
    let title = format!("{} ", cat_name);
    Ok(Html::new(title.as_str(), main)
        .sub_nav(Some("我的文章"))
        .path(Some(path.as_str()))
        .my_huxi(&user)
        .page(&ctx))
}
async fn hu_pub_cat(
    State(ctx): State<WebContext>,
    session: Session<SessionPgPool>,
    pagination: Option<Query<Pagination>>,
    Path(cat): Path<String>,
    path: MatchedPath,
) -> Result<Page> {
    let Query(pagination) = pagination.unwrap_or_default();
    let page = pagination.page;

    let cat_id = ctx.config.hu.category.id(cat.as_str());
    //let page_link = format!("/hu/cat/{}", cat.as_str());
    let cid = cat.as_str();
    let cat_name = ctx
        .config
        .hu
        .category
        .name(cat.as_str())
        .ok_or_else(|| AppError::InvalidArg("cat name error".to_string()))?;
    let sudo = check_sudo(&session).await;
    let left = html!(
        div class="" {
            @for ty in &ctx.config.hu.content_type {
                @let list = List::new(&ctx, ListBy::AllPublic, page)
                    .cat(cat_id)
                    .size(5)
                    .admin(false)
                    .sudo(sudo)
                    .search(false)
                    .show_cat_type_name()
                    //.pager(Some(page_link.as_str()))
                    .i_type(Some(ty.0));
                @let (total, data) = db_list(&list).await?;
                @if total > 0 {
                    div class="border m-2" {
                        div class="bg-light p-2"{
                            a href={"/hu/cat/" (cid) "/" (ty.1)} {(ty.2)}
                        }
                        div class="p-2 border border-info" {
                            (list.show(total, data))
                        }
                    }
                }
            }
        }
    );
    let list = List::new(&ctx, ListBy::AllPublic, page)
        .cat(cat_id)
        .admin(false)
        .sudo(sudo)
        .good(Some(1));
    let (total, data) = db_list(&list).await?;
    let right = list.show(total, data);
    let main = split(left, right);
    let title = format!("{} ", cat_name);
    Ok(Html::new(title.as_str(), main)
        .path(Some(path.as_str()))
        .page(&ctx))
}

async fn hu_my_cat_type(
    State(ctx): State<WebContext>,
    session: Session<SessionPgPool>,
    pagination: Option<Query<Pagination>>,
    Path((cat, tid)): Path<(String, String)>,
    path: MatchedPath,
) -> Result<Page> {
    let user = get_user_from(&session).await?;
    let Query(pagination) = pagination.unwrap_or_default();
    let page = pagination.page;

    let cat_id = ctx.config.hu.category.id(cat.as_str());
    let cat_name = ctx
        .config
        .hu
        .category
        .name(cat.as_str())
        .ok_or_else(|| AppError::InvalidArg("cat name error".to_string()))?;
    let type_id = ctx.config.hu.content_type.id(tid.as_str());
    let type_name = ctx
        .config
        .hu
        .content_type
        .name(tid.as_str())
        .ok_or_else(|| AppError::InvalidArg("type name error".to_string()))?;
    // let page_link = format!("/hu/cat/{}/{}", cat, tid);
    // let sudo = check_sudo(&session).await;
    let list = List::new(&ctx, ListBy::UserId(user.id), page)
        .cat(cat_id)
        .show_cat_type_name()
        .admin(true)
        .sudo(is_sudo_role(user.role))
        .i_type(type_id)
        .pager(Some(path.as_str()));
    let (total, data) = db_list(&list).await?;
    let main = list.show(total, data);
    let title = format!("{} -- {}", cat_name, type_name);
    Ok(Html::new(title.as_str(), main)
        .path(Some(path.as_str()))
        .sub_nav(Some("我的文章"))
        .my_huxi(&user)
        .highlight()
        .page(&ctx))
}
async fn hu_pub_cat_type(
    State(ctx): State<WebContext>,
    session: Session<SessionPgPool>,
    pagination: Option<Query<Pagination>>,
    Path((cat, tid)): Path<(String, String)>,
    path: MatchedPath,
) -> Result<Page> {
    let Query(pagination) = pagination.unwrap_or_default();
    let page = pagination.page;

    let cat_id = ctx.config.hu.category.id(cat.as_str());
    let cat_name = ctx
        .config
        .hu
        .category
        .name(cat.as_str())
        .ok_or_else(|| AppError::InvalidArg("cat name error".to_string()))?;
    let type_id = ctx.config.hu.content_type.id(tid.as_str());
    let type_name = ctx
        .config
        .hu
        .content_type
        .name(tid.as_str())
        .ok_or_else(|| AppError::InvalidArg("type name error".to_string()))?;
    let page_link = format!("/hu/cat/{}/{}", cat, tid);
    let sudo = check_sudo(&session).await;

    let list = List::new(&ctx, ListBy::AllPublic, page)
        .cat(cat_id)
        .admin(false)
        .sudo(sudo)
        .show_cat_type_name()
        .i_type(type_id)
        .pager(Some(page_link.as_str()));
    let (total, data) = db_list(&list).await?;
    let left = list.show(total, data);

    let list = List::new(&ctx, ListBy::AllPublic, page)
        .cat(cat_id)
        .admin(false)
        .sudo(sudo)
        .good(Some(1));
    let (total, data) = db_list(&list).await?;
    let right = list.show(total, data);

    let main = split(left, right);
    let title = format!("{} -- {}", cat_name, type_name);
    Ok(Html::new(title.as_str(), main)
        .path(Some(path.as_str()))
        .page(&ctx))
}

pub async fn list_category_top(ctx: &WebContext, sudo: bool) -> Result<Markup> {
    let res = html!(
        div class="row row-cols-md-2" {
            @for i in &ctx.config.hu.category {
                @let list = List::new(ctx, ListBy::AllPublic, 1)
                    .cat(Some(i.0))
                    .sudo(sudo)
                    .admin(false)
                    .search(false)
                    .size(5);
                @let (total, data) = db_list(&list).await?;
                @if total > 0 {
                    div class="col" {
                        div class="border m-2" {
                            div class="bg-light p-2 border-bottom" {
                                a href={"/hu/cat/" (i.1)} {(i.2)}
                            }
                            div class="p-2" {
                                (list.show(total, data))
                            }
                        }
                    }
                }
            }
        }

    );
    Ok(res)
}
pub async fn list_pub_top(ctx: &WebContext, sudo: bool) -> Result<Markup> {
    let list = List::new(ctx, ListBy::AllPublic, 1)
        .size(5)
        .search(false)
        .show_cat_type_name()
        .sudo(sudo)
        .admin(false);
    let (total, data) = db_list(&list).await?;
    let res = if total > 0 {
        list.show(total, data)
    } else {
        tip("暂无数据")
    };
    Ok(res)
}

pub fn list_category_name(ctx: &WebContext, my: bool) -> Markup {
    let title = if my { "我的文章" } else { "文章分类" };
    let res = html!(
        div class="bg-light m-2 border p-2 mt-3" {
            h5 class="text-center mb-2 p-2 border-bottom border-secondary border-2" {(title)}
            @for i in &ctx.config.hu.category {
                div class="p-2 text-center" {
                    @if my {
                        a href={"/my/hu/cat2/" (i.1) "#start"} {(i.2)}
                    } @else {
                        a href={"/hu/cat/" (i.1)} {(i.2)}
                    }
                }
            }
        }
    );
    res
}

#[derive(Deserialize, Debug)]
pub struct Search {
    pub page: Option<u32>,
    pub key: Option<String>,
    pub i_type: Option<i16>,
    pub user: Option<i16>, // None: no user, Some: 1: user all, 2: user public, 3: user private
}

impl Default for Search {
    fn default() -> Self {
        Self {
            page: Some(1),
            key: None,
            i_type: None,
            user: None,
        }
    }
}

async fn hu_search(
    State(ctx): State<WebContext>,
    session: Session<SessionPgPool>,
    args: Option<Query<Search>>,
    path: MatchedPath,
) -> Result<Page> {
    let user = get_user_from(&session).await;
    let Query(search) = args.unwrap_or_default();

    let page = search.page.unwrap_or(1);
    let key = search.key.as_ref().unwrap().as_str();
    let list = List::new(&ctx, ListBy::AllPublic, page)
        .pager(Some(path.as_str()))
        .sudo(is_sudo_role(user.map_or(0, |u| u.role)))
        .admin(false)
        .web_search_key(Some(key));
    let (total, data) = db_list(&list).await?;
    let main = list.show(total, data);
    Ok(Html::new("搜索文章", main)
        .path(Some(path.as_str()))
        .page(&ctx))
}

async fn hu_search_my(
    State(ctx): State<WebContext>,
    session: Session<SessionPgPool>,
    args: Option<Query<Search>>,
    path: MatchedPath,
) -> Result<Page> {
    let user = get_user_from(&session).await?;
    let Query(search) = args.unwrap_or_default();
    dbg!(&search);
    let page = search.page.unwrap_or(1);
    let key = search.key.as_ref().unwrap().as_str();
    let list = List::new(&ctx, ListBy::UserId(user.id), page)
        .pager(Some(path.as_str()))
        .sudo(is_sudo_role(user.role))
        .title_search_key(Some(key))
        .admin(true);
    let (total, data) = db_list(&list).await?;
    let main = list.show(total, data);
    Ok(Html::new("搜索我的文章", main)
        .path(Some(path.as_str()))
        .my_huxi(&user)
        .page(&ctx))
}
