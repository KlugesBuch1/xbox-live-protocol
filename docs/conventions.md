# Conventions
## Route notation

Paths are written as they appear on the wire, with placeholders in braces:

```text
/users/xuid({xuid})/titles/titleId({title_id})/decoration/detail,image,scid,productId
```

| Placeholder | Value |
| --- | --- |
| `{xuid}` | Numeric Xbox user id |
| `{xuids}` | Comma-separated xuid list |
| `{title_id}` | Decimal title id |
| `{product_id}` | 12-character Store product id |
| `{scid}` | Service config id, a UUID |
| `{pfn}` | Package family name |
| `{gamertag}` | Percent-encoded gamertag |
| `{club_id}` | Numeric club id |
| `{endpoint_id}` | UUID returned by the notification registration |
| `{sigl}` | Game Pass group id |
| `{market}` | Two-letter market code, for example `AT` |
| `{locale}` | BCP 47 locale, for example `en-US` |

Parentheses are part of the path. They are not optional. `xuid(123)` and `xuid/123` are different routes.

## Headers

| Header | Value | When |
| --- | --- | --- |
| `Authorization` | `XBL3.0 x={uhs};{token}` | Most Xbox hosts |
| `X-Xbl-Contract-Version` | `1`, `2`, `3`, `4` or `7` | Per route. Some routes reject a missing header |
| `Accept` | `application/json` | Most routes |
| `Accept-Language` | `en-US` or a full locale | Profile, titles, achievements. The app often uses the Windows locale |
| `Content-Type` | `application/json; charset=utf-8` | JSON bodies |
| `Signature` | Base64 PoP block | Device and SISU only |
| `User-Agent` | Client-specific | Some routes accept anything |
| `authxtoken`, `tickets` | `x:XBL3.0 x={uhs};{token}` | OneCollector |

`X-Xbl-Contract-Version` is the closest thing the API has to a version. A route can return a different shape or response without it. Keep the value from the entry.

`Accept-Language` is used for title names and achievement descriptions. A request without it usually falls back to `en-US`.

## Status codes

| Status | What it usually proves |
| --- | --- |
| 200 | The route exists and the request was accepted |
| 201, 202, 204 | The route exists and the write was accepted |
| 304 | The route exists and the cached copy is still valid |
| 400 | The route exists - a parameter, header or body is missing or malformed |
| 401 | The route exists - an identity is required |
| 403 | The route exists - the identity is not allowed |
| 404 | The route or the object does not exist. Check the notes |
| 405 | The route exists but the method is wrong |
| 415 | The route exists but the content type or body is wrong |
| 429 | The route exists and you are going too fast |

## Observed and required

The `auth` field in the index says what the client sent, not what the server requires. A capture cannot prove that a header is required. It can only prove that it was present.

The same goes for query parameters. The app sends `maxItems=10000` on the title history route - that does not mean 10000 is required. A smaller value works.

When a route has a required header, the doc says so. When it does not, treat the header list as a starting point.

## Errors

Most Xbox hosts return a JSON error with a `code` and a `description`, or a short string. The code is service-specific. e.g.:

```json
{ "code": 1028, "description": "Official Club does not exist." }
```

```json
{ "code": 2038, "source": "TitleStorage", "description": "The requested storage container does not have a lock." }
```

Do not map a numeric code to an HTTP status without checking the route. The same number can mean different things on different hosts.

## IDs

- An xuid is a 64-bit decimal number. It is not the gamertag and it is not the Microsoft account id.
- A title id is a decimal number, usually 7 to 10 digits. The modern id and the Xbox 360 id differ.
- A product id is a 12-character base32-like string. It is used by the Store, not by titlehub.
- A service config id is a UUID. It is per title, not per account.
- A package family name looks like `Name_PublisherId`. It is used by the installed app and by the `pfn()` routes.

## Time

Timestamps in responses are UTC ISO 8601, mostly with seven fractional digits. The stat write timestamp uses six. The PoP signature uses FILETIME, which is a different clock - see [signing.md](signing.md).

## Rate limits

There is no published rate limit. The capture hit a 429 on the title card route after about 60 calls in a short window. A retry with a delay is usually enough.

Do not parallelize a whole library refresh. The app requests one card per visible tile and reuses the response.

## Redaction

The catalogue does not contain tokens, gamertags, real xuids or real message bodies. Examples use `{xuid}` and `Contoso`.

If you publish a capture, redact at least:

- `Authorization`, `x-ms-authorization`, `x-ms-authorization-social`
- `Cookie`, `Set-Cookie`
- `Signature`, `authxtoken`, `tickets`
- `endpointUri`, `AuthKey`, `gsToken`
- JWT-shaped strings
- gamertags and real xuids
