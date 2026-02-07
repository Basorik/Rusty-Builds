use reqwest::{Client, StatusCode};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PoeClientError {
    #[error("Network request failed")]
    Network(#[from] reqwest::Error),
}

pub struct PoeClient {
    http: Client,
}
