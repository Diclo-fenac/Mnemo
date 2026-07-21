# Mnemo

> Your clipboard, with memory.

Mnemo is a local-first desktop clipboard manager that reconstructs research sessions, relates copied items across time, and remembers the context around what you saved. It uses local storage and local intelligence only: no account, cloud, or server is required.

**Project:** [github.com/Diclo-fenac/Mnemo](https://github.com/Diclo-fenac/Mnemo)

## Why Mnemo

Mnemo was inspired by a common research problem: a clipboard remembers the
text, but not why it was copied or how it relates to something copied later.
Mnemo turns scattered snippets, links, commands, and notes into a private
research memory. It can reconstruct a session, show the available sources and
topics, connect related clips across sessions, and produce a short grounded
answer that links back to the original evidence.

## Judge demo flow

1. Launch Mnemo and complete onboarding. Capture starts paused by design.
2. Enable **Clipboard Capture** from the sidebar or Settings.
3. Copy a few safe snippets from documentation, a code editor, and a terminal.
4. Open **Timeline** to see the captures grouped into a research session.
5. Press `Ctrl/Cmd + K`, search for a topic, and inspect the cited clips in the
   grounded answer.
6. Open **Memory graph** to explore semantic and temporal connections.
7. Optional: enable **Browser Context**, install the Context Bridge extension,
   and copy from a web page to attach its verified URL and title.

The first local embedding model downloads after onboarding. Keyword search,
capture, sessions, and the rest of the interface remain usable while the model
prepares or if model download is unavailable.

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

## Optional AI providers

Core Mnemo functionality does not require an API key. Local Ollama support is
optional, and OpenAI or Gemini can be configured in Settings for grounded
answers. Cloud answers require explicit consent for each request and receive
only selected excerpts from the local search results. API keys are not included
in this repository or required for judging.

## Built with Codex and GPT-5.6

Codex, powered by GPT-5.6, was used as an AI engineering collaborator during
development. It helped inspect the existing Rust and React codebase, reason
through the local-first architecture, implement and refine clipboard capture,
SQLite migrations, embedding/search behavior, session reconstruction, graph
connections, browser-context integration, optional AI providers, and release
documentation.

GPT-5.6 was also used to challenge design decisions around privacy, source
provenance, prompt size, citation validation, model migration, fallback
behavior, and cross-platform active-window detection. The implementation was
reviewed and verified through Rust compilation/tests, frontend builds/tests,
extension manifest checks, and manual product decisions. Codex did not replace
testing or product judgment: provider credentials, desktop permissions, and
real browser-extension behavior still require validation on the target machine.

## Known beta limitations

- Active-window metadata is best-effort and varies by desktop environment.
- Linux Wayland may provide less metadata than X11; `xdotool` is an optional
  X11 enhancement, not a hard dependency.
- The browser extension must be installed separately and Browser Context must
  be enabled in Mnemo.
- OpenAI and Gemini require the tester's own API key; the local and Ollama paths
  do not.

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
