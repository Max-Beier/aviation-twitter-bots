use apalis::{
    postgres::PostgresStorage,
    prelude::{Data, Job},
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use shuttle_runtime::{Error, SecretStore};

use crate::{
    apis::{AeroApi, XApi},
    types::Flight,
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

pub async fn checker_job(
    _job: Checker,
    data: Data<(PostgresStorage<Checker>, SecretStore)>,
) -> Result<(), Error> {
    let pool = data.0.pool();
    let secrets = &data.1;

    let aero_api = AeroApi::new(secrets.get("AERO_API_KEY").unwrap());
    let x_api = XApi::new_and_authorize(
        secrets.get("X_CLIENT_ID").unwrap(),
        secrets.get("X_CLIENT_SECRET").unwrap(),
        &pool,
    )
    .await;

    let ranking_count = 3;

    let filter = vec!["HBAL"];
    let mut flights: Vec<Flight> = vec![];
    let mut search_alt = 500;

    while flights.len() < ranking_count {
        flights = aero_api
            .get_flights_above_fl(search_alt, &filter)
            .await
            .unwrap();

        search_alt -= 10;
    }

    flights.sort_by_key(|f| f.altitude);
    flights.truncate(3);

    let mut db_flights: Vec<Flight> = sqlx::query_as("SELECT * FROM Flights;")
        .fetch_all(pool)
        .await
        .unwrap();

    if !db_flights.is_empty() {
        db_flights.sort_by_key(|f| f.altitude);

        if db_flights.first().unwrap() == flights.first().unwrap() {
            return Ok(());
        }

        sqlx::query("TRUNCATE TABLE Flights;")
            .execute(pool)
            .await
            .unwrap();
    }

    for f in &flights {
        sqlx::query("INSERT INTO Flights (ident, altitude) VALUES ($1, $2)")
            .bind(&f.ident)
            .bind(f.altitude)
            .execute(pool)
            .await
            .unwrap();
    }

    let flight = flights.first().unwrap();
    let link = format!("https://www.flightaware.com/live/flight/{}", flight.ident);

    let alt_readout;
    let spd_readout;

    if let Some(alt_fl) = flight.altitude {
        let alt_feet = alt_fl * 100;
        let alt_meters = alt_feet as f32 * 0.3048;
        alt_readout = format!("{}ft ({:.2}m)", alt_feet, alt_meters);
    } else {
        alt_readout = "N/A".to_string();
    }

    if let Some(spd_knots) = flight.groundspeed {
        let spd_kmh = spd_knots as f32 * 1.852;
        spd_readout = format!("{}kts ({:.2}km/h)", spd_knots, spd_kmh);
    } else {
        spd_readout = "N/A".to_string();
    }

    let origin = flight.origin.clone().unwrap_or("Unknown".to_string());
    let destination = flight.destination.clone().unwrap_or("Unknown".to_string());

    x_api
        .tweet(format!(
            "Current highest flight: {}\n\
            Altitude: {}\n\
            Groundspeed: {}\n\
            Origin: {}\n\
            Destination: {}\n\n\
            More info:\n{}",
            flight.ident, alt_readout, spd_readout, origin, destination, link
        ))
        .await;

    Ok(())
}
