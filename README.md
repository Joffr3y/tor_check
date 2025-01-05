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
    let client = ureq::AgentBuilder::new()
        .proxy(ureq::Proxy::new("socks5://127.0.0.1:9050")?)
        .build()
        .tor_check()?;

    // Ok, I am connected to Tor.
    client.get("https://example.com/").call()?;

    Ok(())
}
```

## Troubles and error handling

The check depends on [TorButton](https://check.torproject.org/?TorButton=True) Web page result.  
If you suspect a Web page update or an issue in the parsing process, you can obtain debug information with the `log` feature.

```rust,no_run
use tor_check::{TorCheck, TorCheckError as Error};

fn main() -> Result<(), Error<ureq::Error>> {
    // Enable logger output
    env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .init();

    // Print the check result
    match ureq::Agent::new().tor_check() {
        Ok(_) => println!("Crongratulation!"),
        Err(Error::HttpClient(err)) => println!("HTTP error: {err}"),
        Err(Error::PageParsing(err)) => println!("I/O error: {err}"),
        Err(err) => println!("Danger! {err}"),
    };

    Ok(())
}
```
