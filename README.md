# tor_check

[![crates.io](https://img.shields.io/crates/v/tor_check.svg)](https://crates.io/crates/tor_check)
[![Documentation](https://docs.rs/tor-check/badge.svg)](https://docs.rs/tor-check)
[![MIT](https://img.shields.io/crates/l/tor_check.svg)](./LICENSE)
[![CI](https://github.com/joffr3y/tor_check/workflows/CI/badge.svg)](https://github.com/joffr3y/tor_check/actions?query=workflow:CI)

Extend your favorite HTTP client with a Tor verification feature.

## Usage

Configure your client to use the Tor proxy and call `tor_check` before any other requests.

### Reqwest

The `reqwest` feature is required.

```rust,no_run
use tor_check::{TorCheck, TorCheckError};

#[tokio::main]
async fn main() -> Result<(), TorCheckError<reqwest::Error>> {
    let client = reqwest::Client::builder()
        .proxy(reqwest::Proxy::all("socks5://127.0.0.1:9050")?)
        .build()?
        .tor_check()
        .await?;

    // Ok, I am connected to Tor.
    client.get("https://example.com/").send().await?;

    Ok(())
}
```

### Ureq

The `ureq` feature is required.

```rust,no_run
use tor_check::{TorCheck, TorCheckError};

fn main() -> Result<(), TorCheckError<ureq::Error>> {
    let config = ureq::Agent::config_builder()
        .proxy(Some(ureq::Proxy::new("socks5://127.0.0.1:9050")?))
        .build();
    let client = ureq::Agent::from(config).tor_check()?;

    // Ok, I am connected to Tor.
    client.get("https://example.com/").call()?;

    Ok(())
}
```

## Troubles and error handling

This crate use [check.torproject.org](https://check.torproject.org/api/ip)
API result.

Possible errors returned.

```rust,no_run
use tor_check::{TorCheck, TorCheckError as Error};

match ureq::agent().tor_check() {
    Ok(_) => println!("Crongratulation!"),
    Err(err) if err.is_decode() => {
        eprintln!("Malformed response: {err}");
        eprintln!("Check https://check.torproject.org/api/ip");
    }
    Err(Error::HttpClient(err)) => {
        eprintln!("HTTP error: {err}");
        eprintln!("Check Tor services https://status.torproject.org/");
    }
    Err(err @ Error::YouAreNotUsingTor) => eprintln!("Danger! {err}"),
}
```