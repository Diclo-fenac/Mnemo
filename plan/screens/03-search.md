# Search

> **Purpose:** Let users query Mnemo’s memory using semantic and keyword search.
>
> **Access:** Main route in the app shell.
>
> **Entry Points:** `/search`, sidebar click, handoff from popup.
>
> **Primary Outcome:** Help users find a past clip quickly, even when they only remember the idea rather than the exact text.

---

## Core UI

- large chat-style search input at top
- rotating placeholder examples when idle
- filter chips above results
- vertical result list using clip-like cards
- subtle metadata and search-type badges

## Data Shown

- search query
- ranked results
- result preview content
- source app or page title
- copied timestamp
- search type: semantic, keyword, or both
- match reason

## User Actions

- submit a memory query
- refine with filters
- open a clip detail result
- repeat or clear the search

## States

- Loading: active search state with skeleton result cards
- Empty: warm prompt state with example queries
- Error: search error with retry and query preserved
- Degraded / fallback: keyword-only results if embedding search is unavailable

## Events and Side Effects

- calls merged semantic and FTS search
- caches recent query state in the store for smoother navigation return
- may re-rank or merge duplicate results client-side

## Navigation and Dependencies

- links to Clip Detail
- depends on search command, filters, clip metadata, and shared card styling

## Edge Cases

- empty or whitespace-only queries should not trigger noisy errors
- if semantic ranking fails, keyword search should still feel useful
- long code snippets need clipped previews without destroying readability

## Acceptance Notes

- query entry feels central and fast
- result cards explain why the clip appeared
- keyword-only fallback does not break the screen
