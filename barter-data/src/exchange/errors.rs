use barter_integration::error::SocketError;
use error_chain::error_chain;
use serde::Deserialize;
use thiserror::Error;

#[derive(Debug, Deserialize)]
pub struct ExchangeContentError {
    pub code: i16,
    pub msg: String,
}

#[derive(Debug, Error)]
pub enum ExecutionError {
    #[error("request authorisation invalid: {0}")]
    Unauthorised(String),

    #[error("SocketError: {0}")]
    Socket(#[from] SocketError),
}

error_chain! {
    errors {
        ExchangeError(response: ExchangeContentError)

        KlineValueMissingError(index: usize, name: &'static str) {
            description("invalid Vec for Kline"),
            display("{} at {} is missing", name, index),
        }

        MarketError(response: ExecutionError)
     }

    foreign_links {
        ReqError(reqwest::Error);
        InvalidHeaderError(reqwest::header::InvalidHeaderValue);
        IoError(std::io::Error);
        ParseFloatError(std::num::ParseFloatError);
        UrlParserError(url::ParseError);
        Json(serde_json::Error);
        // Tungstenite(tungstenite::Error);
        TimestampError(std::time::SystemTimeError);
    }
}
