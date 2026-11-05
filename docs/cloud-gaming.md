# Cloud gaming and consoles

The cloud-gaming stack takes a token in the body and returns a short-lived streaming token. The console list is separate and uses the normal XBL3.0 header.

## Consoles

```http
GET https://xccs.xboxlive.com/lists/devices
Authorization: XBL3.0 x=...
x-xbl-contract-version: 4
```

Response:

```json
{
  "result": [],
  "agentUserId": null,
  "status": { "errorCode": "OK", "errorMessage": null }
}
```

`result[]` is the list of linked consoles and devices. An empty list is normal when the account has no console or when the console is offline. The app uses this for the Consoles panel and the remote-play entry point.

## Cloud login

There are three hosts with the same route:

| Host | Offering | Captured result |
| --- | --- | --- |
| `xhome.gssv-play-prod.xboxlive.com` | `xhome` | 200 |
| `xgpupcapp.gssv-play-prod.xboxlive.com` | `XGPUPCAPP` | 403 `InvalidCountry` |
| `uks.core.gssv-play-prodxhome.xboxlive.com` | `xhome` | 200 |

The body:

```json
{
  "offeringId": "xhome",
  "token": "..."
}
```

The token comes from the Xbox auth chain. The response:

```json
{
  "offeringSettings": {
    "allowRegionSelection": false,
    "regions": [
      {
        "name": "UkSouth",
        "baseUri": "https://uks.core.gssv-play-prodxhome.xboxlive.com",
        "isDefault": true,
        "fallbackPriority": -1
      }
    ],
    "clientCloudSettings": {
      "Environments": [
        { "Name": "Prod", "AuthBaseUri": null }
      ]
    }
  },
  "market": "RO",
  "gsToken": "...",
  "tokenType": "bearer",
  "durationInSeconds": 14400
}
```

`gsToken` is a credential. It is valid for four hours. Do not log or publish it.

`offeringSettings.regions[]` is the important part for a client. It gives the base URI for each region and marks the default. The `uks` host in the index is the base URI that the `xhome` login returned for one account. A client should use the `baseUri` from the response instead of hard-coding a region.

`xgpupcapp` returned 403 `InvalidCountry` for my region (Austria). The route is live but the offering is region-gated.

## Example

[`cloud-login.json`](../examples/cloud-login.json).

## What is not here

The streaming session, input channel and media transport are not HTTP APIs. The login handshake is documented because the app performs it and because it is the entry point for everything else.

The old `xgpuweb.gssv-play-prod.xboxlive.com/v2/login` path returned 404. The current `xhome` and `xgpupcapp` hosts are the ones that answer.
