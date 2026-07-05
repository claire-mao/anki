# GRE Atlas sync — local dev setup

Copy-paste workflow for exercising GRE Atlas practice sync (`greatlas.db` sidecar) against the bundled **SimpleServer**. AnkiWeb does not expose `/gre/sync/*`; you need a self-hosted server from this repo.

GRE routes are registered automatically in `SimpleServer` (`rslib/src/sync/http_server/mod.rs`):

- `POST /gre/sync/download`
- `POST /gre/sync/upload`

Both require the standard `anki-sync` header (hkey + sync version 11) and a zstd-compressed body, same as collection sync.

## 1. Start SimpleServer (terminal 1)

From the repo root, pick a port (default `8080`; use another if busy).

**Important:** users and passwords are read from `SYNC_USER*` env vars **only at server startup**. If you change `SYNC_USER1`, stop the old process and start a new one with the updated env. `SYNC_BASE` stores collection/media files per user; it does **not** override login credentials.

```bash
export SYNC_USER1=dev:dev
export SYNC_HOST=127.0.0.1
export SYNC_PORT=8080
export SYNC_BASE="$HOME/.syncserver-gre-dev"
mkdir -p "$SYNC_BASE"

cargo run -p anki-sync-server
```

Wait for `listening` on `127.0.0.1:8080`.

If port 8080 is already in use, kill the old sync-server or pick another port (`SYNC_PORT=18080`) and update client URLs. An old process may have been started without `SYNC_USER1=dev:dev`.

**Alternatives (same server, GRE routes included):**

```bash
# After `just run` / build, from pyenv:
SYNC_USER1=dev:dev ./run --syncserver

# Packaged Anki (macOS):
SYNC_USER1=dev:dev /Applications/Anki.app/Contents/MacOS/anki --syncserver
```

There is no separate “enable GRE routes” flag — any SimpleServer built from this tree serves them.

## 2. Verify login before opening Anki

Run this **before** signing in from Anki. If it returns HTTP 403, the server is not accepting `dev:dev` — fix the server env/restart first; Anki login will fail the same way.

```bash
# Health
curl -s http://127.0.0.1:8080/health

# Host key (login) — must return HTTP 200 with {"key":"96fb1623..."}
echo '{"u":"dev","p":"dev"}' | zstd -q | curl -s -w "\nHTTP:%{http_code}\n" \
  -X POST http://127.0.0.1:8080/sync/hostKey \
  -H 'anki-sync: {"v":11,"k":"","c":"dev","s":"0123456789abcdef"}' \
  --data-binary @-

# GRE download (empty bundle on fresh server)
HKEY=$(python3 -c "import hashlib; print(hashlib.sha1(b'dev:dev').hexdigest())")
echo '{"after_usn":0}' | zstd -q | curl -s -X POST http://127.0.0.1:8080/gre/sync/download \
  -H "anki-sync: {\"v\":11,\"k\":\"$HKEY\",\"c\":\"dev\",\"s\":\"0123456789abcdef\"}" \
  --data-binary @-
# → {"bundle":[]} with HTTP 200
```

The hkey for `SYNC_USER1=dev:dev` is always `sha1("dev:dev")`:

```bash
python3 -c "import hashlib; print(hashlib.sha1(b'dev:dev').hexdigest())"
# 96fb1623e2b9a953fcc5cd9e8da27b6916ae0a09
```

## 3. Configure desktop Anki (terminal 2)

```bash
just run
```

In Anki **Preferences → Syncing**:

1. In **Custom sync server**, set URL to `http://127.0.0.1:8080/` (**trailing slash required**). There is no separate checkbox — any non-empty URL switches away from AnkiWeb.
2. Click **Sign in** on the same Syncing tab (this saves the URL before login). **Do not** use your AnkiWeb email or AnkiWeb password — the dialog still says “AnkiWeb ID”, but for self-hosted sync you must enter the credentials from `SYNC_USER1`:
   - **Username:** `dev`
   - **Password:** `dev`
3. Click **OK** on Preferences to persist the custom URL.
4. Run a normal collection sync once (uploads collection to SimpleServer).
5. Open GRE **Settings → Practice sync → Sync practice data now** (or finish a practice session).

If you sign in from GRE **Settings → Account → Sign in** instead, save Preferences (step 3) first — that button does not save an unsaved custom sync URL.

Desktop reads `hkey` and endpoint from the signed-in profile (`syncKey` + custom sync URL); no manual hkey entry on desktop.

## 4. Configure iOS companion

**Simulator:** use `http://127.0.0.1:8080/`.

**Physical device:** use your Mac’s LAN IP, e.g. `http://192.168.1.42:8080/`. Ensure the device can reach the Mac (same Wi‑Fi; allow local network for the app if prompted).

In the companion app:

1. **Settings → GRE Atlas sync**
2. **Sync server URL:** `http://127.0.0.1:8080/` (or LAN URL)
3. **Host key:** paste the hkey from step 2 (`96fb1623…` for `dev:dev`) or from desktop after sign-in
4. **Save credentials** → **Sync now**

Credentials are stored in `UserDefaults` key `greAtlasSyncCredentials` (JSON: `hkey`, `endpoint`, `ioTimeoutSecs`).

## 5. End-to-end smoke test

| Step | Action                                                        | Expected                                                                 |
| ---- | ------------------------------------------------------------- | ------------------------------------------------------------------------ |
| A    | Practice on desktop → GRE sync                                | Settings shows merged counts; `$SYNC_BASE/dev/gre_atlas_sync.pb` appears |
| B    | **Sync now** on iOS                                           | Progress attempt count matches desktop                                   |
| C    | Practice on iOS → **Sync now**                                | Desktop GRE Progress updates                                             |
| D    | Airplane mode on both → practice each → reconnect → sync both | Attempt totals merge (LWW on `mtime_secs`)                               |

## Troubleshooting

| Symptom                                                                                       | Fix                                                                                                                                                                                                                                                                                                                                           |
| --------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| “Email or password was incorrect” / server log `invalid user/pass in get_host_key` (HTTP 403) | Anki is hitting your server but sending the wrong credentials. Use username `dev` and password `dev` — **not** your AnkiWeb email/password. Confirm with the curl in step 2; if curl is 403, restart the server in a shell that has `export SYNC_USER1=dev:dev` (and unset `PASSWORDS_HASHED` unless you intentionally use hashed passwords). |
| “GRE Atlas practice sync requires a self-hosted…”                                             | Custom sync URL still points at AnkiWeb, or server lacks GRE routes (use this repo’s SimpleServer).                                                                                                                                                                                                                                           |
| HTTP 404 on upload                                                                            | Wrong base URL (missing trailing `/`) or old server binary without `/gre/sync/*`.                                                                                                                                                                                                                                                             |
| HTTP 403 on GRE sync (after login works)                                                      | Stale hkey — sign out and sign in again, or re-fetch via `/sync/hostKey`.                                                                                                                                                                                                                                                                     |
| iOS cannot connect (device)                                                                   | Use LAN IP, not `127.0.0.1`; check firewall; toggle local network access for the app.                                                                                                                                                                                                                                                         |
| `Address already in use`                                                                      | Change `SYNC_PORT` (e.g. `18080`) and update client URLs.                                                                                                                                                                                                                                                                                     |
| Want a clean server state                                                                     | Stop the server, `rm -rf "$SYNC_BASE"`, recreate the directory, restart with the same `SYNC_USER1`. This resets collections/media, not login rules.                                                                                                                                                                                           |

## Reference

- Server env vars: [sync-server manual](https://docs.ankiweb.net/sync-server.html) (`SYNC_USER1`, `SYNC_BASE`, `SYNC_HOST`, `SYNC_PORT`)
- Automated merge tests: `docs/gre-atlas-submission/SYNC-VERIFICATION.md`
- Transport: `rslib/src/gre_atlas/sync_transport.rs`
- Handlers: `rslib/src/sync/http_server/gre_atlas_sync.rs`
