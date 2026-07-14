# Mnemo Chrome Extension Plan

> **Goal:** Define the optional browser extension that sends richer copy context into Mnemo so web research clips retain URL and page-title metadata.
>
> **Priority:** Optional, after core desktop MVP.

---

## Purpose

The desktop app works without the extension, but browser-origin clips are much more useful when Mnemo can capture:

- current page URL
- page title
- selected text at copy time

This improves:

- session labeling
- context generation
- recurring-source detection
- clip-detail source explanations

## Scope

The extension should remain minimal:

- listen for copy events
- extract browser metadata
- send payload to a local Mnemo listener

It should not:

- store data remotely
- introduce accounts
- become a full browser history tracker
- capture content without a copy action

## Extension Structure

Use the planned directory:

- `extension/manifest.json`
- `extension/content.js`
- `extension/background.js`

## Browser Behavior

### content.js

Responsibilities:

- listen for `copy`
- read `window.getSelection().toString()`
- capture `window.location.href`
- capture `document.title`
- pass structured payload to background logic

Rule:

- if selected text is empty, do not send meaningless payloads

### background.js

Responsibilities:

- receive message from content script
- POST to local Mnemo endpoint
- keep logic small and retry-safe

## Local App Listener

Mnemo should optionally expose a small localhost endpoint, such as:

- `POST http://127.0.0.1:17531/clip`

Payload shape:

- `content`
- `source_url`
- `page_title`
- optional `source_app` hint like browser name if available
- timestamp if useful, though server-side timestamp remains authoritative

Rule:

- app still works when the extension is absent
- extension payload supplements clipboard events rather than replacing the desktop pipeline

## Coordination Strategy

Preferred flow:

1. user copies text in browser
2. browser extension sends payload to localhost listener
3. desktop app receives metadata and caches it briefly
4. next clipboard event matches copied text and enriches the inserted clip with cached browser metadata

This avoids bypassing the main clipboard pipeline while still attaching browser context.

## Matching Strategy

To avoid duplicate clip creation:

- the extension should not directly create a full clip row independent of the clipboard watcher
- instead, it should cache metadata keyed by copied text and short timestamp proximity
- the watcher should merge matching browser metadata into the next inserted text clip

Fallback:

- if cache match fails, store clip normally without browser metadata

## Privacy Rules

- only send data on explicit copy action
- only send selected text, current URL, and page title
- never collect page content passively
- keep all communication local to `127.0.0.1`

## Failure Handling

- if localhost listener is unavailable, the extension should fail silently or log minimally
- failed POST should not interrupt the user’s copy behavior
- no extension UI is required for MVP

## Security Notes

- bind listener to localhost only
- validate payload size and content type
- reject malformed payloads safely
- do not expose a broad local API surface

## Suggested Milestones

### Phase 1

- create extension files
- send payload to localhost
- build local listener

### Phase 2

- cache metadata in app
- merge with clipboard pipeline

### Phase 3

- verify source URL/title appears in Timeline, Clip Detail, and session reconstruction

## Acceptance Criteria

- copied text from a browser can carry URL/title into the resulting clip
- app remains fully usable without extension
- no duplicate clip rows are created from extension + watcher interplay
- privacy stance remains explicit-copy-only
