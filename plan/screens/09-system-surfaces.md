# System Surfaces

> **Purpose:** Document non-route surfaces that still shape the Mnemo experience.
>
> **Access:** System- or interaction-driven.
>
> **Entry Points:** Tray interactions, clipboard events, toasts, context menus, stage upgrades.
>
> **Primary Outcome:** Ensure every off-route interaction feels deliberate and consistent with the desktop utility tone.

---

## Core UI

- tray menu
- toast notifications
- sensitive-content blocked notification
- intelligence-upgraded notification
- clip-card context menu

## Data Shown

- concise action/status messaging
- current app status where relevant
- clip-level actions inside context menus

## User Actions

- reopen app from tray
- dismiss notifications
- act on clip context-menu options
- observe sensitive-content blocking

## States

- Loading: not applicable for most surfaces
- Empty: not applicable
- Error: if a tray action fails, show compact feedback
- Degraded / fallback: notifications still function even if richer UI surfaces are unavailable

## Events and Side Effects

- tray menu can open main window or popup
- blocked-sensitive notification appears before storage
- intelligence-upgrade toast appears only on stage transition
- clip context menu can trigger copy, pin, and delete

## Navigation and Dependencies

- depends on tray integration, notifications, and shared clip actions
- bridges system events and route-based UI

## Edge Cases

- notification spam must be avoided during rapid clipboard changes
- tray interactions should not spawn duplicate windows
- destructive context-menu actions need confirmation or undo strategy when implemented

## Acceptance Notes

- system surfaces feel native and quiet
- nothing off-route breaks the minimal tone of the app
