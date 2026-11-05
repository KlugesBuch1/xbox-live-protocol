# Profile and people

The profile host serves the settings that make up a gamercard. The people host serves friends, recent players and profile cards. They are separate services with different contract versions, so a client usually calls both on startup.

## Profile settings

| Route | Auth | Contract |
| --- | --- | --- |
| `GET profile.xboxlive.com/users/me/profile/settings` | XBL3.0 | 2 |
| `GET profile.xboxlive.com/users/xuid({xuid})/profile/settings` | XBL3.0 | 2 |
| `GET profile.xboxlive.com/users/gt({gamertag})/profile/settings` | XBL3.0 | 2 |

The `settings` query parameter is a comma-separated list. The app asks for:

```text
Gamertag,ModernGamertag,Gamerscore,GameDisplayPicRaw,AccountTier,Bio,Location,XboxOneRep
```

A smaller list is allowed. Asking only for `Gamertag` is the cheapest way to turn a gamertag into an xuid.

The response is:

```json
{
  "profileUsers": [
    {
      "id": "{xuid}",
      "hostId": "{xuid}",
      "settings": [
        { "id": "Gamertag", "value": "Contoso" },
        { "id": "Gamerscore", "value": "12345" }
      ],
      "isSponsoredUser": false
    }
  ]
}
```

`profileUsers[0]` is the person. An empty array means the lookup failed, even if the HTTP status is 200. Each entry in `settings` has an `id` that matches the key you asked for.

The gamertag form is percent-encoded. Letters, digits, `-`, `_` and `.` are left alone; everything else is encoded. An unencoded space or `#` can change the path and produce a 404.

A gamertag lookup is not a username search. It returns exactly one account or nothing.

## People

| Route | Auth | Contract |
| --- | --- | --- |
| `GET peoplehub.xboxlive.com/users/me/people/friends/decoration/detail,preferredColor,presenceDetail,multiplayerSummary` | XBL3.0 | 7 |
| `GET peoplehub.xboxlive.com/users/xuid({xuid})/people/friends/decoration/bio,detail,multiplayerSummary,preferredColor,presenceDetail` | XBL3.0 | 7 |
| `GET peoplehub.xboxlive.com/users/me/people/xuids({xuids})/decoration/detail,preferredColor,presenceDetail` | XBL3.0 | 7 |
| `POST peoplehub.xboxlive.com/users/me/people/batch/decoration/detail,preferredColor` | XBL3.0 | 7 |
| `GET peoplehub.xboxlive.com/users/me/people/recentplayers` | XBL3.0 | 7 |
| `GET peoplehub.xboxlive.com/users/me/people/playedTitle({title_id})/decoration/presenceDetail,titleHistory` | XBL3.0 | 7 |
| `GET peoplehub.xboxlive.com/users/me/accountlink` | XBL3.0 | 1 |

The decoration segment is not optional if you want presence or profile detail. It decides which blocks the server fills in. `detail` alone gives the relationship flags and counts, `presenceDetail` adds `presenceDetails[]`, `titleHistory` adds the recent titles for that person.

The response envelope for the people routes is:

```json
{
  "people": [],
  "recommendationSummary": null,
  "friendFinderState": null,
  "accountLinkDetails": null,
  "friendRequestSummary": null
}
```

Each person has `xuid`, `gamertag`, `displayPicRaw`, `gamerScore`, `presenceState`, `presenceText` and an optional `detail` block. The detail block carries `isFriend`, `isFollowingCaller`, `isFollowedByCaller`, `friendCount`, `followerCount`, `hasGamePass` and the relationship flags.

The batch form takes a body:

```json
{ "xuids": ["{xuid}"] }
```

It returns the same `people[]` list. The app uses it when it has a row of xuid values and does not want one request per card.

`recentplayers` can return an empty list. That is normal for an account that has not played multiplayer recently. `playedTitle({title_id})` is the game-page variant - it returns the people who played that decimal title id.

`accountlink` is the odd one. It uses the people envelope but the useful field is `accountLinkDetails`. The app calls it on startup and after an account switch.

## Privacy

`GET privacy.xboxlive.com/users/me/privacy/settings` returns a flat list:

```json
{
  "settings": [
    { "setting": "ShareFriendList", "value": "Everyone" },
    { "setting": "SharePresence", "value": "Everyone" },
    { "setting": "AllowProfileViewing", "value": "Everyone" }
  ]
}
```

Values are strings such as `Everyone`, `PeopleOnMyList` and `Blocked`. The set is larger than the three examples - the full list in the capture had 22 entries.

The contract version is 3.

## Gamertag generation

`GET accounts.xboxlive.com/users/current/profile/gamertags/generate` returns 401 without a token. The route is live but the query and response are not documented here.

## Examples

[`profile-me.json`](../examples/profile-me.json) and [`people-friends.json`](../examples/people-friends.json).

## What the app does

On a cold start the app asks for the signed-in profile, the friend list, the recent players and the privacy settings. After an account switch it repeats the profile and account-link calls. The profile and people calls are independent - one can fail without blocking the other, and the UI shows a partial card when that happens.

The old `social.xboxlive.com` routes are documented in [social.md](social.md). They still answer, but the current app uses peoplehub for the same data.
