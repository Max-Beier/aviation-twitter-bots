use reqwest::{Client, Error};
use serde_json::Value;

use crate::types::{BotType, Flight};

pub struct AeroApi {
    client: Client,
    url: String,
    api_key: String,
}

impl AeroApi {
    pub fn new(api_key: String) -> Self {
        let url = "https://aeroapi.flightaware.com/aeroapi/".to_string();
        Self {
            client: Client::new(),
            url,
            api_key,
        }
    }

    pub async fn get_flights_above_fl(
        &self,
        fl: u32,
        filter: &Vec<&str>,
    ) -> Result<Vec<Flight>, Error> {
        let params = format!("-aboveAltitude {}", fl);

        let response = self
            .client
            .get(format!("{}/flights/search?query={}", &self.url, params))
            .header("x-apikey", &self.api_key)
            .send()
            .await?;

        let response_json = response.json::<Value>().await?;

        let flights = match response_json.get("flights") {
            Some(flights) => flights,
            None => return Ok(vec![]),
        };

        let flights: Vec<Flight> = flights
            .as_array()
            .unwrap()
            .iter()
            .filter(|value| {
                if let Some(ident) = value.get("ident").and_then(|v| v.as_str()) {
                    !filter.iter().any(|&f| ident.starts_with(f))
                } else {
                    false
                }
            })
            .map(|value| {
                let ident = value.get("ident").unwrap().as_str().unwrap().to_string();
                let altitude = value
                    .get("last_position")
                    .unwrap()
                    .get("altitude")
                    .unwrap()
                    .as_i64()
                    .map(|i| i as i32);
                let groundspeed = value
                    .get("last_position")
                    .unwrap()
                    .get("groundspeed")
                    .unwrap()
                    .as_i64()
                    .map(|i| i as i32);

                let origin = value.get("origin").and_then(|origin| {
                    let name = origin.get("name")?.as_str()?;
                    let city = origin.get("city")?.as_str()?;
                    let code_icao = origin.get("code_icao")?.as_str()?;
                    Some(format!("{}, {} [{}]", name, city, code_icao))
                });

                let destination = value.get("destination").and_then(|destination| {
                    let name = destination.get("name")?.as_str()?;
                    let city = destination.get("city")?.as_str()?;
                    let code_icao = destination.get("code_icao")?.as_str()?;
                    Some(format!("{}, {} [{}]", name, city, code_icao))
                });

                Flight {
                    ident,
                    ranking: BotType::ALTITUDE,
                    altitude,
                    groundspeed,
                    origin,
                    destination,
                }
            })
            .collect();

        Ok(flights)
    }

    pub async fn get_flights_above_gspd(&self, gspd: u32) -> Result<Vec<Flight>, Error> {
        let params = format!("-aboveGroundspeed {}", gspd);

        let response = self
            .client
            .get(format!("{}/flights/search?query={}", &self.url, params))
            .header("x-apikey", &self.api_key)
            .send()
            .await?;

        let response_json = response.json::<Value>().await?;

        let flights = match response_json.get("flights") {
            Some(flights) => flights,
            None => return Ok(vec![]),
        };

        let flights: Vec<Flight> = flights
            .as_array()
            .unwrap()
            .iter()
            .map(|value| {
                let ident = value.get("ident").unwrap().as_str().unwrap().to_string();
                let altitude = value
                    .get("last_position")
                    .unwrap()
                    .get("altitude")
                    .unwrap()
                    .as_i64()
                    .map(|i| i as i32);
                let groundspeed = value
                    .get("last_position")
                    .unwrap()
                    .get("groundspeed")
                    .unwrap()
                    .as_i64()
                    .map(|i| i as i32);

                let origin = value.get("origin").and_then(|origin| {
                    let name = origin.get("name")?.as_str()?;
                    let city = origin.get("city")?.as_str()?;
                    let code_icao = origin.get("code_icao")?.as_str()?;
                    Some(format!("{}, {} [{}]", name, city, code_icao))
                });

                let destination = value.get("destination").and_then(|destination| {
                    let name = destination.get("name")?.as_str()?;
                    let city = destination.get("city")?.as_str()?;
                    let code_icao = destination.get("code_icao")?.as_str()?;
                    Some(format!("{}, {} [{}]", name, city, code_icao))
                });

                Flight {
                    ident,
                    ranking: BotType::GROUNDSPEED,
                    altitude,
                    groundspeed,
                    origin,
                    destination,
                }
            })
            .collect();

        Ok(flights)
    }
}
