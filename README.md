# Mnemo

> Your clipboard, with memory.

Mnemo is a local-first desktop clipboard manager that reconstructs research sessions, relates copied items across time, and remembers the context around what you saved. It uses local storage and local intelligence only: no account, cloud, or server is required.

## What Mnemo does

Mnemo captures clipboard items locally when you explicitly enable capture,
groups them into research sessions, connects related memories, and lets you
search by exact text or meaning. Browser context is optional: the Chrome or
Firefox Context Bridge can attach a copied page's URL and title through a
loopback-only connection. No account, cloud sync, or remote clipboard server is
required.

The private beta starts with capture paused and retention set to Never. The
local semantic model downloads after onboarding; keyword search remains usable
while it prepares or when the download is unavailable.

## Private beta install

Download the signed Linux AppImage from the latest GitHub Release, make it
executable, and launch it:

```bash
chmod +x Mnemo.AppImage
./Mnemo.AppImage
```

The AppImage is the supported beta distribution. It creates a fresh local
database on first launch. Follow the optional browser setup in
[`extension/README.md`](extension/README.md) for Chrome/Chromium and Firefox.

## Privacy model

- Capture is off until the user enables it.
- `CmdOrCtrl+Shift+V` searches existing memories even when capture is paused.
- `CmdOrCtrl+Shift+M` toggles real clipboard capture.
- Browser context is opt-in and sends only explicit copy context to
  `127.0.0.1:17531`.
- Retention defaults to Never; automatic cleanup is an explicit setting.
- If source metadata cannot be verified, Mnemo says `Source unavailable` and
  does not guess from the clip text.

## Development

```bash
npm install
npm run tauri dev
```

Rust stable and Tauri's platform prerequisites are required. See the planning documents in `plan/` for the architecture and feature breakdown.

```bash
npm test
npm run build
npm run package:extension -- --verify
cargo test --manifest-path src-tauri/Cargo.toml
```

## Release notes

Signed update checks use the GitHub Release `latest.json` feed. The updater
private key must remain outside the repository and be provided to the release
workflow as `TAURI_SIGNING_PRIVATE_KEY`. See
[`docs/private-beta.md`](docs/private-beta.md) before inviting testers.
