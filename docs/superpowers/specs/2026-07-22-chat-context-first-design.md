# Chat with Mnemo — Context-first redesign

## Goal

Turn the Chat page from a mostly empty conversation shell into a focused, production-like workspace. The page should make recent memory visible before the first question while keeping the conversation, grounded answers, and citations as the primary interaction.

## Design direction

Use a centered, context-first layout with restrained dark surfaces, sage/blue provider accents, and compact controls. The header stays quiet; the current context feed supplies useful density; the composer remains the strongest interactive element.

## UI structure

1. Header
   - Title: “Chat with Mnemo”.
   - Compact `New chat` and `Preview demo` controls.
   - No explanatory privacy banner, empty-state headline, decorative star, or provider marketing copy.

2. Current Context feed
   - Show recent real sessions, recent clips, and meaningful graph/connection activity.
   - Use existing session and clip data; do not fabricate production activity.
   - Items link to existing session reconstruction, clip detail, or graph views.
   - Provide loading, empty, and error states that remain useful without implying activity that does not exist.

3. Start with actions
   - Compact action cards for the existing prompt shortcuts.
   - Include project and system shortcuts only where they map to existing routes or commands.
   - Actions populate or submit the chat composer; they do not create demo data.

4. Composer
   - Larger, dark, high-contrast multi-line input with a prominent send control.
   - Enter submits; Shift+Enter inserts a newline.
   - Keep the composer in the normal page flow so it does not obscure the context feed.
   - Do not add attachment or voice controls until those capabilities exist.

5. Conversation
   - Preserve distinct user and assistant bubbles.
   - Preserve follow-up context by including recent messages in grounded searches.
   - Show loading feedback while searching or generating.
   - Show provider badge and citations inline beneath each assistant answer.
   - Keep the no-relevant-memory fallback explicit and actionable.

## Data flow

- Load clips through the existing clips store.
- Load recent sessions through `list_sessions`.
- Use existing `hybrid_search` and `generate_grounded_answer` for production chat.
- Keep demo mode deterministic and UI-only, with safe seeded responses and citations.
- Reuse existing navigation routes for clips, sessions, graph, Settings, and search.

## Responsive behavior

- Desktop: centered chat column with a two-column context/action area where space allows.
- Narrow screens: single-column feed and action cards; composer remains full width.
- Preserve keyboard focus states and readable contrast for all controls.

## Verification

- TypeScript/Vite production build.
- Existing Vitest suite.
- Manual checks for empty, loading, demo, real answer, citation, follow-up, and narrow viewport states.
- `git diff --check`.
