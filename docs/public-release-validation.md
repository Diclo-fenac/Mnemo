# Public Release Validation

This is the final release gate. Automated checks run in CI; the remaining
items require a clean machine, a real desktop session, browser extensions, or
GitHub signing credentials. Do not mark an item passed without recording the
artifact version, OS/browser version, and evidence.

## Automated checks

Run from the repository root:

```bash
npm test
npm run build
npm run package:extension -- --verify
cargo fmt --check --manifest-path src-tauri/Cargo.toml
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml
node scripts/verify-release-config.mjs --config-only
git diff --check
```

CI must also complete the Tauri build jobs on Ubuntu, macOS, and Windows.

## Clean Linux AppImage

- [ ] Download the exact AppImage attached to the release and verify its
  checksum/signature.
- [ ] Run it as a fresh user or with a new `XDG_DATA_HOME`.
- [ ] Confirm onboarding appears, capture is off, Browser Context is off, and
  Auto-delete is `Never`.
- [ ] Finish onboarding without enabling capture. Confirm the model enters
  loading/deferred state and keyword search remains usable.

## Capture and shortcuts

- [ ] Enable capture from the sidebar and copy a unique safe string; exactly one
  clip appears.
- [ ] Disable capture from Settings and with `Ctrl/Cmd+Shift+M`; a subsequent
  copy is not stored.
- [ ] Use `Ctrl/Cmd+Shift+V` while capture is off; existing clips remain
  searchable and the shortcut does not enable capture.
- [ ] Confirm sensitive values are absent from activity previews and logs.

## Model failure and retry

- [ ] Start once with model download/network access unavailable; the app stays
  usable, shows unavailable state, and offers Retry.
- [ ] Search by an exact keyword while unavailable and confirm results appear.
- [ ] Restore access, press Retry, and confirm the model reaches Ready without
  losing existing clips.

## Browser enrichment

- [ ] With Browser Context enabled, install the Chrome package, copy selected
  text from a real page, and confirm one clip contains the exact URL and title.
- [ ] Repeat with the Firefox temporary `.xpi`; record the Firefox version and
  confirm the temporary-install limitation is visible to testers.
- [ ] Disable Browser Context and repeat; confirm no browser metadata and no
  duplicate clip are inserted.
- [ ] Stop Mnemo and copy in the browser; the browser copy still succeeds.

## Native source detection

- [ ] Copy from a terminal and editor on each target OS; record the detected app
  and title when available.
- [ ] Repeat without optional Linux utilities such as `xdotool`; capture still
  works.
- [ ] Force an unavailable provider and confirm the UI says `Source unavailable`
  rather than guessing from the clip text.

## Signed updater

- [ ] Configure `TAURI_SIGNING_PRIVATE_KEY` and its password in the protected
  `private-beta` GitHub environment; keep the private key out of the repository.
- [ ] Run the manual release workflow and confirm Linux, macOS, Windows,
  extension packages, `latest.json`, and `.sig` assets are attached.
- [ ] Launch the previous release and confirm it detects the newer signed
  release without blocking startup.
- [ ] Repeat offline or with the feed unavailable; Mnemo remains usable and
  reports a calm unavailable status.

## Retention and sensitive content

- [ ] Confirm a fresh database defaults to `Never` and keeps old clips.
- [ ] Set a positive retention period deliberately; verify only old unpinned
  clips are removed and pinned clips remain.
- [ ] Copy representative passwords, tokens, and payment-card-like values;
  confirm configured sensitive rules prevent persistence and previews.

Record failures as release blockers with the exact version and reproduction
steps. The repository cannot certify these environment-dependent checks by
itself.
