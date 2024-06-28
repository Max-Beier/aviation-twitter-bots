#[derive(Debug, sqlx::FromRow, serde::Serialize, serde::Deserialize, Clone, PartialEq)]
pub struct Flight {
    pub ident: String,
    pub altitude: i32,
}
