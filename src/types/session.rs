use super::{AuthProvider, BotType};

#[derive(Debug, sqlx::FromRow, serde::Serialize, serde::Deserialize, Clone, PartialEq)]
pub struct Session {
    pub provider: AuthProvider,
    pub bot_type: BotType,
    pub access_token: String,
}
