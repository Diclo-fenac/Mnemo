# Settings

> **Purpose:** Let the user control privacy, storage, shortcut visibility, and memory-state understanding.
>
> **Access:** Main route in the app shell.
>
> **Entry Points:** `/settings`, sidebar click.
>
> **Primary Outcome:** Give the user confidence and control over how Mnemo stores and filters local memory.

---

## Core UI

- grouped settings sections
- privacy rules list and add-rule form
- hotkey display and rebind placeholder behavior
- storage controls and summary
- intelligence stage summary

## Data Shown

- filter rules
- current hotkey
- storage limits and retention settings
- clip/session/edge/fact counts
- current stage meaning

## User Actions

- enable or disable filter rules
- add or delete a custom rule
- update settings values
- clear history if implemented

## States

- Loading: grouped skeleton sections
- Empty: empty custom-rule list still shows built-in guidance
- Error: section-level recoverable error
- Degraded / fallback: show read-only settings if some write actions fail

## Events and Side Effects

- updates local settings rows
- may affect future clipboard capture behavior immediately
- destructive actions should require confirmation when implemented

## Navigation and Dependencies

- depends on settings commands, filter-rule commands, and memory-state command

## Edge Cases

- invalid regex rules need clear validation
- built-in blocked patterns should remain understandable to non-technical users
- storage cleanup actions should never feel accidental

## Acceptance Notes

- privacy controls are easy to understand
- intelligence stage copy stays explanatory, not gamified
