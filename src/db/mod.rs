#[derive(serde::Serialize, serde::Deserialize, sqlx::FromRow, Debug)]
pub struct Total {
    pub total: i64,
}

pub fn check_none(str: &String) -> Option<String> {
    if str.is_empty() {
        None
    } else {
        Some(str.to_string())
    }
}
