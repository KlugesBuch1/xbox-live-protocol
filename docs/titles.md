# Titles

The title host is the busiest service in the app. It serves the games library, title cards, the store page and the Game Pass rows. Most of the routes look similar and differ in what you put in the path and which decoration list you ask for.

## The four ids

| Value | Example | Where it goes |
| --- | --- | --- |
| Title id | `704208617` | `titleid(...)`, `titleId(...)`, most query parameters |
| Product id | `9MV0B5HZVK9Z` | Store routes, `lists/pins/{product_id}` |
| Package family name | `Microsoft.GamingApp_8wekyb3d8bbwe` | `pfn(...)` routes |
| Service config id | `a5ee0100-7093-4a78-b098-471729f95ee9` | Stats, achievements and cloud saves |

They are not interchangeable.

## Routes

| Route | Method | Contract | Purpose |
| --- | --- | --- | --- |
| `/users/xuid({xuid})/titles/titleHistory/decoration/Achievement` | GET | 2 | Played titles |
| `/users/xuid({xuid})/titles/titleid({title_id})/decoration/achievement,image,detail,titleHistory` | GET | 2 | One played title with art |
| `/titles/titleid({title_id})/decoration/detail` | GET | 2 | Catalog detail for one title |
| `/titles/batch/decoration/detail` | POST | 2 | Detail for several title ids |
| `/titles/search` | GET | 2 | Title search by name |
| `/users/xuid({xuid})/titles/titleId({title_id})/decoration/detail,image,scid,productId` | GET | 2 | Title card with scid and product id |
| `/users/xuid({xuid})/titles/pfn({pfn})/decoration/achievement,detail,image,scid,productId` | GET | 2 | Title card addressed by package |
| `/titles/pfn({pfn})/decoration/Scid` | GET | 2 | Service config id for a package |
| `/users/xuid({xuid})/titles/batch/decoration/GamePass,Achievement,Stats` | POST | 2 | Batch cards with Game Pass and stats |

The `titleid` and `titleId` spelling is not consistent. The service accepts both.

## Title history

```http
GET https://titlehub.xboxlive.com/users/xuid({xuid})/titles/titleHistory/decoration/Achievement?maxItems=10000
Authorization: XBL3.0 x=...
x-xbl-contract-version: 2
```

`maxItems=10000` is what the app sends. It is not a page size - the response comes back in one list in normal use.

The response:

```json
{
  "xuid": "{xuid}",
  "titles": [
    {
      "titleId": "704208617",
      "name": "Halo Infinite",
      "type": "Game",
      "devices": ["PC"],
      "achievement": {
        "currentAchievements": 25,
        "totalAchievements": 80,
        "currentGamerscore": 400,
        "totalGamerscore": 1000,
        "progressPercentage": 31.25
      },
      "titleHistory": {
        "lastTimePlayed": "2026-11-04T20:00:00.000Z"
      }
    }
  ]
}
```

`type` can be `Game`, `Application`, `Bundle` or `Dlc`. The app filters the games page on that field and on the local name.

## Title cards

`title-user-card` is the route the app uses for library rows and store tiles:

```http
GET https://titlehub.xboxlive.com/users/xuid({xuid})/titles/titleId(704208617)/decoration/detail,image,scid,productId
```

It returns `titles[]` with `titleId`, `productId`, `productIds`, `serviceConfigId`, `pfn`, `name`, `type`, `devices`, `displayImage` and `images[]`. The `images[]` entries have `url`, `type` and `caption` - types include `Logo`, `Tile`, `Poster`, `BoxArt`, `SuperHeroArt` and `Screenshot`.

The same route is called once per visible tile. In a long session it is easy to hit a 429. The captured run had one 429 after about 60 calls. Back off and retry.

The `pfn` variant is for a package family name:

```http
GET https://titlehub.xboxlive.com/users/xuid({xuid})/titles/pfn(Microsoft.GamingApp_8wekyb3d8bbwe)/decoration/achievement,detail,image,scid,productId
```

It is useful when you have an installed package but not a title id. `titles/pfn({pfn})/decoration/Scid` is the shorter form that returns just the title record with the service config id.

## Batches

`POST /titles/batch/decoration/detail`:

```json
{
  "titleIds": ["704208617"],
  "pfns": null
}
```

The batch route decorates ids you already have. If the response cannot be parsed, fall back to one `GET /titles/titleid(...)` per id - that is what the client does.

`POST /users/xuid({xuid})/titles/batch/decoration/GamePass,Achievement,Stats` uses the same body but adds the per-user decoration:

```json
{
  "pfns": null,
  "titleIds": ["704208617"]
}
```

The response has `achievementSummary` at the root and `achievement`, `stats` and `gamePass` on each title. `achievementSummary.mythicsUnlocked` and `achievementSummary.totalAchievementsUnlocked` are account-wide, not per title.

## Search

`GET /titles/search?query=halo` returns 401 without a token. The app's search page mainly uses the Game Pass search service, so this route is a fallback. The query parameter is `query` - the response shape was not captured.

`dbox.tools/api/title_ids/` is a third-party title index. It is not an Xbox host and does not need a token. It is useful for finding a title id from a name, but confirm the id with titlehub before you store it.

## Pins

`GET eplists.xboxlive.com/users/xuid({xuid})/lists/pins/{product_id}` uses contract 3. The response:

```json
{
  "ImpressionId": "Lists.00000000-0000-0000-0000-000000000000",
  "ListItems": [],
  "ListMetadata": {
    "ListTitle": null,
    "ListVersion": 0,
    "ListCount": 0,
    "AllowDuplicates": false,
    "MaxListSize": 200,
    "AccessSetting": "OwnerOnly"
  }
}
```

An empty `ListItems` means the product is not pinned. The list is per product, not a global pins list. To render a pins row you need the product ids from somewhere else and one call per product.

## Cloud-save locks

`GET titlestorage.xboxlive.com/connectedstorage/users/xuid({xuid})/scids/{scid}/lock` checks whether a save container is locked. A 404 with:

```json
{
  "code": 2038,
  "source": "TitleStorage",
  "description": "The requested storage container does not have a lock."
}
```

means the container exists but has no lock, or there is no container yet. The app uses it as a probe before showing a cloud-save action.

## Example

[`title-history.json`](../examples/title-history.json).

## Store and catalogue

The product routes are in [store.md](store.md):

- `displaycatalog.mp.microsoft.com/v7.0/products/{product_id}`
- `storeedgefd.dsx.mp.microsoft.com/v9.0/products/{product_id}`
- `catalog.gamepass.com/sigls/v2`
- `catalog.gamepass.com/misc/pc-pfns-list`

They are grouped with the store because they share product ids and market parameters, even though the title host also returns product ids.
