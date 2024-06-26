use bot_service::BotService;
use shuttle_runtime::SecretStore;
use sqlx::PgPool;

mod aero_api;
mod bot_service;
mod checker_job;
mod x_api;

#[shuttle_runtime::main]
async fn shuttle_main(
    #[shuttle_runtime::Secrets] secrets: SecretStore,
    #[shuttle_shared_db::Postgres] pool: PgPool,
) -> Result<BotService, shuttle_runtime::Error> {
    Ok(BotService { secrets, pool })
}
