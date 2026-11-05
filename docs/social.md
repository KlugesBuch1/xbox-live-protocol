# Social and clubs

Most of the social graph is on peoplehub now. This page keeps the older social host and the official-club route, because both still answer and both show up in older code.

## social.xboxlive.com

| Route | Auth | Status without a token |
| --- | --- | --- |
| `GET social.xboxlive.com/users/me/people` | XBL3.0 | 403 |
| `GET social.xboxlive.com/users/xuid({xuid})/people/followers` | XBL3.0 | 403 |
| `GET social.xboxlive.com/users/xuid({xuid})/people/friends` | XBL3.0 | 403 |

A 403 here means the path exists and the caller is not allowed.

The current app does not use these routes. It gets friends, followers and presence from `peoplehub.xboxlive.com` instead. If you are writing a new client, use peoplehub and treat these as compatibility routes.

## Official clubs

`GET clubhub-public.xboxlive.com/clubs/officialclub({club_id})` returns the official club for a title. The contract version is 7 and the response contains:

```json
{
  "clubs": [],
  "clubDeeplinks": null,
  "recommendationCounts": null,
  "searchFacetResults": null
}
```

The `clubs[]` entry has `id`, `clubType`, `profile`, `followersCount`, `membersCount`, `clubPresenceCount` and `titleDeeplinks`. The profile block is a map of `{ "value": ..., "allowedValues": ... }` objects, not plain strings.

A title without a club returns 404 and:

```json
{ "code": 1028, "description": "Official Club does not exist." }
```

A tokenless request returns 400. The app sends the XBL3.0 header and the contract header.

## Hosts that are gone

These were checked on 2026-11-05 and are not in the index:

| Host or route | Result |
| --- | --- |
| `clubs.xboxlive.com` | DNS does not resolve. |
| `clubhub.xboxlive.com/clubs` | 404. |
| `reputation.xboxlive.com/users/xuid(...)/reputation` | 404 in three path variants. |
| `multiplayeractivity.xboxlive.com/users/xuid(...)/activities` | 404. |
| `mediahub.xboxlive.com/users/xuid(...)/media` | 404. |

`clubhub-public` is the host that still works for official clubs.

## Privacy and visibility

A club response can be empty even when the club exists, depending on the caller's privacy settings and the club's visibility. The `state` field and `profile.isSearchable` are the fields to check before treating an empty `clubs[]` as a missing club.
