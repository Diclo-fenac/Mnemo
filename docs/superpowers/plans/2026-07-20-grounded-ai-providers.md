# Grounded AI Providers Implementation Plan

> **For agentic workers:** REQUIRED: Use subagent-driven development or executing-plans when available. Steps use checkbox syntax for tracking.

**Goal:** Add optional Ollama, OpenAI, and Gemini support for compact, cited search answers while preserving deterministic local search briefs and offline behavior.

**Architecture:** Rust owns provider configuration, prompt construction, HTTP requests, response validation, redaction, and fallback. React only selects the provider, configures it through Tauri commands, and renders a provider-grounded answer when available. The provider is invoked only after local hybrid search identifies the top evidence clips.

**Tech Stack:** Tauri 2, Rust, rusqlite, `ureq`, serde/serde_json, React 19, TypeScript, Vitest.

---

## File Map

- Create `src-tauri/src/services/ai/mod.rs`: provider-neutral types, compact prompt builder, validation, safe errors.
- Create `src-tauri/src/services/ai/ollama.rs`: loopback Ollama adapter.
- Create `src-tauri/src/services/ai/openai.rs`: HTTPS OpenAI adapter.
- Create `src-tauri/src/services/ai/gemini.rs`: HTTPS Gemini adapter with header authentication.
- Create `src-tauri/src/commands/ai.rs`: settings-safe commands, provider health check, grounded answer command.
- Create `src-tauri/src/services/ai/settings.rs`: authoritative AI settings persistence, migration mapping, redacted DTOs, and validation.
- Modify `src-tauri/src/services/db.rs`: settings migrations/defaults for provider configuration and cloud consent.
- Modify `src-tauri/src/commands/mod.rs`, `src-tauri/src/services/mod.rs`, `src-tauri/src/lib.rs`: register modules and commands.
- Modify `src-tauri/src/state.rs`: shared AI request cancellation/cooldown state owned by the backend.
- Modify `src/pages/Settings.tsx`: provider selector, model/URL/key controls, consent and test status.
- Modify `src/components/GroundedBrief.tsx`: async provider answer, local-only action, loading/error/source labels.
- Modify `src/pages/Search.tsx`: request grounded answer using visible result IDs and reject stale responses.
- Modify `src/types/index.ts`: provider settings/status and grounded answer types.
- Modify `src/index.css`: compact AI settings and grounded-answer states.
- Create `src-tauri/src/services/ai/tests.rs` or module tests: prompt budgets, validation, redaction, endpoint policy.
- Create `src/lib/ai.test.ts` only if frontend state helpers need independent tests.

## Chunk 1: Settings Schema and Provider Configuration

### Task 1: Add settings columns and migration tests

**Files:** `src-tauri/src/services/db.rs`, `src-tauri/src/services/ai/settings.rs`

- [ ] Read legacy Ollama columns before adding new provider columns, then migrate only legacy-enabled Ollama settings to `ai_provider = 'ollama'`; keep legacy-disabled settings as `none`.
- [ ] Add columns with safe defaults:
  - `ai_provider TEXT NOT NULL DEFAULT 'none'`
  - `ai_model TEXT NOT NULL DEFAULT 'llama3.2:3b'`
  - `ai_api_key TEXT`
  - `ai_ollama_url TEXT NOT NULL DEFAULT 'http://localhost:11434'`
  - `ai_cloud_consent INTEGER NOT NULL DEFAULT 0`
- [ ] Preserve existing `ollama_enabled`/`ollama_url` values during migration and initialize provider-specific defaults: Ollama `llama3.2:3b`, OpenAI `gpt-4o-mini`, Gemini `gemini-2.0-flash`.
- [ ] Add migration tests for fresh databases and existing databases.
- [ ] Add validation for provider enum, provider-specific model defaults, model length, URL length, and consent requirements.
- [ ] Ensure the read model exposes `hasApiKey`, never the key.
- [ ] Test clearing the key and revoking cloud consent.

### Task 2: Add provider settings commands

**Files:** `src-tauri/src/commands/ai.rs`, `src-tauri/src/lib.rs`, `src-tauri/src/commands/mod.rs`

- [ ] Implement `get_ai_settings` with redacted API key state.
- [ ] Implement `update_ai_settings` with write-only key semantics: a missing key preserves the current key, an explicit clear flag removes it.
- [ ] Reject OpenAI/Gemini activation without a key and cloud consent.
- [ ] Reject arbitrary cloud URLs. Permit a non-loopback Ollama URL only behind an explicit advanced warning; never send cloud credentials to Ollama.
- [ ] Never include keys, request bodies, or full configured URLs in errors/logs.
- [ ] Register commands and write command-level tests.

## Chunk 2: Provider Adapters and Safety Core

### Task 3: Build compact evidence and prompt validation first

**Files:** `src-tauri/src/services/ai/mod.rs`, AI tests

- [ ] Define `Provider`, `ProviderConfig`, `EvidenceClip`, `GroundedAnswer`, `ProviderHealth`, and redacted `AiError` types.
- [ ] Build evidence from query and selected clip IDs, preserving only ID/content/source/time/topics.
- [ ] Enforce limits: query 256 chars, max 5 clips, max 700 chars per clip, max 3,500 evidence chars, max 5,000 serialized input chars, max 500 output tokens.
- [ ] Delimit captured content as untrusted evidence and include prompt-injection instructions.
- [ ] Deduplicate clip IDs and content before serialization.
- [ ] Parse strict JSON only; validate answer, confidence, unique known citations, and citation count.
- [ ] Add tests for Unicode truncation, JSON escaping, malicious evidence, oversized fields, unknown/duplicate citations, and empty answers.

### Task 4: Implement endpoint and request policy

**Files:** `src-tauri/src/services/ai/mod.rs`, AI tests

- [ ] Validate OpenAI/Gemini HTTPS hosts against explicit allowlists.
- [ ] Validate Ollama as loopback HTTP/HTTPS by default.
- [ ] Disable redirects in the HTTP agent.
- [ ] Use bounded connect/read timeouts and a single in-flight provider request.
- [ ] Add backend-owned single-flight request state, cancellation token/request generation, and failure cooldown handling without blocking local search.
- [ ] Redact API keys, evidence, authorization headers, and provider URLs from logs.

### Task 5: Implement Ollama adapter

**Files:** `src-tauri/src/services/ai/ollama.rs`

- [ ] Implement model health check without evidence.
- [ ] Implement compact generation request using the configured local URL/model.
- [ ] Set output limits and request JSON format where supported.
- [ ] Parse provider output into the shared answer type and route through shared validation.
- [ ] Add fixture tests for success, unavailable server, malformed response, and model-not-found response.

### Task 6: Implement OpenAI adapter

**Files:** `src-tauri/src/services/ai/openai.rs`

- [ ] Use the current official Responses API request shape after verifying the official API contract.
- [ ] Send authentication only through the authorization header.
- [ ] Request structured JSON output where supported, with a plain JSON extraction fallback only if the response contract permits it.
- [ ] Configure output token cap and timeout.
- [ ] Add fixture tests for success, authentication failure, rate limit, malformed output, and timeout.

### Task 7: Implement Gemini adapter

**Files:** `src-tauri/src/services/ai/gemini.rs`

- [ ] Use the current official Gemini REST request shape after verifying the official API contract.
- [ ] Send the API key through the approved header mechanism only; never query parameters.
- [ ] Request JSON response mode and output cap where supported.
- [ ] Add fixture tests for success, invalid key, quota failure, malformed output, and timeout.

## Chunk 3: Grounded Search Integration

### Task 8: Add grounded answer command

**Files:** `src-tauri/src/commands/ai.rs`, `src-tauri/src/lib.rs`

- [ ] Implement `generate_grounded_answer(query, clip_ids)`.
- [ ] Re-read clip data by ID from SQLite instead of trusting frontend content.
- [ ] Apply sensitive-content filtering before cloud requests by excluding clips marked `is_sensitive = 1` and clips matching enabled block rules; never redact-and-send uncertain secrets. Add tests proving excluded content is absent from the serialized request.
- [ ] Invoke only the selected provider.
- [ ] Return a typed result indicating provider/local source and fallback reason without exposing secrets.
- [ ] Move deterministic brief construction into the Rust AI service and return a complete typed local answer for every fallback path.
- [ ] On any provider error, return the complete deterministic local brief so search remains successful.
- [ ] Add tests for no provider, cloud consent missing, local-only bypass, successful provider answer, and fallback.

### Task 9: Update Search and GroundedBrief UI

**Files:** `src/pages/Search.tsx`, `src/components/GroundedBrief.tsx`, `src/types/index.ts`, `src/index.css`

- [ ] Keep local search results visible immediately.
- [ ] Request the provider answer using the first five result IDs.
- [ ] Track a request ID/query key and ignore stale responses.
- [ ] Ensure only the active query/provider request can update the brief; backend generation state and frontend request IDs must both invalidate stale work.
- [ ] Add loading state and provider/local source label.
- [ ] Add a local-only action that skips provider requests for the current search.
- [ ] Keep thumbs-up/down/edit/hide/show-less feedback working.
- [ ] Render citations as links to the exact cited clips.
- [ ] Handle no results, provider failure, and malformed response without blanking search results.

### Task 10: Add Settings UI

**Files:** `src/pages/Settings.tsx`, `src/types/index.ts`, `src/index.css`

- [ ] Add provider selector with `None`, `Ollama`, `OpenAI`, and `Gemini`.
- [ ] Show provider-specific URL/model/key fields.
- [ ] Keep API key input write-only and never hydrate the value from Tauri.
- [ ] Clear the API key input from React state immediately after a successful save or clear operation.
- [ ] Add cloud disclosure and consent control before OpenAI/Gemini activation.
- [ ] Before any OpenAI/Gemini request, including connection tests, show a provider-named confirmation explaining that selected excerpts or request metadata leave the device; handle confirmation race-safely and persist consent only after confirmation.
- [ ] Before saving a cloud API key, disclose that SQLite settings storage exposes the key to local database readers and backups.
- [ ] Add test connection action that sends no clipboard evidence and visibly warns that cloud health checks may incur provider charges.
- [ ] Add clear-key and revoke-consent actions.
- [ ] Display redacted status and safe error messages.

## Chunk 4: Verification

### Task 11: Test and audit

- [ ] Run Rust unit tests and integration tests for every adapter/prompt policy.
- [ ] Run `npm test` and add UI tests for provider selection, local-only search, stale response handling, and citation links.
- [ ] Run `npm run build` and `cargo check`.
- [ ] Verify no API key appears in logs, Tauri responses, browser state, SQLite feedback, or clip records.
- [ ] Verify no request is sent when provider is `none` or local-only is selected.
- [ ] Verify cloud requests require explicit consent and safe endpoints.
- [ ] Verify backend cancellation, single-flight, and cooldown state prevents stale or repeated provider side effects.
- [ ] Verify local deterministic answers remain available when all providers are offline.
