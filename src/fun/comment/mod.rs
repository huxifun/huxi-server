//! 评论模块

mod db;

use axum::extract::{Form, MatchedPath, Path, Query, State};
use axum::response::Redirect;
use axum::{
    routing::{get, post},
    Router,
};

use axum_session::{Session, SessionPgPool};
use maud::{html, Markup, PreEscaped};

use crate::fun::layout::Html;
use crate::fun::user::{get_user_from, is_sudo_role, SessUser};
use crate::fun::widget::*;
use crate::http::types::Page;
use crate::http::{error::AppError, Result, WebContext};

use db::*;

pub fn router() -> Router<WebContext> {
    Router::new()
        .route("/my/:ty/comment/add", post(add_do))
        .route("/my/:ty/comment/edit/:id", get(edit_input).post(edit_do))
        .route("/my/:ty/comment/hide/:id", get(hide))
        .route("/my/:ty/comment/rm/:id", get(rm))
}

async fn add_do(
    State(ctx): State<WebContext>,
    session: Session<SessionPgPool>,
    Path(ty): Path<String>,
    Form(input): Form<Input>,
) -> Redirect {
    let user = get_user_from(&session).await;
    if user.is_err() {
        return Redirect::to("/user/login");
    }
    let user = user.unwrap();
    let check = input.check();
    let url = input.url.as_ref().unwrap();
    let error_url = "/error";
    if check.is_err() {
        return Redirect::to(error_url);
    }
    let id = db_insert(&ctx, user.id, user.name.as_str(), &input, &ty).await;
    if id.is_err() {
        return Redirect::to(error_url);
    }

    Redirect::to(url)
}

fn edit_form(_ctx: &WebContext, input: &Input, error: ErrorMessage, url: &str, ty: &str) -> Markup {
    let id = input.id.unwrap();
    let post_url = format!("/my/{}/comment/edit/{}", ty, id);
    html! {
        div class="row justify-content-center" {
            div class="col col-md-10" {
                (error_message(error))
                    form action={(post_url) "#start"} method="post" {
                        input type="hidden" name="url" value=(url);
                        input type="hidden" name="id" value=(id);
                        div {
                            label class="m-2" {"评论："}
                            (TextArea::new("body", "body", true).text(Some(&input.body)).show())
                        }
                            div class="text-center" {
                                (submit("确定"))
                            }
                    }
            }
        }
    }
}

async fn edit_input(
    session: Session<SessionPgPool>,
    args: Query<UrlArgs>,
    State(ctx): State<WebContext>,
    Path((ty, id)): Path<(String, i32)>,
    path: MatchedPath,
) -> Result<Page> {
    let cm: Comment = db_get_one(&ctx, id, &ty).await?;
    let user = check_owner(cm.user_id, &session).await?;
    let de = urlencoding::decode(args.url.as_str());
    if de.is_err() {
        return Err(AppError::InvalidArg("invalid url arg".to_string()));
    }
    let url = &de.unwrap();
    let main = edit_form(&ctx, &cm.input(), None, url, &ty);
    Ok(Html::new("修改评论", main)
        .path(Some(path.as_str()))
        .my_huxi(&user)
        .page(&ctx))
}
#[derive(serde::Deserialize, Debug)]
struct UrlArgs {
    url: String,
}

async fn check_owner(hu_user_id: i32, session: &Session<SessionPgPool>) -> Result<SessUser> {
    let user = get_user_from(session).await?;
    if hu_user_id == user.id || is_sudo_role(user.role) {
        Ok(user)
    } else {
        Err(AppError::InvalidLogin("/user/error".into()))
    }
}

async fn edit_do(
    State(ctx): State<WebContext>,
    session: Session<SessionPgPool>,
    Path((ty, id)): Path<(String, i32)>,
    path: MatchedPath,
    Form(input): Form<Input>,
) -> Result<Page> {
    let cm = db_get_one(&ctx, id, &ty).await?;
    let user = check_owner(cm.user_id, &session).await?;
    let check = input.check();
    let url = input.url.as_ref().unwrap();
    if let Err(e) = check {
        let main = edit_form(&ctx, &input, Some(e), url, &ty);
        return Ok(Html::new("修改评论", main)
            .path(Some(path.as_str()))
            .my_huxi(&user)
            .page(&ctx));
    }

    let update = db_update(&ctx, id, &input, &ty).await?;

    if !update {
        let main = tip("数据错误");
        return Ok(Html::new("修改评论", main)
            .path(Some(path.as_str()))
            .my_huxi(&user)
            .page(&ctx));
    }
    let main = html!(
        (tip("修改成功"))
        div class="text-center" {
            a href={(url) "#start"} {"继续查看"}
        }
    );
    Ok(Html::new("修改成功", main)
        .path(Some(path.as_str()))
        .my_huxi(&user)
        .page(&ctx))
}

async fn rm(
    State(ctx): State<WebContext>,
    session: Session<SessionPgPool>,
    args: Query<UrlArgs>,
    Path((ty, id)): Path<(String, i32)>,
    path: MatchedPath,
) -> Result<Page> {
    let cm = db_get_one(&ctx, id, &ty).await?;
    let user = check_owner(cm.user_id, &session).await?;
    let de = urlencoding::decode(args.url.as_str());
    if de.is_err() {
        return Err(AppError::InvalidArg("invalid url arg".to_string()));
    }
    let url = &de.unwrap();
    let result_ok = db_rm(&ctx, id, &ty).await?;
    let main = if result_ok {
        html!(
        (tip("删除评论成功"))
            div class="text-center" {
                a href={(url)} {"继续查看"}
            }
        )
    } else {
        tip("错误")
    };
    Ok(Html::new("删除成功", main)
        .path(Some(path.as_str()))
        .my_huxi(&user)
        .page(&ctx))
}

async fn hide(
    State(ctx): State<WebContext>,
    session: Session<SessionPgPool>,
    Path((ty, id)): Path<(String, i32)>,
    path: MatchedPath,
) -> Result<Page> {
    let cm = db_get_one(&ctx, id, &ty).await?;
    let user = check_owner(cm.user_id, &session).await?;
    let result_ok = db_hide(&ctx, id, &ty).await?;
    let main = if result_ok {
        tip("私有成功")
    } else {
        tip("错误")
    };
    Ok(Html::new("私有成功", main)
        .path(Some(path.as_str()))
        .my_huxi(&user)
        .page(&ctx))
}

pub async fn list_comment(
    ctx: &WebContext,
    id: i32,
    url: &str,
    login: bool,
    admin: bool,
    ty: &str,
) -> Result<Markup> {
    let data = db_list(ctx, id, ty).await?;
    let res = html!(
        div {
            @if data.is_empty() {
                p {"暂无评论"}
            } @else{
                h5 {"评论："}
                @for c in data {
                   div class="border m-2 rounded" {
                       div class="bg-light p-2 border-bottom border-2" {
                           div class="d-flex align-content-start flex-wrap" {
                               div class="mx-2 text-nowrap" {
                                   "作者: " (c.user_name)
                               }
                               div class="mx-2 text-nowrap" {
                                   "时间: " (show_date(c.created_at))
                               }
                               @if admin {
                                   div class="text-nowrap" {
                                       a class="mx-2" href={"/my/" (ty) "/comment/edit/" (c.id) "?url=" (urlencoding::encode(url)) "#start"} {"修改"}
                                       a class="mx-2" href={"javascript:if(confirm('确实要删除吗?'))location='/my/" (ty) "/comment/rm/"
                                                            (c.id)
                                                            "?url=" (urlencoding::encode(url)) "'"
                                       } {"删除"}
                                   }
                               }
                           }
                       }
                       @if let Some(html) = c.html {
                           div class="p-2 md" {
                               (PreEscaped(html))
                           }
                       }
                   }
                }
            }
            div class="row justify-content-center" {
                div class="col col-md-10" {
                    @if login {
                        div class="text-center p-2 m-2" {
                            form action={"/my/" (ty) "/comment/add"} method="post" {
                                input type="hidden" name="url" value={(url) "#start"};
                                input type="hidden" name="oid" value=(id);
                                (TextArea::new("body", "body", false).md().show())
                                (submit("评论"))
                            }
                        }
                    } @else {
                        div class="text-center p-2 m-2 border border-primary" {
                            p {"添加评论，请登录"}
                            a class="mx-2" href={"/user/login?from=" (urlencoding::encode(url))}{"登录"}
                            a class="mx-2" href="/user/reg" {"注册"}
                        }
                    }
                }
            }
        }
    );
    Ok(res)
}
