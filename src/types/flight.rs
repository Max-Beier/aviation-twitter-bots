#[derive(Debug, sqlx::FromRow, serde::Serialize, serde::Deserialize, Clone, PartialEq)]
pub struct Flight {
    pub ident: String,
    pub altitude: i32,
    pub groundspeed: i32,
    pub destination: String,
    pub origin: String,
}
