use apalis::prelude::Job;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use shuttle_runtime::Error;

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct Checker(DateTime<Utc>);

impl From<DateTime<Utc>> for Checker {
    fn from(date_time: DateTime<Utc>) -> Self {
        Self(date_time)
    }
}

impl Job for Checker {
    const NAME: &'static str = "checker::CheckerJob";
}

pub async fn checker_job(job: Checker) -> Result<(), Error> {
    println!("{:?}", job.0.time());

    Ok(())
}
