# Mnemo Private Beta Release Design

## Status

Approved by the user on 2026-07-17. This specification defines the Linux
private-beta release and the first source-reliability increment. It supersedes
the earlier automatic-retention assumption in the Memory Workspace design:
retention defaults to `Never` for this beta.

## Goal

Ship a trustworthy, installable Mnemo private beta for a small Linux cohort.
The beta must make capture consent obvious, keep all data local, enrich browser
clips accurately when the user opts in, and fail safely when embeddings or
native source detection are unavailable.

## Scope

### Included

- Linux AppImage distribution through GitHub Releases.
- Signed update checks backed by GitHub Releases.
- Fresh-database first-run onboarding.
- Capture disabled by default until explicitly enabled.
- Automatic post-onboarding download of the default local semantic model.
- Keyword-only operation plus clear retryable state while model loading or
  download fails.
- Optional browser-context enrichment for Chrome/Chromium and Firefox.
- Best-effort native app/window detection on AppImage-compatible Linux.
- Clear source provenance and uncertainty treatment.
- Clean-install, extension, retention, update, and failure-path validation.

### Excluded

- Migration support for old Mnemo development databases.
- Native packages for individual Linux distributions.
- macOS and Windows installers or platform acceptance testing.
- An in-app diagnostics exporter or telemetry pipeline.
- Mandatory extension installation.
- Inference of a source from clip text.

## Release Architecture

### Distribution and Updates

- The beta ships as an AppImage so it is usable on broadly compatible Linux
  distributions without a package-manager dependency.
- GitHub Releases holds the AppImage, updater manifest, signature, checksums,
  release notes, and browser-extension installation links.
- Tauri update checks use a public updater key in application configuration.
  The matching private signing key is generated and stored outside the repo.
- Update checks must be non-blocking. Offline, malformed, or unavailable
  release metadata leaves the running application usable and reports a calm
  status in Settings.

### First Run

1. A clean install creates a new local database and enters onboarding.
2. Onboarding explains that Mnemo is local-first and does not capture anything
   yet.
3. Capture starts disabled. The user can enable it during onboarding or later
   from the persistent capture control, Settings, or its global shortcut.
4. Once onboarding finishes, Mnemo begins downloading the default semantic
   model in the background, regardless of capture state.
5. The UI exposes download progress and a retry action. Until the model is
   ready, capture and keyword search still work; semantic features state that
   they are preparing or unavailable rather than failing silently.
6. The default retention policy is `Never`. Any automatic cleanup must be
   enabled deliberately in Settings.

### Capture Guarantees

- Capture-off means Mnemo neither reads nor persists clipboard-change events.
- `CmdOrCtrl+Shift+V` remains available for searching existing local memories
  while capture is off.
- The capture-toggle shortcut remains available and must update the real
  watcher state, not only the interface.
- Sensitive-content rules always run before persistence when capture is on.

## Browser Context

### Product Positioning

The browser extension is an optional recommended enhancement. Mnemo remains
useful without it, but browser researchers receive verified URL and title
metadata when they install it.

### Browser Delivery

- Chrome/Chromium: distribute an unlisted Chrome Web Store extension.
- Firefox: distribute a manual/temporary package for the beta and state that
  temporary installations are removed after a browser restart.
- Onboarding and Settings show browser-context setup only as an opt-in choice.
  They explain what is sent locally: copied selection, current URL, page title,
  and favicon when available.

### Data Flow

1. A user explicitly copies selected browser content.
2. The extension sends a local-only payload to `127.0.0.1`.
3. Mnemo validates and temporarily caches that payload.
4. The clipboard watcher matches it to the next copied text event within the
   allowed time window.
5. A single clip is written with verified browser metadata; the extension never
   inserts a duplicate clip independently.

## Source Provenance

Mnemo must describe what it knows, not what it guesses.

| Priority | Provenance | UI treatment |
| --- | --- | --- |
| 1 | Verified browser context | URL/title presented normally; no confidence warning. |
| 2 | Detected native app/window | App name and title when available; no warning. |
| 3 | Optional Linux enhancement | `xdotool` or desktop-environment integration may improve titles at runtime. It is never required for AppImage use. |
| 4 | No metadata | Show `Source unavailable` with subdued explanatory help. |

- No source is inferred from clip content.
- `Source unavailable` is shown only for uncertain clips, not as a noisy badge
  on all reliable clips.
- Clip Detail explains how the browser extension can provide verified web
  provenance without implying that it can identify every application.

## Validation Gates

The beta is releasable only after the following pass on a clean AppImage
environment:

1. Launch succeeds with no existing Mnemo database.
2. Onboarding completes with capture still disabled.
3. Default model download displays progress; failure exposes retry and keeps
   keyword search usable.
4. Enabling and disabling capture changes actual persistence behavior while
   Quick Search remains available.
5. Retention is set to `Never` by default and automatic cleanup requires a
   deliberate Settings change.
6. Chrome/Chromium extension copy inserts one enriched clip with page URL and
   title.
7. Firefox manual extension copy inserts one enriched clip with page URL and
   title; documentation states its temporary-install limitation.
8. A browser-context mismatch or native-source failure produces a normal clip
   with `Source unavailable`, not a duplicate, crash, or source guess.
9. An update check succeeds from signed GitHub Releases and degrades cleanly
   when offline.
10. Automated tests, formatter, lint, production frontend build, and Rust test
    suite pass before publishing an artifact.

## Feedback and Support

- Beta feedback is collected outside the application through GitHub Issues.
- Releases link to issue templates for installation, capture, model, source,
  and extension defects.
- Mnemo does not add diagnostics upload, telemetry, accounts, or cloud
  collection for this beta.

## Phase 8 Follow-On

After the beta is stable, strengthen source detection without weakening the
privacy model:

- probe optional Linux desktop integrations safely at runtime;
- improve app/window-title detection where available;
- keep browser extension metadata authoritative for web provenance;
- add source-confidence tests and visible, calm uncertainty handling.
