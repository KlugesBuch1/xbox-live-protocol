# Telemetry and adjacent services

These routes are in the index because the app calls them, not because they are part of Xbox Live. They use different auth and different data.

## OneCollector

| Route | Auth | Status without a body |
| --- | --- | --- |
| `POST v10.events.data.microsoft.com/OneCollector/1.0/` | events token | 415 |
| `POST mobile.events.data.microsoft.com/OneCollector/1.0/` | events token | 415 |

OneCollector is Microsoft's event ingestion service. The body is `application/x-json-stream` - each line is a JSON event. The headers are:

```http
authxtoken: {events token}
tickets: "1"="{events token}"
Content-Type: application/x-json-stream
```

The events token is the one SISU returns for the relying party `http://events.xboxlive.com`. It already starts with `x:`. The `tickets` header repeats it in a quoted key-value form.

The event flow in this catalogue uses the `v10` host. The `mobile` host appears in the telemetry DLL - I did not see a request to it.

The event body is application-specific. The Xbox app uses OneCollector for its own telemetry, and some titles use it for event-based achievements.

## Impressions

`POST arc.msn.com/v3/Delivery/Events/Impression` records a merchandising impression. No Xbox token. The app calls it when a promoted tile is shown. The response is XML.


## Microsoft account profile

| Route | Auth | Status |
| --- | --- | --- |
| `GET substrate.office.com/profileb2/v2.0/me/V1Profile` | Microsoft account | 401 |
| `GET substrate.office.com/imageB2/v1.0/me/image/$value` | Microsoft account | 401 |

These are Office substrate routes used for the signed-in user's name and avatar. The app calls them next to the Xbox profile - when they fail, it falls back to the Xbox gamerpic.

## Redaction

If you collect traffic, redact before the capture leaves the machine:

- `Authorization`, `x-ms-authorization`, `x-ms-authorization-social`
- `Cookie`, `Set-Cookie`
- `Signature`, `authxtoken`, `tickets`
- `endpointUri`, `AuthKey`, `gsToken`
- any string that looks like a JWT
