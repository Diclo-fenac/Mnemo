# Mnemo Ollama Enhancement Plan

> **Goal:** Define the optional local LLM enhancement path using Ollama for richer natural-language memory responses.
>
> **Priority:** Stretch feature, after MVP search and context features are stable.

---

## Purpose

Mnemo does not require an LLM for MVP. Semantic search plus deterministic context generation already covers the core use case.

Ollama is an optional enhancement for:

- more natural responses to open-ended search queries
- synthesizing a short explanation from multiple matching clips
- making session reconstruction feel more narrative when helpful

## Scope

Ollama should enhance existing results, not replace them.

It should not:

- become mandatory
- block core search
- create opaque answers with no clip grounding
- require cloud connectivity

## Product Behavior

Default behavior:

- user enters a query
- Mnemo runs standard semantic + keyword retrieval
- UI shows normal result list

If Ollama is enabled and available:

- Mnemo may also generate a short answer or summary above the result list
- that answer must be grounded in returned clips

The same pattern may optionally enhance:

- Clip Detail explanation
- Session Reconstruction summary

## Configuration

Settings should control:

- `ollama_enabled`
- `ollama_url`

Defaults:

- disabled by default
- URL: `http://localhost:11434`

## Integration Model

Preferred architecture:

- keep the backend responsible for Ollama calls
- frontend requests enhancement through existing or adjacent commands
- backend assembles prompt context from already retrieved clips

Reason:

- keeps prompt construction close to DB/search results
- preserves a single trust boundary

## Query Enhancement Strategy

Recommended flow:

1. run normal search
2. take top N grounded clips
3. build compact prompt using clip excerpts, titles, tags, and timestamps
4. ask Ollama for a short answer or synthesis
5. return answer plus cited clip IDs

Rule:

- never replace the raw result list with an LLM-only answer

## Prompting Rules

Prompt should instruct Ollama to:

- answer only from provided clips
- admit uncertainty when support is weak
- stay concise
- cite relevant clip references by ID or position

Avoid:

- free-form speculation
- broad assistant persona language
- invented facts beyond the retrieved memory set

## Failure Handling

If Ollama is:

- disabled -> do nothing extra
- unavailable -> show normal search only
- slow -> time out and preserve normal search UX
- malformed -> ignore enhancement result and continue normally

LLM failure must never break search.

## UI Behavior

If enabled and successful:

- show a short answer block above results
- include “based on your clips” framing
- allow the user to inspect supporting result cards below

Do not:

- turn Search into a full chatbot
- hide the source clips
- create conversational memory threads for MVP

## Backend Touchpoints

Potential additions:

- `services/ollama.rs` for local client logic
- optional enhancement path inside search service/command
- optional enhancement field on search response payload

Suggested response addition:

- `summary_answer: Option<String>`
- `summary_citations: Vec<String>`

Keep this additive so current search still works unchanged when disabled.

## Performance Rules

- core search results should return first
- Ollama enhancement can be secondary or async if needed
- do not block UI for long generations

## Privacy Notes

- all inference remains local
- only retrieved clip excerpts are sent to local Ollama
- users must explicitly enable the feature

## Suggested Milestones

### Phase 1

- settings support
- Ollama availability check

### Phase 2

- backend prompt assembly
- optional search enhancement response

### Phase 3

- UI answer block
- citation linking to result clips

## Acceptance Criteria

- normal search works unchanged with Ollama disabled
- enabled Ollama can produce grounded short summaries
- failures or timeouts do not degrade baseline search
- answer blocks clearly map back to real saved clips
