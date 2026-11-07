# Xbox Live protocol

[![CI](https://github.com/KlugesBuch1/xbox-live-protocol/actions/workflows/ci.yml/badge.svg)](https://github.com/KlugesBuch1/xbox-live-protocol/actions/workflows/ci.yml)

A catalogue of the HTTP API used by the Xbox App on Windows. It covers sign-in, profiles, the games library, achievements, stats, presence, friends, messages, captures, the store catalogue and cloud gaming.

It is an unofficial protocol and does not cover the entire Xbox Live surface.

## This repo contains

| Path | What it is |
| --- | --- |
| [`endpoints.yaml`](endpoints.yaml) | All 76 captured endpoints |
| [`docs/auth.md`](docs/auth.md) | Microsoft account sign-in, device tokens, SISU and XSTS |
| [`docs/signing.md`](docs/signing.md) | Proof of possession, the `Signature` header and the payload layout |
| [`docs/profile.md`](docs/profile.md) | Profile settings, peoplehub, privacy and account links |
| [`docs/social.md`](docs/social.md) | The older social host and official clubs |
| [`docs/titles.md`](docs/titles.md) | Title history, title cards, pins and cloud-save locks |
| [`docs/store.md`](docs/store.md) | Display catalog, Store edge, Game Pass lists and app config |
| [`docs/achievements.md`](docs/achievements.md) | Achievement lists and the progress write. |
| [`docs/stats.md`](docs/stats.md) | The three `/batch` bodies and the stat write |
| [`docs/presence.md`](docs/presence.md) | Heartbeats and current-title presence |
| [`docs/messaging.md`](docs/messaging.md) | Inboxes, notifications, chat and real-time activity |
| [`docs/media.md`](docs/media.md) | Clips and screenshots |
| [`docs/cloud-gaming.md`](docs/cloud-gaming.md) | Consoles, xHome and the Game Pass cloud login |
| [`docs/telemetry.md`](docs/telemetry.md) | OneCollector, impressions and the Microsoft account profile |
| [`docs/conventions.md`](docs/conventions.md) | Route notation, headers, contract versions and status codes |
| [`docs/endpoint-index.md`](docs/endpoint-index.md) | The same 76 routes as a table |
| [`tools/probe.py`](tools/probe.py) | Sends a tokenless request to the read routes and prints the status |
| [`examples/`](examples) | Trimmed response bodies with fake ids |

## Rust client

[`crates/xbl`](crates/xbl) is the reference Rust client for these endpoints. (WIP)

## The index

Each entry in `endpoints.yaml` looks like this:

```yaml
- id: title-history
  area: titles
  method: GET
  host: titlehub.xboxlive.com
  path: /users/xuid({xuid})/titles/titleHistory/decoration/Achievement
  auth: xbl3.0
  contract: "2"
  class: read
  status: 401
  doc: docs/titles.md
  summary: Played titles for an account.
  query:
    - maxItems
  notes: The app sends maxItems=10000 and a 100 second timeout.
```

`status` is what the route returned in the capture or probe that established the entry. `probe_status` is the result of a request without an `Authorization` header, when I measured it separately. A captured people route therefore has `status: 200` and `probe_status: 401`: the first is with a token, the second without.

A 401 means the path exists and the service wants an identity. A 400 usually means the body or a required header is missing. A 404 is either a dead path or a missing object.

`contract` is the `X-Xbl-Contract-Version` header. An empty value means the client does not set the header on that route.

`class` is one of `read`, `write`, `auth` or `websocket`. Nine of the ten write routes are marked `test_account: true`. The tenth is an ad impression, not account state. Do not use account writes for an account you care about.

`progress/update` is the one write with a fixed `Signature` instead of a PoP signature. A locked-achievement test confirmed that the value is a static client credential, not a signature over the request body. The value is not published and there is no copy-ready bundle.

## Auth explained

The app ends up with two headers:

```http
Authorization: XBL3.0 x={uhs};{token}
X-Xbl-Contract-Version: 2
```

The token comes from SISU in the current app:

```text
Microsoft access token
  -> device.auth.xboxlive.com/device/authenticate
  -> sisu.xboxlive.com/authorize
  -> XBL3.0 header
```

The older path uses `xsts.auth.xboxlive.com/xsts/authorize` instead of SISU. Both need a proof-of-possession signature. The full flow, the bodies and the error cases are in [`docs/auth.md`](docs/auth.md).

For events, the same SISU call is made with the relying party `http://events.xboxlive.com`. The resulting value starts with `x:XBL3.0` and is sent to OneCollector as `authxtoken` and `tickets`, not as an `Authorization` header.

## Read a route before you call it

The docs use placeholders:

- `{xuid}` is the numeric Xbox user id.
- `{title_id}` is the decimal title id, for example `704208617`.
- `{product_id}` is the Store product id, for example `9MV0B5HZVK9Z`.
- `{scid}` is the service config id, a UUID.
- `{pfn}` is the package family name, for example `Microsoft.GamingApp_8wekyb3d8bbwe`.
- `{gamertag}` is percent-encoded.

A title id, a product id and a package family name are three different things. Mixing them up is the most common reason a route returns 404.

All ids in the docs and examples are placeholders. `{xuid}` is not an account id, and the repo does not contain a real gamertag or xuid. The probe uses `0000000000000000` because a tokenless request only needs a path.

## Probe

`tools/probe.py` reads `endpoints.yaml`, skips the write and websocket routes, and sends a tokenless request to the rest. It sets the contract header when the entry has one and sends `{}` for POST routes that are reads. It also skips `dbox.tools`, which is a third-party index rather than an Xbox host.

```powershell
python -m pip install pyyaml
python tools/probe.py
```

It prints one line per route with the expected and actual status. A mismatch is not always a failure. A 400, 401 or 403 all prove the path exists.

The probe is deliberately boring. It does not log in, does not send a token and does not try to mutate anything.

## What is not here

- Purchases and billing
- Console commands
- Internals of an xCloud session - login handshake is here - media and input channels are not because they are not HTTP
- Static assets
- Windows telemetry

I also left out routes that returned 404 in three variants or whose host no longer resolves. `clubs.xboxlive.com` is gone. `clubhub.xboxlive.com` returns 404. `reputation.xboxlive.com` returns 404. `xgpuweb` and the old `streaming.xboxlive.com` paths did not answer. `userposts.xboxlive.com/users/xuid(...)/posts` answered 405 for GET, so the path exists, but I did not confirm the method or the body so I left it out.

## Accuracy

The service changes. A route that once returned 200 can disappear. The contract version is not a guarantee that the response shape is stable.

Make sure to: 

1. Check the route with `tools/probe.py` before you trust it.
2. Keep the `X-Xbl-Contract-Version` value from the entry. Some routes return a different shape, or a 400, without it.
3. Treat headers as observed, not required. Even though I tried to minimize every header.

Last check: 2026-11-05
