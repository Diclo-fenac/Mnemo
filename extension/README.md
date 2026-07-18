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

## Private beta install

1. Enable **Browser context** in Mnemo Settings.
2. Chrome/Chromium beta users install the unlisted Web Store package shared by
   the beta coordinator. The source package can also be loaded with Developer
   Mode and **Load unpacked** for local testing.
3. Firefox beta users download the `.xpi` package from the GitHub Release, open
   `about:debugging#/runtime/this-firefox`, choose **Load Temporary Add-on**,
   and select the package. Firefox temporary extensions are removed after a
   browser restart, so repeat this step when needed.
4. Copy text from a normal web page and open the resulting clip in Mnemo. A
   page URL/title confirms the bridge is working; no URL means the app remains
   safe and labels the source as unavailable.

## Transport and privacy

The private beta uses the loopback bridge at `127.0.0.1:17531`. The app accepts
the payload only while Browser Context is enabled and matches it to the next
clipboard event; the extension never creates an independent clip row. No page
content is collected passively and no payload leaves the device.
