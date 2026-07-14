# Clip Detail

> **Purpose:** Show a single remembered clip with its full content and generated context.
>
> **Access:** Deep-link route from Timeline, Search, Graph, or Session Reconstruction.
>
> **Entry Points:** `/clip/:id`, any clip-card navigation.
>
> **Primary Outcome:** Let the user inspect the clip, understand why it mattered, and act on it again.

---

## Core UI

- full clip content area
- syntax-friendly preformatted content rendering when relevant
- context card explaining source, likely purpose, tags, and related clips
- action row for copy, pin, and delete

## Data Shown

- full clip content
- app/source/timestamp metadata
- generated context
- related clips
- associated memory fact when applicable

## User Actions

- copy clip again
- pin or unpin
- delete
- open related clips

## States

- Loading: content and context skeleton
- Empty: only if clip is missing or deleted
- Error: route-safe error state
- Degraded / fallback: render clip content even if AI context is unavailable

## Events and Side Effects

- may generate context on demand if missing
- writes content back to clipboard on copy
- updates stores after pin or delete actions

## Navigation and Dependencies

- entered from all major exploration screens
- depends on clip retrieval, context retrieval, related clips, and clip actions

## Edge Cases

- code-heavy clips need readable wrapping/scroll behavior
- deleted clips should fail gracefully if opened from stale links
- missing source metadata should not weaken the context card structure

## Acceptance Notes

- core clip content is always prioritized
- context makes the clip easier to recall, not harder to parse
