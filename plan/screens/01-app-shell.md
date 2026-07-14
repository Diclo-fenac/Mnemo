# App Shell

> **Purpose:** Define the persistent structure around every Mnemo route and system-level desktop behavior.
>
> **Access:** Always available after app launch.
>
> **Entry Points:** App launch, tray reopen, route navigation, popup handoff.
>
> **Primary Outcome:** Give users a stable navigation frame for moving through memory, search, graph, and settings.

---

## Core UI

- left sidebar with Mnemo brand mark and primary routes
- main content region with route outlet
- fixed bottom-right mascot/intelligence indicator
- consistent page padding, max-width behavior, and responsive collapse rules

## Data Shown

- current route
- memory stage
- total clip count
- route-level loading or error shell states when needed

## User Actions

- navigate between routes
- open clip detail from shared interactions
- observe stage changes
- return from popup into full app when needed

## States

- Loading: route skeletons within the main content frame
- Empty: handled by individual routes, not the shell
- Error: route-level error panel without collapsing navigation
- Degraded / fallback: shell renders even if one route fails to load

## Events and Side Effects

- listens for `clip-added` and refreshes stores
- listens for `intelligence-upgraded` and shows compact toast
- preserves navigation state while data refreshes in background

## Navigation and Dependencies

- depends on router and shared layout components
- sidebar links to Timeline, Search, Memory Graph, Settings
- shell owns mascot indicator placement and cross-route chrome

## Edge Cases

- popup-origin navigation into a deep link should preserve back behavior sensibly
- shell should stay usable if graph or search data temporarily fails
- very small desktop widths should collapse sidebar without feeling mobile-first

## Acceptance Notes

- all primary routes are reachable in one click
- shell remains visually consistent across screens
- mascot stays minimal and non-gamified
