# Changelog

## 0.2.0

- Add `Client` and the profile settings route (`GET users/me/profile/settings`).
- Add the crate-wide `Error` type. It is `#[non_exhaustive]`, so new variants
  are not a breaking change.
- Add a 30 second timeout to the HTTP client.
- Update the crate README for the profile route.

## 0.1.0

- Device-code sign-in chain and the two signed auth requests.
- P-256 proof-of-possession signing, including the 76-byte header block.
- Refresh-token exchange.
