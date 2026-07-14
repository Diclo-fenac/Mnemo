# Mnemo Testing Plan

> **Goal:** Define the checks needed to keep Mnemo demo-safe, regression-resistant, and usable across the core memory workflow.

---

## Testing Priorities

Test according to user risk, not code novelty.

Highest priority:

- clipboard capture reliability
- sensitive-content blocking
- DB initialization correctness
- semantic + keyword search behavior
- session grouping
- clip deletion cleanup
- core route rendering and navigation

Medium priority:

- context generation heuristics
- graph payload stability
- settings persistence
- popup interaction behavior

Lower priority:

- purely cosmetic UI details
- advanced pattern-detection nuance

## Backend Tests

### Database

- initializes schema idempotently
- seeds default rows and filter rules
- enables WAL/foreign keys
- creates vector table path safely

### Filtering

- blocks OTP patterns
- blocks credit-card-like strings
- blocks `.env`-like secrets
- allows ordinary content

### Session Builder

- attaches nearby clips to same session
- creates new session past gap threshold
- updates counts and timestamps correctly
- produces generic fallback label when keyword extraction is weak

### Search

- keyword search returns expected exact matches
- semantic merge does not duplicate identical clip result
- fallback keyword mode still returns results when embedding layer is unavailable

### Clip Lifecycle

- insert clip
- pin clip
- copy clip path
- delete clip and clean related FTS/vector/edge state

### Intelligence State

- stage thresholds compute correctly
- upgrade event only fires on transition

## Frontend Tests

### Route Rendering

- app shell renders all main routes
- Timeline empty state appears on fresh install
- Search empty state is friendly and non-broken
- Clip Detail handles missing context gracefully

### Interaction

- expanding/collapsing a session works
- search submission renders results
- opening a result navigates correctly
- pin/delete actions update visible state
- settings toggles reflect backend updates

### Event Handling

- `clip-added` refreshes timeline/search-relevant stores
- `intelligence-upgraded` updates mascot indicator

## Manual Test Scenarios

### First Launch

- install/run app
- verify DB bootstrap
- verify empty Timeline onboarding
- verify routes still open without data

### Clipboard Flow

- copy normal text
- verify clip appears
- copy blocked secret-like value
- verify notification and absence from DB/UI

### Search Flow

- copy several related snippets
- run exact-text query
- run conceptual query
- verify useful merged result ordering

### Session Flow

- copy multiple snippets within 30 minutes
- verify single grouped session
- wait or simulate gap, copy again
- verify new session creation

### Detail and Graph

- open clip detail
- verify context/related clips
- open graph when enough data exists
- verify node click-through

### Settings

- toggle a custom filter rule
- change retention/storage values
- verify persistence after app restart

### Popup

- open shortcut popup
- verify immediate input focus
- search and copy a result
- verify popup dismiss behavior

## Acceptance Bar

Mnemo is ready for hackathon demo use when:

- fresh launch works without setup friction
- copied content reliably appears in Timeline
- blocked content never appears
- Search can recover clips both exactly and semantically
- Session Reconstruction tells a coherent story
- Clip Detail explains context clearly
- the app remains usable when non-core enrichment fails
