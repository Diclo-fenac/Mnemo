# Mnemo Private Beta Checklist

This checklist is for the Linux AppImage beta. It assumes a fresh local
database and a small tester group. Feedback stays outside the app through
GitHub Issues.

For the public-release gate, use [`public-release-validation.md`](public-release-validation.md),
which distinguishes automated checks from clean-machine and signed-release
validation.

## Release prerequisites

- [ ] Generate a Tauri updater key outside the repository.
- [ ] Store the private key and password, if used, in GitHub Actions secrets.
- [ ] Put the public key in `src-tauri/tauri.conf.json`.
- [ ] Create the unlisted Chrome Web Store listing.
- [ ] Verify the Firefox temporary-install package from the release assets.
- [ ] Confirm GitHub Actions can create releases and upload `latest.json`.

## Clean install

- [ ] Download the signed AppImage from GitHub Releases.
- [ ] Run it with a clean `$XDG_DATA_HOME` or a clean user account.
- [ ] Confirm onboarding appears and capture is off.
- [ ] Confirm Browser Context is off.
- [ ] Confirm Auto-delete reads `Never`.
- [ ] Finish onboarding without enabling capture.
- [ ] Confirm the model changes from deferred to loading after onboarding.
- [ ] Confirm keyword search remains available while the model loads.
- [ ] Confirm model failure shows retry and does not crash the app.

## Capture and privacy

- [ ] Enable capture from the sidebar or Settings.
- [ ] Copy a unique safe string and confirm exactly one clip appears.
- [ ] Disable capture from the sidebar and `CmdOrCtrl+Shift+M`.
- [ ] Copy another unique string and confirm it is not stored.
- [ ] Use `CmdOrCtrl+Shift+V` while capture is off and confirm existing clips
  remain searchable.
- [ ] Confirm sensitive content is not displayed in the activity rail or logs.

## Browser context

- [ ] Enable Browser Context in Mnemo Settings.
- [ ] Install the unlisted Chrome/Chromium extension, copy from a real page,
  and confirm one clip includes URL and title.
- [ ] Load the Firefox `.xpi` temporarily, copy from a real page, and confirm
  one clip includes URL and title.
- [ ] Disable Browser Context, repeat a browser copy, and confirm no duplicate
  clip is inserted.
- [ ] Confirm Firefox temporary-install behavior is explained to testers.

## Native source behavior

- [ ] Copy from a terminal or editor and confirm an app name appears when the
  desktop integration can identify it.
- [ ] Test without optional `xdotool`/desktop utilities and confirm capture
  still works.
- [ ] Confirm uncertain clips show `Source unavailable` rather than a guessed
  website or application.

## Update behavior

- [ ] Publish a signed beta release with AppImage, `.sig`, extension packages,
  checksums, and `latest.json`.
- [ ] Launch an older beta and confirm the update check reports the newer
  version without blocking startup.
- [ ] Repeat offline and confirm Mnemo remains usable with a calm unavailable
  status.

## Final commands

```bash
npm test
npm run build
npm run package:extension -- --verify
cargo fmt --check --manifest-path src-tauri/Cargo.toml
cargo clippy --all-targets -- -D warnings --manifest-path src-tauri/Cargo.toml
cargo test --manifest-path src-tauri/Cargo.toml
git diff --check
```
