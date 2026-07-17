# Semantic Engine Hardening Implementation Plan

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make Mnemo's local semantic engine measurable, model-switchable, duplicate-aware, hybrid-ranked, and recoverable during background re-embedding.

**Architecture:** Keep SQLite as the source of truth. Store provenance and migration state in ordinary tables, keep one active sqlite-vec table plus a temporary migration table, and expose migration progress through Tauri commands. Search uses routed FTS plus vector retrieval, optional reranking, and deterministic weighted scoring.

**Tech Stack:** Rust, Tauri 2, rusqlite/FTS5, sqlite-vec, fastembed 3.14.1, React, Zustand.

---

## Chunk 1: Schema and provenance

- [x] Pin `fastembed` to `3.14.1`.
- [x] Add schema migration versioning and provenance columns.
- [x] Add `embedding_registry`, `search_feedback`, `clips_fts_code`, and migration metadata.
- [x] Add normalization, BLAKE3-compatible content hashing, duplicate lookup, and source-intent helpers.
- [x] Add unit tests for normalization and source intent.

## Chunk 2: Embedding lifecycle

- [x] Replace the current embedder loop with a batch job using model metadata.
- [x] Mark clips pending/embedded/failed/skipped.
- [x] Embed new canonical clips and migrate with a dimension-specific temporary vector table.
- [x] Add model switch command with resumable batches and atomic active-index swap.
- [x] Add embedding coverage metrics.

## Chunk 3: Search quality

- [x] Implement safe FTS query construction for prose and code routes.
- [x] Normalize semantic, FTS, and recency scores into `[0, 1]`.
- [x] Apply configured 55/35/10 fusion.
- [x] Load one fixed BGE reranker lazily and rerank top 5 when available.
- [x] Return match reasons and search type.
- [x] Log impressions, copy-again, and empty searches.

## Chunk 4: Graph quality and dashboard

- [x] Generate real semantic-temporal edges with max-edge and diversity limits.
- [x] Store edge type and temporal weight separately from raw similarity.
- [x] Replace mock graph links with database edges.
- [x] Add quality dashboard metrics and `/quality` route.

## Chunk 5: Verification

- [x] Add 50 local labeled benchmark pairs and threshold/F1 helper.
- [ ] Add migration crash/restart, duplicate, search routing, and graph-limit tests.
- [ ] Run `cargo fmt`, `cargo test`, `cargo check`, and frontend build where host dependencies permit.
- [ ] Document Linux `pkg-config`/GTK prerequisites and model cache behavior.
