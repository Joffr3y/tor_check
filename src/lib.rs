#![doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/README.md"))]

use std::{error, fmt};
#[cfg(feature = "reqwest")]
use std::{future::Future, pin::Pin};

use serde::Deserialize;

/// `Result` alias for blocking HTTP client.
type Result<T, E> = std::result::Result<T, TorCheckError<E>>;
/// `Result` alias for asynchronous HTTP client.
#[cfg(feature = "reqwest")]
type FutureResult<T, E> = Pin<Box<dyn Future<Output = Result<T, E>>>>;

/// Tor check response status.
#[derive(Debug, Deserialize)]
struct TorCheckStatus {
    #[serde(rename = "IsTor")]
    is_tor: bool,
}

impl TorCheckStatus {
    const API_URL: &str = "https://check.torproject.org/api/ip";

    fn result<E: error::Error>(&self) -> Result<(), E> {
        if self.is_tor {
            return Ok(());
        }

        Err(TorCheckError::YouAreNotUsingTor)
    }
}

/// Trait for Tor connection checking.
pub trait TorCheck {
    type Result;

    /// Verify if you are correctly connected to Tor.
    ///
    /// Return the HTTP client on success.
    fn tor_check(self) -> Self::Result;
}

/// Potentials errors returned on the Tor verification process.
///
/// Where `E` is the HTTP client error type.
#[derive(Debug, PartialEq)]
pub enum TorCheckError<E> {
    /// Error returned by the HTTP client.
    HttpClient(E),
    /// The check page indicate that you are not using Tor.
    YouAreNotUsingTor,
}

#[cfg(feature = "reqwest")]
impl TorCheckError<reqwest::Error> {
    /// Returns true if the error is related to the JSON response
    /// deserialization.
    pub fn is_decode(&self) -> bool {
        if let Self::HttpClient(err) = self {
            return err.is_decode();
        }

        false
    }
}

#[cfg(feature = "ureq")]
impl TorCheckError<ureq::Error> {
    /// Returns true if the error is related to the JSON response
    /// deserialization.
    pub fn is_decode(&self) -> bool {
        matches!(self, Self::HttpClient(ureq::Error::Json(_)))
    }
}

impl<E: error::Error> fmt::Display for TorCheckError<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::HttpClient(err) => write!(f, "{err}"),
            Self::YouAreNotUsingTor => write!(f, "You are not using Tor"),
        }
    }
}

impl<E: error::Error> error::Error for TorCheckError<E> {}

#[cfg(feature = "ureq")]
impl From<ureq::Error> for TorCheckError<ureq::Error> {
    fn from(err: ureq::Error) -> Self {
        Self::HttpClient(err)
    }
}

#[cfg(feature = "reqwest")]
impl From<reqwest::Error> for TorCheckError<reqwest::Error> {
    fn from(err: reqwest::Error) -> Self {
        Self::HttpClient(err)
    }
}

#[cfg(feature = "ureq")]
impl TorCheck for ureq::Agent {
    type Result = Result<Self, ureq::Error>;

    fn tor_check(self) -> Self::Result {
        self.get(TorCheckStatus::API_URL)
            .call()?
            .body_mut()
            .read_json::<TorCheckStatus>()?
            .result()
            .map(move |_| self)
    }
}

#[cfg(feature = "reqwest")]
impl TorCheck for reqwest::Client {
    type Result = FutureResult<Self, reqwest::Error>;

    fn tor_check(self) -> Self::Result {
        Box::pin(async move {
            self.get(TorCheckStatus::API_URL)
                .send()
                .await?
                .json::<TorCheckStatus>()
                .await?
                .result()
                .map(move |_| self)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{TorCheckError as Error, *};

    #[derive(Debug, PartialEq)]
    struct MockError;

    impl fmt::Display for MockError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "Mock error")
        }
    }

    impl error::Error for MockError {}

    #[test]
    fn test_tor_check_result() {
        let failure = serde_json::from_str::<TorCheckStatus>(
            r#"{"IsTor":false,"IP":"192.0.2.1"}"#,
        )
        .unwrap();
        let success = serde_json::from_str::<TorCheckStatus>(
            r#"{"IsTor":true,"IP":"192.0.2.1"}"#,
        )
        .unwrap();
        let expected_error = Error::YouAreNotUsingTor;

        assert_eq!(failure.result::<MockError>(), Err(expected_error));
        assert_eq!(success.result::<MockError>(), Ok(()));
    }
}
