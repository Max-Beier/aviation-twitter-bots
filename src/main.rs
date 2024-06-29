use bot_service::BotService;
use shuttle_runtime::SecretStore;
use sqlx::PgPool;

mod altitude_bot;
mod apis;
mod bot_service;
mod groundspeed_bot;
mod types;
mod utils;

#[shuttle_runtime::main]
async fn shuttle_main(
    #[shuttle_runtime::Secrets] secrets: SecretStore,
    #[shuttle_shared_db::Postgres] pool: PgPool,
) -> Result<BotService, shuttle_runtime::Error> {
    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    Ok(BotService { secrets, pool })
}
