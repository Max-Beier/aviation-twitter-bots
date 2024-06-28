use super::AuthProvider;

#[derive(Debug, sqlx::FromRow, serde::Serialize, serde::Deserialize, Clone, PartialEq)]
pub struct Session {
    pub provider: AuthProvider,
    pub access_token: String,
}
