# Mnemo Memory Workspace Implementation Plan

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Turn Mnemo into a private, grounded memory workspace with an information-rich dashboard, explicit capture agency, and a useful graph.

**Architecture:** Keep Tauri commands as the contract boundary. Add small Rust services and commands for capture preferences, retention, manual session corrections, feedback, and dashboard aggregates. Keep React route components focused: Dashboard composes read models, Timeline owns archive operations, Graph owns view-state derivation, and Settings owns user preferences.

**Tech Stack:** Tauri 2, Rust, rusqlite/SQLite, React 19, TypeScript, Zustand, Tailwind CSS 4, d3-force, Vitest.

---

## File Map

| Path | Responsibility |
| --- | --- |
| `src-tauri/src/services/capture_state.rs` | Atomic capture enable/disable and persisted preference helpers |
| `src-tauri/src/services/retention.rs` | Expire eligible clips without deleting pinned memories |
| `src-tauri/src/commands/dashboard.rs` | Grounded dashboard read model |
| `src-tauri/src/commands/settings.rs` | Capture, appearance, browser-context, retention, and typed-clear settings commands |
| `src-tauri/src/commands/graph.rs` | Cluster-first graph data and explicit edge/build state |
| `src-tauri/src/commands/sessions.rs` | Rename, pin, merge, and split session commands |
| `src-tauri/src/commands/feedback.rs` | Local feedback persistence and hide/show-less controls |
| `src-tauri/src/services/watcher.rs` | Respect capture state and emit generic blocked events only |
| `src-tauri/src/services/active_window.rs` | Per-platform native app/window attribution |
| `src-tauri/src/services/http_server.rs` | Local browser-context receiver and opt-in gate |
| `extension/` | Cross-browser extension that provides authoritative web provenance |
| `src-tauri/src/services/db.rs` | Preference, feedback, session-override, and migration schema |
| `src-tauri/src/lib.rs` | Command registration and capture-toggle global shortcut |
| `src/types/index.ts` | Exact TypeScript contracts for new commands |
| `src/store/app.ts` | Theme, capture, and dashboard lifecycle state |
| `src/pages/Memory.tsx` | New dashboard route |
| `src/pages/Timeline.tsx` | Archive route and manual session operations |
| `src/pages/MemoryGraph.tsx` | Cluster-first and edge-free graph states |
| `src/pages/Settings.tsx` | Theme-aware settings and destructive confirmation flow |
| `src/components/ActivityRail.tsx` | Responsive live capture activity surface |
| `src/components/GroundedBrief.tsx` | Cited answer and feedback controls |
| `src/components/CaptureControl.tsx` | Persistent capture status/toggle |
| `src/components/ThemeSelect.tsx` | Theme-safe custom select/menu |
| `src/App.tsx` | Memory and Timeline routes |
| `src/components/Sidebar.tsx` | Primary navigation and persistent capture control |
| `src/index.css` | Theme tokens, dashboard layout, graph, and safety states |

## Chunk 1: Capture Agency and Preferences

### Task 1: Persist capture and appearance preferences

**Files:**
- Modify: `src-tauri/src/services/db.rs`
- Create: `src-tauri/src/services/capture_state.rs`
- Test: `src-tauri/src/services/capture_state.rs`

- [ ] Add migration columns or a `preferences` table for `capture_enabled`,
  `appearance`, `browser_context_enabled`, and `auto_delete_days`.
- [ ] Write failing unit tests for default preferences and atomically toggling
  capture state.
- [ ] Implement a small `CaptureState` shared by watcher and commands; persist
  every change before returning success.
- [ ] Run `cargo test capture_state` and `cargo check`.

### Task 2: Make the watcher respect capture state

**Files:**
- Modify: `src-tauri/src/services/watcher.rs`
- Modify: `src-tauri/src/lib.rs`
- Test: `src-tauri/src/services/watcher.rs`

- [ ] Add a failing test that capture-disabled rejects a clipboard event before
  normalizing, filtering, or writing it.
- [ ] Check capture state before every clipboard read/persist cycle.
- [ ] Register `CmdOrCtrl+Shift+M`; toggle the shared state and emit a
  `capture-state-changed` event.
- [ ] Preserve `CmdOrCtrl+Shift+V` search behavior regardless of capture state.
- [ ] Run `cargo test`, `cargo fmt --check`, and `cargo check`.

### Task 3: Expose and render privacy controls

**Files:**
- Modify: `src-tauri/src/commands/settings.rs`
- Modify: `src/types/index.ts`
- Modify: `src/store/app.ts`
- Create: `src/components/CaptureControl.tsx`
- Modify: `src/components/Sidebar.tsx`
- Test: `src/components/CaptureControl.test.tsx`

- [ ] Add command contracts for reading/updating privacy preferences.
- [ ] Write a component test for the capture state label and disabled/enabled
  interaction.
- [ ] Render the persistent capture control in the shell with source-context
  state visible without opening Settings.
- [ ] Run `npm test` and `npm run build`.

## Chunk 2: Memory Dashboard and Activity

### Task 4: Create a grounded dashboard read model

**Files:**
- Create: `src-tauri/src/commands/dashboard.rs`
- Modify: `src-tauri/src/commands/mod.rs`
- Modify: `src-tauri/src/lib.rs`
- Test: `src-tauri/src/commands/dashboard.rs`

- [ ] Define `DashboardData` with recent clips, resumable session, connection
  health, unconnected count, and latest activity.
- [ ] Write a fixture-based command test for an empty database and a database
  with clips, sessions, and edges.
- [ ] Query only bounded records and use existing session/edge structures.
- [ ] Register `get_dashboard` and verify serialization uses camelCase.
- [ ] Run `cargo test dashboard` and `cargo check`.

### Task 5: Build the Memory dashboard

**Files:**
- Create: `src/pages/Memory.tsx`
- Create: `src/components/ActivityRail.tsx`
- Modify: `src/App.tsx`
- Modify: `src/components/Sidebar.tsx`
- Modify: `src/hooks/useClipEvents.ts`
- Modify: `src/index.css`
- Test: `src/pages/Memory.test.tsx`

- [ ] Add failing rendering tests for loading, empty, and populated dashboard
  states.
- [ ] Implement the dashboard with Ask Mnemo entry, resume card, connection
  health, recent captures, and live rail.
- [ ] Move Timeline to `/timeline`, make Memory the `/` route, and preserve
  deep links.
- [ ] Update activity rail from `clip-added` events without refetch loops.
- [ ] Make the rail a drawer below the desktop breakpoint.
- [ ] Run `npm test` and `npm run build`.

### Task 6: Add grounded brief and local feedback UX

**Files:**
- Create: `src-tauri/src/commands/feedback.rs`
- Create: `src/components/GroundedBrief.tsx`
- Modify: `src/pages/Search.tsx`
- Modify: `src/types/index.ts`
- Test: `src/components/GroundedBrief.test.tsx`

- [ ] Add a local-only feedback table and commands for vote, hide, and
  show-less-like-this.
- [ ] Ensure a brief's citations are IDs/links to source clips, never an
  ungrounded synthetic claim.
- [ ] Write tests for citation rendering and negative-feedback menu actions.
- [ ] Add extractive brief rendering before ranked search results; preserve
  current results when no brief is possible.
- [ ] Run `cargo test`, `npm test`, and `npm run build`.

## Chunk 3: Sessions, Graph, and Retention

### Task 7: Support manual session corrections

**Files:**
- Create: `src-tauri/src/commands/sessions.rs`
- Modify: `src-tauri/src/services/db.rs`
- Modify: `src/pages/Timeline.tsx`
- Test: `src-tauri/src/commands/sessions.rs`

- [ ] Add schema for session pin/label overrides and membership overrides.
- [ ] Write failing command tests for rename, pin, merge, and split validation.
- [ ] Implement transactional commands; prevent cross-session clip loss.
- [ ] Add compact archive actions with accessible confirmation and error states.
- [ ] Run `cargo test sessions`, `npm test`, and `npm run build`.

### Task 8: Replace graph dots with cluster-first behavior

**Files:**
- Modify: `src-tauri/src/commands/graph.rs`
- Modify: `src/pages/MemoryGraph.tsx`
- Modify: `src/index.css`
- Test: `src-tauri/src/commands/graph.rs`
- Test: `src/pages/MemoryGraph.test.tsx`

- [ ] Write backend tests for `building`, `edge_free`, and `connected` graph
  states.
- [ ] Return only valid links whose endpoints are part of the selected node
  set, plus cluster summaries and unconnected count.
- [ ] Add client derivation tests that exclude isolated nodes from canvas mode.
- [ ] Render cluster cards first; open a selected cluster/clip in connection
  explorer mode; label semantic and temporal edges.
- [ ] Render an explanatory edge-free state when embeddings are pending or no
  qualifying relationship exists.
- [ ] Run `cargo test graph`, `npm test`, and `npm run build`.

### Task 9: Enforce retention without harming pinned memories

**Files:**
- Create: `src-tauri/src/services/retention.rs`
- Modify: `src-tauri/src/services/watcher.rs`
- Modify: `src-tauri/src/commands/settings.rs`
- Test: `src-tauri/src/services/retention.rs`

- [ ] Write failing tests for expiry cutoff, disabled retention, and pinned
  clip preservation.
- [ ] Implement transactional removal of expired clips, embeddings, edges, and
  FTS rows using existing foreign-key behavior.
- [ ] Run the cleanup at bounded intervals, never on the UI thread.
- [ ] Expose the selected retention period in Settings.
- [ ] Run `cargo test retention` and `cargo check`.

## Chunk 4: Settings, Themes, and Final Verification

### Task 10: Make source attribution reliable across platforms

**Files:**
- Modify: `src-tauri/src/services/active_window.rs`
- Modify: `src-tauri/src/services/http_server.rs`
- Modify: `src-tauri/src/services/watcher.rs`
- Create: `extension/manifest.json`
- Create: `extension/background.js`
- Create: `extension/content.js`
- Create: `extension/README.md`
- Test: `src-tauri/src/services/active_window.rs`

- [ ] Treat extension metadata as authoritative for browser clips: URL, page
  title, favicon URL, and optional selected text.
- [ ] Keep browser provenance explicitly opt-in; reject and discard local
  context requests when the preference is disabled.
- [ ] Implement native app/window attribution per platform:
  Windows uses foreground window, title, PID, and process name; macOS uses
  frontmost app plus Accessibility focused-window title when permission exists;
  Linux keeps X11 and Hyprland support and adds GNOME Wayland best-effort app
  attribution.
- [ ] Never infer a website URL from a browser window title. Persist `app_name`
  even when browser context is unavailable, and use URL/title only from the
  opted-in extension.
- [ ] Build a Manifest V3 Chromium extension and Firefox-compatible manifest
  variant. Send only the minimum local payload to Mnemo's loopback receiver.
- [ ] Document optional native-messaging host packaging as the hardened
  transport for production; retain the local loopback receiver for MVP
  development and extension compatibility.
- [ ] Write fixture tests for X11/Hyprland/GNOME command parsing and verify
  unknown remains a safe fallback.
- [ ] Run `cargo test active_window`, `cargo check`, and manually verify clips
  from a browser, terminal, and editor.

### Task 11: Rebuild settings as a theme-safe system surface

**Files:**
- Modify: `src/pages/Settings.tsx`
- Create: `src/components/ThemeSelect.tsx`
- Modify: `src/pages/Quality.tsx`
- Modify: `src/index.css`
- Test: `src/pages/Settings.test.tsx`

- [ ] Add tests for theme selection and a two-step typed clear confirmation.
- [ ] Replace native model select with a controlled themed menu.
- [ ] Add Dark/System/Light controls using semantic CSS variables; test each
  route with dark and light tokens.
- [ ] Move quality diagnostics under an advanced Settings section.
- [ ] Move Clear Database to Local Data and require exact `DELETE` confirmation.
- [ ] Run `npm test` and `npm run build`.

### Task 12: Full integration verification

**Files:**
- Modify: files identified by verification only

- [ ] Run `npm test`, `npm run build`, `cargo fmt --check`, `cargo check`,
  `cargo test`, and `git diff --check`.
- [ ] Run `npm run tauri dev` and verify: capture state, both global shortcuts,
  dark/light/system appearance, dashboard activity updates, graph states,
  settings confirmation, session corrections, and native/browser source
  attribution.
- [ ] Verify no blocked clip content appears in UI/log payloads.
- [ ] Check keyboard navigation and `prefers-reduced-motion` behavior.
