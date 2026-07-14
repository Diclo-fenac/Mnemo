# Mnemo

> Your clipboard, with memory.

Mnemo is a local-first desktop clipboard manager that reconstructs research sessions, relates copied items across time, and remembers the context around what you saved. It uses local storage and local intelligence only: no account, cloud, or server is required.

## Foundation

The current Milestone 1 workspace provides the Tauri + React application shell, SQLite schema boot path, route placeholders, and the visual system. Clipboard capture, embeddings, semantic search, and session intelligence follow in later milestones.

## Development

```bash
npm install
npm run tauri dev
```

Rust stable and Tauri's platform prerequisites are required. See the planning documents in `plan/` for the architecture and feature breakdown.
