use crate::http::WebContext;
use crate::md;
use chrono::{DateTime, Utc};

#[derive(serde::Serialize, serde::Deserialize, sqlx::FromRow, Debug)]
pub struct Comment {
    pub id: i32,
    pub user_id: i32,
    pub user_name: String,
    pub obj_id: i32,
    pub i_public: i16,
    pub body: String,
    pub html: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

impl Comment {
    pub fn input(self) -> Input {
        Input {
            id: Some(self.id),
            oid: Some(self.obj_id),
            url: None,
            body: self.body,
        }
    }
}

#[derive(serde::Deserialize, Debug, Default)]
pub struct Input {
    pub id: Option<i32>,
    pub oid: Option<i32>, //obj id
    pub url: Option<String>,
    pub body: String,
}
impl Input {
    pub fn check(&self) -> std::result::Result<(), Vec<String>> {
        let mut error: Vec<String> = Vec::new();
        if self.body.is_empty() {
            error.push("请输入正文".to_string());
        }
        if !error.is_empty() {
            return Err(error);
        }
        Ok(())
    }
}

pub async fn db_insert(
    ctx: &WebContext,
    user_id: i32,
    user_name: &str,
    input: &Input,
    ty: &str,
) -> anyhow::Result<()> {
    let html = md::to_html(input.body.as_str());
    let sql = format!(
        r#"insert into {}_comment
             (user_id, user_name, obj_id, i_public, body, html)
           values ($1, $2, $3, 1, $4, $5)
           returning id"#,
        ty
    );
    let _rec = sqlx::query(sql.as_str())
        .bind(user_id)
        .bind(user_name)
        .bind(input.oid)
        .bind(input.body.as_str())
        .bind(html)
        .fetch_one(&ctx.db)
        .await?;
    Ok(())
}

pub async fn db_update(ctx: &WebContext, id: i32, input: &Input, ty: &str) -> anyhow::Result<bool> {
    let html = md::to_html(&input.body);
    let sql = format!(
        r#"update {}_comment
           set body=$1, html=$2
           where id=$3"#,
        ty
    );
    let rows = sqlx::query(sql.as_str())
        .bind(input.body.as_str())
        .bind(html)
        .bind(id)
        .execute(&ctx.db)
        .await?
        .rows_affected();

    Ok(rows > 0)
}

pub async fn db_get_one(ctx: &WebContext, id: i32, ty: &str) -> anyhow::Result<Comment> {
    let sql = format!(
        r#"select *
           from {}_comment where id=$1"#,
        ty
    );
    let rec = sqlx::query_as::<_, Comment>(sql.as_str())
        .bind(id)
        .fetch_one(&ctx.db)
        .await?;
    Ok(rec)
}

pub async fn db_rm(ctx: &WebContext, id: i32, ty: &str) -> anyhow::Result<bool> {
    let sql = format!(
        r#"delete from {}_comment
           where id=$1"#,
        ty
    );
    let rows = sqlx::query(sql.as_str())
        .bind(id)
        .execute(&ctx.db)
        .await?
        .rows_affected();

    Ok(rows == 1)
}

pub async fn db_hide(ctx: &WebContext, id: i32, ty: &str) -> anyhow::Result<bool> {
    let sql = format!(
        r#"update {}_comment
           set i_public=0
           where id=$1"#,
        ty
    );
    let rows = sqlx::query(sql.as_str())
        .bind(id)
        .execute(&ctx.db)
        .await?
        .rows_affected();

    Ok(rows == 1)
}

pub async fn db_list(ctx: &WebContext, id: i32, ty: &str) -> anyhow::Result<Vec<Comment>> {
    let sql = format!(
        r#"select * from {}_comment where obj_id=$1 and i_public=1 order by id asc"#,
        ty
    );
    let rows = sqlx::query_as::<_, Comment>(sql.as_str())
        .bind(id)
        .fetch_all(&ctx.db)
        .await?;
    Ok(rows)
}
