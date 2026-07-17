# Mnemo Adaptive Signal Implementation Plan

> **For agentic workers:** REQUIRED: Use subagent-driven development if subagents are available, otherwise execute tasks in order with tests after each task.

**Goal:** Ship an original Trace Loop brand and a reactive, adaptive-density UI while preserving all current Mnemo routes, stores, Tauri commands, and local-first behavior.

**Architecture:** Keep React route components and Zustand stores as they are. Introduce a small CSS-token and presentation layer for visual state, use CSS transitions for all interaction feedback, and replace only frontend/bundle visual assets. No backend contract changes are required.

**Tech Stack:** React 19, TypeScript, Vite, Tailwind CSS 4, Zustand, lucide-react, CSS custom properties, Vitest.

---

## File Map

| Path | Responsibility |
| --- | --- |
| `src/assets/trace-loop.svg` | color Trace Loop brand mark |
| `src/assets/trace-loop-mono.svg` | monochrome tray-safe mark |
| `src-tauri/icons/icon.png` | new app bundle icon rendered from Trace Loop |
| `src-tauri/icons/32x32.png` | small bundle icon rendered from Trace Loop |
| `src/index.css` | Adaptive Signal tokens, responsive shell, motion primitives, component styles |
| `src/components/BrandMark.tsx` | shared mark/wordmark variants |
| `src/components/Sidebar.tsx` | responsive brand/navigation rail |
| `src/components/ClipCard.tsx` | compact/detailed density and action feedback |
| `src/pages/Timeline.tsx` | spacious timeline, new-clip visual feedback |
| `src/pages/Search.tsx` | dense keyboard-first result surface |
| `src/pages/QuickSearchPopup.tsx` | compact command-surface behavior and selected state |
| `src/pages/ClipDetail.tsx` | reading-first responsive two-zone composition |
| `src/pages/SessionReconstruction.tsx` | deliberate staged playback and source hierarchy |
| `src/pages/MemoryGraph.tsx` | muted clusters and selection/fade states |
| `src/pages/Settings.tsx` | system-dense settings grouping |
| `src/lib/presentation.ts` | pure helpers for UI state classes and motion-safe time formatting |
| `src/lib/presentation.test.ts` | regression tests for new pure UI helpers |
| `plan/style.md` | replace obsolete warm/amber style rules |
| `plan/brand-icon.md` | replace obsolete Bookmark Arc decision with Trace Loop |

## Chunk 1: Brand and Foundation

### Task 1: Replace the brand assets

**Files:**
- Create: `src/assets/trace-loop.svg`
- Create: `src/assets/trace-loop-mono.svg`
- Modify: `src/components/Sidebar.tsx`
- Modify: `src/pages/Timeline.tsx`
- Modify: `src/pages/QuickSearchPopup.tsx`
- Modify: `src-tauri/icons/icon.png`
- Modify: `src-tauri/icons/32x32.png`

- [ ] Define Trace Loop as original SVG geometry: two non-wave paths plus offset node.
- [ ] Render a charcoal rounded-square 512px app icon and a true 32px version.
- [ ] Create `BrandMark` with `full`, `mono`, and `wordmark` variants.
- [ ] Replace every `mnemo-mark.svg` import and empty-state mark with `BrandMark`.
- [ ] Verify no shipping UI imports the old copyrighted asset.
- [ ] Run `npm run build` and `cargo check`.
- [ ] Commit: `feat(ui): replace Mnemo brand assets`.

### Task 2: Install Adaptive Signal tokens and motion primitives

**Files:**
- Modify: `src/index.css`
- Modify: `src/main.tsx`
- Modify: `plan/style.md`
- Modify: `plan/brand-icon.md`

- [ ] Write a failing presentation test for the `prefers-reduced-motion` class helper if one is introduced.
- [ ] Add mineral-dark semantic tokens and WCAG-safe text/border tokens.
- [ ] Define `--motion-fast: 140ms`, `--motion-standard: 180ms`, and `--motion-panel: 220ms`.
- [ ] Add shared focus-visible, hover, pressed, selected, skeleton, and inserted-item primitives.
- [ ] Preserve and extend the reduced-motion override; do not remove keyboard focus outlines.
- [ ] Update both planning docs to mark Trace Loop and Adaptive Signal as the source of truth.
- [ ] Run `npm test` and `npm run build`.
- [ ] Commit: `feat(ui): add adaptive signal design tokens`.

## Chunk 2: App Shell and Reactive Surfaces

### Task 3: Rebuild navigation as a responsive signal rail

**Files:**
- Modify: `src/components/Sidebar.tsx`
- Modify: `src/App.tsx`
- Modify: `src/index.css`

- [ ] Add clear active navigation state with mist edge, text contrast, and `140ms` feedback.
- [ ] Keep desktop wordmark rail; collapse to icons below the defined narrow width.
- [ ] Ensure keyboard focus order and route labels remain accessible.
- [ ] Keep the popup route outside main-shell navigation.
- [ ] Run `npm run build`.
- [ ] Commit: `feat(ui): redesign responsive app shell`.

### Task 4: Make Timeline visibly live without becoming noisy

**Files:**
- Modify: `src/pages/Timeline.tsx`
- Modify: `src/components/ClipCard.tsx`
- Modify: `src/hooks/useClipEvents.ts`
- Modify: `src/index.css`

- [ ] Add a store/event marker for the newest clip ID after `clip-added`.
- [ ] Apply a one-time `160–180ms` inserted state only to that clip/session entry.
- [ ] Rework session groups into spacious, topic-railed artifacts with clear expand/collapse states.
- [ ] Preserve existing fetch/error/loading behavior and expand-all actions.
- [ ] Verify collapse/expand is usable with keyboard and reduced motion.
- [ ] Run `npm test` and `npm run build`.
- [ ] Commit: `feat(ui): make timeline stateful and reactive`.

### Task 5: Compact Search and Quick Search

**Files:**
- Modify: `src/pages/Search.tsx`
- Modify: `src/pages/QuickSearchPopup.tsx`
- Modify: `src/components/ClipCard.tsx`
- Modify: `src/index.css`

- [ ] Keep search rows compact and content-first; avoid expanded card spacing used by Timeline.
- [ ] Make selected keyboard result unambiguous with a pastel edge and surface tint.
- [ ] Add short loading-to-result state transition with no input delay.
- [ ] Preserve arrow, Enter, Escape, and copy-back behavior.
- [ ] Ensure no selected-state animation exceeds `160ms`.
- [ ] Run `npm run build`.
- [ ] Commit: `feat(ui): refine search command surfaces`.

## Chunk 3: Reading, Graph, and System Screens

### Task 6: Recompose Clip Detail and Session Reconstruction

**Files:**
- Modify: `src/pages/ClipDetail.tsx`
- Modify: `src/pages/SessionReconstruction.tsx`
- Modify: `src/index.css`

- [ ] Implement wide-screen content/context zones; collapse to one column at narrow widths.
- [ ] Keep clip content primary, context secondary, and actions visible but quiet.
- [ ] Rework reconstruction playback into a controlled `180–220ms` chronological reveal.
- [ ] Preserve long-content expansion, copy confirmation, loading, and error states.
- [ ] Test reduced motion manually: reveal controls still advance content without movement.
- [ ] Run `npm run build`.
- [ ] Commit: `feat(ui): improve reading and reconstruction surfaces`.

### Task 7: Clarify graph state and system routes

**Files:**
- Modify: `src/pages/MemoryGraph.tsx`
- Modify: `src/pages/Settings.tsx`
- Modify: `src/pages/Quality.tsx`
- Modify: `src/index.css`

- [ ] Apply pastel topic cluster colors at low saturation against a mineral canvas.
- [ ] Add selected/related/dimmed graph states without changing graph data or simulation.
- [ ] Make graph controls consistently compact and keyboard-focusable.
- [ ] Tighten Settings and Quality into dense system panels with explicit destructive/disabled states.
- [ ] Run `npm run build`.
- [ ] Commit: `feat(ui): clarify graph and system surfaces`.

## Chunk 4: Verification and Release Assets

### Task 8: Audit for old branding and visual regressions

**Files:**
- Modify as required by audit results
- Test: `src/lib/presentation.test.ts`

- [ ] Run `rg -n 'mnemo-mark|Bookmark Arc|Golden Amber|F8C557' src src-tauri plan` and remove stale shipping references.
- [ ] Test all routes at desktop and narrow widths: Timeline, Search, Session, Graph, Clip Detail, Settings, Quality, popup.
- [ ] Confirm shortcut, search selection, clip copy/pin/delete, graph controls, and session expansion remain functional.
- [ ] Check `prefers-reduced-motion` manually or through browser emulation.
- [ ] Run `npm test`, `npm run build`, `cargo fmt --check`, `cargo check`, `cargo test`, and `git diff --check`.
- [ ] Run `npm run tauri dev`; verify no console errors, no Vite reload loops, and correct app/tray icons.
- [ ] Commit: `test(ui): verify adaptive signal release`.

## Acceptance Criteria

- No copyrighted reference mark remains in a shipping route or icon.
- Trace Loop reads clearly at 32px and as a monochrome tray mark.
- Timeline and Detail are visibly more spacious than Search and Quick Search.
- All feedback uses `120–220ms` motion or less and honors reduced-motion preferences.
- New clipboard events visibly enter the Timeline without reanimating the whole page.
- Existing Tauri commands, keyboard shortcuts, and user flows remain unchanged and build successfully.
