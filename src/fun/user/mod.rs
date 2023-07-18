//! 用户

pub mod db;

use axum::extract::{Form, MatchedPath, Multipart, Path, Query, State};
use axum::response::Redirect;
use axum::{routing::get, Router};

use axum_session::{Session, SessionPgPool};

use chrono::Local;
use maud::{html, Markup};

use anyhow::anyhow;
use argon2::{Argon2, PasswordHash};
use serde::Deserialize;
use sqlx::types::Uuid;
use std::fs::File;

use crate::fun::image::{get_format_and_ext, get_orientation, img_rotate};
use crate::fun::layout::Html;
use crate::fun::message;
use crate::fun::widget::*;
use crate::http::email;
use crate::http::types::Page;
use crate::http::{error::AppError, Result, WebContext};

use db::*;

const RESET_PW_INVALID_SECS: i64 = 1800;

#[allow(dead_code)]
enum Role {
    Normal = 1,
    Sudo = 5,
}

pub fn router() -> Router<WebContext> {
    Router::new()
        .route("/user/reg", get(reg_input).post(reg_do))
        .route("/user/reg/v/:uid", get(reg_ok))
        .route("/user/login", get(login_input).post(login_do))
        .route("/user/status", get(status))
        .route("/my/hx", get(my_huxi))
        .route("/my/new/pw", get(pw_new_input).post(pw_new_do))
        .route("/my/pw", get(pw1_input).post(pw1_do))
        .route("/my/info", get(my_info))
        .route("/my/img", get(img_update_input).post(img_update_do))
        .route("/my/pw/new/:id", get(pw2_input).post(pw2_do))
        .route("/user/logout", get(logout))
}

/// 注册
async fn reg_input(
    mess: Query<UrlArgs>,
    State(ctx): State<WebContext>,
    session: Session<SessionPgPool>,
    path: MatchedPath,
) -> Result<Page> {
    let code = sess_code(&session).await;
    let mm = if let Some(true) = mess.error {
        Some(vec!["有错误".to_string()])
    } else {
        None
    };
    let input: RegInput = Default::default();
    let main = reg_form(&input, mm, code);
    Ok(Html::new("新用户注册", main)
        .path(Some(path.as_str()))
        .show_title(false)
        .page(&ctx))
}

#[derive(Deserialize, Default, Debug)]
struct UrlArgs {
    from: Option<String>,
    error: Option<bool>,
}

// 目前先重点实现，以后再考虑增建字段
fn reg_form(user: &RegInput, error: ErrorMessage, code: u16) -> Markup {
    html! {
        div class="container m-3" {
            div class="row justify-content-center" {
                div class="col col-md-7  col-xl-5 border p-3 shadow-lg mb-5 bg-body rounded " {
                    form action="" method="post" {
                        div class="w-100 text-center mb-5" { h2 {"新用户注册"}}
                        (error_message(error))
                        hr;
                        // p class="text-center text-success" {"请填写以下内容"}

                        div class="row mb-3" {
                            label for="name" class="col-sm-4 col-form-label text-md-end" {"用户名："}
                            div class="col-sm-6" {
                                (TextInput::new("name", "name", true).value(Some(&user.name)).show())
                                div class="text-secondary" {"请输入 2-20 个字符"}
                            }
                            div class="col-sm-2" {}
                        }
                        hr;
                        div class="row mb-3" {
                            label for="email"  class="col-sm-4 col-form-label text-md-end" {"邮箱："}
                            div class="col-sm-6" {
                                (TextInput::new("email", "email", true).with_type("email").value(Some(&user.email)).show())
                            }
                            div class="col-sm-2" {}
                        }
                        hr;
                        div class="row mb-3" {
                            label for="password"  class="col-sm-4 col-form-label text-md-end" {"密码："}
                            div class="col-sm-6" {
                                (TextInput::new("password", "password", true).with_type("password").show())
                            }
                            div class="col-sm-2" {}
                        }
                        div class="row mb-3" {
                            label for="password2" class="col-sm-4 col-form-label text-md-end"  {"确认密码："}
                            div class="col-sm-6" {
                                (TextInput::new("password2", "password2", true).with_type("password").show())
                            }
                            div class="col-sm-2" {}
                        }
                        hr;
                        div class="row mb-3" {
                            label for="validate" class="col-sm-4 col-form-label text-md-end" {"验证码："}
                            div class="col-sm-6" {
                                div class="d-flex" {
                                    (TextInput::new("validate", "validate", true).show())
                                    label for="validate" class="p-2 border border-info bg-success text-dark bg-opacity-25 m-2" {(code)}
                                }
                                div class="text-secondary" {"请输入右边的数字"}
                            }
                            div class="col-sm-2" {}
                        }
                        hr;
                        div class="text-center" {
                            (submit("确定"))
                        }
                    }
                }
            }
        }
    }
}

async fn reg_do(
    State(ctx): State<WebContext>,
    session: Session<SessionPgPool>,
    path: MatchedPath,
    Form(mut input): Form<RegInput>,
) -> Result<Page> {
    let mut error: Vec<String> = Vec::new();
    input.trim();
    if input.name.is_empty() {
        error.push("请输入用户名".to_string());
    } else if input.name.contains('@') {
        error.push("用户名中不能包含字符 @".to_string());
    }
    if input.email.is_empty() {
        error.push("请输入邮箱".to_string());
    } else {
        let email_regex = regex::Regex::new(r"^[a-zA-Z0-9.!#$%&'*+/=?^_`{|}~-]+@[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?(?:\.[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?)*$").unwrap();
        if !email_regex.is_match(&input.email) {
            error.push("邮箱格式错误".to_string());
        }
    }
    if input.password.is_empty() {
        error.push("请输入密码".to_string());
    }
    if input.password2.is_empty() {
        error.push("请输入确认密码".to_string());
    }
    if input.validate.is_empty() {
        error.push("请输入验证码".to_string());
    } else {
        let res = input.validate.parse::<u16>();
        if let Ok(v) = res {
            let sess = session.get::<u16>("validate");
            if let Some(sess_v) = sess {
                if v != sess_v {
                    error.push("验证码错误".to_string());
                }
            } else {
                error.push("验证码错误".to_string());
            }
        } else {
            error.push("验证码错误".to_string());
        }
    }
    let user = db_get_user(&ctx, By::Name(input.name.clone())).await;
    if let Ok(Some(_)) = user {
        error.push("用户名已被使用，请重新输入用户名".to_string());
    }
    let user = db_get_user(&ctx, By::Email(input.email.clone())).await;
    if let Ok(Some(_)) = user {
        error.push("邮箱已被使用，请重新输入邮箱".to_string());
    }

    if !error.is_empty() {
        let code = sess_code(&session).await;
        let main = reg_form(&input, Some(error), code);
        return Ok(Html::new("add input", main)
            .path(Some(path.as_str()))
            .page(&ctx));
    }
    let email = input.email.clone();
    let name = input.name.clone();
    let uid = db_add_user(&ctx, input).await?;

    // send email
    let url = format!("{}/user/reg/v/{}", &ctx.config.host.www, uid);
    let send = email::send_email(
        &ctx.config,
        &email,
        format!("{} 用户注册激活", &ctx.config.host.name.as_str()).as_str(),
        format!(
            r#"<p>您好，{}</p>
<p>您当前正在注册 {}，请点击以下网址激活用户：</p>
<p><a href="{}">{}</a></p>
<p>
{}
</p>
"#,
            &name,
            &ctx.config.host.name.as_str(),
            &url,
            &url,
            &ctx.config.host.domain.as_str()
        ),
    )
    .await;

    let main = if send.is_ok() {
        tip(format!("发送注册激活邮件到了 {}, 请查收", email).as_str())
    } else {
        tip(format!("验证邮件{}发送失败，请重新注册", email).as_str())
    };
    Ok(Html::new("新用户注册", main)
        .path(Some(path.as_str()))
        .page(&ctx))
}

async fn reg_ok(
    State(ctx): State<WebContext>,
    Path(uid): Path<Uuid>,
    path: MatchedPath,
) -> Result<Page> {
    let update = db_update_user_reg(&ctx, uid).await;
    let main;
    if let Ok(name) = update {
        //send message
        let sm = message::db::Input {
            title: "你好，欢迎！".to_string(),
            to_user_name: name.clone(),
            body: format!(
                "你好，{}，欢迎！

保持联系，有问题，随时留言 :)
",
                &name
            ),
        };
        let _res = message::send_message(&ctx, 1, "huxi", sm).await;

        main = tip("新用户注册成功，请登录。");
    } else {
        main = tip("发生错误，请重新注册");
    }

    Ok(Html::new("用户注册", main)
        .path(Some(path.as_str()))
        .page(&ctx))
}

async fn db_update_user_reg(ctx: &WebContext, uid: Uuid) -> anyhow::Result<String> {
    let rec = sqlx::query!(
        r#"update users
           set i_role=1
           where uid=$1
           returning name"#,
        uid
    )
    .fetch_one(&ctx.db)
    .await?;
    Ok(rec.name)
}

/// 登录
async fn login_input(
    State(ctx): State<WebContext>,
    args: Option<Query<UrlArgs>>,
    path: MatchedPath,
) -> Result<Page> {
    let Query(args) = args.unwrap_or_default();
    let mm = if let Some(true) = args.error {
        Some(vec!["用户名或密码错误，请重新输入".to_string()])
    } else {
        None
    };
    login_html(&ctx, mm, path, args.from)
}

fn login_html(
    ctx: &WebContext,
    error: ErrorMessage,
    path: MatchedPath,
    from: Option<String>,
) -> Result<Page> {
    let main = html! {
        div {
            div class="container" {
                div class="row justify-content-center" {
                    div class="col col-md-5  col-xl-3 border p-3 shadow-lg m-5 bg-body rounded" {
                        form action="" method="post" {
                            div class="w-100 text-center mb-5" {
                                h2 {"用户登录"}
                            }
                            (error_message(error))
                                input type="hidden" name="from" value=(from.map_or(String::new(), |v| v));
                            div class="m-2" {
                                label for="name" class="form-label" {"用户名或邮箱："}
                                (TextInput::new("name", "name", true).show())
                            }
                            div class="m-2" {
                                label for="password" {"密码："}
                                (TextInput::new("password", "password", true).with_type("password").show())
                            }
                            div class="m-2" {
                                (checkbox("remember", "remember", "1", false))
                                    label for="remember" class="p-1" {"记住我"}
                            }
                            div class="text-end" {
                                a href="/my/pw" class="me-2" {"忘记密码？"}
                                a href="/user/reg" {"新用户注册"}
                            }
                            div class="text-center" {
                                (submit("确定"))
                            }
                        }
                    }
                }
            }
        }
    };
    Ok(Html::new("用户登录", main)
        .path(Some(path.as_str()))
        .show_title(false)
        .page(ctx))
}

#[derive(Deserialize, Debug)]
struct LoginInput {
    name: String,
    password: String,
    from: String,
    remember: Option<u8>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct SessUser {
    pub id: i32,
    pub name: String,
    pub email: String,
    pub role: i16,
}

async fn login_do(
    State(ctx): State<WebContext>,
    session: Session<SessionPgPool>,
    //_path: MatchedPath,
    Form(input): Form<LoginInput>,
) -> Redirect {
    let name = input.name.trim();
    let user = if name.contains('@') {
        db_get_user(&ctx, By::Email(name.to_string())).await
    } else {
        db_get_user(&ctx, By::Name(name.to_string())).await
    };
    if let Ok(Some(u)) = user {
        let v = verify_password(input.password, u.password).await;
        if v.is_ok() {
            session.set(
                "user",
                SessUser {
                    id: u.user_id,
                    name: u.name,
                    email: u.email,
                    role: u.i_role,
                },
            );
            if input.remember.is_some() {
                session.set_longterm(true);
            } else {
                session.set_longterm(false);
            }
            if input.from.is_empty() {
                return Redirect::to("/my/hx");
            } else {
                return Redirect::to(input.from.as_str());
            }
        }
    }

    Redirect::to("/user/login?error=true")
}

async fn status(session: Session<SessionPgPool>) -> String {
    let v: Option<SessUser> = session.get("user");
    if let Some(user) = v {
        return user.name;
    }
    "".to_string()
}

/// 我的呼吸
async fn my_huxi(
    State(ctx): State<WebContext>,
    session: Session<SessionPgPool>,
    path: MatchedPath,
) -> Result<Page> {
    let user = get_user_from(&session).await?;
    let new_total = message::db::db_new_total(&ctx, user.id).await?;
    let main = html! {

        div class="container" {
            div class="row justify-content-center" {
                div class="col col-md-10 col-xl-10 border shadow-lg p-3 mb-5 bg-body rounded" {
                    div class="m-2 p3" {
                        h3 {"你好，" (&user.name) "， " span class="mx-2" {"欢迎！"}}
                        br;
                        @if new_total > 0 {
                            div class="text-dark bg-light m-3 p-2 border border-info" {
                                "提示：发现新信件 " (new_total) " 封。"

                                    a class="btn btn-primary m-2" href="/my/inbox#start" {"打开收信箱"}
                            }
                        }
                    }
                }
            }
        }
    };
    Ok(Html::new(&ctx.config.host.my_name, main)
        .my_huxi(&user)
        .path(Some(path.as_str()))
        .page(&ctx))
}

async fn my_info(
    State(ctx): State<WebContext>,
    session: Session<SessionPgPool>,
    path: MatchedPath,
) -> Result<Page> {
    let user = get_user_from(&session).await?;
    let main = html! {
        div class="container" {
            div class="row justify-content-center" {
                div class="col col-md-10 col-xl-10 border shadow-lg p-3 mb-5 bg-body rounded" {
                    div class="m-2 p3" {
                       h5 class="mb-2 p-2 border-bottom border-secondary border-2" {"基本信息"}
                        "电子邮件：" (user.email)
                    }
                    div class="m-2 p3" {
                        h5 class="mb-2 p-2 border-bottom border-secondary border-2" {"密码"}
                        a href="/my/new/pw" {"修改密码"}
                    }
                    div class="m-2 p3" {
                       h5 class="mb-2 p-2 border-bottom border-secondary border-2" {"头像"}
                       a href="/my/img" {"修改头像"}
                    }
                }
            }
        }
    };
    Ok(Html::new("个性化设置", main)
        .my_huxi(&user)
        .path(Some(path.as_str()))
        .page(&ctx))
}

/// 重置密码: 输入用户名
async fn pw1_input(
    State(ctx): State<WebContext>,
    _session: Session<SessionPgPool>,
) -> Result<Page> {
    pw1_html(&ctx, "找回密码", "请输入用户名或邮箱", false)
}
fn pw1_html(ctx: &WebContext, title: &str, message: &str, sent: bool) -> Result<Page> {
    let main = html! {
        div class="container" {
            div class="row justify-content-center" {
                div class="col col-md-5  col-xl-3 border p-3 shadow-lg m-5 bg-body rounded " {
                    p {(message)}
                    @if !sent {
                        form action="" method="post" {
                            div class="" {
                                (TextInput::new("name", "name", true).show())
                            }
                            div class="text-center" {
                                (submit("确定"))
                            }
                        }
                    }
                }
            }
        }
    };
    Ok(Html::new(title, main).page(ctx))
}

#[derive(Deserialize, Debug)]
struct UserNameInput {
    name: String,
}

async fn pw1_do(
    State(ctx): State<WebContext>,
    //_session: Session<SessionPgPool>,
    path: MatchedPath,
    Form(input): Form<UserNameInput>,
) -> Result<Page> {
    let name = input.name.trim();
    let user = if name.contains('@') {
        db_get_user(&ctx, By::Email(name.to_string())).await
    } else {
        db_get_user(&ctx, By::Name(name.to_string())).await
    };
    if let Ok(Some(u)) = user {
        // db insert a new data, return uuid
        let reset_id: Uuid = db_add_pw_reset(&ctx, &u).await?;
        // send email
        let url = format!("{}/my/pw/new/{}", &ctx.config.host.www, reset_id);
        let send = email::send_email(
            &ctx.config,
            &u.email,
            format!("{} 重置密码", &ctx.config.host.name.as_str()).as_str(),
            format!(
                r#"<p>您好，{}</p>
<p>您当前正在重新设置密码，请点击以下网址设置新密码：</p>
<p><a href="{}">{}</a></p>
<p>
{}
</p>
"#,
                &u.name,
                &url,
                &url,
                &ctx.config.host.domain.as_str()
            ),
        )
        .await;
        if send.is_ok() {
            let main = tip(format!("找到了用户{}，请查收邮箱{}", u.name, u.email).as_str());
            Ok(Html::new("找回密码", main)
                .path(Some(path.as_str()))
                .page(&ctx))
        } else {
            pw1_html(&ctx, "找回密码", "发送邮件失败，请重新输入", false)
        }
    } else {
        pw1_html(
            &ctx,
            "找回密码",
            "用户不存在，请重新输入用户名或邮箱",
            false,
        )
    }
}

async fn pw2_input(
    State(ctx): State<WebContext>,
    _session: Session<SessionPgPool>,
    Path(id): Path<String>,
    path: MatchedPath,
) -> Result<Page> {
    let pw_req = pw2_db_get_pwreq_by_id(&ctx, &id).await;
    let main;
    if let Ok(r) = pw_req {
        let req_time = r.created_at.with_timezone(&Local).timestamp();
        let local = Local::now().timestamp();
        if local - req_time < RESET_PW_INVALID_SECS {
            let message = format!("你好{}, 请输入新密码", r.user_name);
            return pw2_html(&ctx, &message);
        } else {
            main = tip("已经失效");
        }
    } else {
        main = tip("不存在错误");
    }
    Ok(Html::new("找回密码", main)
        .path(Some(path.as_str()))
        .page(&ctx))
}
async fn pw_new_input(
    State(ctx): State<WebContext>,
    session: Session<SessionPgPool>,
    path: MatchedPath,
) -> Result<Page> {
    let user = get_user_from(&session).await?;
    let message = format!("你好{}, 请输入以下信息", user.name);
    let main = pw_new_html(None, &message);

    Ok(Html::new("设置新密码", main)
        .path(Some(path.as_str()))
        .my_huxi(&user)
        .page(&ctx))
}
fn pw_new_html(error: ErrorMessage, message: &str) -> Markup {
    html! {
        div class="container" {
            div class="row justify-content-center" {
                div class="col col-md-8  col-xl-5 border p-3 shadow-lg m-5 bg-body rounded" {
                    p {(message)}
                    div {
                        (error_message(error))
                    }
                    form action="" method="post" {
                        div class="" {
                            label for="old" {"旧密码"} (TextInput::new("old", "old", true).with_type("password").show())
                        }
                        div class="" {
                            label for="password" {"新密码"} (TextInput::new("password", "password", true).with_type("password").show())
                        }
                        div class="" {
                            label for="password2" {"确认新密码"} (TextInput::new("password2", "password2", true).with_type("password").show())
                        }
                        div class="text-center" {
                            (submit("确定"))
                        }
                    }
                }
            }
        }
    }
}

#[derive(Deserialize, Debug)]
struct PwNewInput {
    old: String,
    password: String,
    password2: String,
}

async fn pw_new_do(
    State(ctx): State<WebContext>,
    session: Session<SessionPgPool>,
    path: MatchedPath,
    Form(input): Form<PwNewInput>,
) -> Result<Page> {
    let user = get_user_from(&session).await?;

    let db_user = db_get_user(&ctx, By::Name(user.name.clone())).await;
    let mut error: Vec<String> = Vec::new();
    if let Ok(Some(u)) = db_user {
        let v = verify_password(input.old, u.password).await;
        if v.is_err() {
            error.push("旧密码输入错误".to_string());
        }
    } else {
        error.push("数据库错误".to_string());
    }
    if input.password != input.password2 {
        error.push("新密码和确认密码不匹配".to_string());
    }
    if !error.is_empty() {
        let message = "输入错误，请重新输入";
        let main = pw_new_html(Some(error), message);
        return Ok(Html::new("设置新密码", main)
            .path(Some(path.as_str()))
            .my_huxi(&user)
            .page(&ctx));
    }
    db_update_user_pw(&ctx, user.id, input.password).await?;
    let main = tip("新密码设置成功");
    Ok(Html::new("新密码设置成功", main)
        .path(Some(path.as_str()))
        .my_huxi(&user)
        .page(&ctx))
}
// ==================================================
/// 重置密码: 输入新密码
fn pw2_html(ctx: &WebContext, message: &str) -> Result<Page> {
    let main = html! {
        div class="container" {
            div class="row justify-content-center" {
                div class="col col-md-5  col-xl-3 border p-3 shadow-lg m-5 bg-body rounded" {
                    p {(message)}
                    form action="" method="post" {
                        div class="" {
                            label for="password" {"密码"} (TextInput::new("password", "password", true).with_type("password").show())
                        }
                        div class="" {
                            label for="password2" {"确认密码"} (TextInput::new("password2", "password2", true).with_type("password").show())
                        }
                        div class="text-center" {
                            (submit("确定"))
                        }
                    }
                }
            }
        }
    };
    Ok(Html::new("重置密码", main).page(ctx))
}

#[derive(Deserialize, Debug)]
struct PwInput {
    password: String,
    password2: String,
}

async fn pw2_do(
    State(ctx): State<WebContext>,
    //_session: Session<SessionPgPool>,
    Path(id): Path<String>,
    path: MatchedPath,
    Form(input): Form<PwInput>,
) -> Result<Page> {
    let pw_req = pw2_db_get_pwreq_by_id(&ctx, &id).await;
    let main;
    if let Ok(r) = pw_req {
        let req_time = r.created_at.with_timezone(&Local).timestamp();
        let local = Local::now().timestamp();
        if local - req_time < RESET_PW_INVALID_SECS {
            if input.password == input.password2 {
                // db update user password
                db_update_user_pw(&ctx, r.user_id, input.password).await?;
                // db set status 1
                db_update_pw_reset(&ctx, &id).await?;
                main = tip("密码重置成功，请重新登录");
            } else {
                let message = format!("你好{}, 密码输入错误！", r.user_name);
                return pw2_html(&ctx, &message);
            }
        } else {
            main = tip("失效了");
        }
    } else {
        main = tip("不存在错误");
    }
    Ok(Html::new("找回密码", main)
        .path(Some(path.as_str()))
        .page(&ctx))
}

/// logout
async fn logout(session: Session<SessionPgPool>) -> Redirect {
    let v: Option<SessUser> = session.get("user");
    if v.is_some() {
        session.remove("user");
    }
    Redirect::temporary("/")
}

// 常用函数
/// 获得登录用户
pub async fn get_user_from(session: &Session<SessionPgPool>) -> Result<SessUser> {
    let v: Option<SessUser> = session.get("user");
    v.ok_or_else(|| AppError::InvalidLogin("/user/login".into()))
}

/// 检查管理员权限
pub async fn check_sudo(session: &Session<SessionPgPool>) -> bool {
    let user: Option<SessUser> = session.get("user");
    if let Some(u) = user {
        if u.role >= Role::Sudo as i16 {
            return true;
        }
    }
    false
}

pub fn is_sudo_role(user_role: i16) -> bool {
    user_role >= Role::Sudo as i16
}

/// 验证密码
async fn verify_password(password: String, password_hash: String) -> Result<()> {
    let hash = PasswordHash::new(&password_hash)
        .map_err(|_| AppError::InvalidArg("error password".to_owned()))?;

    hash.verify_password(&[&Argon2::default()], password)
        .map_err(|e| match e {
            argon2::password_hash::Error::Password => {
                AppError::InvalidArg("error password".to_owned())
            }
            _ => AppError::InvalidArg("error password".to_owned()),
        })
}

async fn sess_code(session: &Session<SessionPgPool>) -> u16 {
    let vv: u16 = rand::random::<u8>() as u16 + 2000;
    session.set("validate", vv);
    vv
}

async fn img_update_input(
    State(ctx): State<WebContext>,
    session: Session<SessionPgPool>,
    path: MatchedPath,
) -> Result<Page> {
    let user = get_user_from(&session).await?;
    let main = input_form(&ctx, None, true);
    Ok(Html::new("修改头像", main)
        .path(Some(path.as_str()))
        .my_huxi(&user)
        .page(&ctx))
}

fn input_form(_ctx: &WebContext, error: ErrorMessage, is_add: bool) -> Markup {
    html! {
        div class="container" {
            div class="row justify-content-center" {
                div class="col col-md-10 col-xl-10 border shadow-lg p-3 mb-5 bg-body rounded" {
                    div {
                        (error_message(error))
                    }
                    form action="" method="post" enctype="multipart/form-data" {
                        div class="row mb-3 pb-3 border-bottom" {
                            label for="file" class="col-md-2 col-form-label text-md-end" {"头像文件："}
                            div class="col-md-7" {
                                (TextInput::new("file", "file", is_add).with_type("file").show())
                                    "(文件小于500K，支持jpg，png或gif格式图片)"
                            }
                            div class="col-md-3" {
                            }
                        }
                        div class="text-center" {
                            (submit("确定"))
                        }
                    }
                }
            }
        }
    }
}

async fn img_update_do(
    session: Session<SessionPgPool>,
    State(ctx): State<WebContext>,
    path: MatchedPath,
    multipart: Multipart,
) -> Result<Page> {
    let user = get_user_from(&session).await?;
    let mut error: Vec<String> = vec![];
    let result = form_data(multipart).await;
    if result.is_err() {
        error.push("上传错误，请重新上传".to_owned());
        let main = input_form(&ctx, Some(error), true);
        return Ok(Html::new("上传错误", main)
            .path(Some(path.as_str()))
            .my_huxi(&user)
            .page(&ctx));
    }
    let data = result.unwrap();
    if data.is_none() {
        error.push("上传错误，文件数据错误，重新上传".to_owned());
    }
    if !error.is_empty() {
        let main = input_form(&ctx, Some(error), true);
        return Ok(Html::new("上传错误", main)
            .path(Some(path.as_str()))
            .my_huxi(&user)
            .page(&ctx));
    }
    let result = save(data.unwrap(), user.id, &ctx).await;
    if result.is_err() {
        let main = input_form(&ctx, Some(vec!["上传错误，请重新上传".to_owned()]), true);
        return Ok(Html::new("上传错误", main)
            .path(Some(path.as_str()))
            .my_huxi(&user)
            .page(&ctx));
    }

    let tip = tip("头像更新成功");

    Ok(Html::new("头像更新成功", tip)
        .my_huxi(&user)
        .path(Some(path.as_str()))
        .page(&ctx))
}

async fn form_data(mut multipart: Multipart) -> anyhow::Result<Option<Vec<u8>>> {
    let mut data: Option<Vec<u8>> = None;
    while let Some(field) = multipart.next_field().await? {
        let f_name = field.name().unwrap_or("").to_string();
        let bytes: Vec<u8> = field.bytes().await?.into_iter().collect();
        match &*f_name {
            "file" => {
                if !bytes.is_empty() {
                    data = Some(bytes);
                }
            }
            _ => return Err(anyhow!("Invalid Parameter")),
        }
    }
    Ok(data)
}

async fn save(data: Vec<u8>, user_id: i32, ctx: &WebContext) -> Result<()> {
    let img_bytes = data;
    let orientation = get_orientation(&img_bytes);
    let (format, ext) = get_format_and_ext(&img_bytes)?;
    let image_small_file = create_file_name(ext, user_id);
    let save_path_small = format!("{}/{}", ctx.config.user.upload_path, &image_small_file);
    match image::load_from_memory_with_format(&img_bytes, format) {
        Ok(img) => {
            // save small image
            let mut new_img_small = img.thumbnail(150, 150);
            new_img_small = img_rotate(new_img_small, orientation);
            let mut output = File::create(save_path_small).unwrap();
            new_img_small.write_to(&mut output, format).unwrap();
        }
        Err(_) => return Err(AppError::InvalidFileFormat),
    }
    Ok(())
}

pub fn create_file_name(ext: &str, user_id: i32) -> String {
    format!("s-{}.{}", user_id, ext)
}
