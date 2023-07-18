use crate::http::{error::AppError, Result, WebContext};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2,
};

use chrono::{DateTime, Utc};
use sqlx::types::Uuid;

#[derive(serde::Serialize, serde::Deserialize, sqlx::FromRow, Debug)]
pub struct User {
    pub user_id: i32,
    pub name: String,
    pub email: String,
    pub password: String,
    pub i_role: i16,
}

#[derive(serde::Deserialize, Debug, Default)]
pub struct RegInput {
    pub name: String,
    pub email: String,
    pub password: String,
    pub password2: String,
    pub validate: String,
}
impl RegInput {
    pub fn trim(&mut self) {
        self.name = self.name.trim().to_string();
        self.email = self.email.trim().to_string();
        self.password = self.password.trim().to_string();
        self.password2 = self.password2.trim().to_string();
    }
}

pub async fn db_add_user(ctx: &WebContext, input: RegInput) -> anyhow::Result<Uuid> {
    let pw = hash_password(input.password).await?;
    let rec = sqlx::query_scalar!(
        r#"insert into users
             (name, email, password)
           values ($1, $2, $3)
           returning uid"#,
        input.name,
        input.email,
        pw,
    )
    .fetch_one(&ctx.db)
    .await?;
    Ok(rec)
}

#[derive(Debug)]
pub enum By {
    Email(String),
    Name(String),
    Id(i32),
}

pub async fn db_get_user(ctx: &WebContext, by: By) -> anyhow::Result<Option<User>> {
    let filter = match by {
        By::Email(s) => format!("email='{}'", s),
        By::Name(s) => format!("name='{}'", s),
        By::Id(i) => format!("user_id={}", i),
    };
    let sql = format!(
        r#"select user_id, name, email, password, i_role
           from users
           where {} and i_role > 0;"#,
        filter
    );
    let rec = sqlx::query_as::<_, User>(&sql)
        .fetch_optional(&ctx.db)
        .await?;
    Ok(rec)
}

pub async fn db_add_pw_reset(ctx: &WebContext, user: &User) -> anyhow::Result<Uuid> {
    let rec = sqlx::query!(
        r#"insert into reset_pw_req
             (user_id, user_name, user_email)
           values ($1, $2, $3)
           returning id"#,
        user.user_id,
        user.name,
        user.email,
    )
    .fetch_one(&ctx.db)
    .await?;
    Ok(rec.id)
}

#[derive(serde::Serialize, serde::Deserialize, sqlx::FromRow, Debug)]
pub struct PwReq {
    pub created_at: DateTime<Utc>,
    pub user_name: String,
    pub user_id: i32,
}

pub async fn pw2_db_get_pwreq_by_id(ctx: &WebContext, id: &str) -> anyhow::Result<PwReq> {
    let rec = sqlx::query_as!(
        PwReq,
        r#"select created_at, user_name, user_id
           from reset_pw_req where id=$1"#,
        Uuid::parse_str(&id).unwrap()
    )
    .fetch_one(&ctx.db)
    .await?;
    Ok(rec)
}

pub async fn db_update_user_pw(ctx: &WebContext, id: i32, pw: String) -> anyhow::Result<bool> {
    let pw = hash_password(pw).await?;
    let rows_affected = sqlx::query!(
        r#"update users
           set password=$1
           where user_id=$2"#,
        pw,
        id,
    )
    .execute(&ctx.db)
    .await?
    .rows_affected();

    Ok(rows_affected > 0)
}

pub async fn db_update_pw_reset(ctx: &WebContext, id: &str) -> anyhow::Result<bool> {
    let rows_affected = sqlx::query!(
        r#"update reset_pw_req
           set i_status = 1
           where id = $1"#,
        Uuid::parse_str(id).unwrap()
    )
    .execute(&ctx.db)
    .await?
    .rows_affected();

    Ok(rows_affected > 0)
}

#[derive(serde::Serialize, serde::Deserialize, sqlx::FromRow, Debug)]
pub struct UserSimple {
    pub id: i32,
    pub name: String,
    pub email: String,
    pub i_role: i16,
}

pub async fn _db_user_list(ctx: &WebContext) -> anyhow::Result<Vec<UserSimple>> {
    let rows = sqlx::query_as!(
        UserSimple,
        r#"
            select
                user_id as id, name, email, i_role
            from users 
            order by user_id desc"#,
    )
    .fetch_all(&ctx.db)
    .await?;

    Ok(rows)
}

/// hash 密码
async fn hash_password(password: String) -> Result<String> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    Ok(argon2
        .hash_password(password.as_str().as_bytes(), &salt)
        .map_err(|_| AppError::InvalidArg("error password".to_owned()))?
        .to_string())
}
