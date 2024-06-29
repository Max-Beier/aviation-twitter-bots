use apalis::{
    postgres::PostgresStorage,
    prelude::{Data, Job},
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use shuttle_runtime::{Error, SecretStore};

use crate::{
    apis::{AeroApi, XApi},
    types::{BotType, Flight},
    utils::{format_tweet, FormatOrder},
};

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct Checker {
    pub time: DateTime<Utc>,
}

impl From<DateTime<Utc>> for Checker {
    fn from(time: DateTime<Utc>) -> Self {
        Self { time }
    }
}

impl Job for Checker {
    const NAME: &'static str = "checker::CheckerJob";
}

pub async fn groundspeed_job(
    _job: Checker,
    data: Data<(PostgresStorage<Checker>, SecretStore)>,
) -> Result<(), Error> {
    // Need for first time oauth - first run
    // sleep(Duration::from_secs(60)).await;

    let pool = data.0.pool();
    let secrets = &data.1;

    let aero_api = AeroApi::new(secrets.get("AERO_API_KEY").unwrap());
    let x_api = XApi::new_and_authorize(
        secrets.get("X_GSPD_CLIENT_ID").unwrap(),
        secrets.get("X_GSPD_CLIENT_SECRET").unwrap(),
        BotType::GROUNDSPEED,
        &pool,
    )
    .await;

    let ranking_count = 3;

    let mut flights: Vec<Flight> = vec![];
    let mut search_gspd = 650;

    while flights.len() < ranking_count {
        flights = aero_api.get_flights_above_gspd(search_gspd).await.unwrap();

        search_gspd -= 10;
    }

    flights.sort_by_key(|f| f.groundspeed);
    flights.truncate(3);

    let mut db_flights: Vec<Flight> =
        sqlx::query_as("SELECT * FROM Flights WHERE Flights.ranking = 'GROUNDSPEED';")
            .fetch_all(pool)
            .await
            .unwrap();

    if !db_flights.is_empty() {
        db_flights.sort_by_key(|f| f.groundspeed);

        if db_flights.first().unwrap().ident == flights.first().unwrap().ident {
            return Ok(());
        }

        sqlx::query("DELETE FROM Flights WHERE Flights.ranking = 'GROUNDSPEED';")
            .execute(pool)
            .await
            .unwrap();
    }

    for f in &flights {
        sqlx::query("INSERT INTO Flights (ident, ranking, altitude, groundspeed, origin, destination) VALUES ($1, $2, $3, $4, $5, $6)")
            .bind(&f.ident)
            .bind(&f.ranking)
            .bind(&f.altitude)
            .bind(&f.groundspeed)
            .bind(&f.origin)
            .bind(&f.destination)
            .execute(pool)
            .await
            .unwrap();
    }

    let flight = flights.first().unwrap();

    x_api
        .tweet(format_tweet(flight, FormatOrder::GROUNDSPEED))
        .await;

    Ok(())
}
