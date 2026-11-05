# Presence

Presence is split across two hosts. `presence-heartbeat.xboxlive.com` keeps a title marked as running and uses a token from the Xbox app process. `userpresence.xboxlive.com` sets the current title or device state with a normal XBL3.0 header.

The app uses both, but they are not interchangeable.

## Heartbeat

```http
POST https://presence-heartbeat.xboxlive.com/users/xuid({xuid})/devices/current/
Authorization: {app token}
x-xbl-contract-version: 3
Content-Type: application/json; charset=utf-8
```

Body:

```json
{
  "titles": [
    {
      "expiration": 600,
      "id": "704208617",
      "state": "active",
      "sandbox": "RETAIL"
    }
  ]
}
```

`id` is a string. `expiration` is in seconds. The client sends a heartbeat every five minutes and uses HTTP/1 only; the host did not accept HTTP/2 in the client's configuration.

The token is not the XBL3.0 header. The app obtains it from its own process and passes it to the heartbeat. This is the main reason the route is hard to reproduce outside the app.

A successful response has no interesting body. The `x-heartbeat-after` header tells the client how long to wait before the next beat - the captured value was 300.

Status codes: 401 means the app token is missing or expired. 403 means the account or title is not allowed.

The route changes account state: it makes the title look active. Use a test account.

## Current title

```http
POST https://userpresence.xboxlive.com/users/xuid({xuid})/devices/current/titles/current
Authorization: XBL3.0 x=...
x-xbl-contract-version: 3
Content-Type: application/json; charset=UTF-8
```

Body:

```json
{ "id": 704208617, "State": "Active" }
```

This is the route the app uses to show the running title in the friend list. `id` is a number here, unlike the heartbeat body where it is a string.

The response is empty. The `x-heartbeat-after` header is 300.

## Device state

`POST userpresence.xboxlive.com/users/xuid({xuid})/devices/current` returns 401 without a token. The client uses the `titles/current` form for title presence - the device form is for the device-level state and is listed because it is part of the same family.

The body shape is not documented here.

## Reading presence

There is no dedicated read route in this catalogue. Presence comes back through `peoplehub` when the decoration includes `presenceDetail`:

```text
GET peoplehub.xboxlive.com/users/me/people/friends/decoration/detail,preferredColor,presenceDetail,multiplayerSummary
```

Each person has `presenceState`, `presenceText` and `presenceDetails[]`. A detail entry has `Device`, `State`, `TitleId`, `IsPrimary`, `IsGame` and `RichPresenceText`. `State` values include `Online`, `Away`, `Offline` and `LastSeen`.

This is why a client that only polls `userpresence` will not see the same thing as the friend list.

## Common mistakes

- Sending the SISU token to the heartbeat host. It expects the app token.
- Sending `id` as a number in the heartbeat body. It is a string.
- `State` is case-sensitive.
- Polling every few seconds. The server returns `x-heartbeat-after`.
- A 403 usually means the account is a child account or the title is not allowed.

x-xbl-contract-version: 3
Content-Type: application/json; charset=utf-8
```

Body:

```json
{
  "titles": [
    {
      "expiration": 600,
      "id": "704208617",
      "state": "active",
      "sandbox": "RETAIL"
    }
  ]
}
```

`id` is a string, not a number. `expiration` is in seconds. The client sends a heartbeat every five minutes and uses HTTP/1 only - the host did not accept HTTP/2 in the client's configuration.

The token is not the XBL3.0 header. The app obtains it from its own process and passes it to the heartbeat. This is the main reason the route is hard to reproduce outside the app.

A successful response has no interesting body. The `x-heartbeat-after` header tells the client how long to wait before the next beat - the captured value was 300.

Status codes: 401 means the app token is missing or expired. 403 means the account or title is not allowed. A GET on this path returns 405, which is a useful confirmation that the route is POST-only.

The route changes account state: it makes the title look active. Use a test account.

## Current title

```http
POST https://userpresence.xboxlive.com/users/xuid({xuid})/devices/current/titles/current
Authorization: XBL3.0 x=...
x-xbl-contract-version: 3
Content-Type: application/json; charset=UTF-8
```

Body:

```json
{ "id": 704208617, "State": "Active" }
```

This is the route the app uses to show the running title in the friend list. `id` is a number here, unlike the heartbeat body where it is a string. `State` is capitalized.

The response is empty. The `x-heartbeat-after` header is 300.

## Device state

`POST userpresence.xboxlive.com/users/xuid({xuid})/devices/current` returns 401 without a token. The client uses the `titles/current` form for title presence - the device form is for the device-level state and is listed because it is part of the same family.

The body shape is not documented here.

## Reading presence

There is no dedicated read route in this catalogue. Presence comes back through `peoplehub` when the decoration includes `presenceDetail`:

```text
GET peoplehub.xboxlive.com/users/me/people/friends/decoration/detail,preferredColor,presenceDetail,multiplayerSummary
```

Each person has `presenceState`, `presenceText` and `presenceDetails[]`. A detail entry has `Device`, `State`, `TitleId`, `IsPrimary`, `IsGame` and `RichPresenceText`. `State` values include `Online`, `Away`, `Offline` and `LastSeen`.

This is why a client that only polls `userpresence` will not see the same thing as the friend list. The friend list is peoplehub's view of presence, not the raw device state.

## Common mistakes

- Sending the SISU token to the heartbeat host. It expects the app token.
- Sending `id` as a number in the heartbeat body. It is a string.
- Sending `State` in lowercase on the userpresence route. The client sends `Active`.
- Polling every few seconds. The server returns `x-heartbeat-after`.
- Treating a 403 as a dead route. It usually means the account is a child account or the title is not allowed.
