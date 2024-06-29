use reqwest::{Client, Error};
use serde_json::Value;

use crate::types::Flight;

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
            .await
            .unwrap();

        let response_json = response.json::<Value>().await.unwrap();

        let flights: Vec<Flight> = response_json
            .get("flights")
            .unwrap()
            .as_array()
            .unwrap()
            .iter()
            .filter(|value| {
                let ident = value.get("ident").unwrap().as_str().unwrap().to_string();
                for f in filter {
                    if ident.starts_with(f) {
                        return false;
                    }
                }

                true
            })
            .map(|value| Flight {
                ident: value.get("ident").unwrap().as_str().unwrap().to_string(),
                altitude: value
                    .get("last_position")
                    .unwrap()
                    .get("altitude")
                    .unwrap()
                    .as_i64()
                    .unwrap() as i32,
                groundspeed: value
                    .get("last_position")
                    .unwrap()
                    .get("groundspeed")
                    .unwrap()
                    .as_i64()
                    .unwrap() as i32,
                origin: format!(
                    "{}, {} [{}]",
                    value
                        .get("origin")
                        .unwrap()
                        .get("name")
                        .unwrap()
                        .as_str()
                        .unwrap()
                        .to_string(),
                    value
                        .get("origin")
                        .unwrap()
                        .get("city")
                        .unwrap()
                        .as_str()
                        .unwrap()
                        .to_string(),
                    value
                        .get("origin")
                        .unwrap()
                        .get("code_icao")
                        .unwrap()
                        .as_str()
                        .unwrap()
                        .to_string()
                ),
                destination: format!(
                    "{}, {} [{}]",
                    value
                        .get("destination")
                        .unwrap()
                        .get("name")
                        .unwrap()
                        .as_str()
                        .unwrap()
                        .to_string(),
                    value
                        .get("destination")
                        .unwrap()
                        .get("city")
                        .unwrap()
                        .as_str()
                        .unwrap()
                        .to_string(),
                    value
                        .get("destination")
                        .unwrap()
                        .get("code_icao")
                        .unwrap()
                        .as_str()
                        .unwrap()
                        .to_string()
                ),
            })
            .collect();

        Ok(flights)
    }
}
