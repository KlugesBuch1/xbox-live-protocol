# Achievements

The achievement host serves two lists of the same route and one write. The list has a contract version that changes the response, and the write has a second `Signature` header that is not the proof-of-possession signature used for login.

## The list

```http
GET https://achievements.xboxlive.com/users/xuid({xuid})/achievements?titleId=704208617&maxItems=1000
Authorization: XBL3.0 x=...
x-xbl-contract-version: 2
```

Query parameters:

| Name | Required | Value |
| --- | --- | --- |
| `titleId` | yes | Decimal title id |
| `maxItems` | no | Page size - Xbox app uses 1000 |
| `continuationToken` | no | Token from the previous page |

The response:

```json
{
  "xuid": "{xuid}",
  "titleId": "704208617",
  "name": "Halo Infinite",
  "achievements": [
    {
      "id": "1",
      "name": "First Steps",
      "description": "...",
      "lockedDescription": "...",
      "progressState": "Achieved",
      "progression": { "timeUnlocked": "2026-11-04T20:00:00.000Z" },
      "mediaAssets": [],
      "rewards": [{ "name": "Gamerscore", "value": "10", "type": "Gamerscore" }]
    }
  ],
  "pagingInfo": { "continuationToken": null, "totalRecords": 80 }
}
```

The app stops after 25 pages, an empty page or a repeated continuation token.

`progressState` is `Achieved`, `InProgress` or `NotStarted`. The request will return the title's achievements the account has not unlocked, they still count toward `totalAchievements`.

## Contract 2 and contract 4

The same path with `x-xbl-contract-version: 4` returns an extra requirement block. The field that matters is the requirement id:

```json
{
  "id": "1",
  "progression": { "requirements": [{ "id": "00000000-0000-0000-0000-000000000000" }] }
}
```

A requirement id that is not all zeros marks an event-based title. The app reads one page under contract 4 and uses that to choose between the progress write and the event path.

Contract 2 returns the paged list described above. Contract 4 is not a newer version of the same page, it is a different payload structure. Do not ask for contract 4 and expect `pagingInfo` to behave the same.

A tokenless GET returns 400, not 401. The service validates both the contract header and identity before parsing the query.

## The write

```http
POST https://achievements.xboxlive.com/users/xuid({xuid})/achievements/a5ee0100-7093-4a78-b098-471729f95ee9/update
Authorization: XBL3.0 x=...
x-xbl-contract-version: 2
Content-Type: application/json
User-Agent: XboxServicesAPI/2021.10.20211005.0 c
Signature: {fixed value}
```

Body:

```json
{
  "action": "progressUpdate",
  "serviceConfigId": "a5ee0100-7093-4a78-b098-471729f95ee9",
  "titleId": "704208617",
  "userId": "{xuid}",
  "achievements": [
    { "id": "1", "percentComplete": "100" }
  ]
}
```

`titleId` and `percentComplete` are strings. `100` unlocks. A lower value updates progress on titles that support partial progress.

`serviceConfigId` must be the scid for the title. You can get it from `title-user-card` or `title-pfn-user`.

The `Signature` header here is not a PoP signature. It is a fixed value in the client. Measured on 2026-11-05: a request for a locked achievement, using the same fixed value that was captured from a request for a different achievement, returned 200 and unlocked the achievement. The header is not bound to the achievement id or the body.

Status mapping:

| Status | Meaning |
| --- | --- |
| 200, 204 | The service accepted the update |
| 304 | No change - the achievement was already at that value |
| 401 | The Xbox token is missing or expired |
| 403 | The token is valid but the account or title is not allowed |
| Other | Rejected - the body usually contains a short error string |

There is a delay between a successful write and the achievement appearing in a new list request. The client waits two seconds before refreshing.

## Event-based titles

Some titles do not use `progress/update`. Their achievements are driven by events posted to OneCollector with the events token. The event body is title-specific and is not documented here. See [telemetry.md](telemetry.md).

The contract 4 list is how you tell the two apart. If the requirement id is all zeros, the title uses progress. If it is not, the title uses events.

## Examples

[`achievements-contract-2.json`](../examples/achievements-contract-2.json) and [`achievements-contract-4.json`](../examples/achievements-contract-4.json).

## Disclaimer

If you test it, use an account you can lose and a title you do not care about. The write changes account state and there is no way to undo an unlock.
