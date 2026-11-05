# Store and catalogue

The store is the part of the app that works without an Xbox token. Product metadata, Game Pass lists and the service status page are public or semi-public, which makes them useful for tools that do not want to handle a full sign-in.

## Routes

| Route | Auth | Purpose |
| --- | --- | --- |
| `GET displaycatalog.mp.microsoft.com/v7.0/products/{product_id}` | none | Product metadata, images, SKUs |
| `GET storeedgefd.dsx.mp.microsoft.com/v9.0/products/{product_id}` | none | Store edge product data |
| `GET catalog.gamepass.com/sigls/v2?id={sigl}&market={market}&language={locale}` | none | Product ids in a Game Pass group |
| `GET catalog.gamepass.com/misc/pc-pfns-list` | none | Package family names in the PC Game Pass catalogue |
| `GET beige.xboxservices.com/pcgafd/config/offlinepc` | XBL3.0 | Offline-permission settings |
| `GET xnotify.xboxlive.com/servicestatusv6/{market}/{locale}` | none | Xbox service status |

## displaycatalog

```http
GET https://displaycatalog.mp.microsoft.com/v7.0/products/9MV0B5HZVK9Z?market=AT&languages=en-US
```

`market` is a two-letter market code and `languages` is a comma-separated list. The response:

```json
{
  "Product": {
    "ProductId": "9MV0B5HZVK9Z",
    "ProductType": "Game",
    "Title": "Halo Infinite",
    "PublisherName": "Xbox Game Studios",
    "Description": "...",
    "ReleaseDate": "2021-12-08T00:00:00Z",
    "Images": [
      { "Url": "https://store-images.s-microsoft.com/...", "ImagePurpose": "BoxArt" }
    ],
    "Properties": {
      "Categories": ["Games"],
      "Keywords": ["Halo"]
    }
  }
}
```

The real response is nested more deeply than this. The useful paths are `Product.LocalizedProperties[0].ProductTitle`, `Product.LocalizedProperties[0].PublisherName`, `Product.LocalizedProperties[0].Images[]` and `Product.Properties.Categories[]`.

The product id is the 12 character id from the Store URL, not the title id. A title can have several product ids (base game, bundle, DLC), which is why the title response has `productIds[]` and `productIdsWithDeviceTypes[]`.

## Store edge

```http
GET https://storeedgefd.dsx.mp.microsoft.com/v9.0/products/9MV0B5HZVK9Z?market=AT&locale=en-US&deviceFamily=Windows.Desktop
```

The response has `Product`, `ProductIds`, `SkuIds` and availability data. The fields are close to displaycatalog but the casing and nesting differ. 

`deviceFamily=Windows.Desktop` matters for availability. A product that is available on console can be unavailable on PC.

## Game Pass lists

A SIGL is an opaque group id. The catalogue returns product ids for that group:

```http
GET https://catalog.gamepass.com/sigls/v2?id=f6f1f99f-9b49-4ccd-b3bf-4d9767a77f5e&market=AT&language=en-US
```

The body is a JSON array of product id strings. It does not include titles, images or prices - you need displaycatalog for each id.

`misc/pc-pfns-list` returns the package family names for the PC catalogue. The app caches it with an ETag, so a warm start returns 304. The body is a JSON array of package family names, not product ids.

These lists change. Do not hard-code a SIGL id.

## Offline permissions

`GET beige.xboxservices.com/pcgafd/config/offlinepc` needs the XBL3.0 header. The response:

```json
{
  "offlinePermissions": {
    "enabled": false,
    "requestData": {
      "success": true,
      "maxTogglesPerPeriod": 3,
      "remainingTogglesForPeriod": 3,
      "periodLengthDays": 365,
      "nextTimeToAdd": "1601-01-01T00:00:00+00:00"
    }
  }
}
```

`enabled` is the current setting. `remainingTogglesForPeriod` is how many times the user can change it before `nextTimeToAdd`. The 1601 date means there is no cooldown.

## Service status

`GET xnotify.xboxlive.com/servicestatusv6/{market}/{locale}` returns the status page data:

```json
{
  "Status": {
    "Overall": { "State": "None", "Id": 1, "LastUpdated": "..." },
    "SelectedScenarios": { "State": "None", "Id": 1, "LastUpdated": "..." }
  },
  "CoreServices": [],
  "Titles": []
}
```

The captured request had a double slash before the market:

```text
/servicestatusv6//AT/en-US
```

The server accepted it. The clean form is `/servicestatusv6/AT/en-US` - both work.

`State` is `None` for normal operation. The app uses this to show the service-status banner.

## Examples

[`displaycatalog-product.json`](../examples/displaycatalog-product.json) and [`gamepass-sigls.json`](../examples/gamepass-sigls.json).

## What is not here

Purchases, billing and payment instruments are out of scope. The app has routes under `purchase.mp.microsoft.com` and `paymentsinstruments.mp.microsoft.com` but they are not documented here.
