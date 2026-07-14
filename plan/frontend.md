# Mnemo Frontend Plan

> **Goal:** Define the React frontend needed to turn Mnemo’s local memory data into a usable desktop experience.

---

## Frontend Structure

Use the planned structure:

- `App.tsx`: routes and shell
- `pages/*`: route-level screens
- `components/*`: reusable UI pieces
- `store/*`: Zustand state
- `hooks/*`: invoke wrappers and route-facing helpers
- `types/index.ts`: TS contracts matching Rust models

## Routing

Routes:

- `/` -> Timeline
- `/search` -> Search
- `/session/:id` -> Session Reconstruction
- `/graph` -> Memory Graph
- `/clip/:id` -> Clip Detail
- `/settings` -> Settings

Behavior:

- App Shell persists across all routes
- popup search window can reuse shared search UI with compact layout rules where practical

## Shared State

### Clips Store

Owns:

- paginated clips
- refresh/invalidate flow
- clip mutation state for pin/delete/copy UX

### Memory Store

Owns:

- memory stage
- sessions
- memory facts
- graph data

### Settings Store

Owns:

- settings row
- filter rules
- save/update flags

Rule:

- stores coordinate command calls, but backend remains source of truth

## Screen Plans

### App Shell

- sidebar navigation
- route outlet
- bottom-right mascot indicator
- global event listeners for `clip-added` and `intelligence-upgraded`

### Timeline

- fetch sessions + memory state + optional fact banner
- render collapsible session groups
- render unsorted group
- include first-run onboarding in empty state
- add virtualization when history grows large

### Search

- search input at top
- rotating example prompts
- result list with search badges
- filter chips for content-type/time/pin style narrowing if implemented on client or query-side later

### Quick Search Popup

- compact search field
- keyboard-first result navigation
- minimal metadata
- fast copy/open action path

### Session Reconstruction

- fetch single session reconstruction payload
- show source breakdown, topics, timeline, and cross-session links

### Memory Graph

- render force graph with capped node count
- color by topic
- click-through to clip detail

### Clip Detail

- fetch clip + context
- render content
- render explanatory context card
- expose copy, pin, delete

### Settings

- privacy rules
- storage settings
- hotkey info
- intelligence stage explanation

## Component Responsibilities

- `Sidebar`: navigation and app identity
- `MascotIndicator`: stage name, icon, clip count, subtle message rotation
- `SessionGroup`: expandable session shell
- `ClipCard`: preview card with source, timestamp, language, connections, pin state
- `SearchBar`: main search entry component
- `ClipContextCard`: explanatory panel in Clip Detail
- `MemoryBadge`: inline connection count
- `MemoryFactBanner`: top-of-timeline surfaced pattern
- `GraphView`: force-graph wrapper
- `SensitiveBadge`: optional indicator when blocked-state history is ever surfaced later

## Event Model

Frontend should listen for:

- `clip-added`
- `intelligence-upgraded`

On `clip-added`:

- invalidate clips and sessions
- update counts
- refresh views without full reload

On `intelligence-upgraded`:

- update mascot state
- show compact toast

## UI Rules

- preserve the warm premium visual language defined in `style.md`
- avoid generic SaaS scaffolding
- prioritize readability of clip content over ornamental UI
- keep graph and mascot secondary to the core memory workflow

## Degradation Rules

- if graph data is sparse, show a sparse-state explanation
- if context generation is missing, still show clip content immediately
- if semantic search is unavailable, render keyword results without breaking layout
- if sessions are weak, display raw clip chronology cleanly

## Frontend Acceptance Criteria

- route transitions feel stable
- the first-run empty state is helpful
- search is understandable without reading docs
- clip detail is action-oriented
- settings are trust-building and clear
