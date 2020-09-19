use crate::{config, log, utils::Result};
use reqwest::StatusCode;

pub fn send_get_request<T: serde::de::DeserializeOwned>(
    client: &reqwest::blocking::Client,
    endpoint: &String,
) -> Result<T> {
    let url = format!("{}{}", config::get_server_endpoint()?, endpoint);
    match client.get(&url).send() {
        Ok(resp) => {
            if resp.status() != StatusCode::OK {
                return Err(format!(
                    "Request returned an unexpected status code: {}",
                    resp.status()
                ));
            }

            match resp.json::<T>() {
                Ok(obj) => Ok(obj),
                Err(e) => {
                    log::println(format!("network: {}", e));
                    Err("Failed to decode json response".to_string())
                }
            }
        }
        Err(_) => Err("Failed to send request".to_string()),
    }
}

pub fn send_post_request<T: serde::Serialize, R: serde::de::DeserializeOwned>(
    client: &reqwest::blocking::Client,
    endpoint: &String,
    body: &T,
) -> Result<R> {
    let url = format!("{}{}", config::get_server_endpoint()?, endpoint);
    match client.post(&url).json(&body).send() {
        Ok(resp) => {
            if resp.status() != StatusCode::CREATED {
                return Err(format!(
                    "Request returned an unexpected status code: {}",
                    resp.status()
                ));
            }

            match resp.json::<R>() {
                Ok(obj) => Ok(obj),
                Err(e) => {
                    log::println(format!("network: {}", e));
                    Err("Failed to decode json response".to_string())
                }
            }
        }
        Err(e) => Err(format!("Failed to send request: {}", e)),
    }
}
