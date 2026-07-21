# Public Release Readiness Design

## Status

Approved by the user on 2026-07-20. This design extends the Linux private-beta
release work toward a public release without changing Mnemo's local-first
privacy model.

## Goal

Make the repository ready for a public release by hardening runtime behavior,
adding cross-platform build coverage, tightening release workflow checks, and
leaving a reproducible manual validation checklist for environments unavailable
to CI.

## Scope

### Included

- Automated tests for capture on/off behavior, shortcuts' command path,
  model failure/retry and keyword fallback, sensitive filtering, retention
  defaults, browser-context matching, and active-window parsing.
- Safer and more testable active-window detection on Linux, macOS, and Windows;
  detection remains best effort and never blocks capture.
- GitHub Actions validation for least-privilege permissions, signing-secret
  presence, artifact completeness, and signed updater metadata.
- CI build coverage for Linux, macOS, and Windows. Public release publishing
  remains gated by the existing manual workflow and signing environment.
- Browser extension packaging and deterministic payload tests for both Chrome
  and Firefox.
- A release validation document covering clean AppImage installation, real
  keyboard shortcuts, real browser enrichment, signed GitHub update detection,
  and sensitive-content/retention acceptance checks.
- Archiving the untracked local plan file after repository-side work and
  documentation are complete.

### Excluded

- Claiming successful clean-machine, macOS, Windows, browser, or signed-release
  validation without those environments and credentials.
- Adding telemetry, cloud storage, accounts, or a diagnostics upload path.
- Replacing the existing Tauri architecture or introducing platform-specific
  installer products beyond the builds supported by the release workflow.

## Architecture

Runtime behavior stays behind existing service boundaries. Pure parsing and
policy functions gain unit tests; command and watcher behavior is verified at
the smallest practical boundary without requiring a desktop session. The
active-window service uses platform-specific providers with sanitized output
and a common `WindowInfo` fallback. Any provider failure returns `Unknown` and
capture continues.

The release workflow is split conceptually into verification, platform build,
and publishing concerns. CI builds each supported desktop target with
read-only repository permissions. The manual release job keeps `contents: write`
only where publishing is required, checks signing inputs before invoking the
Tauri action, and verifies that updater metadata and signatures exist before
uploading browser artifacts.

## Behavior and failure handling

- Capture disabled means clipboard events are not persisted; search and the
  search shortcut remain available.
- Capture shortcut and UI updates must pass through the same persisted settings
  path and actual watcher state.
- Model download/initialization errors transition to an unavailable state,
  preserve keyword search, and expose retry without crashing startup.
- Sensitive rules run before persistence and before activity previews/logging.
- Retention defaults to `Never`; invalid values are rejected and no cleanup is
  performed unless an explicit positive day count is configured.
- Browser metadata is accepted only when enabled, fresh, and matched to the
  copied content; failed or disabled context yields one ordinary clip.
- Active-window probes are optional, bounded, and never infer a source from
  clip text.
- Update checks remain non-blocking; malformed, unsigned, unavailable, or
  offline metadata leaves the app usable.

## Validation

Automated validation will include frontend tests/build, extension packaging,
Rust formatting, clippy, Rust tests, and platform build jobs. Manual gates will
be written with exact setup and expected results for:

1. Clean Linux AppImage launch and onboarding.
2. Capture off/on and both keyboard shortcuts.
3. Model success, failure, retry, and keyword-only fallback.
4. Chrome and Firefox URL/title enrichment and disabled-context behavior.
5. Signed GitHub release updater detection and offline fallback.
6. Sensitive content suppression and retention defaults.
7. Native source detection on supported desktop environments.

The final checklist will identify the owner, environment, release artifact, and
evidence required for each manual gate. The local untracked plan file is moved
to an archive location only after these repository changes and documents are
finished; no unrelated user changes are deleted.
