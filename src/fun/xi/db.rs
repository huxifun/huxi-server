use chrono::{DateTime, Utc};

use crate::db;
use crate::fun::user::SessUser;
use crate::fun::widget::list::{DbList, List, ListBy};
use crate::http::WebContext;
use crate::md;

#[derive(serde::Serialize, serde::Deserialize, sqlx::FromRow, Debug)]
pub struct Xi {
    #[sqlx(rename = "xi_id")]
    pub id: i32,
    pub user_id: i32,
    pub user_name: String,
    pub title: String,
    pub body: String,
    pub html: Option<String>,
    pub i_public: i16,
    pub i_type: i16,
    pub i_category: i16,
    pub i_good: i16, // 申请推荐
    pub url: Option<String>,
    pub tags: Option<String>,
    pub star: i32,
    pub good: i16, // 推荐结果 1: 推荐，0: 不推荐
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
    pub good_at: Option<DateTime<Utc>>,
    pub click: i32,
}

impl Xi {
    pub fn input(self) -> Input {
        Input {
            title: self.title,
            i_public: self.i_public,
            i_type: self.i_type,
            i_category: self.i_category,
            i_good: Some(self.i_good),
            url: self.url,
            tags: self.tags,
            body: self.body,
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
        Ok(())
    }
}

#[derive(serde::Serialize, serde::Deserialize, sqlx::FromRow, Debug)]
pub struct XiSimple {
    pub title: String,
    pub user_name: String,
    pub xi_id: i32,
    pub i_type: i16,
    pub i_category: i16,
    pub body: String,
    pub html: String,
    pub good: i16,
    pub url: Option<String>,
    pub i_public: i16,
    pub i_good: i16,
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
            s_cat, s_type, s_good, s_title, s_web
        ),
        ListBy::AllPublic => format!(
            "where i_public=1 {} {} {} {} {}",
            s_cat, s_type, s_good, s_title, s_web
        ),
    };
    let sql_total = format!("select count(*) as total from xi {};", &sql_where);
    let row = sqlx::query_as::<_, db::Total>(&sql_total)
        .fetch_one(&list.ctx.db)
        .await?;
    let total = row.total;
    let sql = format!(
        r#"
            select
                xi_id, title, user_name, body, html, i_public, i_type, i_category, good, created_at, url, i_good
            from xi
            {}
            order by xi_id desc limit {} offset {};"#,
        sql_where, list.size, offset
    );
    let rows = sqlx::query_as::<_, XiSimple>(&sql)
        .fetch_all(&list.ctx.db)
        .await?;
    Ok((total, DbList::Xi(rows)))
}

pub async fn db_insert(ctx: &WebContext, user: &SessUser, input: Input) -> anyhow::Result<i32> {
    let html = md::to_html(&input.body);

    let i_good = input.i_good.map_or(0, |v| v);

    let rec = sqlx::query!(
        r#"insert into xi
             (user_id, user_name, title, body, html, i_category, i_public, url, tags, i_good, i_type)
           values ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
           returning xi_id"#,
        user.id,
        user.name,
        input.title,
        input.body,
        html,
        input.i_category,
        input.i_public,
        input.url,
        input.tags,
        i_good,
        input.i_type,
    )
    .fetch_one(&ctx.db)
    .await?;
    Ok(rec.xi_id)
}

pub async fn db_update(ctx: &WebContext, id: i32, input: &Input) -> anyhow::Result<bool> {
    let html = md::to_html(&input.body);
    let i_good = input.i_good.map_or(0, |v| v);
    let rows = sqlx::query!(
        r#"update xi
           set body=$1, html=$2, i_category=$3, i_public=$4, i_type=$5, url=$6, tags=$7, i_good=$8, title=$9 
           where xi_id=$10"#,
        input.body,
        html,
        input.i_category,
        input.i_public,
        input.i_type,
        input.url,
        input.tags,
        i_good,
        input.title,
        id
    )
    .execute(&ctx.db)
    .await?
    .rows_affected();

    Ok(rows > 0)
}

pub async fn db_get_one(ctx: &WebContext, id: i32) -> anyhow::Result<Xi> {
    let rec = sqlx::query_as!(
        Xi,
        r#"select xi_id as id, title, user_id, user_name, body, html, i_category, i_type, i_public, i_good, star,
                url, tags, good, created_at, updated_at, good_at, click
           from xi where xi_id=$1"#,
        id
    )
    .fetch_one(&ctx.db)
    .await?;

    Ok(rec)
}

pub async fn db_update_click(ctx: &WebContext, id: i32) -> anyhow::Result<bool> {
    let rows = sqlx::query!(
        r#"update xi
           set click=click+1
           where xi_id=$1"#,
        id
    )
    .execute(&ctx.db)
    .await?
    .rows_affected();

    Ok(rows == 1)
}

pub async fn db_rm(ctx: &WebContext, id: i32) -> anyhow::Result<bool> {
    let rows = sqlx::query!(
        r#"delete from xi
           where xi_id=$1"#,
        id
    )
    .execute(&ctx.db)
    .await?
    .rows_affected();

    Ok(rows == 1)
}

pub async fn db_good(ctx: &WebContext, id: i32, value: i16) -> anyhow::Result<bool> {
    let rows = sqlx::query!(
        r#"update xi
           set good=$1, good_at=now()
           where xi_id=$2"#,
        value,
        id
    )
    .execute(&ctx.db)
    .await?
    .rows_affected();

    Ok(rows == 1)
}
