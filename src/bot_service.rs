use std::str::FromStr;

use apalis::{
    cron::{CronStream, Schedule},
    postgres::PostgresStorage,
    prelude::{Monitor, WorkerBuilder, WorkerFactoryFn},
    utils::TokioExecutor,
};

use shuttle_runtime::SecretStore;
use sqlx::PgPool;
use tower::ServiceBuilder;

use crate::checker_job::{checker_job, Checker};

pub struct BotService {
    pub secrets: SecretStore,
    pub pool: PgPool,
}

#[shuttle_runtime::async_trait]
impl shuttle_runtime::Service for BotService {
    async fn bind(self, _addr: std::net::SocketAddr) -> Result<(), shuttle_runtime::Error> {
        let storage: PostgresStorage<Checker> = PostgresStorage::new(self.pool);

        let schedule = Schedule::from_str("0 */10 * ? * * *").expect("Couldn't start scheduler.");
        let service = ServiceBuilder::new().service(checker_job);

        let worker = WorkerBuilder::new("cron-worker")
            .with_storage(storage.clone())
            .stream(CronStream::new(schedule).into_stream())
            .build_fn(service.clone());

        Monitor::<TokioExecutor>::new()
            .register(worker)
            .run()
            .await
            .expect("Unable to start worker.");

        Ok(())
    }
}
