use std::{fmt, num::ParseFloatError};

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

// 定义自定义错误类型
#[derive(Debug)]
pub enum KVParseError {
    ParseError(ParseFloatError),
    InvalidType(String),
}

impl fmt::Display for KVParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            KVParseError::ParseError(e) => write!(f, "Failed to parse float: {}", e),
            KVParseError::InvalidType(ty) => write!(f, "Unsupported type for conversion: {}", ty),
        }
    }
}
error_chain! {
    errors {
        ExchangeError(response: ExchangeContentError)

        KlineValueMissingError(index: usize, name: &'static str) {
            description("invalid Vec for Kline"),
            display("{} at {} is missing", name, index),
        }
        
        KlineValueParseError(parse_error: KVParseError)

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
