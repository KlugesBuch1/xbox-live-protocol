# Auth

The app does not use one login. It turns a Microsoft account into an Xbox identity in four or five requests. The token it ends up with is not the Microsoft token.

```text
login.live.com/oauth20_authorize.srf
  -> login.live.com/oauth20_token.srf
  -> device.auth.xboxlive.com/device/authenticate
  -> sisu.xboxlive.com/authorize
  -> Authorization: XBL3.0 x={uhs};{token}
```

The current app signs in through Windows Web Account Manager. The older path uses `xsts.auth.xboxlive.com` instead of SISU. Both end at the same `XBL3.0` header.

The two signed requests are covered in [signing.md](signing.md). This page covers the route flow and the important fields.

## Routes

| Route | Method | Purpose |
| --- | --- | --- |
| `login.live.com/oauth20_authorize.srf` | GET | Interactive sign-in |
| `login.live.com/oauth20_connect.srf` | POST | Starts the device-code flow |
| `login.live.com/oauth20_token.srf` | POST | Code, refresh or device-code exchange |
| `login.live.com/RST2.srf` | POST | WS-Trust request made by Windows WAM |
| `device.auth.xboxlive.com/device/authenticate` | POST | Creates a device token |
| `sisu.xboxlive.com/authorize` | POST | Turns the Microsoft token into an XBL3.0 header |
| `xsts.auth.xboxlive.com/xsts/authorize` | POST | The older device-token exchange |

## The Microsoft side

The desktop client uses the public client id `000000004424da1f`. It appears in the authorize request. The scope is:

```text
service::user.auth.xboxlive.com::MBI_SSL
```

Without that scope, SISU has nothing to exchange and returns an error that does not explain why.

The authorize request asks for a code:

```text
client_id=000000004424da1f
response_type=code
redirect_uri=https://login.live.com/oauth20_desktop.srf
response_mode=query
scope=service::user.auth.xboxlive.com::MBI_SSL
prompt=select_account
```

The redirect is parsed for `code` or `error`. The code is then posted to `oauth20_token.srf` with `grant_type=authorization_code`, the same `client_id`, the same `redirect_uri` and the same scope.

The refresh grant uses `grant_type=refresh_token` and a refresh token. The device-code grant uses `grant_type=urn:ietf:params:oauth:grant-type:device_code` and the device code. There is no client secret in any of these requests.

WAM asks Windows for a token and Windows talks to `login.live.com/RST2.srf` with a WS-Trust envelope. I captured the request but did not reproduce the body. It contains the returned token material and the exact XML is not needed to understand the rest of the chain.

## Device token

`device/authenticate` is the first signed request. The body:

```json
{
  "Properties": {
    "AuthMethod": "ProofOfPossession",
    "Id": "{...}",
    "DeviceType": "Win32",
    "SerialNumber": "{...}",
    "Version": "0.0.0",
    "ProofKey": { "kty": "EC", "crv": "P-256", "alg": "ES256", "use": "sig", "x": "...", "y": "..." }
  },
  "RelyingParty": "http://auth.xboxlive.com",
  "TokenType": "JWT"
}
```

`Id` and `SerialNumber` are random GUIDs in braces. They do not have to match the machine. `Version` is the client version string and `0.0.0` is accepted.

The response has a `Token`, which is the device token. It is passed to SISU unchanged.

A fresh P-256 key pair is generated per relying party. The app performs this flow twice, once for `http://xboxlive.com` and once for `http://events.xboxlive.com`.

## SISU

`sisu.xboxlive.com/authorize` is the second signed request:

```json
{
  "AccessToken": "t={microsoft_access_token}",
  "AppId": "000000004424da1f",
  "DeviceToken": "{device token}",
  "Sandbox": "RETAIL",
  "UseModernGamertag": true,
  "SiteName": "user.auth.xboxlive.com",
  "RelyingParty": "http://xboxlive.com",
  "ProofKey": { "kty": "EC", "crv": "P-256", "alg": "ES256", "use": "sig", "x": "...", "y": "..." }
}
```

The `t=` prefix is part of the value.

`Sandbox` is `RETAIL` for a normal account. Other sandboxes issue tokens for other environments and the rest of the API will not accept them.

`SiteName` decides the token type. `user.auth.xboxlive.com` is the one that produces an `XBL3.0` header.

The response contains:

```json
{
  "AuthorizationToken": {
    "Token": "...",
    "DisplayClaims": {
      "Xui": [{ "uhs": "..." }]
    }
  }
}
```

The header is built from `DisplayClaims.Xui[0].uhs` and `AuthorizationToken.Token`:

```http
Authorization: XBL3.0 x={uhs};{token}
```

For events, the relying party is `http://events.xboxlive.com` and the value is prefixed with `x:`:

```text
x:XBL3.0 x={uhs};{token}
```

OneCollector sends that value in `authxtoken` and `tickets`, not in `Authorization`. The `x:` is part of the value.

## XSTS

`xsts/authorize` is the route most older projects use. It takes the device token and returns an XSTS token. The current app does not call it, but the route is live.

The body is not documented here because I did not capture a working request. The shape is close to SISU: a `Properties` object, a `RelyingParty` and a `TokenType`. Do not copy a body from an old guide without checking the relying party and the sandbox.

## Errors

| Status | What it usually means |
| --- | --- |
| 400 at SISU | Missing `t=` prefix, malformed `ProofKey`, or a missing field |
| 401 at SISU | The Microsoft token is expired, has the wrong scope or belongs to another client id |
| 403 at SISU | The account or sandbox is not allowed - child accounts and restricted accounts show up here |
| 401 at an API | The `Authorization` header is missing, has the wrong shape, or the token expired |
| 403 at an API | The identity is valid but the account is not allowed to see the resource |

## Tokens

| Value | Lifetime | Where it goes |
| --- | --- | --- |
| Microsoft access token | About an hour | `AccessToken` in SISU, with `t=` |
| Microsoft refresh token | Weeks to months | Local session storage, then the refresh grant |
| Device token | Bound to the proof key | `DeviceToken` in SISU or XSTS |
| Xbox token and `uhs` | About an hour | `Authorization` on every Xbox host |
| Events token | About an hour | `authxtoken` and `tickets` on OneCollector |

When the Xbox token expires, run SISU again. Do not try to refresh the Xbox token with the Microsoft refresh token directly - they are different grants.
