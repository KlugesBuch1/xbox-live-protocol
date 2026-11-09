# xbl

Xbox Live authentication and proof-of-possession signing for Rust.

The crate implements the device-code sign-in chain used by the Xbox app and the
two signed requests behind it, `device/authenticate` and `sisu/authorize`. The
result is an `XBL3.0` authorization header. The API client for the Live
endpoints is in progress.

Unofficial. Not affiliated with Microsoft.

## Install

```toml
[dependencies]
xbl = "0.1"
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
```

## Sign in

```rust,no_run
use xbl::auth::Auth;

#[tokio::main]
async fn main() -> Result<(), xbl::auth::AuthError> {
    let auth = Auth::new();
    let code = auth.request_device_code().await?;

    println!("Open {}", code.verification_uri());
    println!("Enter code {}", code.user_code());

    let live = auth.poll_device_code(&code).await?;
    let xbox = auth.authenticate(&live).await?;

    println!("Authorization: {}", xbox.authorization_header());
    Ok(())
}
```

## What is covered

- Device-code flow (`oauth20_connect.srf`, `oauth20_token.srf`).
- P-256 proof-of-possession signing with the 76-byte header block.
- Refresh-token exchange.
- No token storage.

## Documentation

The protocol notes, including the byte layout and the measured error cases, are
in the repository: <https://github.com/KlugesBuch1/xbox-live-protocol>

## License

MIT.
