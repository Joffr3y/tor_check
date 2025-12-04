#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/README.md"))]

use std::io::{self, BufRead};
use std::{error, fmt};
#[cfg(feature = "reqwest")]
use std::{future::Future, pin::Pin};

/// TorButton Web page URL.
const TOR_CHECK_URL: &str = "https://check.torproject.org/?TorButton=True";
/// Content to find on success.
const TOR_CHECK_SUCCESS_RESULT: &str = concat!(
    r#"<a id="TorCheckResult" target="success" href="/">"#,
    r#"</a>"#,
);

/// `Result` alias for blocking HTTP client.
type Result<T, E> = std::result::Result<T, TorCheckError<E>>;
/// `Result` alias for asynchronous HTTP client.
#[cfg(feature = "reqwest")]
type FutureResult<T, E> = Pin<Box<dyn Future<Output = Result<T, E>>>>;

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
#[derive(Debug)]
pub enum TorCheckError<E> {
    /// Error returned by the HTTP client.
    HttpClient(E),
    /// Error occurred in the page parsing.
    PageParsing(io::Error),
    /// The check page indicate that you are not using Tor.
    YouAreNotUsingTor,
}

impl<E: error::Error> fmt::Display for TorCheckError<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::HttpClient(err) => write!(f, "{err}"),
            Self::PageParsing(err) => write!(f, "{err}"),
            Self::YouAreNotUsingTor => write!(f, "You are not using Tor"),
        }
    }
}

impl<E: error::Error> error::Error for TorCheckError<E> {}

impl<T: error::Error> From<io::Error> for TorCheckError<T> {
    fn from(err: io::Error) -> TorCheckError<T> {
        Self::PageParsing(err)
    }
}

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

/// Parse the Tor check Web page.
#[inline]
fn tor_check_result<T, B, E>(client: T, reader: B) -> Result<T, E>
where
    B: BufRead,
    E: error::Error,
{
    for line in reader.lines() {
        let line = line?;

        #[cfg(feature = "log")]
        log::debug!("{line}");

        if line.trim() == TOR_CHECK_SUCCESS_RESULT {
            return Ok(client);
        }
    }

    Err(TorCheckError::YouAreNotUsingTor)
}

#[cfg(feature = "ureq")]
impl TorCheck for ureq::Agent {
    type Result = Result<Self, ureq::Error>;

    fn tor_check(self) -> Self::Result {
        let resp = self.get(TOR_CHECK_URL).call()?;
        let body = resp.into_body();
        let reader = body.into_reader();

        tor_check_result(self, io::BufReader::new(reader))
    }
}

#[cfg(feature = "reqwest")]
impl TorCheck for reqwest::Client {
    type Result = FutureResult<Self, reqwest::Error>;

    fn tor_check(self) -> Self::Result {
        Box::pin(async move {
            let resp = self.get(TOR_CHECK_URL).send().await?.bytes().await?;

            tor_check_result(self, io::Cursor::new(resp))
        })
    }
}

#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::io::BufReader;

    use super::{TorCheckError as Error, *};

    #[derive(Debug, PartialEq)]
    struct MockError;

    impl fmt::Display for MockError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "Mock error")
        }
    }

    impl error::Error for MockError {}

    impl PartialEq for TorCheckError<MockError> {
        fn eq(&self, other: &Self) -> bool {
            match (self, other) {
                (Self::YouAreNotUsingTor, Self::YouAreNotUsingTor) => true,
                (Self::HttpClient(a), Self::HttpClient(b)) => a == b,
                (Self::PageParsing(_), Self::PageParsing(_)) => {
                    unimplemented!("io::Error doesn't implement PartialEq");
                }
                (_, _) => false,
            }
        }
    }

    #[test]
    fn test_tor_check_result() {
        let fn_test = tor_check_result::<(), BufReader<File>, MockError>;
        let failure = BufReader::new(File::open("tests/failure.html").unwrap());
        let success = BufReader::new(File::open("tests/success.html").unwrap());

        assert_eq!(fn_test((), failure), Err(Error::YouAreNotUsingTor));
        assert_eq!(fn_test((), success), Ok(()));
    }
}
