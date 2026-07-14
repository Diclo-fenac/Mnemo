# Mnemo Implementation Plan

> **Goal:** Build Mnemo as a local-first Tauri desktop app that captures clipboard history, enriches it with embeddings and session/context metadata, and exposes it through a refined desktop UI.
>
> **Primary Constraint:** No auth, no cloud, no remote server, no required external dependency for MVP.

---

## Build Order

The app should be built in this order:

1. app shell and local database boot
2. clipboard capture pipeline
3. clip persistence and retrieval
4. embedding generation and semantic search
5. session grouping and timeline UI
6. clip context and related clips
7. memory graph and pattern detection
8. settings, storage controls, and polish

This keeps the Day 1 MVP aligned with the original priority order while preserving the higher-level UX already documented.

## Milestone 1: Foundation

- scaffold Tauri 2 + React 19 + Vite + TypeScript + Tailwind 4 project structure
- create the Rust module layout from the spec
- initialize SQLite in WAL mode in the app data directory
- create migrations/schema bootstrap in a single deterministic startup path
- create `AppState` with shared database and embedder handles
- register Tauri commands and route-safe error handling
- wire main window and hidden popup window definitions

Deliverable:

- app launches
- DB initializes correctly
- all major routes render placeholder screens inside the final app shell

## Milestone 2: Clipboard Memory Core

- integrate clipboard change listening
- normalize captured content into supported content types
- run sensitive-content filtering before persistence
- capture fallback source metadata with best-effort active-window lookup
- insert clips into the database
- emit `clip-added` to the frontend

Deliverable:

- copied text shows up in Timeline without requiring restart
- blocked content is not stored

## Milestone 3: Search and Session Intelligence

- initialize fastembed once at startup
- embed new clips after insert
- store vectors in sqlite-vec
- build merged semantic + FTS search
- assign clips to sessions using 30-minute gap logic
- generate readable session labels and summaries

Deliverable:

- searches return useful results for both exact text and remembered meaning
- Timeline groups clips into meaningful sessions

## Milestone 4: Memory Features

- compute related-clip edges for new inserts
- generate clip context and likely purpose
- expose related clips and memory facts to the frontend
- build session reconstruction route
- build graph visualization route with recent-node limit

Deliverable:

- Clip Detail explains why a clip mattered
- Graph and Session Reconstruction demonstrate cross-clip memory

## Milestone 5: Control and Polish

- settings reads/writes
- filter-rule CRUD
- clip pin/copy/delete flows
- storage limits and cleanup paths
- stage indicator and stage-upgrade notifications
- tray interaction and popup polishing
- icon integration and final brand assets

Deliverable:

- the app feels shippable for hackathon demo use

## Architecture Decisions

### Backend

- Rust owns persistence, embedding, filtering, sessioning, graph edges, and search
- frontend calls Tauri commands and listens to events
- no frontend-side business logic that duplicates backend decisions

### Data Model

- SQLite is the source of truth
- FTS5 handles text search
- sqlite-vec handles semantic retrieval
- clip context is stored as JSON for fast reuse and simple incremental enhancement

### UI State

- Zustand stores should cache route-relevant data only
- commands remain the authority for persistence mutations
- stores should optimistically update only for low-risk actions like pin toggles if needed

## Implementation Rules

- fail soft on active-window lookup
- do not block clip capture on enrichment features unless storage safety requires it
- degrade to keyword search if embeddings are unavailable
- degrade to raw clips if session/context generation partially fails
- prioritize determinism and demo reliability over overly clever async orchestration

## Acceptance Criteria

- new clips appear without manual refresh
- local privacy controls are understandable
- semantic search returns non-exact matches
- sessions reconstruct the user’s recent work plausibly
- graph view remains stable and readable
- the app remains useful even if advanced enrichment partially fails
