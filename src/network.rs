use crate::{log, utils::Result};
use reqwest::StatusCode;

pub fn send_get_request<T: serde::de::DeserializeOwned>(
    client: &reqwest::blocking::Client,
    url: &String,
    token: Option<&String>,
) -> Result<T> {
    let mut request = client.get(url);
    if token.is_some() {
        request = request.bearer_auth(token.unwrap());
    }

    match request.send() {
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
    url: &String,
    body: &T,
    token: Option<&String>,
) -> Result<R> {
    let mut request = client.post(url).json(&body);
    if token.is_some() {
        request = request.bearer_auth(token.unwrap());
    }

    match request.send() {
        Ok(resp) => {
            if resp.status() != StatusCode::CREATED && resp.status() != StatusCode::OK {
                return Err(format!(
                    "Request to {} returned an unexpected status code: {}",
                    url,
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
