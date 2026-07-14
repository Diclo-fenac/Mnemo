# Mnemo Backend Plan

> **Goal:** Define the Rust/Tauri backend needed to capture, enrich, store, and retrieve clipboard memory reliably.

---

## Runtime Structure

Backend modules should be organized exactly as the product spec describes:

- `main.rs`: builder, plugin registration, windows, shortcut, startup wiring
- `lib.rs`: module exports
- `state.rs`: `AppState`
- `commands/*`: Tauri command surface
- `services/*`: DB and intelligence pipeline
- `models/*`: serializable data contracts

## AppState

`AppState` should contain:

- shared SQLite connection guard
- shared embedder instance guard
- optional runtime config cache if needed later

Use synchronized shared state because both command handlers and event-driven pipeline code will access the same core resources.

## Database Layer

`services/db.rs` should:

- create app data directories
- create image storage directory
- open SQLite with WAL mode
- enforce foreign keys
- run schema creation idempotently
- load sqlite-vec
- create the vector table
- seed default settings, memory state, and filter rules

Decisions:

- use a single startup initialization path
- keep schema bootstrap local and code-driven for MVP
- preserve deterministic startup even on existing databases

## Clipboard Pipeline

`services/watcher.rs` is the orchestrator.

Pipeline order:

1. receive clipboard change
2. ignore unsupported/empty content
3. run sensitive-content check
4. resolve active app/window with fallback to `Unknown`
5. insert clip row
6. generate embedding
7. store vector row and back-reference
8. compute memory edges
9. generate context JSON
10. assign session
11. every 10 clips, detect patterns
12. update memory state
13. emit frontend event

Rule:

- if enrichment steps fail after clip insert, the clip should still remain stored unless integrity is broken

## Embeddings

`services/embedder.rs` should:

- initialize the fastembed model once
- expose `embed_text` and `embed_query`
- normalize short and long text safely
- return meaningful errors without panicking the app

Fallback:

- if embeddings fail at runtime, keyword search still works and the clip remains persisted

## Filtering

`services/filter.rs` should:

- load enabled filter rules
- evaluate regex patterns
- return blocked/allowed decision

For MVP:

- treat `block` as authoritative
- treat `ask` as future-capable but initially degrade to `block` or a simple local notification note if interactive notification buttons are not ready
- treat `allow` as pass-through

## Session Builder

`services/session_builder.rs` should:

- locate nearest qualifying recent session
- update or create session row
- relabel session after assignment
- compute keyword-based title and summary

Fallback rules:

- if keyword extraction is weak, use a generic label like `Research Session`
- never leave a session structurally broken because labeling failed

## Memory Graph

`services/memory_graph.rs` should:

- search nearest neighbors from sqlite-vec
- convert distance to similarity
- write deduplicated semantic edges
- expose related-clip lookup

Rule:

- only edges above threshold become persistent graph links

## Context Generator

`services/context_generator.rs` should:

- infer source from page title or app name
- infer topic tags from heuristics
- infer likely purpose from source + tags
- attach top related clip IDs
- save JSON to `clips.ai_context`

Rule:

- context should be explanatory, not verbose

## Pattern Detector

`services/pattern_detector.rs` should:

- scan accumulated clips periodically
- group recurring tags and recurring domains
- persist facts with dedupe semantics

Rule:

- only generate facts when there is enough repetition to feel meaningful

## Intelligence State

`services/intelligence.rs` should:

- refresh counters
- compute stage
- emit upgrade event only on actual transition

Stages:

- `clippy`
- `bindor`
- `archivor`

## Commands

### Clips

- list clips with pagination
- read single clip
- delete clip and clean related data
- pin/unpin
- copy clip back to clipboard
- list clips by session

### Search

- merged semantic + keyword search
- dedupe and score merge
- attach `search_type` and `match_reason`

### Memory

- read memory state
- list sessions
- reconstruct session
- read facts
- read/generate clip context
- list related clips
- build graph payload

### Settings

- read/write settings
- list/add/delete/toggle filter rules

## Active Window Detection

Use best-effort platform-specific lookup, but do not block persistence if it fails.

Initial stance:

- macOS, Windows, Linux all allow fallback to `Unknown`
- stubs are acceptable if the feature boundary is isolated cleanly

## Error Handling

- never crash the app on enrichment failure
- surface frontend-safe command errors
- use structured logging for pipeline failures
- preserve core clip persistence whenever possible

## Backend Acceptance Criteria

- startup is idempotent
- clipboard capture is reliable
- blocked content never lands in DB
- search remains usable when embeddings are unavailable
- deletion cleans associated search/vector/edge records
- stage counters remain consistent with DB truth
