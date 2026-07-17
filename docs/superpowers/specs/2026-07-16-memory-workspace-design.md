# Mnemo Memory Workspace Design

## Status

Approved by the user on 2026-07-16. This specification supersedes the current
Timeline-first presentation, without changing Mnemo's local-first storage or
capture architecture.

## Product Intent

Mnemo must feel like a private, useful memory workspace rather than a passive
clipboard logger. It prioritizes immediate retrieval, clear user agency over
capture, and grounded insights that can be traced back to the source clips.

## Information Architecture

- `/` becomes **Memory**, an information-rich dashboard.
- `/timeline` becomes the session archive.
- Search is entered from the persistent sidebar command field or `CmdOrCtrl+K`.
  `/search` remains the focused results surface and quick-search popup target,
  but is not primary navigation.
- `/graph` opens as a cluster summary and drills into a focused connection
  explorer.
- `/settings` owns product settings, privacy controls, appearance, retention,
  model selection, and advanced quality diagnostics.
- `Quality` is removed from primary navigation and remains accessible through
  Settings as advanced diagnostics.

## Memory Dashboard

The dashboard contains these ordered surfaces:

1. A persistent `Ask Mnemo` entry point.
2. A short, grounded and cited memory brief for completed queries.
3. A resume-session card for the most relevant recent session.
4. A connection-health summary: real edges, active clusters, and a direct link
   to explore them.
5. Recent capture cards.
6. A desktop-right live activity rail that shows recent captures as they arrive.
   It becomes a slide-over panel on narrow windows.

Each activity item shows a two-line preview, source, time, and session. Hover
actions are Pin, Copy, and Open.

## Capture Agency and Privacy

- Capture has an explicit, persistent on/off control.
- When capture is off, Mnemo does not read or persist clipboard changes.
- `CmdOrCtrl+Shift+V` continues to search existing local memories while capture
  is off.
- `CmdOrCtrl+Shift+M` toggles capture by default and is rebindable in Settings.
- Browser URL/title enrichment is independently opt-in and disabled by default.
- Browser extension metadata is authoritative for web provenance: canonical URL,
  page title, favicon, and optionally selected text. Native window detection is
  used for non-browser apps and as the browser fallback only.
- Sensitive-content blocks produce a generic activity event only. No preview,
  source, normalized value, or retained hash is stored or shown.
- The default retention policy is automatic deletion. The exact period remains
  a Settings value and must be visible before the first capture.

## Sessions

Sessions auto-group by default. Users can rename, pin, merge, and split sessions
when the grouping is incorrect. Manual corrections must survive later automatic
analysis.

## Grounded Intelligence and Feedback

- Query results begin with a short, local, grounded memory brief and cite the
  clips or sessions supporting each claim.
- Deterministic extractive summaries are always available offline.
- Optional Ollama may improve phrasing only; it cannot replace evidence or
  create uncited claims.
- Every AI-generated memory brief, context card, and session label provides
  thumbs up/down feedback.
- Thumbs down offers Edit, Hide, and Show less like this.
- Feedback is local-only and may tune rank/presentation behavior; it must not
  leave the device.
- Insights appear beside relevant clips and queries. An optional daily brief is
  available and disabled by default.

## Graph Behavior

The graph is not a canvas of all clips. It is a real-relationship explorer.

- Initial view: topic clusters with counts, connection types, and cluster
  summaries.
- Drill-down: select a cluster or clip to open a focused connection explorer.
- Isolated clips do not appear as meaningless dots. They appear as an
  `Unconnected captures` count/list.
- While embeddings or edges are unavailable, show a clear building state that
  describes what will create the first connection and links to recent captures.
- Semantic and temporal relationships are visibly labeled.

## Settings and Appearance

- Dark is the default appearance. System and Light are manual options.
- Native light form controls must not leak into the dark UI; model selection is
  a custom, theme-aware control.
- Brighter muted pastels communicate meaningful state, data grouping, capture
  status, and connection strength. They are not decorative noise.
- Clear Database moves under Local Data and uses a two-step typed confirmation
  (`DELETE`). It is discoverable but not a persistent alarm-red control.

## Visual and Interaction Direction

- Dark mineral surfaces, brighter muted pastel accents, readable contrast, and
  meaningful empty states replace broad blank canvas space.
- Timeline and detail views remain spacious; dashboard, search, and quick
  search are compact and information-dense.
- Motion is causal and respects reduced-motion preferences: 140ms controls,
  180ms selections, 220ms panels maximum.

## Reliability Rules

- A model-loading or edge-generation delay cannot look like a broken graph.
- UI state must distinguish: no clips, clips pending embeddings, clips with no
  edges, and graph data failure.
- Capture status and browser-context status must be available without opening
  Settings.
- Every destructive action has clear scope and a recoverable cancellation path.

## Verification

- Unit-test capture state, retention settings, grounded-feedback state, and
  graph-state derivation.
- Verify all routes in Dark, Light, and System themes.
- Verify `CmdOrCtrl+Shift+V` works with capture disabled.
- Verify `CmdOrCtrl+Shift+M` toggles actual watcher behavior, not just UI.
- Verify model-loading, edge-free, and graph-error states independently.
