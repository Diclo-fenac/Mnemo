# Timeline

> **Purpose:** Show Mnemo’s remembered clips as research sessions over time.
>
> **Access:** Default route for all users of the local app.
>
> **Entry Points:** `/`, sidebar click, app launch.
>
> **Primary Outcome:** Help the user review what they copied, grouped into meaningful sessions.

---

## Core UI

- page header with `Mnemo` title and current memory-stage summary
- optional memory-fact banner near the top
- session list ordered by most recent
- collapsible session groups with summary and clip previews
- unsorted clip group at the bottom when needed

## Data Shown

- sessions with labels, summaries, source apps, time ranges, clip count
- clips inside each expanded session
- latest surfaced memory fact
- clip connection counts where relevant

## User Actions

- expand or collapse a session
- open full session reconstruction
- open individual clip detail
- dismiss or act on memory-fact banner
- scan unsorted clips

## States

- Loading: session-group skeletons
- Empty: first-run onboarding state with local-only privacy message, clipboard guidance, shortcut hint, and “copy something to begin”
- Error: recoverable timeline error panel with retry
- Degraded / fallback: show raw clips without rich grouping if session metadata is unavailable

## Events and Side Effects

- refreshes on `clip-added`
- updates counts and stage summary as intelligence state changes
- may virtualize large lists for performance

## Navigation and Dependencies

- links to Session Reconstruction and Clip Detail
- depends on clips, sessions, memory state, and memory facts
- uses session grouping and clip card components

## Edge Cases

- clips without `session_id` appear in Unsorted
- very large histories must remain responsive
- sessions with weak auto-labels still need readable fallback titles
- banner dismissal should not permanently suppress future facts unless explicitly designed later

## Acceptance Notes

- first launch feels intentional, not blank
- sessions are clearly scannable
- clip previews remain readable without requiring detail navigation
