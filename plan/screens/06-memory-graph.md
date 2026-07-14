# Memory Graph

> **Purpose:** Visualize semantic connections between remembered clips.
>
> **Access:** Main route in the app shell.
>
> **Entry Points:** `/graph`, sidebar click.
>
> **Primary Outcome:** Let users inspect relationship clusters and jump into related memory nodes.

---

## Core UI

- graph canvas
- compact page intro
- topic legend
- hover tooltip behavior

## Data Shown

- nodes representing clips
- edges representing similarity
- topic color grouping
- basic preview on hover

## User Actions

- pan and inspect the graph
- hover for context
- click node to open Clip Detail
- use legend for interpretation

## States

- Loading: canvas placeholder and legend skeleton
- Empty: sparse-state message when not enough edges exist yet
- Error: graph load failure with retry
- Degraded / fallback: render recent nodes only if edge density is too low or the graph payload is partial

## Events and Side Effects

- renders only recent data for performance
- visually updates as new memory edges appear over time

## Navigation and Dependencies

- links to Clip Detail on node click
- depends on graph command and force-graph wrapper

## Edge Cases

- graphs with very few edges should not look broken
- dense clusters need readable hover behavior
- topic-color mapping must remain stable within a session

## Acceptance Notes

- the graph is informative without becoming visually loud
- node interaction feels stable and fast
