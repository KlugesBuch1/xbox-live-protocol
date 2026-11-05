# Messaging, notifications and chat

There are four different things in this area that look similar:

- Xbox Live conversations on `xblmessaging.xboxlive.com`
- The notification inbox on `notificationinbox.xboxlive.com`
- Service messages on `inbox.assist.gameservices.xboxlive.com`
- Push registration on `notify.xboxlive.com`

They use different contracts and different auth.

## Xbox Live conversations

| Route | Auth | Contract |
| --- | --- | --- |
| `GET xblmessaging.xboxlive.com/network/xbox/users/me/inbox` | XBL3.0 | 1 |
| `GET xblmessaging.xboxlive.com/network/xbox/users/me/inbox/Secondary` | XBL3.0 | 1 |
| `GET xblmessaging.xboxlive.com/network/xbox/users/me/groups` | XBL3.0 | 1 |
| `GET xblmessaging.xboxlive.com/network/xbox/users/xuid({xuid})/safetySettings` | XBL3.0 | 2 |

The inbox response:

```json
{
  "primary": {
    "folder": "Primary",
    "totalCount": 127,
    "unreadCount": 0,
    "conversations": [
      {
        "conversationId": "00000000-0000-0000-0000-000000000000",
        "participants": ["0", "{xuid}"],
        "isRead": true,
        "muted": false,
        "lastMessage": {
          "contentPayload": {
            "content": {
              "parts": [
                { "contentType": "text", "text": "..." },
                { "contentType": "deeplink", "buttonText": "...", "appUri": "..." }
              ]
            }
          }
        }
      }
    ]
  },
  "folders": [],
  "safetySettings": {}
}
```

`participants` are strings. `"0"` is the Xbox system account.

The secondary inbox is where message requests land. The app calls it separately so it can show a badge without loading the primary list.

`groups` is the group-conversation list. It is not the same as a group chat - it is the list of conversations that have more than two participants or a named group.

`safetySettings` is per account:

```json
{
  "settings": {
    "version": 3,
    "primaryInboxMedia": "Mature",
    "primaryInboxText": "AllowAll",
    "primaryInboxUrl": "Mature",
    "secondaryInboxMedia": "Medium",
    "secondaryInboxText": "Medium",
    "secondaryInboxUrl": "BlockAll",
    "canUnobscure": true
  }
}
```

The app reads this before rendering filtered content.

## Notification inbox

| Route | Auth | Status without a token |
| --- | --- | --- |
| `GET notificationinbox.xboxlive.com/users/xuid({xuid})/inbox` | XBL3.0 | 401 |
| `GET notificationinbox.xboxlive.com/users/me/inbox` | XBL3.0 | 401 |

This is the notification service. It carries system notifications, achievement notifications and social notifications. The app uses the xuid form because it already has the id.

The response shape was not captured.

## Service messages

`GET inbox.assist.gameservices.xboxlive.com/messages` returns:

```json
{
  "Messages": [],
  "Errors": []
}
```

The app sends an authorization header. The capture does not show the header form because the value was redacted. This inbox carries support and service messages.

## Push registration

`POST notify.xboxlive.com/system/notifications/endpoints` registers a Windows Push Notification channel:

```json
{
  "systemId": "{00000000-0000-0000-0000-000000000000}",
  "endpointUri": "https://wns2-example.invalid/?token=...",
  "platform": "windowsOneCore",
  "platformVersion": "SG.1.0",
  "deviceName": "windows",
  "locale": "de-AT",
  "titleId": 704208617,
  "filters": [
    { "action": "Include", "source": 6, "type": 1 },
    { "action": "Include", "source": 6, "type": 8 }
  ]
}
```

The response is:

```json
{ "endpointId": "00000000-0000-0000-0000-000000000000" }
```

`endpointUri` is a WNS channel and is a credential. Do not publish it. The `filters` list decides which notification sources are delivered.

`DELETE notify.xboxlive.com/system/notifications/endpoints/{endpoint_id}` returns 202 with an empty body. The app calls it on sign-out and when it rotates the channel.

Both routes use contract 1. Both change account state. Use a test account.

## Chat

| Route | Auth | Status |
| --- | --- | --- |
| `GET chat.xboxlive.com/chat/connect` | XBL3.0 | 101 |
| `GET chat.xboxlive.com/users/xuid({xuid})/chat/auth` | XBL3.0 | 200 |

`chat/connect` is a WebSocket upgrade. The 101 is the handshake - the body after that is not HTTP. The socket carries chat messages and presence events.

`chat/auth` returns:

```json
{ "AuthKey": "00000000-0000-0000-0000-000000000000" }
```

The key is a short-lived credential. The app requests it immediately before opening the socket. Do not publish it.

## Real-time activity

| Route | Auth | Status |
| --- | --- | --- |
| `GET rta.xboxlive.com/connect` | XBL3.0 | 101 |
| `GET rta.xboxlive.com/nonce` | XBL3.0 | 200 |

`rta/nonce` returns a nonce that the client uses when opening `rta/connect`. The socket carries presence and rich-presence updates. The app uses it while a game is running.
