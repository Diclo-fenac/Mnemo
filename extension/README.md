# Mnemo Context Bridge

This optional browser extension sends page context to Mnemo only when the user
copies content and Browser Context is enabled in Mnemo Settings.

## What it sends

- Current page URL
- Page title
- Favicon URL when the page exposes one
- Selected text, capped at 100 KB

The current Mnemo MVP receives URL/title over a loopback-only listener at
`127.0.0.1:17531`. It never sends this data to a remote server.

## Development install

1. Enable **Browser context** in Mnemo Settings.
2. Chromium: open `chrome://extensions`, enable Developer Mode, choose **Load
   unpacked**, then select this `extension` directory.
3. Firefox: use `about:debugging#/runtime/this-firefox`, choose **Load
   Temporary Add-on**, and select `manifest.firefox.json` after copying it to
   the extension directory as `manifest.json` for the temporary load.

## Production transport

For packaged releases, use a native-messaging host registered by the Mnemo
installer. Native messaging restricts access to the known extension ID and
avoids a loopback HTTP listener. The loopback bridge remains the low-friction
development transport.
