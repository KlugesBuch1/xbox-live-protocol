//! Runs the live device-code auth chain.
//!
//! The example prints the user code, the verification URL, the user hash and
//! the first 12 characters of the Xbox token.

use xbl::auth::Auth;

#[tokio::main]
async fn main() -> Result<(), xbl::auth::AuthError> {
    let auth = Auth::new();
    let code = auth.request_device_code().await?;

    println!("Open {}", code.verification_uri());
    println!("Enter code {}", code.user_code());
    println!("Waiting for sign-in...");

    let live = auth.poll_device_code(&code).await?;
    let xbox = auth.authenticate(&live).await?;
    let token = xbox.token();
    let prefix: String = token.chars().take(12).collect();

    println!("uhs: {}", xbox.uhs());
    if token.chars().count() > 12 {
        println!("token: {prefix}...");
    } else {
        println!("token: <shorter than 12 characters, not shown>");
    }

    Ok(())
}
