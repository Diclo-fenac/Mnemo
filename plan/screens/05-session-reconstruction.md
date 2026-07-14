# Session Reconstruction

> **Purpose:** Reconstruct a research or work session as a coherent story.
>
> **Access:** Deep-link route from Timeline.
>
> **Entry Points:** `/session/:id`, “View Full Reconstruction” action.
>
> **Primary Outcome:** Help the user understand how a line of work unfolded across sources and time.

---

## Core UI

- back action
- large session title
- time range and duration
- sources section
- key topics section
- chronological vertical timeline
- cross-session memory connections section when available

## Data Shown

- session metadata
- clips in chronological order
- source breakdown
- topic tags
- timeline entries
- related clips from outside the session

## User Actions

- review the session arc
- open clips from the timeline
- inspect cross-session connections
- navigate back to Timeline

## States

- Loading: full-page skeleton with timeline placeholders
- Empty: only if reconstruction data is corrupted or missing
- Error: failure state with route-safe retry
- Degraded / fallback: show raw chronological clips without enriched sections

## Events and Side Effects

- marks session as reconstructed when viewed
- may trigger on-demand context generation for missing pieces

## Navigation and Dependencies

- linked from Timeline
- links onward to Clip Detail
- depends on session reconstruction command and clip timeline rendering

## Edge Cases

- sessions with weak labels still need a readable heading fallback
- sessions with no external links should hide the connection section cleanly
- source labels must handle missing URL data gracefully

## Acceptance Notes

- the screen feels like a readable story, not just another list
- chronology is unambiguous
