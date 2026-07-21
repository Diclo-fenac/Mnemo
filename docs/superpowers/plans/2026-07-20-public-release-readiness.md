# Public Release Readiness Implementation Plan

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Harden Mnemo for public release, add cross-platform build coverage, and document the manual gates that cannot be certified in this workspace.

**Architecture:** Preserve the current Tauri services and browser bridge. Add focused pure-function tests and platform-provider safety improvements, then strengthen CI/release workflows with verification and platform build jobs. Finish with a release validation document and archive the existing untracked plan file.

**Tech Stack:** Rust/Tauri 2, SQLite, React/Vitest, GitHub Actions, Chrome/Firefox Manifest V3.

---

## Chunk 1: Runtime behavior and test coverage

### Task 1: Test policy defaults and sensitive filtering

**Files:**
- Modify: `src-tauri/src/services/filter.rs`
- Modify: `src-tauri/src/services/retention.rs`
- Test: existing Rust module tests in those files

- [ ] Add table-driven tests for sensitive-content matches and safe content.
- [ ] Add tests proving `Never` retention does not delete data and positive retention rejects invalid values.
- [ ] Run `cargo test --manifest-path src-tauri/Cargo.toml services::filter services::retention`.

### Task 2: Test capture and model fallback contracts

**Files:**
- Modify: `src-tauri/src/services/capture_state.rs`
- Modify: `src-tauri/src/services/embedder.rs`
- Modify: `src-tauri/src/commands/settings.rs`

- [ ] Add tests for capture disabled/enabled state synchronization and onboarding defaults.
- [ ] Add tests for unavailable model state preserving keyword operation and retry eligibility.
- [ ] Run targeted Rust tests, then the full Rust suite.

### Task 3: Improve active-window providers

**Files:**
- Modify: `src-tauri/src/services/active_window.rs`
- Test: `src-tauri/src/services/active_window.rs`

- [ ] Centralize trimming/sanitization and reject empty or malformed provider output.
- [ ] Improve Linux provider ordering and parsing without making optional utilities mandatory.
- [ ] Make macOS/Windows command output bounded and safe, retaining `Unknown` fallback.
- [ ] Add parser tests for X11, compositor JSON, and uncertain output.

### Task 4: Verify browser enrichment behavior

**Files:**
- Modify: `extension/context-bridge.test.ts`
- Modify: `extension/background.js`
- Modify: `scripts/package-extension.mjs`

- [ ] Test Chrome and Firefox payload forwarding, URL/title preservation, disabled/missing runtime behavior, and packaging metadata.
- [ ] Ensure extension failures never interfere with the browser copy event.
- [ ] Run `npm test` and `npm run package:extension -- --verify`.

## Chunk 2: CI and release workflow

### Task 5: Add cross-platform verification builds

**Files:**
- Modify: `.github/workflows/ci.yml`

- [ ] Keep Linux tests as the authoritative full suite.
- [ ] Add macOS and Windows frontend/Tauri build jobs with read-only permissions.
- [ ] Use platform-specific Tauri prerequisites and explicit target labels.
- [ ] Validate workflow YAML and run all locally available checks.

### Task 6: Harden signed release publishing

**Files:**
- Modify: `.github/workflows/release.yml`
- Modify: `src-tauri/tauri.conf.json`
- Create: `scripts/verify-release-artifacts.mjs`

- [ ] Add preflight checks for signing secrets, public updater key, version, and expected artifacts.
- [ ] Keep write permission limited to the publishing job and use least privilege elsewhere.
- [ ] Verify `latest.json` and signature files before extension upload.
- [ ] Add macOS and Windows release build artifacts while preserving Linux AppImage publishing.

## Chunk 3: Manual validation and cleanup

### Task 7: Publish reproducible release checklist

**Files:**
- Create: `docs/public-release-validation.md`
- Modify: `docs/private-beta.md`

- [ ] Document clean-machine steps and expected evidence for AppImage, shortcuts, model failure/retry, extensions, updater signing, filtering, retention, and native sources.
- [ ] Clearly mark environment-dependent items as unverified until run.

### Task 8: Archive the local plan file

**Files:**
- Move: `docs/superpowers/plans/2026-07-17-private-beta-release.md` to `docs/superpowers/plans/archive/2026-07-17-private-beta-release.md`

- [ ] Confirm the file is the intended untracked local plan.
- [ ] Move it without deleting unrelated files.
- [ ] Run `git diff --check` and final status inspection.
