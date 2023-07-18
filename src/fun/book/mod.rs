//! 好书

use axum::{routing::get, Router};

use crate::md;
use anyhow::anyhow;
use axum::extract::{MatchedPath, Multipart, Path, Query, State};
use axum_session::{Session, SessionPgPool};
use maud::{html, Markup, PreEscaped};
use serde::Deserialize;
use std::fs::File;

use crate::config::CategoryType;
use crate::fun::comment;
use crate::fun::image::{create_file_name, get_format_and_ext, get_orientation, img_rotate};
use crate::fun::layout::{split, vsplit, Html};
use crate::fun::user::{check_sudo, get_user_from, is_sudo_role, SessUser};
use crate::fun::widget::list::*;
use crate::fun::widget::*;
use crate::http::types::Page;
use crate::http::{error::AppError, Result, WebContext};
//use futures_util::stream::StreamExt;

pub mod db;
use db::*;

static PUBLIC_STATUS: [(i16, &str, &str); 2] = [(0, "craft", "草稿"), (1, "published", "公布")];

pub fn router() -> Router<WebContext> {
    Router::new()
        .route("/book", get(book_pub))
        .route("/book/cat/:cat", get(book_pub_cat))
        .route("/book/cat/:cat/:tid", get(book_pub_cat_type))
        .route("/book/view/:id/index.html", get(book_view))
        .route("/book/search", get(book_search))
        .route("/my/book", get(book_my))
        .route("/my/book/add", get(book_add_input).post(book_add_do))
        .route("/my/book/edit/:id", get(book_edit_input).post(book_edit_do))
        .route("/my/book/rm/:id", get(book_rm))
        .route("/my/book/good/:id", get(book_good))
        .route("/my/book/good/cancel/:id", get(book_good_cancel))
        .route("/my/book/cat/:cat", get(book_my_cat))
        .route("/my/book/cat2/:cat", get(book_my_cat2))
        .route("/my/book/cat/:cat/:tid", get(book_my_cat_type))
        .route("/my/book/search", get(book_search_my))
}

/// 用户登录以后管理
async fn book_my(
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
    Ok(Html::new("我的好书", main)
        .path(Some(path.as_str()))
        .my_huxi(&user)
        .page(&ctx))
}

async fn book_pub(
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
    Ok(Html::new("好书", main).path(Some(path.as_str())).page(&ctx))
}

async fn book_add_input(
    State(ctx): State<WebContext>,
    session: Session<SessionPgPool>,
    cat_type: Option<Query<CatType>>,
    path: MatchedPath,
) -> Result<Page> {
    let user = get_user_from(&session).await?;
    let Query(cat_type) = cat_type.unwrap_or_default();
    let book: Input = Input {
        i_category: cat_type.cat,
        i_type: cat_type.typ,
        ..Default::default()
    };
    let main = input_form(&ctx, &book, None, false);
    Ok(Html::new("新建好书", main)
        .path(Some(path.as_str()))
        .my_huxi(&user)
        // .mde()
        .page(&ctx))
}

/// 字段：body2 补充内容
fn input_form(ctx: &WebContext, book: &Input, error: ErrorMessage, edit: bool) -> Markup {
    let title = if edit { "修改" } else { "新建" };
    html! {
        div {
            div class="container" {
                div class="row justify-content-center" {
                    div class="col col-md-10 col-xl-10 border p-3 shadow-lg mb-5 bg-body rounded" {
                        // <form>
                        form action="" method="post" enctype="multipart/form-data" {
                            // div class="w-100 text-center mb-5" {
                            //     h2 {(title)}
                            // }
                            div {
                                (error_message(error))
                            }
                            div class="row mb-3 border-bottom" {
                                label for="title" class="col-md-2 col-form-label text-md-end" {"* 标题："}
                                div class="col-md-7" {
                                    (TextInput::new("title", "title", true).value(Some(&book.title)).show())
                                }
                                div class="col-md-3" {
                                }
                            }
                            div class="row mb-3 border-bottom" {
                                label for="author" class="col-md-2 col-form-label text-md-end" {"* 作者："}
                                div class="col-md-7" {
                                    (TextInput::new("author", "author", true).value(Some(&book.author)).show())
                                }
                                div class="col-md-3" {
                                }
                            }

                            div class="row mb-3 border-bottom" {
                                label class="col-md-2 col-form-label text-md-end" {"分类："}
                                div class="col-md-10" {
                                    @for (_i, c) in ctx.config.book.category.iter().enumerate() {
                                        (radio(&c.1, "i_category", &c.0.to_string(), book.i_category == c.0 as i16, &c.2))
                                    }
                                }
                            }
                            div class="row mb-3 border-bottom" {
                                label class="col-md-2 col-form-label text-md-end" {"格式："}
                                div class="col-md-10" {
                                    @for (_i, t) in ctx.config.book.content_type.iter().enumerate() {
                                        (radio(&t.1, "i_type", &t.0.to_string(), book.i_type == t.0 as i16, &t.2))
                                    }
                                }
                            }
                            div class="row mb-3 border-bottom" {
                                label class="col-md-2 col-form-label text-md-end" for="version" {"出版社："}
                                div class="col-md-5" {
                                    (TextInput::new("press", "press", false).value(book.press.as_ref()).show())
                                }
                                div class="col-md-5" {
                                }
                            }
                            div class="row mb-3 border-bottom" {
                                label class="col-md-2 col-form-label text-md-end" for="version" {"版本："}
                                div class="col-md-5" {
                                    (TextInput::new("version", "version", false).value(book.version.as_ref()).show())
                                }
                                div class="col-md-5" {
                                }
                            }
                            div class="row mb-3 border-bottom" {
                                label class="col-md-2 col-form-label text-md-end" for="price" {"* 价格："}
                                div class="col-md-5" {
                                    (TextInput::new("price", "price", true).value(book.price.as_ref()).show())
                                }
                                div class="col-md-5" {
                                }
                            }
                            div class="row mb-3 border-bottom" {
                                label class="col-md-2 col-form-label text-md-end" {"* 简介："}
                                div class="col-md-10" {
                                    (TextArea::new("brief", "brief", true).text(book.brief.as_ref()).md().show())
                                }
                            }
                            div class="row mb-3 border-bottom" {
                                label class="col-md-2 col-form-label text-md-end" {"* 详细介绍："}
                                div class="col-md-10" {
                                    (TextArea::new("body", "body", true).text(Some(&book.body)).rows(10).md().show())
                                }
                                // script {
                                //     "const easyMDE = new EasyMDE({element: document.getElementById('body')});"
                                // }
                            }
                            div class="row mb-3 border-bottom" {
                                label class="col-md-2 col-form-label text-md-end" {"相关链接："}
                                div class="col-md-10" {
                                    (TextArea::new("body2", "body2", false).text(book.body2.as_ref()).md().show())
                                }
                            }
                            div class="row mb-3 border-bottom" {
                                label class="col-md-2 col-form-label text-md-end" {"目录："}
                                div class="col-md-10" {
                                    (TextArea::new("log", "log", false).text(book.log.as_ref()).md().show())
                                }
                            }
                            div class="row mb-3 border-bottom" {
                                label class="col-md-2 col-form-label text-md-end" for="tags" {"Tags："}
                                div class="col-md-6" {
                                    (TextInput::new("tags", "tags", false).value(book.tags.as_ref()).show())
                                }
                                div class="col-md-4" {
                                }
                            }
                            div class="row mb-3 border-bottom" {
                                label class="col-md-2 col-form-label text-md-end" for="url" {"阅读网址："}
                                div class="col-md-6" {
                                    (TextInput::new("url", "url", false).value(book.url.as_ref()).show())
                                }
                                div class="col-md-4" {
                                }
                            }
                            div class="row mb-3 pb-3 border-bottom" {
                                label class="col-md-2 col-form-label text-md-end" {"封面图片："}
                                div class="col-md-6" {
                                    @if edit {
                                        @if let Some(ref file) = book.file {
                                            img class="m-3" src={(ctx.config.book.public_url) "/s-" (&file)};
                                        }
                                    }
                                    (TextInput::new("file", "file", !edit).with_type("file").show())
                                        "(文件小于500K，支持jpg，png或gif格式图片)"
                                }
                                div class="col-md-4" {
                                }
                            }
                            div class="row mb-3 border-bottom" {
                                label class="col-md-2 col-form-label text-md-end" {"推荐："}
                                div class="col-md-10" {
                                    (checkbox("i_good", "i_good", "1", book.i_good.map_or(false, |v| v!=0 )))
                                        label for="i_good" class="mx-2" {"申请推荐"}
                                }
                            }
                            div class="row mb-3 border-bottom" {
                                label class="col-md-2 col-form-label text-md-end" {"状态："}
                                div class="col-md-10" {
                                    @for (_i, p) in PUBLIC_STATUS.iter().enumerate() {
                                        (radio(p.1, "i_public", &p.0.to_string(), book.i_public == p.0, p.2))
                                    }
                                }
                            }
                            div class="text-center bg-light" {
                                (submit(title))
                                    a class="btn btn-primary mx-3" href="javascript:window.history.back()" {"取消"}
                                a class="btn btn-primary" href="/my/book#start" {"列表"}
                            }

                        }
                        // </form>
                    }
                }
            }
        }
    }
}

async fn book_add_do(
    State(ctx): State<WebContext>,
    session: Session<SessionPgPool>,
    path: MatchedPath,
    multipart: Multipart,
) -> Result<Page> {
    let user = get_user_from(&session).await?;
    let mut error: Vec<String> = vec![];
    let result = form_data(multipart, None).await;
    if result.is_err() {
        error.push("上传错误，请重新上传".to_owned());
        let book: Input = Default::default();
        let main = input_form(&ctx, &book, Some(error), false);
        return Ok(Html::new("上传错误", main)
            .path(Some(path.as_str()))
            .my_huxi(&user)
            .page(&ctx));
    }
    let mut input = result.unwrap();

    let check = input.check();
    if let Err(e) = check {
        let main = input_form(&ctx, &input, Some(e), false);
        return Ok(Html::new("新建好书", main)
            .my_huxi(&user)
            .path(Some(path.as_str()))
            .page(&ctx));
    }

    if input.data.is_none() {
        error.push("上传错误，文件数据错误，重新上传".to_owned());
    }
    if !error.is_empty() {
        let main = input_form(&ctx, &input, Some(error), false);
        return Ok(Html::new("上传错误", main)
            .path(Some(path.as_str()))
            .my_huxi(&user)
            .page(&ctx));
    }
    let clone = input.clone();
    let result = save(input, &user, &ctx, None).await;
    if result.is_err() {
        let main = input_form(
            &ctx,
            &clone,
            Some(vec!["上传错误，请重新上传".to_owned()]),
            true,
        );
        return Ok(Html::new("上传错误", main)
            .path(Some(path.as_str()))
            .my_huxi(&user)
            .page(&ctx));
    }
    let (_new_file, id) = result.unwrap();
    let url = format!("/book/view/{}/index.html#start", id.unwrap());
    let main = html! {
        div {
            (tip("好书添加成功"))
            div class="text-center" {
                a href=(url) class="m-2" {"继续查看"}
                a href="/my/book" class="m-2" {"显示列表"}
            }
            (PreEscaped(redirect_script("/my/book")))
        }
    };
    Ok(Html::new("好书添加成功", main)
        .my_huxi(&user)
        .path(Some(path.as_str()))
        .page(&ctx))
}

async fn form_data(mut multipart: Multipart, file_src: Option<String>) -> anyhow::Result<Input> {
    let mut title = String::new();
    let mut author = String::new();
    let mut i_public: i16 = 0;
    let mut i_type: i16 = 0;
    let mut i_category: i16 = 0;
    let mut i_good: Option<i16> = None;
    let mut version: Option<String> = None;
    let mut press: Option<String> = None;
    let mut price: Option<String> = None;
    let mut tags: Option<String> = None;
    let mut url: Option<String> = None;
    let mut body = String::new();
    let mut body2: Option<String> = None;
    let mut brief: Option<String> = None;
    let mut log: Option<String> = None;
    let mut file: Option<String> = file_src;
    let mut data: Option<Vec<u8>> = None;

    while let Some(field) = multipart.next_field().await? {
        let f_name = field.name().unwrap_or("").to_string();
        let file_name = field.file_name().unwrap_or("").to_string();
        // let content_type = field.content_type().unwrap_or("").to_string();
        let bytes: Vec<u8> = field.bytes().await?.into_iter().collect();
        match &*f_name {
            "title" => title = String::from_utf8(bytes)?,
            "author" => author = String::from_utf8(bytes)?,
            "version" => version = Some(String::from_utf8(bytes)?),
            "press" => press = Some(String::from_utf8(bytes)?),
            "price" => price = Some(String::from_utf8(bytes)?),
            "tags" => tags = Some(String::from_utf8(bytes)?),
            "url" => url = Some(String::from_utf8(bytes)?),
            "body" => body = String::from_utf8(bytes)?,
            "body2" => body2 = Some(String::from_utf8(bytes)?),
            "brief" => brief = Some(String::from_utf8(bytes)?),
            "log" => log = Some(String::from_utf8(bytes)?),
            "i_public" => {
                let tmp: i16 = std::str::from_utf8(&bytes)?.parse()?;
                if tmp >= 0 {
                    i_public = tmp;
                }
            }
            "i_type" => {
                let tmp: i16 = std::str::from_utf8(&bytes)?.parse()?;
                if tmp >= 0 {
                    i_type = tmp;
                }
            }
            "i_category" => {
                let tmp: i16 = std::str::from_utf8(&bytes)?.parse()?;
                if tmp >= 0 {
                    i_category = tmp;
                }
            }
            "i_good" => {
                let tmp: i16 = std::str::from_utf8(&bytes)?.parse()?;
                if tmp >= 0 {
                    i_good = Some(tmp);
                }
            }
            "file" => {
                if !file_name.is_empty() {
                    file = Some(file_name);
                }
                if !bytes.is_empty() {
                    data = Some(bytes);
                }
            }
            _ => return Err(anyhow!("Invalid Parameter")),
        }
    }
    Ok(Input {
        title,
        author,
        i_public,
        i_type,
        i_category,
        i_good,
        version,
        price,
        tags,
        body,
        body2,
        brief,
        log,
        file,
        data,
        press,
        url,
    })
}

async fn save(
    input: Input,
    user: &SessUser,
    ctx: &WebContext,
    id: Option<i32>,
) -> Result<(Option<String>, Option<u64>)> {
    let mut new_file: Option<String> = None;
    if let Some(data) = input.data {
        let img_bytes = data;
        let orientation = get_orientation(&img_bytes);
        let (format, ext) = get_format_and_ext(&img_bytes)?;
        let (image_new_file, image_small_file) = create_file_name(ext, user.id);
        let save_path = format!("{}/{}", ctx.config.book.upload_path, &image_new_file);
        let save_path_small = format!("{}/{}", ctx.config.book.upload_path, &image_small_file);
        match image::load_from_memory_with_format(&img_bytes, format) {
            Ok(img) => {
                let mut new_img = img.thumbnail(600, 600);
                new_img = img_rotate(new_img, orientation);
                let mut output = File::create(save_path).unwrap();
                new_img.write_to(&mut output, format).unwrap();

                // save small image
                let mut new_img_small = img.thumbnail(200, 200);
                new_img_small = img_rotate(new_img_small, orientation);
                let mut output = File::create(save_path_small).unwrap();
                new_img_small.write_to(&mut output, format).unwrap();
            }
            Err(_) => return Err(AppError::InvalidFileFormat),
        }
        new_file = Some(image_new_file);
    }

    let html = md::to_html(&input.body);
    let mut html2: Option<String> = None;
    let mut log_html: Option<String> = None;
    let mut brief_html: Option<String> = None;
    if let Some(ref text) = input.body2 {
        html2 = Some(md::to_html(text));
    }
    if let Some(ref text) = input.log {
        log_html = Some(md::to_html(text));
    }
    if let Some(ref text) = input.brief {
        brief_html = Some(md::to_html(text));
    }
    let i_good = input.i_good.map_or(0, |v| v);

    // update
    let sql_result: Option<u64> = if let Some(book_id) = id {
        if let Some(ref file) = new_file {
            let rows = sqlx::query!(
                r#"update book
           set title=$1, body=$2, html=$3, i_category=$4, i_public=$5, i_type=$6, version=$7, tags=$8, body2=$9, html2=$10,
               log=$11, log_html=$12, i_good=$13, brief=$14, brief_html=$15, price=$16, file=$17, author=$18, press=$19, url=$20
           where book_id=$21"#,
                input.title,
                input.body,
                html,
                input.i_category,
                input.i_public,
                input.i_type,
                input.version,
                input.tags,
                input.body2,
                html2,
                input.log,
                log_html,
                i_good,
                input.brief,
                brief_html,
                input.price,
                file,
                input.author,
                input.press,
                input.url,
                book_id
            )
            .execute(&ctx.db)
            .await?
            .rows_affected();
            if rows > 0 {
                Some(rows)
            } else {
                None
            }
        } else {
            let rows = sqlx::query!(
                r#"update book
           set title=$1, body=$2, html=$3, i_category=$4, i_public=$5, i_type=$6, version=$7, tags=$8, body2=$9, html2=$10,
               log=$11, log_html=$12, i_good=$13, brief=$14, brief_html=$15, price=$16, author=$17, press=$18, url=$19
           where book_id=$20"#,
                input.title,
                input.body,
                html,
                input.i_category,
                input.i_public,
                input.i_type,
                input.version,
                input.tags,
                input.body2,
                html2,
                input.log,
                log_html,
                i_good,
                input.brief,
                brief_html,
                input.price,
                input.author,
                input.press,
                input.url,
                book_id
            )
            .execute(&ctx.db)
            .await?
            .rows_affected();
            if rows > 0 {
                Some(rows)
            } else {
                None
            }
        }
    } else {
        // add
        let rec = sqlx::query!(
            r#"insert into book
             (user_id, user_name, title, body, html, i_category, i_public, i_type,
               version, tags, body2, html2, log, log_html, i_good, brief, brief_html, price, src, file, author, press, url)
           values ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21, $22, $23)
           returning book_id"#,
            user.id,
            user.name,
            input.title,
            input.body,
            html,
            input.i_category,
            input.i_public,
            input.i_type,
            input.version,
            input.tags,
            input.body2,
            html2,
            input.log,
            log_html,
            i_good,
            input.brief,
            brief_html,
            input.price,
            input.file.unwrap_or("".to_string()),
            &new_file.clone().unwrap(),
            input.author,
            input.press,
            input.url,
        )
        .fetch_one(&ctx.db)
        .await?;
        Some(rec.book_id.try_into().unwrap())
    };
    Ok((new_file, sql_result))
}

async fn book_edit_input(
    State(ctx): State<WebContext>,
    session: Session<SessionPgPool>,
    Path(id): Path<i32>,
    path: MatchedPath,
) -> Result<Page> {
    let book: Book = db_get_one(&ctx, id).await?;
    let user = check_owner(book.user_id, &session).await?;
    let main = input_form(&ctx, &book.to_edit(), None, true);
    Ok(Html::new("修改好书", main)
        .my_huxi(&user)
        .path(Some(path.as_str()))
        // .mde()
        .page(&ctx))
}

async fn check_owner(book_user_id: i32, session: &Session<SessionPgPool>) -> Result<SessUser> {
    let user = get_user_from(session).await?;
    if book_user_id == user.id || is_sudo_role(user.role) {
        Ok(user)
    } else {
        Err(AppError::InvalidLogin("/user/error".into()))
    }
}

async fn book_edit_do(
    State(ctx): State<WebContext>,
    session: Session<SessionPgPool>,
    Path(id): Path<i32>,
    path: MatchedPath,
    multipart: Multipart,
) -> Result<Page> {
    let book = db_get_one(&ctx, id).await?;
    let user = check_owner(book.user_id, &session).await?;
    let mut error: Vec<String> = vec![];
    let result = form_data(multipart, book.file.clone()).await;
    if result.is_err() {
        error.push("上传错误，请重新上传".to_owned());
        let main = input_form(&ctx, &book.to_edit(), Some(error), true);
        return Ok(Html::new("上传错误", main)
            .path(Some(path.as_str()))
            .my_huxi(&user)
            .page(&ctx));
    }
    let mut input = result.unwrap();
    let check = input.check();
    if let Err(e) = check {
        let main = input_form(&ctx, &input, Some(e), true);
        return Ok(Html::new("修改", main)
            .my_huxi(&user)
            .path(Some(path.as_str()))
            .page(&ctx));
    }
    let input_clone = input.clone();
    let result = save(input, &user, &ctx, Some(id)).await;
    if result.is_err() {
        let main = input_form(
            &ctx,
            &input_clone,
            Some(vec!["上传错误，请重新上传2".to_owned()]),
            true,
        );
        return Ok(Html::new("上传错误", main)
            .path(Some(path.as_str()))
            .my_huxi(&user)
            .page(&ctx));
    }

    let (_new_file, rows) = result.unwrap();
    if rows.is_none() {
        let main = input_form(
            &ctx,
            &input_clone,
            Some(vec!["上传错误，请重新上传3".to_owned()]),
            true,
        );
        return Ok(Html::new("上传错误", main)
            .path(Some(path.as_str()))
            .my_huxi(&user)
            .page(&ctx));
    }
    let url = format!("/book/view/{}/index.html#start", id);
    let main = html! {
        div {
            (tip("好书修改成功"))
            div class="text-center" {
                a href=(url) class="m-2" {"继续查看"}
                a href="/my/book" class="m-2" {"显示列表"}
            }
            (PreEscaped(redirect_script("/my/book")))
        }
    };
    Ok(Html::new("修改成功", main)
        .path(Some(path.as_str()))
        .my_huxi(&user)
        .page(&ctx))
}

fn view_url(id: i32) -> String {
    format!("/book/view/{}/index.html", id)
}

async fn book_view(
    State(ctx): State<WebContext>,
    session: Session<SessionPgPool>,
    Path(id): Path<i32>,
    path: MatchedPath,
) -> Result<Page> {
    let book = db_get_one(&ctx, id).await?;

    // 检查权限
    let mut allow = false;
    let mut login = false;
    let mut admin = false;
    let mut sudo = false;
    let mut owner: Option<SessUser> = None;
    if book.i_public > 0 {
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
        if book.user_id == user.id {
            allow = true;
            admin = true;
            owner = Some(user);
        }
    }

    let left = if allow {
        let url = view_url(id);
        let _ = db_update_click(&ctx, id).await;
        let cms = comment::list_comment(&ctx, book.id, url.as_str(), login, admin, "book").await?;
        content_html(&book, cms, &ctx, admin, sudo)
    } else {
        tip("权限错误")
    };

    let main = if owner.is_some() {
        left
    } else {
        let list = List::new(&ctx, ListBy::AllPublic, 1)
            .cat(Some(book.i_category as u8))
            .admin(admin)
            .sudo(sudo)
            .good(Some(1));
        let (total, data) = db_list(&list).await?;
        let cats = list_category_name(&ctx, false);
        let right = vsplit(list.show(total, data), cats);
        split(left, right)
    };

    let mut html = Html::new(&book.title, main)
        .path(Some(path.as_str()))
        .description(book.brief)
        .show_title(false)
        .highlight();
    if owner.is_some() {
        html = html
            .my_huxi(owner.as_ref().unwrap())
            .sub_nav(Some("我的好书"));
    }
    Ok(html.page(&ctx))
}

fn content_html(book: &Book, comment: Markup, ctx: &WebContext, admin: bool, sudo: bool) -> Markup {
    let path = format!(
        "{}/{}",
        ctx.config.book.public_url,
        book.file.as_ref().unwrap()
    );
    html! {
        div class="container" {
            div class="row justify-content-center p-2 shadow-lg mb-5 bg-body rounded row-cols-1" {
                div class="text-end" {
                    (PreEscaped(get_good_status(book.good)))
                }
                div class="col col-md-4" {
                    div class="text-center" {
                        img src=(&path) class="w-100 shadow p-2 mb-3 bg-body rounded";
                        @if let Some(url) = &book.url {
                            a class="fs-5" href={(url)} {"阅读"}
                        }
                    }
                }
                div class="col col-md-8" {
                    div {
                        h3 class="bg-light p-2 m-1" {(book.title)}
                    }
                    div class="row row-cols-2" {
                        div class="col" {
                            "日期：" (show_date(book.created_at))
                        }
                        @if book.updated_at.is_some() {
                            div class="col" {
                                "更新：" (show_date(book.updated_at.unwrap()))
                            }
                        }
                        div class="col" {
                            "作者："  (book.author)
                        }
                        @if let Some(ver) = &book.version {
                            div class="col" {
                                "版本："  (ver)
                            }
                        }
                        @if let Some(press) = &book.press {
                            div class="col" {
                                "出版社："  (press)
                            }
                        }
                        @if let Some((cat_path, cat_name)) = ctx.config.book.category.path_name(book.i_category as u8) {
                            div class="col" {
                                "分类："
                                    mark class="me-2" {
                                        a href={"/book/cat/" (cat_path) } {(cat_name)}
                                    }
                            }
                            @if let Some((type_path, type_name)) = ctx.config.book.content_type.path_name(book.i_type as u8) {
                                div class="col" {
                                    "格式："
                                        mark class="me-2" {
                                            a href={"/book/cat/" (cat_path) "/" (type_path)} {(type_name)}
                                        }
                                }
                            }
                        }
                        div class="col" {
                            "推荐人：" (book.user_name)
                        }
                        div class="col" {
                            "浏览：" (book.click)
                        }
                        @if admin {
                            div class="col" {
                                "状态："(PreEscaped(get_status_name(PUBLIC_STATUS_HTML, book.i_public).map_or("", |v| v)))
                            }
                            div class="col" {
                                @let url = format!("/my/book/edit/{}#start", book.id);
                                "管理：" a href={(url)} class="mx-2" {"编辑"}
                            }
                            div class="col" {
                                "推荐：" (get_igood_status(book.i_good, book.good))
                                @if sudo {
                                    @if book.good == 1 {
                                        a href={"/my/book/good/cancel/" (book.id)} class="ms-2" {"取消推荐"}
                                    } @else {
                                        a href={"/my/book/good/" (book.id)} class="ms-2" {"推荐"}
                                    }
                                }
                            }
                        }
                    }
                    @if let Some(price) = &book.price {
                        div class="border m-2 p-2 bg-light mt-3" {
                            h5 {"价格："  (price)}
                        }
                    }
                    @if let Some(ref html) = book.brief_html {
                        @if !html.is_empty() {
                            div class="mb-3" {
                                div class="border m-2 p-2 md bg-light" {
                                    h5 class="border-bottom border-2 p-2" {"内容简介："}
                                    (PreEscaped(html))
                                }
                            }
                        }
                    }
                    @if let Some(ref html) = book.html2 {
                        @if !html.is_empty() {
                            div class="border m-2 p-2 md bg-light" {
                                (PreEscaped(html))
                            }
                        }
                    }
                }
            }
            div class="row m-2 p-2 shadow-lg mb-5 bg-body rounded" {
                div class="col" {
                    div class="mb-3" {
                        h5 class="bg-light border-bottom border-secondary border-2 p-2" {"详细介绍："}
                        div class="border m-2 p-2 md" {
                            (PreEscaped(&book.html.as_ref().unwrap()))
                        }
                    }
                    @if book.log.is_some() {
                        div class="mb-3" {
                            h5 class="bg-light border-bottom border-secondary border-2 p-2" {"目录："}
                            div class="border m-2 p-2 md" {
                                (PreEscaped(&book.log_html.as_ref().unwrap()))
                            }
                        }
                    }
                    hr;
                    (comment)
                }
            }
        }
    }
}

async fn book_rm(
    State(ctx): State<WebContext>,
    session: Session<SessionPgPool>,
    Path(id): Path<i32>,
    path: MatchedPath,
) -> Result<Page> {
    let book = db_get_one(&ctx, id).await?;
    let user = check_owner(book.user_id, &session).await?;
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
async fn book_good(
    State(ctx): State<WebContext>,
    session: Session<SessionPgPool>,
    Path(id): Path<i32>,
    path: MatchedPath,
) -> Result<Page> {
    let book = db_get_one(&ctx, id).await?;
    let user = check_owner(book.user_id, &session).await?;

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
// 取消推荐
async fn book_good_cancel(
    State(ctx): State<WebContext>,
    session: Session<SessionPgPool>,
    Path(id): Path<i32>,
    path: MatchedPath,
) -> Result<Page> {
    let book = db_get_one(&ctx, id).await?;
    let user = check_owner(book.user_id, &session).await?;

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

async fn book_my_cat(
    State(ctx): State<WebContext>,
    session: Session<SessionPgPool>,
    pagination: Option<Query<Pagination>>,
    Path(cat): Path<String>,
    path: MatchedPath,
) -> Result<Page> {
    let user = get_user_from(&session).await?;
    let Query(pagination) = pagination.unwrap_or_default();
    let page = pagination.page;

    let cat_id = ctx.config.book.category.id(cat.as_str());
    let cat_name = ctx
        .config
        .book
        .category
        .name(cat.as_str())
        .ok_or_else(|| AppError::InvalidArg("cat name error".to_string()))?;

    let list = List::new(&ctx, ListBy::UserId(user.id), page)
        .cat(cat_id)
        .pager(Some(path.as_str()))
        .admin(true)
        .show_cat_type_name()
        .sudo(is_sudo_role(user.role));
    let (total, data) = db_list(&list).await?;
    let main = list.show(total, data);
    let title = format!("{} ", cat_name);
    Ok(Html::new(title.as_str(), main)
        .path(Some(path.as_str()))
        .sub_nav(Some("我的好书"))
        .my_huxi(&user)
        .highlight()
        .page(&ctx))
}

async fn book_my_cat2(
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

    let cat_id = ctx.config.book.category.id(cat.as_str());
    let cid = cat.as_str();
    let cat_name = ctx
        .config
        .book
        .category
        .name(cat.as_str())
        .ok_or_else(|| AppError::InvalidArg("cat name error".to_string()))?;
    let main = html!(
        div class="" {
            @for ty in &ctx.config.book.content_type {
                @let list = List::new(&ctx, ListBy::UserId(user.id), page)
                    .cat(cat_id)
                    .admin(true)
                    .sudo(sudo)
                    .search(false)
                    .size(5)
                    .show_cat_type_name()
                    .i_type(Some(ty.0));
                @let (total, data) = db_list(&list).await?;
                div class="border m-3 shadow p-3 mb-5 bg-body rounded" {
                    div class="bg-light p-2"{
                        a href={"/my/book/cat/" (cid) "/" (ty.1)} {(ty.2)}
                    }
                    div class="p-2 border border-info" {
                        @if total > 0 {
                            (list.show(total, data))
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
        .sub_nav(Some("我的好书"))
        .path(Some(path.as_str()))
        .my_huxi(&user)
        .page(&ctx))
}

async fn book_pub_cat(
    State(ctx): State<WebContext>,
    session: Session<SessionPgPool>,
    pagination: Option<Query<Pagination>>,
    Path(cat): Path<String>,
    path: MatchedPath,
) -> Result<Page> {
    let Query(pagination) = pagination.unwrap_or_default();
    let page = pagination.page;

    let cat_id = ctx.config.book.category.id(cat.as_str());
    let page_link = format!("/book/cat/{}", cat.as_str());
    let cid = cat.as_str();
    let cat_name = ctx
        .config
        .book
        .category
        .name(cat.as_str())
        .ok_or_else(|| AppError::InvalidArg("cat name error".to_string()))?;
    let sudo = check_sudo(&session).await;
    let left = html!(
        div class="" {
            @for ty in &ctx.config.book.content_type {
                @let list = List::new(&ctx, ListBy::AllPublic, page)
                    .cat(cat_id)
                    .admin(false)
                    .sudo(sudo)
                    .show_cat_type_name()
                    .pager(Some(page_link.as_str()))
                    .i_type(Some(ty.0));
                @let (total, data) = db_list(&list).await?;
                @if total > 0 {
                    div class="border m-2" {
                        div class="bg-light p-2"{
                            a href={"/book/cat/" (cid) "/" (ty.1)} {(ty.2)}
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

async fn book_my_cat_type(
    State(ctx): State<WebContext>,
    session: Session<SessionPgPool>,
    pagination: Option<Query<Pagination>>,
    Path((cat, tid)): Path<(String, String)>,
    path: MatchedPath,
) -> Result<Page> {
    let user = get_user_from(&session).await?;
    let Query(pagination) = pagination.unwrap_or_default();
    let page = pagination.page;

    let cat_id = ctx.config.book.category.id(cat.as_str());
    let cat_name = ctx
        .config
        .book
        .category
        .name(cat.as_str())
        .ok_or_else(|| AppError::InvalidArg("cat name error".to_string()))?;
    let type_id = ctx.config.book.content_type.id(tid.as_str());
    let type_name = ctx
        .config
        .book
        .content_type
        .name(tid.as_str())
        .ok_or_else(|| AppError::InvalidArg("type name error".to_string()))?;
    let list = List::new(&ctx, ListBy::UserId(user.id), page)
        .cat(cat_id)
        .show_cat_type_name()
        .sudo(is_sudo_role(user.role))
        .i_type(type_id)
        .pager(Some(path.as_str()));
    let (total, data) = db_list(&list).await?;
    let main = list.show(total, data);
    let title = format!("{} -- {}", cat_name, type_name);
    Ok(Html::new(title.as_str(), main)
        .path(Some(path.as_str()))
        .sub_nav(Some("我的好书"))
        .my_huxi(&user)
        .highlight()
        .page(&ctx))
}

async fn book_pub_cat_type(
    State(ctx): State<WebContext>,
    session: Session<SessionPgPool>,
    pagination: Option<Query<Pagination>>,
    Path((cat, tid)): Path<(String, String)>,
    path: MatchedPath,
) -> Result<Page> {
    let Query(pagination) = pagination.unwrap_or_default();
    let page = pagination.page;

    let cat_id = ctx.config.book.category.id(cat.as_str());
    let cat_name = ctx
        .config
        .book
        .category
        .name(cat.as_str())
        .ok_or_else(|| AppError::InvalidArg("cat name error".to_string()))?;
    let type_id = ctx.config.book.content_type.id(tid.as_str());
    let type_name = ctx
        .config
        .book
        .content_type
        .name(tid.as_str())
        .ok_or_else(|| AppError::InvalidArg("type name error".to_string()))?;
    let page_link = format!("/book/cat/{}/{}", cat, tid);
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
        div class="row row-cols-2" {
            @for i in &ctx.config.book.category {
                @let list = List::new(ctx, ListBy::AllPublic, 1)
                    .cat(Some(i.0))
                    .sudo(sudo)
                    .search(false)
                    .admin(false)
                    .size(5);
                @let (total, data) = db_list(&list).await?;
                @if total > 0 {
                    div class="col" {
                        div class="border m-2" {
                            div class="bg-light p-2 border-bottom" {
                                a href={"/book/cat/" (i.1)} {(i.2)}
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
        .show_cat_type_name()
        .search(false)
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
    let title = if my { "我的好书" } else { "好书分类" };
    let res = html!(
        div class="bg-light m-2 border p-2 mt-3" {
            h5 class="text-center mb-2 p-2 border-bottom border-secondary border-2" {(title)}
            @for i in &ctx.config.book.category {
                div class="p-2 text-center" {
                    @if my {
                        a href={"/my/book/cat2/" (i.1) "#start"} {(i.2)}
                    } @else {
                        a href={"/book/cat/" (i.1)} {(i.2)}
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

async fn book_search(
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
    Ok(Html::new("搜索好书", main)
        .path(Some(path.as_str()))
        .page(&ctx))
}

async fn book_search_my(
    State(ctx): State<WebContext>,
    session: Session<SessionPgPool>,
    args: Option<Query<Search>>,
    path: MatchedPath,
) -> Result<Page> {
    let user = get_user_from(&session).await?;
    let Query(search) = args.unwrap_or_default();
    let page = search.page.unwrap_or(1);
    let key = search.key.as_ref().unwrap().as_str();
    let list = List::new(&ctx, ListBy::UserId(user.id), page)
        .pager(Some(path.as_str()))
        .sudo(is_sudo_role(user.role))
        .title_search_key(Some(key))
        .admin(true);
    let (total, data) = db_list(&list).await?;
    let main = list.show(total, data);
    Ok(Html::new("搜索我的好书", main)
        .path(Some(path.as_str()))
        .my_huxi(&user)
        .page(&ctx))
}
