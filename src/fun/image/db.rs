use crate::fun::image::List;
use crate::http::WebContext;
use chrono::{DateTime, Utc};
use serde::Deserialize;

pub enum ListBy {
    All,
    UserId(i32),
}

#[derive(serde::Serialize, serde::Deserialize, sqlx::FromRow, Debug)]
pub struct Image {
    pub id: i32,
    pub user_id: i32,
    pub title: String,
    pub brief: Option<String>,
    pub tags: Option<String>,
    pub file: Option<String>,
    pub src: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

impl Image {
    pub fn input(self) -> Input {
        Input {
            title: Some(self.title),
            brief: self.brief,
            tags: self.tags,
            file: self.file,
            data: None,
            resize: None,
        }
    }
}
#[derive(Deserialize, Debug, Clone, Default)]
pub struct Input {
    pub title: Option<String>,
    pub brief: Option<String>,
    pub tags: Option<String>,
    pub file: Option<String>,
    pub data: Option<Vec<u8>>,
    pub resize: Option<u32>,
}

#[derive(serde::Serialize, serde::Deserialize, sqlx::FromRow, Debug)]
pub struct ImageSimple {
    pub id: i32,
    pub title: String,
    pub file: Option<String>,
    pub created_at: DateTime<Utc>,
}
#[derive(serde::Serialize, serde::Deserialize, sqlx::FromRow, Debug)]
pub struct DbTotal {
    pub total: i64,
}

pub async fn db_list<'a>(list: &List<'a>) -> anyhow::Result<(i64, Vec<ImageSimple>)> {
    let offset = list.size as u32 * (list.page - 1);
    let sql_where = match list.filter {
        ListBy::All => "".to_owned(),
        ListBy::UserId(id) => format!("where user_id={id}"),
    };

    let sql_total = format!("select count(*) as total from image {};", &sql_where);
    let row = sqlx::query_as::<_, DbTotal>(&sql_total)
        .fetch_one(&list.ctx.db)
        .await?;
    let total = row.total;

    let sql = format!(
        r#"
            select
                id, title, file, created_at
            from image
            {}
            order by id desc limit {} offset {};"#,
        sql_where, list.size, offset
    );
    let rows = sqlx::query_as::<_, ImageSimple>(&sql)
        .fetch_all(&list.ctx.db)
        .await?;
    Ok((total, rows))
}

pub async fn db_get_one(ctx: &WebContext, id: i32) -> anyhow::Result<Image> {
    let rec = sqlx::query_as!(
        Image,
        r#"select id, user_id, title, brief, tags, file, src, created_at, updated_at
           from image where id=$1"#,
        id
    )
    .fetch_one(&ctx.db)
    .await?;

    Ok(rec)
}

pub async fn db_rm(ctx: &WebContext, id: i32) -> anyhow::Result<bool> {
    let rows = sqlx::query!(
        r#"delete from image
           where id=$1"#,
        id
    )
    .execute(&ctx.db)
    .await?
    .rows_affected();

    Ok(rows == 1)
}
