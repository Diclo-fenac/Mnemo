# Mnemo Private Beta Release Implementation Plan

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Ship a Linux AppImage private beta that begins with capture off, downloads local intelligence only after onboarding, offers optional verified browser context, checks signed GitHub Releases for updates, and communicates source uncertainty truthfully.

**Architecture:** Persist first-run completion alongside capture preferences in SQLite so the Rust runtime, global shortcut, and React UI agree after restart. Gate the embedder behind a single-flight starter command and expose its state through the existing bootstrap contract. Keep browser provenance derived from trusted fields already on a clip: browser URL/title means verified browser context, a non-empty native app means detected app context, and all other clips are explicitly unavailable. Use Tauri's updater plugin only for non-blocking update checks; release signing and GitHub Release publishing live in CI secrets and workflow configuration.

**Tech Stack:** Tauri 2, Rust, rusqlite/SQLite, FastEmbed, React 19, TypeScript, Zustand, Vitest, GitHub Actions, `tauri-plugin-updater`, `@tauri-apps/plugin-updater`.

---

## File Map

| Path | Responsibility |
| --- | --- |
| `src-tauri/src/services/db.rs` | Fresh-install preference defaults and additive preference migration. |
| `src-tauri/src/services/capture_state.rs` | Persisted onboarding/capture/retention state and unit tests. |
| `src-tauri/src/services/embedder.rs` | Idempotent post-onboarding model startup and status transitions. |
| `src-tauri/src/state.rs` | Shared atomic guard preventing duplicate model-loader threads. |
| `src-tauri/src/commands/settings.rs` | Read/update/complete-onboarding commands and input validation. |
| `src-tauri/src/commands/system.rs` | Bootstrap state that includes onboarding and model availability. |
| `src-tauri/src/lib.rs` | Defer embedder startup until onboarding and register updater plugin. |
| `src-tauri/src/services/active_window.rs` | Optional `xdotool` fallback and safe source fallback tests. |
| `src-tauri/Cargo.toml` | Updater plugin dependency. |
| `src-tauri/capabilities/default.json` | Check-only updater permission. |
| `src-tauri/tauri.conf.json` | AppImage target, updater artifacts, public key, GitHub Releases endpoint. |
| `src/types/index.ts` | Exact contracts for onboarding, bootstrap, source provenance, and update state. |
| `src/store/app.ts` | Onboarding and update-check state. |
| `src/hooks/useUpdateCheck.ts` | One non-blocking update check per application launch. |
| `src/lib/sourceProvenance.ts` | Pure source-confidence derivation shared by clip surfaces. |
| `src/lib/sourceProvenance.test.ts` | Provenance edge-case tests. |
| `src/pages/Memory.tsx` | Consent-first onboarding and model progress/retry UI. |
| `src/pages/ClipDetail.tsx` | `Source unavailable` explanation only for uncertain clips. |
| `src/components/ClipCard.tsx` | Quiet uncertain-source treatment in list surfaces. |
| `src/pages/Settings.tsx` | Retention default, browser-context setup, updater state, and retry controls. |
| `src/App.tsx` | Bootstrap preference hydration and update-check lifecycle. |
| `src/index.css` | Onboarding, source-uncertainty, and update-state styling. |
| `extension/manifest.json` | Chrome/Chromium MV3 package manifest. |
| `extension/manifest.firefox.json` | Firefox MV3 package manifest. |
| `extension/README.md` | Accurate beta setup/privacy instructions. |
| `scripts/package-extension.mjs` | Deterministic Chrome ZIP and Firefox XPI staging. |
| `.github/workflows/ci.yml` | Required build, test, lint, and extension-package checks. |
| `.github/workflows/release.yml` | Manually dispatched, signed Linux release publication. |
| `README.md` | Current product capabilities, beta status, install, and privacy overview. |
| `docs/private-beta.md` | Clean-install, extension, source, offline-update, and release checklist. |
| `.gitignore` | Generated extension archives and local updater-key paths. |

## External Prerequisites

These must be completed by the repository owner before the release workflow can
publish a signed beta. Do not put any private key, password, browser-store
credential, or GitHub token in the repository.

1. Generate a Tauri updater key with `npm run tauri signer generate` and store
   the private key outside this repository.
2. Add `TAURI_SIGNING_PRIVATE_KEY` and, if used, `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`
   as GitHub Actions secrets.
3. Copy the generated public key into the updater configuration. Public keys may
   be committed; private keys may not.
4. Create the unlisted Chrome Web Store listing manually and record its URL in
   the release configuration/documentation.
5. Enable GitHub Actions write permission for releases and protect the manual
   release workflow with a beta environment if available.

## Chunk 1: Consent-First Bootstrap

### Task 1: Make fresh installs capture-off and retention-never

**Files:**
- Modify: `src-tauri/src/services/db.rs`
- Modify: `src-tauri/src/services/capture_state.rs`
- Test: `src-tauri/src/services/db.rs`
- Test: `src-tauri/src/services/capture_state.rs`

- [ ] Write a failing database test asserting a fresh `settings` row has
  `capture_enabled = 0`, `browser_context_enabled = 0`, `auto_delete_days = NULL`,
  and `onboarding_completed = 0`.
- [ ] Add `onboarding_completed INTEGER NOT NULL DEFAULT 0` to the base schema
  and additive settings migration. Keep migrations non-destructive even though
  existing development databases are out of beta support.
- [ ] Change only fresh-schema defaults to capture-off/retention-never; do not
  silently overwrite an existing user's explicit preferences during migration.
- [ ] Extend `CapturePreferences` with `onboarding_completed`; update `load`,
  `persist`, and test fixtures to read/write it.
- [ ] Run: `cargo test capture_state`.
- [ ] Run: `cargo test services::db`.
- [ ] Commit implementation only: `feat(privacy): default fresh installs to capture off`.

### Task 2: Gate model startup behind completed onboarding

**Files:**
- Modify: `src-tauri/src/state.rs`
- Modify: `src-tauri/src/services/embedder.rs`
- Modify: `src-tauri/src/commands/settings.rs`
- Modify: `src-tauri/src/commands/system.rs`
- Modify: `src-tauri/src/lib.rs`
- Test: `src-tauri/src/services/embedder.rs`
- Test: `src-tauri/src/commands/settings.rs`

- [ ] Write a failing unit test for a single-flight model-start guard: the first
  completed-onboarding request starts loading; repeated requests report the
  existing state and never spawn another loader.
- [ ] Add `model_start_requested: Arc<AtomicBool>` to `AppState` and a focused
  `start_embedder_once` wrapper that sets `Loading` before spawning work.
- [ ] Remove unconditional `start_embedder` from Tauri setup. During setup,
  start it only when persisted onboarding is already complete; otherwise leave
  `EmbeddingStatus::Deferred`.
- [ ] Add `complete_onboarding` command. In one transaction it marks onboarding
  complete, preserves capture off unless the explicit input requests capture on,
  then invokes the single-flight loader after the transaction succeeds.
- [ ] Extend `BootstrapState` with `onboarding_completed`; expose `deferred`,
  `loading`, `ready`, and `unavailable` unchanged as stable status values.
- [ ] Preserve keyword search and capture behavior when model loading fails;
  failed loading must remain retryable through a `retry_embedding_model` command
  that resets only the single-flight guard and starts the loader again.
- [ ] Run: `cargo test embedder`.
- [ ] Run: `cargo test commands::settings`.
- [ ] Run: `cargo fmt --check && cargo check`.
- [ ] Commit implementation only: `feat(onboarding): defer model load until consent`.

### Task 3: Replace browser-local onboarding with persisted onboarding

**Files:**
- Modify: `src/types/index.ts`
- Modify: `src/store/app.ts`
- Modify: `src/App.tsx`
- Modify: `src/pages/Memory.tsx`
- Modify: `src/pages/Settings.tsx`
- Modify: `src/index.css`
- Test: `src/lib/presentation.test.ts`

- [ ] Add TypeScript types for `onboardingCompleted` and model states. Do not
  derive onboarding from `localStorage`.
- [ ] Write a failing component-level pure-state test that maps `deferred`,
  `loading`, `ready`, and `unavailable` to truthful onboarding copy and retry
  eligibility.
- [ ] Hydrate preferences before deciding whether to show onboarding; render a
  neutral loading surface while they are unavailable to avoid a capture-on flash.
- [ ] Make the final onboarding action call `complete_onboarding` with
  `captureEnabled: false`. Label it "Continue with capture paused" and offer a
  separate explicit "Enable capture" action after the privacy explanation.
- [ ] Show background model preparation after completion, progress/state where
  available, and a retry action only for `unavailable`. Never block the route
  on model download.
- [ ] Change Settings empty retention control to "Never" and keep numeric days
  as an explicit opt-in value.
- [ ] Run: `npm test`.
- [ ] Run: `npm run build`.
- [ ] Commit implementation only: `feat(ui): add consent-first onboarding`.

## Chunk 2: Provenance and Browser Extension Delivery

### Task 4: Render source confidence without guessing

**Files:**
- Create: `src/lib/sourceProvenance.ts`
- Create: `src/lib/sourceProvenance.test.ts`
- Modify: `src/components/ClipCard.tsx`
- Modify: `src/pages/ClipDetail.tsx`
- Modify: `src/index.css`

- [ ] Write failing tests for the pure derivation function:
  `sourceUrl + pageTitle` yields `verified_browser`; a non-empty non-`Unknown`
  app yields `detected_app`; blank/`Unknown` data yields `unavailable`.
- [ ] Implement a narrow `deriveSourceProvenance(clip)` helper that accepts the
  existing `Clip` model and returns a label plus optional detail. Do not add a
  guessed-source database field.
- [ ] Keep cards visually quiet for verified-browser and detected-app clips.
  Show only the subdued `Source unavailable` treatment for uncertain clips.
- [ ] In Clip Detail, replace `Unknown source` with an explicit unavailable
  explanation and a link/anchor to Browser Context setup; do not claim that the
  extension can identify terminal/editor clips.
- [ ] Run: `npm test -- sourceProvenance`.
- [ ] Run: `npm run build`.
- [ ] Commit implementation only: `feat(sources): explain unavailable provenance`.

### Task 5: Add optional Linux `xdotool` fallback safely

**Files:**
- Modify: `src-tauri/src/services/active_window.rs`
- Test: `src-tauri/src/services/active_window.rs`
- Modify: `docs/private-beta.md`

- [ ] Write fixture tests for parsing `xdotool getactivewindow`, window name,
  and PID/process output, including blank and failing-command cases.
- [ ] Add an `xdotool` fallback after existing X11/Sway/Hyprland/GNOME probes.
  It must time out/fail soft, return `None` on malformed output, and never
  create a shell command from clipboard data.
- [ ] Keep the existing bundled/best-effort probes first. Treat missing
  `xdotool`, Wayland restrictions, and denied GNOME Shell Eval as normal
  `Source unavailable` conditions.
- [ ] Document optional utilities by desktop environment and the exact fallback
  behavior; do not add a runtime install prompt.
- [ ] Run: `cargo test active_window`.
- [ ] Run: `cargo clippy --all-targets -- -D warnings`.
- [ ] Commit implementation only: `feat(linux): add optional source fallback`.

### Task 6: Produce beta browser-extension packages and setup guidance

**Files:**
- Create: `scripts/package-extension.mjs`
- Modify: `extension/manifest.json`
- Modify: `extension/manifest.firefox.json`
- Modify: `extension/README.md`
- Modify: `.gitignore`
- Modify: `src/pages/Settings.tsx`
- Modify: `src/index.css`
- Test: `extension/context-bridge.test.ts`

- [ ] Add a failing package-script test or dry-run assertion that Chrome output
  contains `manifest.json`, `background.js`, and `content.js`; Firefox output
  must contain the Firefox manifest renamed to `manifest.json` plus the same
  scripts.
- [ ] Implement a deterministic Node script that stages two temporary trees and
  creates `mnemo-context-bridge-chrome.zip` and
  `mnemo-context-bridge-firefox.xpi` outside source control. Read extension
  version from one canonical manifest and fail on missing source files.
- [ ] Preserve the current local-only copy event: content script sends only
  selection, URL, title, favicon, and timestamp; background script propagates
  the fetch promise and never disrupts copying on failure.
- [ ] Add an opt-in Browser Context setup panel in Settings. It explains local
  data flow, links Chrome users to a configurable unlisted-store URL/release
  page, and states Firefox temporary-install restart behavior plainly.
- [ ] Update extension documentation with Chrome install, Firefox temporary
  install, Browser Context toggle, verification steps, and no-duplicate
  guarantee. Remove the unimplemented native-messaging claim from beta docs.
- [ ] Run: `npm test`.
- [ ] Run: `node scripts/package-extension.mjs --verify`.
- [ ] Commit implementation plus documentation together:
  `feat(extension): package private beta context bridge`.

## Chunk 3: Signed AppImage Updates and Release Automation

### Task 7: Configure a check-only Tauri updater

**Files:**
- Modify: `src-tauri/Cargo.toml`
- Modify: `package.json`
- Modify: `src-tauri/capabilities/default.json`
- Modify: `src-tauri/tauri.conf.json`
- Modify: `src-tauri/src/lib.rs`
- Create: `src/hooks/useUpdateCheck.ts`
- Modify: `src/store/app.ts`
- Modify: `src/pages/Settings.tsx`
- Modify: `src/index.css`

- [ ] Add a failing frontend test for update status mapping: unavailable plugin
  or offline endpoint produces a non-blocking `Unable to check` state, not an
  application error; a newer version produces an available version and release
  notes link.
- [ ] Add the official Rust and JavaScript updater plugins at compatible Tauri
  2 versions. Register the Rust plugin in the builder.
- [ ] Grant only `updater:allow-check`; do not expose download/install in the
  private beta.
- [ ] Configure `bundle.createUpdaterArtifacts = true` and a HTTPS GitHub
  Releases `latest.json` endpoint. Add the public updater key only after the
  owner completes the external prerequisite; never use a placeholder that can
  accidentally ship.
- [ ] Implement `useUpdateCheck` to run once after bootstrap completion, with a
  bounded timeout and no retry loop. Store only `idle`, `checking`, `current`,
  `available`, or `error` plus non-sensitive display data.
- [ ] Surface a calm status and manual "Check now" action in Settings. Never
  interrupt onboarding or capture because an update endpoint is offline.
- [ ] Run: `npm test`.
- [ ] Run: `npm run build`.
- [ ] Run: `cargo check`.
- [ ] Commit implementation only: `feat(release): add signed update checks`.

### Task 8: Add CI and manually dispatched Linux release workflow

**Files:**
- Create: `.github/workflows/ci.yml`
- Create: `.github/workflows/release.yml`
- Modify: `.gitignore`
- Modify: `README.md`
- Modify: `docs/private-beta.md`

- [ ] Add CI jobs that run `npm ci`, `npm test`, `npm run build`, `cargo fmt
  --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`, and
  `node scripts/package-extension.mjs --verify` on Ubuntu.
- [ ] Create a `workflow_dispatch` release workflow restricted to Linux x86_64.
  It checks out a version tag, installs the same Node/Rust toolchain, builds
  signed updater artifacts with `TAURI_SIGNING_PRIVATE_KEY` secrets, packages
  the extensions, and publishes an AppImage, its signature, extension archives,
  checksums, and `latest.json` to the tagged GitHub Release.
- [ ] Pin third-party actions to reviewed immutable revisions. Use the official
  `tauri-apps/tauri-action` only after confirming its release workflow matches
  the installed Tauri version; set a tag name so generated `latest.json` URLs
  are stable.
- [ ] Add a preflight step that fails if signing secrets, public updater key, or
  the release tag/version mismatch are absent. Never echo secret contents.
- [ ] Rewrite README to describe the implemented local-first app, Linux private
  beta status, AppImage installation, supported browser-context paths, privacy
  defaults, known Linux source limitations, and developer commands.
- [ ] Add the human-run clean-install and release checklist in
  `docs/private-beta.md`. Documentation changes ride with this release feature;
  do not create a docs-only commit.
- [ ] Validate workflow YAML with the available local linter, or record the
  exact GitHub Actions validation limitation in the PR/release checklist.
- [ ] Commit implementation and release documentation together:
  `ci: automate signed Linux beta releases`.

## Chunk 4: Acceptance Validation

### Task 9: Execute fresh-install and failure-path smoke tests

**Files:**
- Modify: `docs/private-beta.md`
- Modify: files identified by failed validation only

- [ ] Build the AppImage using the same signing configuration as the release
  workflow. Confirm the expected AppImage and `.sig` updater artifact exist.
- [ ] On a clean `$XDG_DATA_HOME`, launch the AppImage and verify no database
  exists before launch, onboarding appears, capture is off, Browser Context is
  off, and retention reads Never.
- [ ] Complete onboarding without enabling capture. Confirm model status moves
  from deferred to loading and that the UI remains usable during download.
- [ ] Simulate an unavailable network/model endpoint. Confirm retry is visible,
  keyword search remains usable, and no crash or capture regression occurs.
- [ ] Enable capture, copy a safe unique string, and verify exactly one stored
  clip. Disable capture via Settings and `CmdOrCtrl+Shift+M`; copy another
  unique string and verify it is not stored while `CmdOrCtrl+Shift+V` still
  searches the first clip.
- [ ] With Browser Context enabled, test one real Chrome/Chromium page and one
  Firefox temporary-install page. Confirm one resulting clip per copy with URL
  and title. Repeat with context disabled and confirm a normal app/unavailable
  source, never a duplicate.
- [ ] Test terminal/editor copy and missing optional utilities. Confirm native
  app metadata appears when available and `Source unavailable` otherwise.
- [ ] Test update check against a valid signed GitHub Release and then offline.
  Confirm Settings reports the result calmly and app startup is unaffected.
- [ ] Run final gate:
  `npm test && npm run build && cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test && git diff --check`.
- [ ] Commit only code fixes discovered during validation. Append checklist
  outcomes to the release notes/issue, not as a standalone documentation commit.

## Definition of Done

- Fresh private-beta installs are capture-off, browser-context-off, and
  retention-never.
- Completing onboarding starts one local model download; model failure never
  disables keyword search or crashes the app.
- Browser context is optional and packaged for Chrome/Chromium plus Firefox
  temporary installation. It produces URL/title metadata without duplicate
  clips.
- Mnemo never guesses a source. Uncertain clips say `Source unavailable`; all
  other source labels are derived from actual metadata.
- A signed Linux AppImage release and `latest.json` update feed can be produced
  from the manual GitHub workflow without committing secrets.
- CI and the private-beta checklist pass before inviting testers.
