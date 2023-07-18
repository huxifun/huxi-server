use chrono::{DateTime, Utc};

use crate::db;
use crate::fun::widget::list::{DbList, List, ListBy};
use crate::http::WebContext;

#[derive(serde::Serialize, serde::Deserialize, sqlx::FromRow, Debug)]
pub struct Book {
    #[sqlx(rename = "book_id")]
    pub id: i32,
    pub user_id: i32,
    pub user_name: String,
    pub title: String,
    pub author: String,
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

    pub version: Option<String>,
    pub press: Option<String>,
    pub price: Option<String>,
    pub src: Option<String>,
    pub file: Option<String>,

    pub tags: Option<String>,
    pub url: Option<String>,
    pub star: i32,
    pub good: i16,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
    pub good_at: Option<DateTime<Utc>>,
    pub click: i32,
}

impl Book {
    pub fn to_edit(self) -> Input {
        Input {
            title: self.title,
            author: self.author,
            i_public: self.i_public,
            i_type: self.i_type,
            i_category: self.i_category,
            i_good: Some(self.i_good),

            version: self.version,
            press: self.press,

            price: self.price,
            file: self.file,
            data: None,

            tags: self.tags,
            url: self.url,
            body: self.body,
            brief: self.brief,
            body2: self.body2,
            log: self.log,
        }
    }
}

#[derive(serde::Deserialize, Debug, Default, Clone)]
pub struct Input {
    pub title: String,
    pub author: String,
    pub i_public: i16,
    pub i_type: i16,
    pub i_category: i16,
    pub i_good: Option<i16>,
    pub version: Option<String>,
    pub press: Option<String>,
    pub price: Option<String>,
    pub tags: Option<String>,
    pub url: Option<String>,
    pub body: String,
    pub body2: Option<String>,
    pub brief: Option<String>,
    pub log: Option<String>,
    pub file: Option<String>,
    pub data: Option<Vec<u8>>,
}

impl Input {
    pub fn check(&mut self) -> std::result::Result<(), Vec<String>> {
        let mut error: Vec<String> = Vec::new();
        self.title = self.title.trim().to_string();
        if self.title.is_empty() {
            error.push("请输入标题".to_string());
        }
        if self.author.is_empty() {
            error.push("请输入作者".to_string());
        }
        if self.body.is_empty() {
            error.push("请输入详细介绍".to_string());
        }
        if !error.is_empty() {
            return Err(error);
        }
        self.version = self.version.as_ref().and_then(db::check_none);
        self.press = self.press.as_ref().and_then(db::check_none);
        self.price = self.price.as_ref().and_then(db::check_none);
        self.tags = self.tags.as_ref().and_then(db::check_none);
        self.url = self.url.as_ref().and_then(db::check_none);
        self.body2 = self.body2.as_ref().and_then(db::check_none);
        self.brief = self.brief.as_ref().and_then(db::check_none);
        self.log = self.log.as_ref().and_then(db::check_none);
        Ok(())
    }
}
#[derive(serde::Serialize, serde::Deserialize, sqlx::FromRow, Debug)]
pub struct BookSimple {
    pub book_id: i32,
    pub title: String,
    pub user_name: String,
    pub i_type: i16,
    pub i_category: i16,
    pub good: i16,
    pub i_public: i16,
    pub i_good: i16,
    pub file: Option<String>,
    pub url: Option<String>,
    pub brief_html: Option<String>,
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
    let sql_total = format!("select count(*) as total from book {};", &sql_where);
    let row = sqlx::query_as::<_, db::Total>(&sql_total)
        .fetch_one(&list.ctx.db)
        .await?;
    let total = row.total;
    let sql = format!(
        r#"
            select
                book_id, title, i_public, i_type, i_category, good, created_at, user_name, file, brief_html, url, i_good
            from book
            {}
            order by book_id desc limit {} offset {};"#,
        sql_where, list.size, offset
    );
    let rows = sqlx::query_as::<_, BookSimple>(&sql)
        .fetch_all(&list.ctx.db)
        .await?;
    Ok((total, DbList::Book(rows)))
}

pub async fn db_get_one(ctx: &WebContext, id: i32) -> anyhow::Result<Book> {
    let rec = sqlx::query_as!(
        Book,
        r#"select book_id as id, user_id, user_name, title, author, body, html, body2, html2, log, log_html, brief, brief_html,
               i_category, i_type, i_public, i_good, star, version, tags, good, created_at, updated_at, good_at, click, 
               price, src, file, press, url
           from book where book_id=$1"#,
        id
    )
    .fetch_one(&ctx.db)
    .await?;

    Ok(rec)
}

pub async fn db_rm(ctx: &WebContext, id: i32) -> anyhow::Result<bool> {
    let rows = sqlx::query!(
        r#"delete from book
           where book_id=$1"#,
        id
    )
    .execute(&ctx.db)
    .await?
    .rows_affected();

    Ok(rows == 1)
}

pub async fn db_update_click(ctx: &WebContext, id: i32) -> anyhow::Result<bool> {
    let rows = sqlx::query!(
        r#"update book
           set click=click+1
           where book_id=$1"#,
        id
    )
    .execute(&ctx.db)
    .await?
    .rows_affected();

    Ok(rows == 1)
}

pub async fn db_good(ctx: &WebContext, id: i32, value: i16) -> anyhow::Result<bool> {
    let rows = sqlx::query!(
        r#"update book
           set good=$1, good_at=now()
           where book_id=$2"#,
        value,
        id
    )
    .execute(&ctx.db)
    .await?
    .rows_affected();

    Ok(rows == 1)
}
