//! Runs the live device-code auth chain and prints the signed-in profile.
//!
//! The example prints the gamertag and the gamerscore. The token is not
//! printed or written.

use xbl::auth::Auth;
use xbl::client::Client;

#[tokio::main]
async fn main() -> Result<(), xbl::Error> {
    let auth = Auth::new();
    let code = auth.request_device_code().await?;

    println!("Open {}", code.verification_uri());
    println!("Enter code {}", code.user_code());
    println!("Waiting for sign-in...");

    let live = auth.poll_device_code(&code).await?;
    let token = auth.authenticate(&live).await?;
    let profile = Client::new(token).profile().await?;

    println!(
        "gamertag: {}",
        profile.gamertag().unwrap_or("<not returned>")
    );
    println!(
        "gamerscore: {}",
        profile.gamerscore().unwrap_or("<not returned>")
    );

    Ok(())
}
