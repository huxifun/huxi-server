use chrono::{DateTime, Utc};
use sqlx::types::Uuid;
use std::str::FromStr;

use crate::fun::message::List;
use crate::fun::user::db::{db_get_user, By};
use crate::http::WebContext;
use crate::md;

#[derive(serde::Serialize, serde::Deserialize, sqlx::FromRow, Debug)]
pub struct Message {
    pub id: Uuid,
    pub user_id: i32,
    pub user_name: String,
    pub to_user_id: i32,
    pub to_user_name: String,
    pub i_type: i16,
    pub title: String,
    pub body: String,
    pub html: String,
    pub i_status: i16,
    pub in_public: i16,
    pub out_public: i16,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(serde::Deserialize, Debug, Default, Clone)]
pub struct Input {
    pub title: String,
    pub to_user_name: String,
    pub body: String,
}
impl Input {
    pub async fn check(&mut self, ctx: &WebContext) -> std::result::Result<i32, Vec<String>> {
        let mut error: Vec<String> = Vec::new();
        self.title = self.title.trim().to_string();
        self.to_user_name = self.to_user_name.trim().to_string();
        let mut to_user_id: i32 = 0;
        if self.to_user_name.is_empty() {
            error.push("请输入收信人".to_string());
        } else {
            let to_user_search = db_get_user(ctx, By::Name(self.to_user_name.clone())).await;
            if let Ok(u) = to_user_search {
                if u.is_none() {
                    error.push("收信人不存在，请重新输入".to_string());
                } else {
                    to_user_id = u.unwrap().user_id;
                }
            }
        }
        if self.title.is_empty() {
            error.push("请输入标题".to_string());
        }
        if self.body.is_empty() {
            error.push("请输入正文".to_string());
        }
        if !error.is_empty() {
            return Err(error);
        }
        Ok(to_user_id)
    }
}

pub async fn db_new_total(ctx: &WebContext, user_id: i32) -> anyhow::Result<i64> {
    let rec = sqlx::query!(
        r#"select count(*) as total from message
                   where in_public=1 and i_status=0 and to_user_id=$1"#,
        user_id
    )
    .fetch_one(&ctx.db)
    .await?;
    Ok(rec.total.unwrap())
}

#[derive(serde::Serialize, serde::Deserialize, sqlx::FromRow, Debug)]
pub struct MessageSimple {
    pub id: Uuid,
    pub title: String,
    pub i_status: i16,
    pub user_name: String,
    pub to_user_name: String,
    pub created_at: DateTime<Utc>,
}
#[derive(serde::Serialize, serde::Deserialize, sqlx::FromRow, Debug)]
pub struct DbTotal {
    pub total: i64,
}

type UserID = i32;

#[derive(Clone)]
pub enum ListBy {
    UserInbox(UserID),
    UserOutbox(UserID),
}

impl ListBy {
    pub fn is_inbox(&self) -> bool {
        matches!(self, ListBy::UserInbox(_))
    }
    pub fn is_outbox(&self) -> bool {
        matches!(self, ListBy::UserOutbox(_))
    }
}

pub async fn db_list<'a>(list: &List<'a>) -> anyhow::Result<(i64, Vec<MessageSimple>)> {
    let offset = list.size as u32 * (list.page - 1);
    let s_title = list.title_search_key.map_or("".to_string(), |k| {
        format!("and lower(title) like '%{}%' ", k.to_lowercase())
    });
    let sql_where = match list.filter {
        ListBy::UserInbox(id) => format!("where to_user_id={id} and in_public=1 {}", s_title),
        ListBy::UserOutbox(id) => format!("where user_id={id} and out_public=1 {}", s_title),
    };
    let sql_total = format!("select count(*) as total from message {};", &sql_where);
    let row = sqlx::query_as::<_, DbTotal>(&sql_total)
        .fetch_one(&list.ctx.db)
        .await?;
    let total = row.total;
    let sql = format!(
        r#"
            select
                id, title, i_status, created_at, user_name, to_user_name
            from message
            {}
            order by created_at desc limit {} offset {};"#,
        sql_where, list.size, offset
    );
    let rows = sqlx::query_as::<_, MessageSimple>(&sql)
        .fetch_all(&list.ctx.db)
        .await?;
    Ok((total, rows))
}

pub async fn db_insert(
    ctx: &WebContext,
    user_id: i32,
    from: &str,
    input: &Input,
    to_user_id: i32,
) -> anyhow::Result<Uuid> {
    let html = md::to_html(input.body.as_str());

    let rec = sqlx::query!(
        r#"insert into message
             (user_id, user_name, to_user_id, to_user_name, title, body, html)
           values ($1, $2, $3, $4, $5, $6, $7)
           returning id"#,
        user_id,
        from,
        to_user_id,
        input.to_user_name,
        input.title,
        input.body,
        html
    )
    .fetch_one(&ctx.db)
    .await?;
    Ok(rec.id)
}

pub async fn db_get_one(ctx: &WebContext, id: &str) -> anyhow::Result<Message> {
    let uuid = Uuid::from_str(id)?;
    let rec = sqlx::query_as!(
        Message,
        r#"select id, user_id, user_name, to_user_id, to_user_name, i_type, title, body, html, i_status, 
               in_public, out_public, created_at, updated_at
           from message where id=$1"#,
        uuid
    )
    .fetch_one(&ctx.db)
    .await?;

    Ok(rec)
}

pub async fn db_update_status(ctx: &WebContext, id: String) -> anyhow::Result<bool> {
    let uid = Uuid::from_str(id.as_str())?;

    let rows = sqlx::query!(
        r#"update message
           set i_status=1 
           where id=$1"#,
        uid
    )
    .execute(&ctx.db)
    .await?
    .rows_affected();

    Ok(rows > 0)
}

pub async fn db_rm(ctx: &WebContext, id: &str) -> anyhow::Result<bool> {
    let uid = Uuid::from_str(id)?;
    let rows = sqlx::query!(
        r#"update message set in_public=0
           where id=$1"#,
        uid
    )
    .execute(&ctx.db)
    .await?
    .rows_affected();

    Ok(rows == 1)
}
