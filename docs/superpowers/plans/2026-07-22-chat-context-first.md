# Chat Context-first Redesign Implementation Plan

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make Chat with Mnemo feel like a focused production workspace by adding real current context, useful action groups, and a stronger composer around the existing grounded conversation flow.

**Architecture:** Keep `Chat.tsx` responsible for chat state and page composition, but extract presentation-only context/action helpers into focused components. Load existing clips from `useClipsStore` and recent sessions through `list_sessions`; do not add new persistence or fabricated production data. Extend the existing chat CSS with a context-first responsive layout while preserving current provider/citation behavior.

**Tech Stack:** React 19, TypeScript, React Router, Tauri `invoke`, Zustand clips store, lucide-react, plain CSS, Vitest.

---

## File map

- Create `src/components/ChatContextFeed.tsx` — renders recent sessions, clips, and connection activity from supplied data.
- Create `src/components/ChatActionGrid.tsx` — renders grouped, route-aware chat shortcuts.
- Modify `src/pages/Chat.tsx` — load context data, compose the new sections, wire actions into the existing composer and routes.
- Modify `src/index.css` — implement the centered context-first layout, feed cards, action groups, composer hierarchy, and responsive states.
- Create `src/components/ChatContextFeed.test.tsx` only if the repository test setup can render React components without adding a new testing dependency; otherwise verify behavior through build and manual states.

## Chunk 1: Context data and presentation boundaries

### Task 1: Add context feed component

**Files:**
- Create: `src/components/ChatContextFeed.tsx`
- Modify: `src/types/index.ts` only if a small view-model type is needed

- [ ] Define props for `clips: Clip[]`, `sessions: SessionSummary[]`, and loading/error state.
- [ ] Render the active/recent session first when available, linking to `/session/:id`.
- [ ] Render the latest clips with relative time, source provenance, and links to `/clip/:id`.
- [ ] Render an empty state that says what to do next without claiming activity exists.
- [ ] Keep the component presentation-only; no Tauri calls or store mutation inside it.
- [ ] Run `npm run build` and confirm the component compiles.

### Task 2: Add action grid component

**Files:**
- Create: `src/components/ChatActionGrid.tsx`

- [ ] Define a small action model with label, group, icon, and callback or route.
- [ ] Include real actions for the three existing prompts, latest project/session, graph, and Settings.
- [ ] Make prompt actions populate the composer; make navigation actions use `Link`.
- [ ] Render grouped sections with accessible button/link labels.
- [ ] Keep the grid compact and usable on narrow screens.
- [ ] Run `npm run build` and confirm no route/type errors.

## Chunk 2: Compose the Chat page

### Task 3: Load context data in `Chat.tsx`

**Files:**
- Modify: `src/pages/Chat.tsx`

- [ ] Use `useClipsStore` and `fetchClips` to load current clips consistently with Memory and Timeline.
- [ ] Load a bounded recent session list using `invoke<SessionSummary[]>("list_sessions", { limit: 6 })`.
- [ ] Track independent session loading/error state so a failed context request does not disable chat.
- [ ] Pass real data to `ChatContextFeed` and `ChatActionGrid`.
- [ ] Preserve demo mode, follow-up context, loading feedback, no-result fallback, provider labels, and citations.

### Task 4: Wire actions into the existing composer

**Files:**
- Modify: `src/pages/Chat.tsx`

- [ ] Replace the empty prompt-only area with the context feed and action grid.
- [ ] Ensure prompt actions populate the textarea without submitting unexpectedly.
- [ ] Ensure the main composer remains the only submit path and still supports Enter/Shift+Enter.
- [ ] Keep `New chat` and demo controls compact in the header.
- [ ] Verify a follow-up question still includes the last six messages in the grounded query.

## Chunk 3: Visual system and responsive polish

### Task 5: Implement the context-first layout

**Files:**
- Modify: `src/index.css`

- [ ] Center the chat workspace at roughly 900px and reduce unused vertical space.
- [ ] Style the context feed as a calm, information-dense surface with clickable rows.
- [ ] Style actions as grouped cards with clear hierarchy for Quick actions, Projects, and System.
- [ ] Make the composer darker with a stronger border, larger textarea, and prominent send button.
- [ ] Keep provider colors limited to answer badges/citation accents.
- [ ] Add narrow-screen rules for one-column context/actions and full-width composer.
- [ ] Preserve visible focus states and avoid decorative controls that do not work.

## Chunk 4: Verification and handoff

### Task 6: Verify behavior

**Files:**
- Test: existing `src/lib/*.test.ts`

- [ ] Run `npm run build`.
- [ ] Run `npm test -- --run`.
- [ ] Run `git diff --check`.
- [ ] Manually verify: recent context, no clips, no sessions, context-load failure, demo answer, real answer, citations, follow-up, new chat, Enter submit, Shift+Enter newline, and narrow viewport.
- [ ] Review the final diff and confirm unrelated working-tree changes remain unstaged.

### Task 7: Commit the implementation

- [ ] Stage only the Chat redesign files and any directly related tests.
- [ ] Commit with `feat: add context-first chat workspace`.
- [ ] Report the commit and verification results; do not push unless explicitly requested.
