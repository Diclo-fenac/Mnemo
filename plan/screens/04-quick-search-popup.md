# Quick Search Popup

> **Purpose:** Provide a lightweight keyboard-first memory lookup without opening the full app flow.
>
> **Access:** Shortcut-driven popup window.
>
> **Entry Points:** `CmdOrCtrl+Shift+V`, tray interaction if added later.
>
> **Primary Outcome:** Let the user search and recover a clip in a compact transient surface.

---

## Core UI

- compact floating window
- single dominant search field
- compact result list
- optional tiny Mnemo brand mark in the header
- minimal chrome and strong keyboard focus

## Data Shown

- active query
- top ranked results
- compact source and time metadata

## User Actions

- type a query immediately on open
- move through results with keyboard
- copy or open a result
- dismiss with escape or blur rules
- hand off to full Search or Clip Detail

## States

- Loading: lightweight list skeleton
- Empty: shortcut hint and example prompts
- Error: inline recoverable message
- Degraded / fallback: keyword-only search if semantic pipeline is unavailable

## Events and Side Effects

- opens near cursor or in predictable focus position
- auto-focuses input on open
- may write selected result back to clipboard
- may open full app windows when deep inspection is needed

## Navigation and Dependencies

- depends on global shortcut registration and popup window management
- shares search logic with full Search screen

## Edge Cases

- popup should not steal focus permanently if dismissed
- repeated shortcut use should re-focus existing popup rather than spawn duplicates
- very long result content should remain compact

## Acceptance Notes

- the popup feels materially faster and lighter than the main Search page
- keyboard-only usage is complete
