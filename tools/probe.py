#!/usr/bin/env python3
"""Send a tokenless request to the read routes in endpoints.yaml.

The probe does not log in and does not send an Authorization header. A 401 or
403 prove the route exists. A 404 means the path is dead or a missing object.
"""

import argparse
import ssl
import sys
import urllib.error
import urllib.parse
import urllib.request
from pathlib import Path

try:
    import yaml
except ImportError:
    sys.exit("pyyaml is required: python -m pip install pyyaml")

ROOT = Path(__file__).resolve().parent.parent
DEFAULTS = {
    "xuid": "0000000000000000",
    "xuids": "0000000000000000",
    "title_id": "704208617",
    "product_id": "9MV0B5HZVK9Z",
    "scid": "a5ee0100-7093-4a78-b098-471729f95ee9",
    "pfn": "Microsoft.GamingApp_8wekyb3d8bbwe",
    "gamertag": "Contoso",
    "club_id": "2043073184",
    "endpoint_id": "00000000-0000-0000-0000-000000000000",
    "sigl": "f6f1f99f-9b49-4ccd-b3bf-4d9767a77f5e",
    "market": "AT",
    "locale": "en-US",
}
QUERY_DEFAULTS = {
    "settings": "Gamertag",
    "maxItems": "10",
    "titleId": "704208617",
    "query": "halo",
    "id": DEFAULTS["sigl"],
    "market": "AT",
    "language": "en-US",
    "languages": "en-US",
    "locale": "en-US",
    "deviceFamily": "Windows.Desktop",
}
SKIP_HOSTS = {"dbox.tools"}


def fill(text):
    for key, value in DEFAULTS.items():
        text = text.replace("{" + key + "}", value)
    return text


def build_url(entry):
    url = "https://" + entry["host"] + fill(entry["path"])
    pairs = []
    for key in entry.get("query", []):
        if key in QUERY_DEFAULTS:
            pairs.append((key, QUERY_DEFAULTS[key]))
    if pairs:
        url += "?" + urllib.parse.urlencode(pairs)
    return url


def request(entry, timeout, context):
    url = build_url(entry)
    headers = {
        "User-Agent": "xbox-live-protocol-probe/1.0",
        "Accept": "application/json",
    }
    contract = str(entry.get("contract", ""))
    if contract:
        headers["X-Xbl-Contract-Version"] = "2" if "or" in contract else contract
    data = None
    if entry["method"] in ("POST", "PATCH", "PUT"):
        headers["Content-Type"] = "application/json"
        data = b"{}"
    req = urllib.request.Request(url, data=data, headers=headers, method=entry["method"])
    try:
        with urllib.request.urlopen(req, timeout=timeout, context=context) as response:
            return url, response.status, None
    except urllib.error.HTTPError as exc:
        return url, exc.code, None
    except Exception as exc:  # noqa: BLE001 - the report wants the reason
        return url, None, f"{type(exc).__name__}: {exc}"


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--area", help="only probe one area")
    parser.add_argument("--timeout", type=float, default=12.0)
    args = parser.parse_args()

    data = yaml.safe_load((ROOT / "endpoints.yaml").read_text(encoding="utf-8"))
    endpoints = [
        e for e in data["endpoints"]
        if e.get("class") == "read" and e["host"] not in SKIP_HOSTS
    ]
    if args.area:
        endpoints = [e for e in endpoints if e.get("area") == args.area]
    if not endpoints:
        sys.exit("no read routes to probe")

    context = ssl.create_default_context()
    width = max(len(e["id"]) for e in endpoints)
    failures = 0

    print(f"{'route':<{width}}  {'want':>4}  {'got':>4}  result")
    for entry in endpoints:
        url, status, error = request(entry, args.timeout, context)
        want = str(entry.get("probe_status", entry.get("status", "")))
        if error:
            failures += 1
            print(f"{entry['id']:<{width}}  {want:>4}  {'--':>4}  {error}")
            continue
        note = "ok" if str(status) == want else "different"
        print(f"{entry['id']:<{width}}  {want:>4}  {status:>4}  {note}  {url}")

    return 1 if failures else 0


if __name__ == "__main__":
    raise SystemExit(main())
