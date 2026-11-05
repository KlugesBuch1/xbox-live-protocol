# Endpoint index

Every route in `endpoints.yaml`, grouped by area.

`Contract` is the `X-Xbl-Contract-Version` header.

## Auth

See [auth.md](auth.md) for the details.

| Route | Method | Host | Path | Auth | Contract |
| --- | --- | --- | --- | --- | --- |
| `oauth-authorize` | GET | `login.live.com` | `/oauth20_authorize.srf` | oauth | - |
| `oauth-device-code` | POST | `login.live.com` | `/oauth20_connect.srf` | oauth | - |
| `oauth-token` | POST | `login.live.com` | `/oauth20_token.srf` | oauth | - |
| `wam-rst2` | POST | `login.live.com` | `/RST2.srf` | wam | - |
| `device-authenticate` | POST | `device.auth.xboxlive.com` | `/device/authenticate` | proof-of-possession | - |
| `sisu-authorize` | POST | `sisu.xboxlive.com` | `/authorize` | proof-of-possession | - |
| `xsts-authorize` | POST | `xsts.auth.xboxlive.com` | `/xsts/authorize` | none | - |

## Profile and people

See [profile.md](profile.md) for the details.

| Route | Method | Host | Path | Auth | Contract |
| --- | --- | --- | --- | --- | --- |
| `profile-me` | GET | `profile.xboxlive.com` | `/users/me/profile/settings` | xbl3.0 | 2 |
| `profile-xuid` | GET | `profile.xboxlive.com` | `/users/xuid({xuid})/profile/settings` | xbl3.0 | 2 |
| `profile-gamertag` | GET | `profile.xboxlive.com` | `/users/gt({gamertag})/profile/settings` | xbl3.0 | 2 |
| `people-friends` | GET | `peoplehub.xboxlive.com` | `/users/me/people/friends/decoration/detail,preferredColor,presenceDetail,multiplayerSummary` | xbl3.0 | 7 |
| `people-friends-xuid` | GET | `peoplehub.xboxlive.com` | `/users/xuid({xuid})/people/friends/decoration/bio,detail,multiplayerSummary,preferredColor,presenceDetail` | xbl3.0 | 7 |
| `people-xuids` | GET | `peoplehub.xboxlive.com` | `/users/me/people/xuids({xuids})/decoration/detail,preferredColor,presenceDetail` | xbl3.0 | 7 |
| `people-batch` | POST | `peoplehub.xboxlive.com` | `/users/me/people/batch/decoration/detail,preferredColor` | xbl3.0 | 7 |
| `people-accountlink` | GET | `peoplehub.xboxlive.com` | `/users/me/accountlink` | xbl3.0 | 1 |
| `people-recent` | GET | `peoplehub.xboxlive.com` | `/users/me/people/recentplayers` | xbl3.0 | 7 |
| `people-played-title` | GET | `peoplehub.xboxlive.com` | `/users/me/people/playedTitle({title_id})/decoration/presenceDetail,titleHistory` | xbl3.0 | 7 |
| `privacy-settings` | GET | `privacy.xboxlive.com` | `/users/me/privacy/settings` | xbl3.0 | 3 |
| `accounts-gamertags` | GET | `accounts.xboxlive.com` | `/users/current/profile/gamertags/generate` | xbl3.0 | - |

## Social and clubs

See [social.md](social.md) for the details.

| Route | Method | Host | Path | Auth | Contract |
| --- | --- | --- | --- | --- | --- |
| `social-people` | GET | `social.xboxlive.com` | `/users/me/people` | xbl3.0 | - |
| `social-followers` | GET | `social.xboxlive.com` | `/users/xuid({xuid})/people/followers` | xbl3.0 | - |
| `social-friends` | GET | `social.xboxlive.com` | `/users/xuid({xuid})/people/friends` | xbl3.0 | - |
| `clubhub-official` | GET | `clubhub-public.xboxlive.com` | `/clubs/officialclub({club_id})` | xbl3.0 | 7 |

## Titles and catalogue

See [titles.md](titles.md) for the details.

| Route | Method | Host | Path | Auth | Contract |
| --- | --- | --- | --- | --- | --- |
| `title-history` | GET | `titlehub.xboxlive.com` | `/users/xuid({xuid})/titles/titleHistory/decoration/Achievement` | xbl3.0 | 2 |
| `title-user` | GET | `titlehub.xboxlive.com` | `/users/xuid({xuid})/titles/titleid({title_id})/decoration/achievement,image,detail,titleHistory` | xbl3.0 | 2 |
| `title-detail` | GET | `titlehub.xboxlive.com` | `/titles/titleid({title_id})/decoration/detail` | xbl3.0 | 2 |
| `title-batch` | POST | `titlehub.xboxlive.com` | `/titles/batch/decoration/detail` | xbl3.0 | 2 |
| `title-search` | GET | `titlehub.xboxlive.com` | `/titles/search` | xbl3.0 | 2 |
| `title-user-card` | GET | `titlehub.xboxlive.com` | `/users/xuid({xuid})/titles/titleId({title_id})/decoration/detail,image,scid,productId` | xbl3.0 | 2 |
| `title-pfn-user` | GET | `titlehub.xboxlive.com` | `/users/xuid({xuid})/titles/pfn({pfn})/decoration/achievement,detail,image,scid,productId` | xbl3.0 | 2 |
| `title-pfn-scid` | GET | `titlehub.xboxlive.com` | `/titles/pfn({pfn})/decoration/Scid` | xbl3.0 | 2 |
| `title-batch-user` | POST | `titlehub.xboxlive.com` | `/users/xuid({xuid})/titles/batch/decoration/GamePass,Achievement,Stats` | xbl3.0 | 2 |
| `eplists-pins` | GET | `eplists.xboxlive.com` | `/users/xuid({xuid})/lists/pins/{product_id}` | xbl3.0 | 3 |
| `titlestorage-lock` | GET | `titlestorage.xboxlive.com` | `/connectedstorage/users/xuid({xuid})/scids/{scid}/lock` | xbl3.0 | - |
| `dbox-search` | GET | `dbox.tools` | `/api/title_ids/` | none | - |
| `gamepass-sigls` | GET | `catalog.gamepass.com` | `/sigls/v2` | none | - |
| `gamepass-pfns` | GET | `catalog.gamepass.com` | `/misc/pc-pfns-list` | none | - |
| `displaycatalog-product` | GET | `displaycatalog.mp.microsoft.com` | `/v7.0/products/{product_id}` | none | - |
| `storeedge-product` | GET | `storeedgefd.dsx.mp.microsoft.com` | `/v9.0/products/{product_id}` | none | - |
| `beige-offline` | GET | `beige.xboxservices.com` | `/pcgafd/config/offlinepc` | xbl3.0 | - |
| `xnotify-status` | GET | `xnotify.xboxlive.com` | `/servicestatusv6/{market}/{locale}` | none | - |

## Achievements

See [achievements.md](achievements.md) for the details.

| Route | Method | Host | Path | Auth | Contract |
| --- | --- | --- | --- | --- | --- |
| `achievements-list` | GET | `achievements.xboxlive.com` | `/users/xuid({xuid})/achievements` | xbl3.0 | 2 or 4 |
| `progress-update` | POST | `achievements.xboxlive.com` | `/users/xuid({xuid})/achievements/{scid}/update` | xbl3.0 | 2 |

## Stats

See [stats.md](stats.md) for the details.

| Route | Method | Host | Path | Auth | Contract |
| --- | --- | --- | --- | --- | --- |
| `stats-hero` | POST | `userstats.xboxlive.com` | `/batch` | xbl3.0 | 2 |
| `stats-scid` | POST | `userstats.xboxlive.com` | `/batch` | xbl3.0 | 1 |
| `stats-minutes` | POST | `userstats.xboxlive.com` | `/batch` | xbl3.0 | 2 |
| `stat-write` | PATCH | `statswrite.xboxlive.com` | `/stats/users/{xuid}/scids/{scid}` | xbl3.0 | 4 |

## Presence

See [presence.md](presence.md) for the details.

| Route | Method | Host | Path | Auth | Contract |
| --- | --- | --- | --- | --- | --- |
| `presence-heartbeat` | POST | `presence-heartbeat.xboxlive.com` | `/users/xuid({xuid})/devices/current/` | app-token | 3 |
| `presence-title` | POST | `userpresence.xboxlive.com` | `/users/xuid({xuid})/devices/current/titles/current` | xbl3.0 | 3 |
| `presence-device` | POST | `userpresence.xboxlive.com` | `/users/xuid({xuid})/devices/current` | xbl3.0 | 3 |

## Messaging and notifications

See [messaging.md](messaging.md) for the details.

| Route | Method | Host | Path | Auth | Contract |
| --- | --- | --- | --- | --- | --- |
| `messaging-inbox` | GET | `xblmessaging.xboxlive.com` | `/network/xbox/users/me/inbox` | xbl3.0 | 1 |
| `messaging-inbox-secondary` | GET | `xblmessaging.xboxlive.com` | `/network/xbox/users/me/inbox/Secondary` | xbl3.0 | 1 |
| `messaging-groups` | GET | `xblmessaging.xboxlive.com` | `/network/xbox/users/me/groups` | xbl3.0 | 1 |
| `messaging-safety` | GET | `xblmessaging.xboxlive.com` | `/network/xbox/users/xuid({xuid})/safetySettings` | xbl3.0 | 2 |
| `inbox-assist` | GET | `inbox.assist.gameservices.xboxlive.com` | `/messages` | xbl3.0 | - |
| `notification-inbox-xuid` | GET | `notificationinbox.xboxlive.com` | `/users/xuid({xuid})/inbox` | xbl3.0 | - |
| `notification-inbox-me` | GET | `notificationinbox.xboxlive.com` | `/users/me/inbox` | xbl3.0 | - |
| `notify-create` | POST | `notify.xboxlive.com` | `/system/notifications/endpoints` | xbl3.0 | 1 |
| `notify-delete` | DELETE | `notify.xboxlive.com` | `/system/notifications/endpoints/{endpoint_id}` | xbl3.0 | 1 |
| `chat-connect` | GET | `chat.xboxlive.com` | `/chat/connect` | xbl3.0 | - |
| `chat-auth` | GET | `chat.xboxlive.com` | `/users/xuid({xuid})/chat/auth` | xbl3.0 | - |
| `rta-connect` | GET | `rta.xboxlive.com` | `/connect` | xbl3.0 | - |
| `rta-nonce` | GET | `rta.xboxlive.com` | `/nonce` | xbl3.0 | - |

## Media

See [media.md](media.md) for the details.

| Route | Method | Host | Path | Auth | Contract |
| --- | --- | --- | --- | --- | --- |
| `gameclips` | GET | `gameclipsmetadata.xboxlive.com` | `/users/xuid({xuid})/clips` | xbl3.0 | - |
| `gameclips-me` | GET | `gameclipsmetadata.xboxlive.com` | `/users/me/clips` | xbl3.0 | - |
| `screenshots` | GET | `screenshotsmetadata.xboxlive.com` | `/users/xuid({xuid})/screenshots` | xbl3.0 | - |
| `screenshots-me` | GET | `screenshotsmetadata.xboxlive.com` | `/users/me/screenshots` | xbl3.0 | - |

## Cloud gaming and consoles

See [cloud-gaming.md](cloud-gaming.md) for the details.

| Route | Method | Host | Path | Auth | Contract |
| --- | --- | --- | --- | --- | --- |
| `xccs-devices` | GET | `xccs.xboxlive.com` | `/lists/devices` | xbl3.0 | 4 |
| `xhome-login` | POST | `xhome.gssv-play-prod.xboxlive.com` | `/v2/login/user` | none | - |
| `xgpupcapp-login` | POST | `xgpupcapp.gssv-play-prod.xboxlive.com` | `/v2/login/user` | none | - |
| `uks-login` | POST | `uks.core.gssv-play-prodxhome.xboxlive.com` | `/v2/login/user` | none | - |

## Telemetry and adjacent

See [telemetry.md](telemetry.md) for the details.

| Route | Method | Host | Path | Auth | Contract |
| --- | --- | --- | --- | --- | --- | 
| `onecollector-v10` | POST | `v10.events.data.microsoft.com` | `/OneCollector/1.0/` | events-token | - |
| `onecollector-mobile` | POST | `mobile.events.data.microsoft.com` | `/OneCollector/1.0/` | events-token | - |
| `arc-impression` | POST | `arc.msn.com` | `/v3/Delivery/Events/Impression` | none | - |
| `substrate-profile` | GET | `substrate.office.com` | `/profileb2/v2.0/me/V1Profile` | microsoft-account | - |
| `substrate-image` | GET | `substrate.office.com` | `/imageB2/v1.0/me/image/$value` | microsoft-account | - |
