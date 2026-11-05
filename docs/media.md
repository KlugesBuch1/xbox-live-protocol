# Media

Game DVR clips and screenshots are two services with the same shape:

| Route | Auth | Status without a token |
| --- | --- | --- |
| `GET gameclipsmetadata.xboxlive.com/users/xuid({xuid})/clips` | XBL3.0 | 401 |
| `GET gameclipsmetadata.xboxlive.com/users/me/clips` | XBL3.0 | 401 |
| `GET screenshotsmetadata.xboxlive.com/users/xuid({xuid})/screenshots` | XBL3.0 | 401 |
| `GET screenshotsmetadata.xboxlive.com/users/me/screenshots` | XBL3.0 | 401 |

The `me` forms are the same service as the xuid forms. The app uses the xuid form because it already has the id from the profile.

The response fields were not captured. The routes are listed because a captures tab needs both hosts, and because the 401 proves the paths are live.
