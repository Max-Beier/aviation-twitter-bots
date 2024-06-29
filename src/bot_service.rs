use std::str::FromStr;

use apalis::{
    cron::{CronStream, Schedule},
    postgres::PostgresStorage,
    prelude::{Data, Monitor, WorkerBuilder, WorkerFactoryFn},
    utils::TokioExecutor,
};

use chrono::Utc;
use shuttle_runtime::{tokio, SecretStore};
use sqlx::PgPool;

use crate::{
    altitude_bot::{altitude_job, Checker as AltChecker},
    groundspeed_bot::{groundspeed_job, Checker as GspdChecker},
};

pub struct BotService {
    pub secrets: SecretStore,
    pub pool: PgPool,
}

#[shuttle_runtime::async_trait]
impl shuttle_runtime::Service for BotService {
    async fn bind(self, _addr: std::net::SocketAddr) -> Result<(), shuttle_runtime::Error> {
        let alt_storage: PostgresStorage<AltChecker> = PostgresStorage::new(self.pool.clone());
        let schedule = Schedule::from_str("0 0 */16 ? * * *").expect("Couldn't start scheduler.");

        let alt_worker = WorkerBuilder::new("cron-worker")
            .with_storage(alt_storage.clone())
            .stream(CronStream::new(schedule.clone()).into_stream())
            .data((alt_storage.clone(), self.secrets.clone()))
            .build_fn(altitude_job);

        let alt_monitor = Monitor::<TokioExecutor>::new().register(alt_worker);

        let gspd_storage: PostgresStorage<GspdChecker> = PostgresStorage::new(self.pool.clone());
        let gspd_worker = WorkerBuilder::new("cron-worker")
            .with_storage(gspd_storage.clone())
            .stream(CronStream::new(schedule).into_stream())
            .data((gspd_storage.clone(), self.secrets.clone()))
            .build_fn(groundspeed_job);

        let gspd_monitor = Monitor::<TokioExecutor>::new().register(gspd_worker);

        let initial_altitude_job = altitude_job(
            AltChecker::from(Utc::now()),
            Data::new((alt_storage.clone(), self.secrets.clone())),
        );

        let initial_groundspeed_job = groundspeed_job(
            GspdChecker::from(Utc::now()),
            Data::new((gspd_storage.clone(), self.secrets.clone())),
        );

        // Run initial jobs concurrently
        let (alt_result, gspd_result) = tokio::join!(initial_altitude_job, initial_groundspeed_job);

        if let Err(e) = alt_result {
            eprintln!("Initial altitude job failed: {:?}", e);
        }

        if let Err(e) = gspd_result {
            eprintln!("Initial groundspeed job failed: {:?}", e);
        }

        // Run monitors concurrently
        tokio::select! {
            _ = alt_monitor.run() => {
                eprintln!("Altitude monitor stopped.");
            },
            _ = gspd_monitor.run() => {
                eprintln!("Groundspeed monitor stopped.");
            },
        }

        Ok(())
    }
}
