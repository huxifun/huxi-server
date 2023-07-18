use chrono::{DateTime, Utc};

use crate::db;
use crate::fun::user::SessUser;
use crate::fun::widget::list::{DbList, List, ListBy};
use crate::http::WebContext;
use crate::md;

#[derive(serde::Serialize, serde::Deserialize, sqlx::FromRow, Debug)]
pub struct Hu {
    #[sqlx(rename = "hu_id")]
    pub id: i32,
    pub user_id: i32,
    pub user_name: String,
    pub title: String,
    pub body: String,
    pub html: Option<String>,
    pub body2: Option<String>,
    pub html2: Option<String>,
    pub brief: Option<String>,
    pub brief_html: Option<String>,
    pub log: Option<String>,
    pub log_html: Option<String>,
    pub i_public: i16,
    pub i_type: i16,
    pub i_category: i16,
    pub i_good: i16, // 申请推荐
    pub url: Option<String>,
    pub tags: Option<String>,
    pub star: i32,
    pub good: i16,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
    pub good_at: Option<DateTime<Utc>>,
    pub click: i32,
}

impl Hu {
    pub fn to_edit(self) -> Input {
        Input {
            title: self.title,
            i_public: self.i_public,
            i_type: self.i_type,
            i_category: self.i_category,
            i_good: Some(self.i_good),
            url: self.url,
            tags: self.tags,
            body: self.body,
            brief: self.brief,
            body2: self.body2,
            log: self.log,
        }
    }
}

#[derive(serde::Deserialize, Debug, Default)]
pub struct Input {
    pub title: String,
    pub i_public: i16,
    pub i_type: i16,
    pub i_category: i16,
    pub i_good: Option<i16>,
    pub url: Option<String>,
    pub tags: Option<String>,
    pub body: String,
    pub body2: Option<String>,
    pub brief: Option<String>,
    pub log: Option<String>,
}

impl Input {
    pub fn check(&mut self) -> std::result::Result<(), Vec<String>> {
        let mut error: Vec<String> = Vec::new();
        self.title = self.title.trim().to_string();
        if self.title.is_empty() {
            error.push("请输入标题".to_string());
        }
        if self.body.is_empty() {
            error.push("请输入正文".to_string());
        }
        if !error.is_empty() {
            return Err(error);
        }
        self.url = self.url.as_ref().and_then(db::check_none);
        self.tags = self.tags.as_ref().and_then(db::check_none);
        self.body2 = self.body2.as_ref().and_then(db::check_none);
        self.brief = self.brief.as_ref().and_then(db::check_none);
        self.log = self.log.as_ref().and_then(db::check_none);
        Ok(())
    }
}
#[derive(serde::Serialize, serde::Deserialize, sqlx::FromRow, Debug)]
pub struct HuSimple {
    pub hu_id: i32,
    pub title: String,
    pub brief: Option<String>,
    pub user_name: String,
    pub i_type: i16,
    pub i_category: i16,
    pub good: i16,
    pub i_public: i16,
    pub i_good: i16,
    pub url: Option<String>,
    pub created_at: DateTime<Utc>,
}

pub async fn db_list<'a>(list: &List<'a>) -> anyhow::Result<(i64, DbList)> {
    let offset = list.size as u32 * (list.page - 1);
    let s_web = list.web_search_key.map_or("".to_string(), |k| {
        format!("and search_ti @@ websearch_to_tsquery('{}') ", k)
    });
    let s_title = list.title_search_key.map_or("".to_string(), |k| {
        format!("and lower(title) like '%{}%' ", k.to_lowercase())
    });
    let s_type = list
        .i_type
        .map_or("".to_string(), |tid| format!("and i_type={} ", tid));
    let s_cat = list
        .cat
        .map_or("".to_string(), |cid| format!("and i_category={} ", cid));
    let s_good = list
        .good
        .map_or("".to_string(), |gid| format!("and good={} ", gid));
    let sql_where = match list.filter {
        ListBy::All => "".to_owned(),
        ListBy::UserId(id) => format!(
            "where user_id={id} {} {} {} {} {}",
            s_cat, s_type, s_good, s_web, s_title
        ),
        ListBy::AllPublic => format!(
            "where i_public=1 {} {} {} {} {}",
            s_cat, s_type, s_good, s_web, s_title
        ),
    };
    let sql_total = format!("select count(*) as total from hu {};", &sql_where);
    let row = sqlx::query_as::<_, db::Total>(&sql_total)
        .fetch_one(&list.ctx.db)
        .await?;
    let total = row.total;
    let sql = format!(
        r#"
            select
                hu_id, title, i_public, i_type, i_category, good, created_at, user_name, brief, url, i_good
            from hu
            {}
            order by hu_id desc limit {} offset {};"#,
        sql_where, list.size, offset
    );
    let rows = sqlx::query_as::<_, HuSimple>(&sql)
        .fetch_all(&list.ctx.db)
        .await?;
    Ok((total, DbList::Hu(rows)))
}

pub async fn db_insert(ctx: &WebContext, user: &SessUser, input: Input) -> anyhow::Result<i32> {
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
    let rec = sqlx::query!(
        r#"insert into hu
             (user_id, user_name, title, body, html, i_category, i_public, i_type,
               url, tags, body2, html2, log, log_html, i_good, brief, brief_html)
           values ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17)
           returning hu_id"#,
        user.id,
        user.name,
        input.title,
        input.body,
        html,
        input.i_category,
        input.i_public,
        input.i_type,
        input.url,
        input.tags,
        input.body2,
        html2,
        input.log,
        log_html,
        i_good,
        input.brief,
        brief_html
    )
    .fetch_one(&ctx.db)
    .await?;
    Ok(rec.hu_id)
}

pub async fn db_update(ctx: &WebContext, id: i32, input: &Input) -> anyhow::Result<bool> {
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
    let rows = sqlx::query!(
        r#"update hu
           set title=$1, body=$2, html=$3, i_category=$4, i_public=$5, i_type=$6, url=$7, tags=$8, body2=$9, html2=$10,
               log=$11, log_html=$12, i_good=$13, brief=$14, brief_html=$15
           where hu_id=$16"#,
        input.title,
        input.body,
        html,
        input.i_category,
        input.i_public,
        input.i_type,
        input.url,
        input.tags,
        input.body2,
        html2,
        input.log,
        log_html,
        i_good,
        input.brief,
        brief_html,
        id
    )
    .execute(&ctx.db)
    .await?
    .rows_affected();

    Ok(rows > 0)
}

pub async fn db_get_one(ctx: &WebContext, id: i32) -> anyhow::Result<Hu> {
    let rec = sqlx::query_as!(
        Hu,
        r#"select hu_id as id, user_id, user_name, title, body, html, body2, html2, log, log_html, brief, brief_html,
               i_category, i_type, i_public, i_good, star, url, tags, good, created_at, updated_at, good_at, click
           from hu where hu_id=$1"#,
        id
    )
    .fetch_one(&ctx.db)
    .await?;

    Ok(rec)
}

pub async fn db_rm(ctx: &WebContext, id: i32) -> anyhow::Result<bool> {
    let rows = sqlx::query!(
        r#"delete from hu
           where hu_id=$1"#,
        id
    )
    .execute(&ctx.db)
    .await?
    .rows_affected();

    Ok(rows == 1)
}

pub async fn db_update_click(ctx: &WebContext, id: i32) -> anyhow::Result<bool> {
    let rows = sqlx::query!(
        r#"update hu
           set click=click+1
           where hu_id=$1"#,
        id
    )
    .execute(&ctx.db)
    .await?
    .rows_affected();

    Ok(rows == 1)
}

pub async fn db_good(ctx: &WebContext, id: i32, value: i16) -> anyhow::Result<bool> {
    let rows = sqlx::query!(
        r#"update hu
           set good=$1, good_at=now()
           where hu_id=$2"#,
        value,
        id
    )
    .execute(&ctx.db)
    .await?
    .rows_affected();

    Ok(rows == 1)
}
