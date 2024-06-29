use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, PartialOrd, sqlx::Type, Deserialize, Serialize)]
#[sqlx(type_name = "BotType")]
pub enum BotType {
    ALTITUDE,
    GROUNDSPEED,
}
