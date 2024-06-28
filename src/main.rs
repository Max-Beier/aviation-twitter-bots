use bot_service::BotService;
use shuttle_runtime::SecretStore;
use sqlx::PgPool;

mod apis;
mod bot_service;
mod checker_job;
mod types;

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
