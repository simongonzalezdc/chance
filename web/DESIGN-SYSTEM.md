# DESIGN-SYSTEM.md — chance web UI

> **Committed direction (one line):**
> *Memphis-postmodern Apollo cockpit — warm, playful, maximal, classic, refined, dense —
> oxblood-on-dark anchor with a coral secondary, Instrument Serif numerals as the signature,
> Hanken Grotesk body, hard-offset poster shadows, soft 11px corners, bento 3-col mosaic.*

Every concrete decision below was rolled by **`chance`** itself (OS CSPRNG, no seed) — the
`design-system-interview` forcing questions plus the finishing values the interview left as
ranges. This file is the single source of truth the rest of the tastecheck pack builds to and
`deslop-ui` audits against.

## Interview answers (rolled by chance)

| # | Question | Rolled |
|---|----------|--------|
| 1 | Reference | NASA Apollo flight-deck console |
| 2 | Personality poles | warm · playful · maximal · classic · refined · dense |
| 3 | Aesthetic direction | memphis postmodern |
| 4 | Type stance | Instrument Serif (display) + Neue Haas Grotesk (body) |
| 5 | Color + mode | oxblood anchor · dark-only |
| 6 | Density & shape | dense · soft 8–12px corners · layered/shadowed |
| 7 | Structure & rhythm | symmetric centered · bento/tessellation · metronomic |
| 8 | Signature move | oversized display numerals as section markers |
| 9 | Imagery + icons | duotone photography · Radix @ 1.5 |
| 10 | Motion | restrained |

## Finishing values (rolled by chance)

| Decision | Rolled |
|----------|--------|
| Corner radius (8–12) | **11px** (cards) · 8px (controls) |
| Memphis secondary accent | **coral** (`oklch(0.72 0.16 40)`) |
| Oxblood hue | **H 26** |
| Type scale ratio | **Minor Third 1.200** (dense cockpit) |
| Elevation | **hard-offset poster shadow** (memphis) |
| Bento shape | **3-col mosaic** |

## Refusals (the system is a set of refusals)

- No indigo→violet gradient, ever. Oxblood is the only brand hue; coral is the single accent.
- No Inter / Roboto / system-default headlines. Instrument Serif or Hanken Grotesk only.
- No pure gray neutrals — every neutral is tinted toward oxblood (H 26, tiny chroma).
- No emoji as icons. Signature numerals mark methods; stroke-1.5 SVG marks affordances.
- No pill text CTAs (8px), no uniform equal-card grid (mosaic varies tile sizes), no animated
  stat-counter band, no gradient blobs. Memphis decoration = flat solid geometric shapes only.
- No perpetual/looping motion. Restrained: focus, hover, a single generate confirmation.

## Imagery deviation (explicit, accepted)

Committed imagery was *duotone photography*. This UI has no photograph source, so the imagery
plane is committed as **pure type & texture**: flat solid memphis geometric shapes (coral disc,
dot-grid, zigzag) as committed decoration — never gradient blobs. Stated, not silently swapped.

## Tokens

### Color (OKLCH — constant hue, stepped lightness, neutrals tinted to H 26)

```css
/* Brand oxblood ramp, H 26 */
--brand-300: oklch(0.72 0.13 26);  /* small accent text — ≥4.5:1 on bg */
--brand-400: oklch(0.62 0.15 26);
--brand-500: oklch(0.54 0.16 26);  /* base oxblood */
--brand-600: oklch(0.46 0.15 26);  /* primary fill */
--brand-700: oklch(0.38 0.12 26);

/* Coral — the one secondary accent */
--accent: oklch(0.72 0.16 40);

/* Neutrals tinted toward H 26 (not dead gray) */
--n-950: oklch(0.15 0.012 26); --n-900: oklch(0.18 0.012 26);
--n-850: oklch(0.21 0.013 26); --n-800: oklch(0.24 0.014 26);
--n-700: oklch(0.30 0.014 26); --n-500: oklch(0.58 0.014 26);

/* Semantic aliases (dark mode) */
--color-bg:        oklch(0.15 0.014 26);
--color-surface-1: var(--n-850);
--color-surface-2: var(--n-800);
--color-border:    var(--n-700);
--color-text:      oklch(0.96 0.005 26);
--color-text-muted:oklch(0.70 0.014 26);
--color-primary:   var(--brand-400);
--color-primary-fill: var(--brand-600);
--color-primary-ink: oklch(0.97 0.01 26);
--color-success: oklch(0.72 0.15 150);
--color-error:   oklch(0.66 0.20 14);
```

Contrast (measured, WCAG AA): text L0.96 on bg L0.15 ≈ 17:1; muted L0.70 on bg ≈ 6.3:1;
brand-300 L0.72 small-accent on bg ≈ 6.9:1; primary-fill L0.46 with ink L0.97 ≈ 7.1:1.

### Type

```css
--font-display: "Instrument Serif", Georgia, serif;          /* signature numerals + headlines */
--font-body:    "Hanken Grotesk", system-ui, sans-serif;     /* Neue Haas Grotesk substitute */
--font-mono:    "JetBrains Mono", ui-monospace, monospace;   /* bytes / uuid / data */
/* Minor Third 1.200, dense; body 1rem, measure 66ch, unitless line-height 1.5 */
```

### Spacing (4px dense base) · Shape · Elevation

```css
--space-1..8: 0.25 / 0.5 / 0.75 / 1 / 1.5 / 2 / 3 / 4 rem;
--space-section: clamp(2rem, 1.5rem + 3vw, 4rem);
--radius-control: 8px; --radius-card: 11px;   /* pills reserved for tags/chips only */
--shadow-hard:  4px 4px 0 oklch(0.10 0.02 26);          /* memphis poster offset */
--shadow-coral: 5px 5px 0 var(--accent);
```

## Pipeline handoff

Built to this spec by `color-system` / `web-typography` / `spacing-system` / `responsive-layout` /
`component-states` / `form-ux` / `empty-states` / `micro-motion`; audited by `deslop-ui` and
gated by `tastecheck-pass` (report ships with the work).
