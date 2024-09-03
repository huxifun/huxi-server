//! 图片

mod db;

use axum::extract::{MatchedPath, Multipart, Path, Query, State};
use axum::{routing::get, Router};
use exif::{In, Reader, Tag};
use image::{guess_format, DynamicImage, ImageFormat};
use std::fs::File;
use std::io::Cursor;

use anyhow::anyhow;
use axum_session::Session;
use axum_session_sqlx::SessionPgPool;
use chrono::Local;
use maud::{html, Markup, PreEscaped};

use super::user::SessUser;
use crate::fun::layout::Html;
use crate::fun::user::{get_user_from, is_sudo_role};
use crate::fun::widget::*;
use crate::http::types::Page;
use crate::http::{error::AppError, Result, WebContext};

use db::*;

pub fn router() -> Router<WebContext> {
    Router::new()
        .route("/my/image", get(image_my))
        .route("/my/image/add", get(image_add_input).post(image_add_do))
        .route(
            "/my/image/edit/:id",
            get(image_edit_input).post(image_edit_do),
        )
        .route("/my/image/rm/:id", get(image_rm))
        .route("/image/view/:id", get(image_view))
}

pub struct List<'a> {
    ctx: &'a WebContext,
    tip: Option<Markup>,
    filter: ListBy,
    page: u32,
    size: u8,
    pager: Option<&'a str>,
}

impl<'a> List<'a> {
    pub fn new(ctx: &'a WebContext, filter: ListBy, page: u32) -> Self {
        List {
            ctx,
            filter,
            page,
            tip: None,
            size: 5,
            pager: None,
        }
    }
    pub fn i_tip(mut self, tip: Option<Markup>) -> Self {
        self.tip = tip;
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

    pub fn show(self, total: i64, data: Vec<ImageSimple>) -> Markup {
        html! {
            div {
                a class="btn btn-outline-primary shadow" href="/my/image/add#start" {"新建"}
            }
            @if let Some(tip_markup) = self.tip {
                (tip_markup)
            }
            div class="container my-3" {
                @for img in data {
                    @let file = img.file.unwrap_or_default();
                    div class="border m-2 p-2 shadow mb-5 bg-body rounded" {
                    div class="row justify-content-center row-cols-1" {
                        div class="col col-md-2" {
                            //@if let Some(file) = img.file {
                            a href={"/image/view/" (img.id) "#start"} {
                                img class="img-thumbnail" src={"/img/pub/s-" (&file)};
                            }
                            br;

                            //}
                        }
                        div class="col col-md-3" {
                            a href={"/image/view/" (img.id) "#start"} {(img.title)}
                        }
                        div class="col col-md-4" {
                            (show_date(img.created_at))
                        }
                        div class="col col-md-3" {
                            a href={"/my/image/edit/" (img.id)} {"编辑"}
                            @let cfm = format!("javascript:if(confirm('确实要删除吗?'))location='/my/image/rm/{}'", img.id);
                            a class="mx-3" href=(cfm) {"删除"}
                        }
                    }
                    div class="text-end" {
                        @let path = format!(
                            "{}/{}",
                            //self.ctx.config.host.www,
                            self.ctx.config.image.public_url,
                            &file
                        );
                        @let md = format!("![{}]({})", &img.title, &path);
                        span class="bg-light m-2 p-2" {
                            (md)
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
}

/// 用户登录以后管理
async fn image_my(
    State(ctx): State<WebContext>,
    session: Session<SessionPgPool>,
    pagination: Option<Query<Pagination>>,
    path: MatchedPath,
) -> Result<Page> {
    let user = get_user_from(&session).await?;
    let Query(pagination) = pagination.unwrap_or_default();
    let page = pagination.page;
    let list = List::new(&ctx, ListBy::UserId(user.id), page).pager(Some(path.as_str()));
    let (total, data) = db_list(&list).await?;
    let main = list.show(total, data);
    Ok(Html::new("我的图片", main)
        .my_huxi(&user)
        .path(Some(path.as_str()))
        .page(&ctx))
}

async fn image_add_input(
    State(ctx): State<WebContext>,
    session: Session<SessionPgPool>,
    path: MatchedPath,
) -> Result<Page> {
    let user = get_user_from(&session).await?;
    let input: Input = Default::default();
    let main = input_form(&ctx, &input, None, true);
    Ok(Html::new("添加图片", main)
        .path(Some(path.as_str()))
        .my_huxi(&user)
        .page(&ctx))
}

fn input_form(ctx: &WebContext, image: &Input, error: ErrorMessage, is_add: bool) -> Markup {
    let title = if is_add { "确定" } else { "修改" };
    html! {
        div class="container" {
            div class="row justify-content-center" {
                div class="col col-md-10 col-xl-10 border p-3 shadow-lg mb-5 bg-body rounded" {
                    div {
                        (error_message(error))
                    }
                    form action="" method="post" enctype="multipart/form-data" {
                        div class="row mb-3 border-bottom" {
                            label for="title" class="col-md-2 col-form-label text-md-end" {"* 标题："}
                            div class="col-md-7" {
                                (TextInput::new("title", "title", true).value(image.title.as_ref()).show())
                            }
                            div class="col-md-3" {
                            }
                        }
                        div class="row mb-3 border-bottom" {
                            label for="brief" class="col-md-2 col-form-label text-md-end" {"简介："}
                            div class="col-md-10" {
                                (TextArea::new("brief", "brief", false).text(image.brief.as_ref()).show())
                            }
                        }
                        div class="row mb-3 border-bottom" {
                            label for="tags" class="col-md-2 col-form-label text-md-end" {"Tags："}
                            div class="col-md-7" {
                                (TextInput::new("tags", "tags", false).value(image.tags.as_ref()).show())
                            }
                            div class="col-md-3" {
                            }
                        }
                        div class="row mb-3 pb-3 border-bottom" {
                            label for="file" class="col-md-2 col-form-label text-md-end" {"文件："}
                            div class="col-md-7" {
                                @if !is_add {
                                    @if let Some(ref file) = image.file {
                                        img class="m-3" src={(ctx.config.image.public_url) "/s-" (&file)};
                                    }
                                }
                                (TextInput::new("file", "file", is_add).with_type("file").show())
                                    "(文件小于500K，支持jpg，png或gif格式图片)"
                            }
                            div class="col-md-3" {
                            }
                        }
                        div  class="row mb-3 border-bottom" {
                            label  class="col-md-2 col-form-label text-md-end" {"缩放："}
                            div class="col-md-10" {
                                @for (i, c) in ctx.config.image.resize.iter().enumerate() {
                                    (radio(&i.to_string(), "resize", &c.1.to_string(),
                                           if let Some(width) = image.resize {
                                               width == c.1
                                           } else {
                                               i == 0
                                           }
                                           , &c.0))
                                }
                            }
                        }
                        div class="text-center" {
                            (submit(title))
                        }
                    }
                }
            }
        }
    }
}

async fn image_add_do(
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
        let image: Input = Default::default();
        let main = input_form(&ctx, &image, Some(error), true);
        return Ok(Html::new("上传错误", main)
            .path(Some(path.as_str()))
            .my_huxi(&user)
            .page(&ctx));
    }
    let image = result.unwrap();
    if image.title.is_none() {
        error.push("上传错误，请输入标题，重新上传".to_owned());
    }
    if image.data.is_none() {
        error.push("上传错误，文件数据错误，重新上传".to_owned());
    }
    if !error.is_empty() {
        let main = input_form(&ctx, &image, Some(error), true);
        return Ok(Html::new("上传错误", main)
            .path(Some(path.as_str()))
            .my_huxi(&user)
            .page(&ctx));
    }
    let clone = image.clone();
    let result = save(image, user.id, &ctx, None).await;
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
    let main = html!(
        (tip("修改添加成功"))
        div class="text-center" {
            a href="/my/image" class="m-2" {"显示列表"}
        }
        (PreEscaped(redirect_script("/my/image")))
    );
    Ok(Html::new("图片添加成功", main)
        .my_huxi(&user)
        .path(Some(path.as_str()))
        .page(&ctx))
}

async fn form_data(mut multipart: Multipart) -> anyhow::Result<Input> {
    let mut title: Option<String> = None;
    let mut brief: Option<String> = None;
    let mut tags: Option<String> = None;
    let mut file: Option<String> = None;
    let mut data: Option<Vec<u8>> = None;
    let mut resize: Option<u32> = None;
    while let Some(field) = multipart.next_field().await? {
        let f_name = field.name().unwrap_or("").to_string();
        let file_name = field.file_name().unwrap_or("").to_string();
        //let content_type = field.content_type().unwrap_or("").to_string();
        let bytes: Vec<u8> = field.bytes().await?.into_iter().collect();
        match &*f_name {
            "title" => title = Some(String::from_utf8(bytes)?),
            "brief" => brief = Some(String::from_utf8(bytes)?),
            "tags" => tags = Some(String::from_utf8(bytes)?),
            "resize" => {
                let tmp: u32 = std::str::from_utf8(&bytes)?.parse()?;
                if tmp > 0 {
                    resize = Some(tmp);
                }
            }
            "file" => {
                file = Some(file_name);
                if !bytes.is_empty() {
                    data = Some(bytes);
                }
            }
            _ => return Err(anyhow!("Invalid Parameter")),
        }
    }
    Ok(Input {
        title,
        brief,
        tags,
        file,
        data,
        resize,
    })
}

async fn save(
    image: Input,
    user_id: i32,
    ctx: &WebContext,
    id: Option<i32>,
) -> Result<(Option<String>, Option<u64>)> {
    let mut new_file: Option<String> = None;
    if let Some(data) = image.data {
        let img_bytes = data;
        let orientation = get_orientation(&img_bytes);
        let (format, ext) = get_format_and_ext(&img_bytes)?;
        let (image_new_file, image_small_file) = create_file_name(ext, user_id);
        let save_path = format!("{}/{}", ctx.config.image.upload_path, &image_new_file);
        let save_path_small = format!("{}/{}", ctx.config.image.upload_path, &image_small_file);
        match image::load_from_memory_with_format(&img_bytes, format) {
            Ok(img) => {
                let img_small = img.clone();
                let mut new_img = if let Some(width) = image.resize {
                    img.thumbnail(width, width)
                } else {
                    img
                };
                new_img = img_rotate(new_img, orientation);
                let mut output = File::create(save_path).unwrap();
                new_img.write_to(&mut output, format).unwrap();

                // save small image
                let mut new_img_small = img_small.thumbnail(100, 100);
                new_img_small = img_rotate(new_img_small, orientation);
                let mut output = File::create(save_path_small).unwrap();
                new_img_small.write_to(&mut output, format).unwrap();
            }
            Err(_) => return Err(AppError::InvalidFileFormat),
        }
        new_file = Some(image_new_file);
    }
    // update
    let sql_result: Option<u64> = if let Some(image_id) = id {
        if let Some(ref file) = new_file {
            let rows = sqlx::query!(
                r#"update image
           set title=$1, brief=$2, tags=$3, file=$4
           where id=$5"#,
                image.title,
                image.brief,
                image.tags,
                file,
                image_id
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
                r#"update image
           set title=$1, brief=$2, tags=$3
           where id=$4"#,
                image.title,
                image.brief,
                image.tags,
                image_id
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
            r#"insert into image
             (user_id, title, brief, tags, src, file)
           values ($1, $2, $3, $4, $5, $6)
           returning id"#,
            user_id,
            image.title.unwrap_or("".to_string()),
            image.brief.unwrap_or("".to_string()),
            image.tags.unwrap_or("".to_string()),
            image.file.unwrap_or("".to_string()),
            &new_file.clone().unwrap()
        )
        .fetch_one(&ctx.db)
        .await?;
        Some(rec.id.try_into().unwrap())
    };
    Ok((new_file, sql_result))
}

async fn image_edit_input(
    session: Session<SessionPgPool>,
    State(ctx): State<WebContext>,
    Path(id): Path<i32>,
    path: MatchedPath,
) -> Result<Page> {
    let image: Image = db_get_one(&ctx, id).await?;
    let user = check_owner(image.user_id, &session).await?;
    let main = input_form(&ctx, &image.input(), None, false);
    Ok(Html::new("图片修改", main)
        .path(Some(path.as_str()))
        .my_huxi(&user)
        .page(&ctx))
}

async fn check_owner(image_user_id: i32, session: &Session<SessionPgPool>) -> Result<SessUser> {
    let user = get_user_from(session).await?;
    if image_user_id == user.id || is_sudo_role(user.role) {
        Ok(user)
    } else {
        Err(AppError::InvalidLogin("/user/error".into()))
    }
}

async fn image_edit_do(
    State(ctx): State<WebContext>,
    session: Session<SessionPgPool>,
    Path(id): Path<i32>,
    path: MatchedPath,
    multipart: Multipart,
) -> Result<Page> {
    let image = db_get_one(&ctx, id).await?;
    let user = check_owner(image.user_id, &session).await?;
    let mut error: Vec<String> = vec![];
    let result = form_data(multipart).await;
    dbg!(&result);
    if result.is_err() {
        error.push("上传错误，请重新上传".to_owned());
        let image: Input = Default::default();
        let main = input_form(&ctx, &image, Some(error), false);
        return Ok(Html::new("上传错误", main)
            .path(Some(path.as_str()))
            .my_huxi(&user)
            .page(&ctx));
    }
    let input = result.unwrap();
    if input.title.is_none() {
        error.push("上传错误，请输入标题，重新上传".to_owned());
    }
    if !error.is_empty() {
        let main = input_form(&ctx, &input, Some(error), false);
        return Ok(Html::new("上传错误", main)
            .path(Some(path.as_str()))
            .my_huxi(&user)
            .page(&ctx));
    }
    let input_clone = input.clone();
    let result = save(input, user.id, &ctx, Some(id)).await;
    if result.is_err() {
        let main = input_form(
            &ctx,
            &input_clone,
            Some(vec!["上传错误，请重新上传2".to_owned()]),
            false,
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
            false,
        );
        return Ok(Html::new("上传错误", main)
            .path(Some(path.as_str()))
            .my_huxi(&user)
            .page(&ctx));
    }
    let main = html!(
        (tip("修改图片成功"))
        div class="text-center" {
            a href={"/image/view/" (id) "#start"} class="m-2" {"继续查看"}
            a href="/my/image" class="m-2" {"显示列表"}
        }
        (PreEscaped(redirect_script("/my/image")))
    );
    Ok(Html::new("修改成功", main)
        .path(Some(path.as_str()))
        .my_huxi(&user)
        .page(&ctx))
}

async fn image_view(
    session: Session<SessionPgPool>,
    State(ctx): State<WebContext>,
    Path(id): Path<i32>,
    path: MatchedPath,
) -> Result<Page> {
    let image = db_get_one(&ctx, id).await?;
    let user = check_owner(image.user_id, &session).await?;
    let main = content_html(&ctx, &image)?;
    Ok(Html::new(&image.title, main)
        .path(Some(path.as_str()))
        .sub_nav(Some("我的图片"))
        .my_huxi(&user)
        .page(&ctx))
}

fn content_html(ctx: &WebContext, image: &Image) -> Result<Markup> {
    let path = format!(
        "{}/{}",
        ctx.config.image.public_url,
        image.file.as_ref().unwrap()
    );
    let md = format!("![{}]({})", &image.title, &path);
    let main = html! {
        div class="container" {
            div class="row justify-content-center" {
                div class="col col-md-10 p-3 shadow-lg mb-5 bg-body rounded" {
                    div class="text-center" {
                        @if let Some(ref tags) = image.tags {
                            span class="col mx-2" {
                                "Tags: " (tags)
                            }
                        }
                        span class="mx-2" {
                            "日期："(show_date(image.created_at))
                        }
                        span class="mx-2" {
                            @if image.updated_at.is_some() {
                                "更新日期："  (show_date(image.updated_at.unwrap()))
                            }
                        }
                    }
                    div class="border m-2 p-2" {
                        (&image.brief.as_ref().unwrap())
                    }
                    div class="border m-2 p-2" {
                        h5 {"Markdown:"}
                        pre class="m-2 p-2" {(md)}
                    }
                    div class="bg-light text-center m-2 p-2" {
                        img src=(&path);
                    }
                }
            }
        }
    };
    Ok(main)
}

async fn image_rm(
    State(ctx): State<WebContext>,
    session: Session<SessionPgPool>,
    Path(id): Path<i32>,
    path: MatchedPath,
) -> Result<Page> {
    let image = db_get_one(&ctx, id).await?;
    let user = check_owner(image.user_id, &session).await?;

    let mut main = tip("删除成功");
    let res = db_rm(&ctx, id).await?;
    if !res {
        main = tip("数据库删除错误");
    }
    Ok(Html::new("删除成功", main)
        .path(Some(path.as_str()))
        .my_huxi(&user)
        .page(&ctx))
}

// common fn =======================
pub fn img_rotate(img: DynamicImage, orientation: u32) -> DynamicImage {
    match orientation {
        1 => img,
        2 => img.flipv(),
        3 => img.rotate180(),
        4 => img.fliph(),
        5 => img.flipv().rotate270(),
        6 => img.rotate90(),
        7 => img.flipv().rotate90(),
        8 => img.rotate270(),
        _ => img,
    }
}

pub fn get_orientation(img_bytes: &Vec<u8>) -> u32 {
    let mut buf = Cursor::new(img_bytes);
    let mut orientation = 1;
    if let Ok(exif) = Reader::new().read_from_container(&mut buf) {
        if let Some(o) = exif.get_field(Tag::Orientation, In::PRIMARY) {
            if let Some(v @ 1..=8) = o.value.get_uint(0) {
                orientation = v;
            }
        }
    }
    orientation
}

pub fn get_format_and_ext(img_bytes: &[u8]) -> Result<(ImageFormat, &'static str)> {
    match guess_format(img_bytes) {
        Ok(ImageFormat::Png) => Ok((ImageFormat::Png, "png")),
        Ok(ImageFormat::Jpeg) => Ok((ImageFormat::Jpeg, "jpg")),
        Ok(ImageFormat::Gif) => Ok((ImageFormat::Gif, "gif")),
        _ => Err(AppError::InvalidFileFormat),
    }
}

pub fn create_file_name(ext: &str, user_id: i32) -> (String, String) {
    let stamp = Local::now().format("%Y%m%d-%H%M%S").to_string();
    (
        format!("{}-{}.{}", user_id, stamp, ext),
        format!("s-{}-{}.{}", user_id, stamp, ext),
    )
}
