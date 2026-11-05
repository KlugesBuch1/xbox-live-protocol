# Stats

`userstats.xboxlive.com/batch` is one path with three different bodies. The path does not tell you which one you want - the body and the contract version do.

## The three bodies

### Hero stats

```json
{
  "arrangebyfield": "xuid",
  "xuids": ["{xuid}"],
  "groups": [{ "name": "Hero", "titleId": "704208617" }]
}
```

Contract version 2. This is the first call for a title page. It returns the stats the title marks as hero stats.

### Service-config stats

```json
{
  "requestedusers": ["{xuid}"],
  "requestedscids": [
    { "scid": "a5ee0100-7093-4a78-b098-471729f95ee9", "requestedstats": [] }
  ]
}
```

Contract version 1. The empty `requestedstats` list asks for every stat on that service config. This is the fallback when the Hero call returns no rows.

### Minutes played

```json
{
  "arrangebyfield": "xuid",
  "xuids": ["{xuid}"],
  "stats": [{ "name": "MinutesPlayed", "titleId": "704208617" }]
}
```

Contract version 2. This is a named stat request, not a group. It is the same path as the Hero call and a different body.

The capture uses `arrangebyfield`. Some client code uses `arrangeByField`.

## The response

The shape is not stable across titles. The client handles these places:

```text
groups[].statlistscollection
groups[].statListsCollection
groups[].statlistcollection
statlistscollection (top level)
users[].scids[]
stats[] (top level)
```

A normal Hero response:

```json
{
  "groups": [
    {
      "name": "Hero",
      "titleid": "704208617",
      "statlistscollection": [
        {
          "arrangebyfield": "xuid",
          "arrangebyfieldid": "{xuid}",
          "stats": [
            {
              "xuid": "{xuid}",
              "scid": "a5ee0100-7093-4a78-b098-471729f95ee9",
              "titleid": "704208617",
              "name": "MinutesPlayed",
              "type": "Integer",
              "value": 1234
            }
          ]
        }
      ]
    }
  ]
}
```

The field names are lowercase in the response even when the request used camelCase. Some titles return `statListsCollection` - some return `statlistcollection`. Check all three before giving up.

`value` can be a number or a string. For `MinutesPlayed`, parse both. For a double stat, the JSON number can be an integer if the value has no fractional part.

An empty `statlistscollection` is a valid answer. It means the title has no stats for that account, not that the request failed.

## Stat write

```http
PATCH https://statswrite.xboxlive.com/stats/users/{xuid}/scids/a5ee0100-7093-4a78-b098-471729f95ee9
Authorization: XBL3.0 x=...
x-xbl-contract-version: 4
Content-Type: application/json; charset=utf-8
```

Body:

```json
{
  "$schema": "http://stats.xboxlive.com/2017-1/schema#",
  "previousRevision": 0,
  "revision": 1793879144,
  "timestamp": "2026-11-05T11:45:44.123456Z",
  "stats": {
    "title": {
      "MinutesPlayed": { "value": "1234" }
    }
  }
}
```

`revision` is the current Unix time in seconds. `previousRevision` is 0 in the client. `timestamp` is UTC with six fractional digits.

`value` is a string for every stat type, including Integer and Double. A body with a JSON number is rejected. Integer values must match `-?[0-9]+`. Double values must be finite JSON numbers. String and unknown types send the trimmed text.

The service config id is the one on the stat row. If you do not have a stat row, get it from the title card. Sending a stat write to the wrong scid can return 403 or a generic rejection.

Retries: the client tries three times, one second apart. It does not retry 401 or 403. A 200 or 204 is a success - the client waits two seconds before refreshing so a follow-up read can see the new value.

Status mapping:

| Status | Meaning |
| --- | --- |
| 200, 204 | Written. |
| 401 | Missing or expired token. |
| 403 | Token valid, account or scid not allowed. |
| Other | Rejected - the body has a short error. |

The write changes account state. Use a test account. Deleting or reverting written stats is not always possible.

## Examples

[`stats-hero.json`](../examples/stats-hero.json) and [`stats-scid.json`](../examples/stats-scid.json).

## What is not here

Leaderboards are not documented. `leaderboards.xboxlive.com/users/xuid(...)/scids/{scid}/leaderboard` returned 404 in the probe, and I did not find a working variant. 

The old `GET userstats.xboxlive.com/users/xuid({xuid})/scids/{scid}` route also returned 404.
