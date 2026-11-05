# Signing

Two requests in the auth chain are signed with a P-256 key:

- `POST device.auth.xboxlive.com/device/authenticate`
- `POST sisu.xboxlive.com/authorize`

The signature proves that the client owns the private key behind the `ProofKey` it sent. It also covers the method, path, query and body, so a captured request cannot be moved to another route or edited.

This is not the same signature as the `Signature` header on `progress/update`. That one is covered at the end of this page.

## The header

The `Signature` header is a standard base64 encoding of a 76 byte block:

| Offset | Length | Value |
| --- | --- | --- |
| 0 | 4 | Policy version, big-endian `u32`, currently `1`. |
| 4 | 8 | The same FILETIME value that is signed, big-endian `u64`. |
| 12 | 64 | The raw ES256 signature as `r || s`, 32 bytes each. |

It is not a DER signature. `Signature::to_bytes()` from the `p256` crate produces the right form; `to_der()` does not.

It is standard base64, not base64url. The padding is kept.

## The signed payload

| Offset | Length | Value |
| --- | --- | --- |
| 0 | 4 | Policy version, big-endian `u32`, `1`. |
| 4 | 1 | `0x00`. |
| 5 | 8 | FILETIME, big-endian `u64`. |
| 13 | 1 | `0x00`. |
| 14 | n | HTTP method as ASCII, for example `POST`. |
| 14+n | 1 | `0x00`. |
| 15+n | n | Path and query, for example `/authorize` or `/authorize?x=1`. |
| ... | 2 | `0x00 0x00`. |
| ... | n | Body, byte for byte as sent. |
| ... | 1 | `0x00`. |

The two zero bytes after the path are the boundary that is easiest to get wrong. The method starts at offset 14, not 15. A version with two zeros before the method returns 403 even when the rest is correct.

There are no separators between the version and timestamp in the header, but there are separators in the payload. Building both the same way returns 401.

## FILETIME

The timestamp is not a Unix timestamp:

```text
filetime = (unix_seconds + 11_644_473_600) * 10_000_000
```

It counts 100 nanosecond intervals since 1 January 1601. The value is signed, but it is not checked as a clock. Measured: `0`, one day in the past and one day in the future all returned 200. A Unix timestamp is not a failure as long as you sign the bytes you send.

## The proof key

The `ProofKey` is a JWK:

```json
{
  "kty": "EC",
  "crv": "P-256",
  "alg": "ES256",
  "use": "sig",
  "x": "base64url, 32 bytes, padded",
  "y": "base64url, 32 bytes, padded"
}
```

`x` and `y` are base64url without padding, but always padded to 32 bytes before encoding. A coordinate that starts with `0x00` can be shortened to 31 bytes by a library. The point is then invalid and the server answers 400.

A fresh key pair is generated per relying party. The app does this twice, once for the Xbox relying party and once for events.

## Verify offline

You do not need the server to check the layout. The request contains the signature and the public key. To verify a captured request:

1. Base64-decode the `Signature` header. It must be 76 bytes.
2. Read the version, FILETIME, `r` and `s`.
3. Rebuild the payload from the method, path with query, and raw body bytes.
4. Run ECDSA/SHA-256 against `x` and `y` from `ProofKey`.

If the signature verifies, the layout is confirmed for that request. If it fails for one variant and passes for another, the variant is wrong.

This is the cheapest way to test a layout change. It costs nothing and cannot unlock anything.

## What is not covered

The `Signature` header on `POST users/xuid({xuid})/achievements/{scid}/update` is a different value. It is a fixed string in the client, not a per-request PoP signature. The client sends it together with `User-Agent: XboxServicesAPI/2021.10.20211005.0 c`.

Measured on 2026-11-05: a request for a locked achievement, using the same fixed value that was captured from a request for a different achievement, returned 200 and unlocked the achievement. The header is not bound to the achievement id or the body. It behaves like a static credential.

## Common failures

These are expectations from the layout and from the measurements noted above.

| What is wrong | Likely status |
| --- | --- |
| Unix timestamp instead of FILETIME | 200, because the clock is not checked. |
| Little-endian fields | 401. |
| Path signed without the query | 401. |
| Full URL with scheme and host signed | 401. |
| DER signature instead of `r || s` | 401. |
| Base64url instead of standard base64 | 401. |
| Header built with the payload separators | 401. |
| Body reserialized after signing | 401. |
| Key coordinate shortened to 31 bytes | 400. |
| `AuthMethod` is not `ProofOfPossession` | 400. |

The last two are the reason to keep the body as bytes until it is sent. Parsing it into a JSON value and serializing it again can change the key order or the whitespace, and the signature is over the bytes, not the object.
