#[derive(Debug, sqlx::FromRow, serde::Serialize, serde::Deserialize, Clone, PartialEq)]
pub struct Flight {
    pub ident: String,
    pub altitude: Option<i32>,
    pub groundspeed: Option<i32>,
    pub destination: Option<String>,
    pub origin: Option<String>,
}
