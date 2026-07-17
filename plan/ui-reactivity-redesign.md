# Mnemo Adaptive Signal Redesign

> **Purpose:** Replace Mnemo's copyrighted visual reference with an original identity and give the desktop app a responsive consumer-product interface without changing its data or command architecture.

## Approved Direction

`Adaptive Signal System` combines the density and responsiveness of Signal Console with the calm, connected quality of Living Index.

- dark mineral base, not generic black
- muted pastel semantic accents, never neon
- strong contrast from typography, layers, and focus states instead of saturated color
- spacious Timeline and Clip Detail
- compact, keyboard-first Search and Quick Search
- functional motion only, capped at `220ms`

## Brand: Trace Loop

The old mark must be removed from every shipping surface. It resembles copyrighted reference material and cannot remain in the app.

`Trace Loop` is original geometry:

- two open, rounded paths loop toward each other
- one small offset node marks the remembered connection
- the paths never form a wave-and-dot silhouette
- the full-color mark uses sage and mist paths with a muted blush node
- the tray version is one-color and uses only the path silhouette

Asset outputs:

| Surface | Asset |
| --- | --- |
| App bundle | rounded-square charcoal icon with Trace Loop |
| Tray | monochrome Trace Loop, no square container |
| Sidebar | Trace Loop plus `mnemo` wordmark |
| Quick Search | compact Trace Loop only |
| Empty state | oversized Trace Loop watermark at low opacity |

## Color and Contrast

| Token | Value | Use |
| --- | --- | --- |
| `--canvas` | `#161B18` | app background |
| `--surface` | `#202824` | cards and sidebar |
| `--surface-raised` | `#2A342F` | hover and active cards |
| `--ink` | `#F1F3EC` | primary text |
| `--muted` | `#AAB4AC` | metadata |
| `--line` | `#3A4840` | borders |
| `--sage` | `#B8CEA9` | related/memory signals |
| `--mist` | `#AFC9D6` | navigation and graph clusters |
| `--blush` | `#D9B3BA` | saved/pinned emphasis |
| `--butter` | `#D7D99F` | focused primary action |

Rules:

- Pastels only appear on small signals, badges, graph clusters, and active controls.
- Text contrast must meet WCAG AA for normal text.
- Cards use one-pixel mineral borders and small elevation changes, not broad shadows.
- Code stays near-black with a restrained syntax palette.

## Interaction Policy

| Interaction | Duration | Behavior |
| --- | --- | --- |
| hover, press, focus | `120–160ms` | tint, border, or 1–2px lift |
| list insertion, filter response | `160–180ms` | opacity and short vertical settle |
| panel/session expansion | `180–220ms` | height/opacity with no bounce |
| route reveal | `180–220ms` | content-only fade/settle |

- No motion may delay input or navigation.
- No spring, continuous bobbing, or decorative animation.
- `prefers-reduced-motion` disables movement and leaves opacity/state feedback.
- New `clip-added` events receive a one-time insertion highlight; existing clips do not reanimate.

## Density and Screen Strategy

### Timeline and Session Reconstruction

- large page headings, generous session spacing, and reading-oriented line lengths
- sessions become grouped timeline artifacts with a muted topic rail and stateful hover
- clip cards show only source, time, and a short preview until expanded
- reconstruction play mode exposes one clip at a time with the same short motion policy

### Clip Detail

- a two-zone layout: readable saved clip first, context/connection rail second on wide screens
- actions remain visible but quiet
- long content expands deliberately rather than dumping full minified text

### Search and Quick Search

- compact result rows optimized for keyboard movement
- query field and current search mode remain visually dominant
- active row has an obvious pastel edge/focus state, not a large card transformation
- result reasons are compact chips with icons and a plain-language label

### Graph

- deep canvas, muted clusters, and clear selected/related/faded states
- topic filtering dims unrelated content rather than removing spatial context
- controls use consistent compact square buttons

### Settings and Quality

- system-panel grouping, condensed data tables, explicit destructive states
- keep these routes dense because users visit them to operate the app, not read a story

## Responsiveness

- desktop: full side rail, page-specific density, two-zone detail layout
- narrow desktop/tablet: icon rail and single-column detail/context sections
- mobile-width webview: bottom navigation only if a route is reached at that size; desktop remains the primary target
- Quick Search stays fixed-size and never inherits the main app layout

## Non-Goals

- no backend schema or Tauri command changes
- no external design system or component library
- no heavy animation runtime
- no reimplementation of working search, graph, session, or clipboard logic
