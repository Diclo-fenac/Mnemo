# Mnemo Visual Style

> **Purpose:** Define the visual language for Mnemo so the app feels calm, premium, and consistent across desktop surfaces.
>
> **Design Intent:** Mnemo should feel like a refined local-first utility. The interface should be warm, quiet, and deliberate, not flashy or overly “AI.”

---

## Core Direction

Mnemo uses an adaptive signal aesthetic:

- editorial spacing instead of dense dashboards
- large, restrained typography
- soft rounded geometry
- deep mineral surfaces with muted pastel signals
- functional motion capped at 220ms
- original Trace Loop branding

The app should feel closer to a polished macOS utility than a generic SaaS admin panel.

## Visual Principles

### Calm Before Clever

The product is technically sophisticated, but the UI should not perform intelligence theatrically. Avoid glowing gradients, mascot-heavy framing, and noisy visualizations.

### One Signature Mark

The brand should rely on one strong symbol that scales from tray icon to empty-state watermark. It must survive monochrome rendering and tiny sizes.

### Subtle Signal Contrast

Primary surfaces use deep mineral charcoal. Sage, mist blue, blush, and butter are reserved for focus, topic, and relationship signals. Structural contrast comes from value, type, and focus states rather than bright accents.

### Big Type, Short Copy

Page titles and hero text should be confident and spacious. Supporting copy should be direct and brief.

## Color System

- `Canvas`: `#161B18`
- `Surface`: `#202824`
- `Raised Surface`: `#2A342F`
- `Sage`: `#B8CEA9`
- `Mist`: `#AFC9D6`
- `Blush`: `#D9B3BA`
- `Butter`: `#D7D99F`
- `Ink`: `#F1F3EC`
- `Line`: `#3A4840`
- `Muted Text`: `#AAB4AC`

Usage rules:

- app background: `Canvas`
- main cards and panels: `Surface`
- hover and active surfaces: `Raised Surface`
- primary text: `Ink`
- icon/logo signals: `Sage`, `Mist`, and `Blush`
- borders: `Line`, never high contrast

## Typography

Recommended families:

- headline/UI: `Geist`, `Satoshi`, or `General Sans`
- clip/code content: `JetBrains Mono` or `IBM Plex Mono`

Usage:

- page title: large, bold, tight line-height
- section titles: medium to bold
- body text: regular, high readability
- metadata: smaller, muted
- code clips: monospace with generous line-height

Avoid:

- playful rounded fonts
- serif-led branding
- overly technical sci-fi faces

## Shape Language

- corner radii: `12`, `16`, `24`
- icon container: rounded square
- cards: soft corners with thin borders
- controls: compact but touch-friendly
- graph nodes: circles or softened capsules

The entire app should feel softened, but not toy-like.

## Layout System

### App Shell

- quiet left sidebar
- generous outer padding
- strong page heading zone
- minimal chrome around main content

### Cards

- clip cards should feel like saved artifacts
- session cards should feel grouped and structured
- context cards should read as explanatory, not chat-like

### Popup

- Spotlight-inspired
- compact, keyboard-first
- centered interaction
- minimal persistent chrome

### Graph

- simple background
- selective use of topic color
- no neon or dense canvas clutter

## Motion

- hover: gentle lift or tint
- transitions: `140ms` to `220ms`
- no springy or celebratory effects
- intelligence-upgrade notifications should be informative, not triumphant

## Components

### Buttons

- primary: butter fill, mineral-dark text
- secondary: surface with mineral border
- destructive: muted blush, used sparingly

### Inputs

- rounded
- soft border
- visible focus ring
- no strong inset styling

### Badges

- small and text-led
- memory badges should stay informational, not gamified

### Notifications

- compact
- native-feeling
- aligned with desktop utility tone

## Screen-Level Notes

### Timeline

Editorial and calm. The first-run empty state belongs here and should introduce the product without looking like a tutorial wizard.

### Search

The search field is the hero. Results stay quiet and legible.

### Session Reconstruction

This is the most story-driven screen. Use rhythm, spacing, and strong section hierarchy.

### Memory Graph

Analytical but restrained. It should look understandable, not dramatic.

### Clip Detail

Feels like examining a saved reference with context layered beside it.

### Settings

System-like, grouped, and highly legible.

## Empty-State Tone

Empty states should:

- reassure the user that data stays local
- explain what happens next
- provide one concrete action
- use an oversized low-opacity brand symbol as a background motif

## Do / Don’t

Do:

- keep interfaces warm and sparse
- use amber sparingly for emphasis
- let typography and spacing carry polish
- keep graph visuals secondary to content

Don’t:

- use purple startup gradients
- turn intelligence stages into a game
- over-animate the mascot indicator
- make every surface equally loud
