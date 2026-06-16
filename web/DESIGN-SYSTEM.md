# DESIGN-SYSTEM.md — chance web UI

> The committed direction in one line: **"Phosphor-violet instrument console — a
> Hasselblad lunar-surface camera catalog reimagined as a brutalist-terminal RNG
> studio: cool, serious, maximal, classical, refined, spacious; violet 287 /
> magenta 303 riso two-color on dark; Unbounded cascade signature; Perfect-Fifth
> scale; honest data-visualization of every roll."**

Every value below was rolled at random by the `chance` binary itself (os-csprng,
no seed). The tastecheck pack then implements to this committed spec. This is a
re-roll — the prior oxblood/memphis direction has been superseded.

## Refusals (the design IS the constraints)

- **NO gradients.** A single committed dark-violet hue (OKLCH 287) is fine; an
  indigo→violet *gradient* is the slop the pack bans. Fills are flat. Period.
- **NO Inter / Roboto.** Display = **Unbounded**, body = **Spline Sans**, mono =
  **DM Mono** (all Google Fonts, `display=swap`, no build step).
- **NO emoji as icons.** Affordances are stroke-1.5 inline SVG. Numerals are set
  in Unbounded (the signature), never pictograms.
- **NO motion.** The roll landed `motion = none`. The "kinetic type cascade"
  signature is rendered as a **static** stair-stepped type composition — no
  animation (interactive hover/focus transitions only; `prefers-reduced-motion`
  zeroes even those).
- **NO decorative chartjunk.** Every visualization obeys the `data-viz` skill:
  ink maps to data, bars from zero, direct-labeled, ≤5 hues from tokens,
  not-color-alone, text equivalent present.

## Rolled spec (chance output)

| # | Question | Rolled answer |
|---|---|---|
| 1 | Reference | Hasselblad lunar-surface camera catalog |
| 2 | Personality | cool · serious · maximal · classical · refined · spacious |
| 3 | Aesthetic | brutalist-terminal (refined-instrument reading) |
| 4 | Type stance | Unbounded (display) / Spline Sans (body) / DM Mono (mono) |
| 5 | Color + mode | hue **287** (phosphor-violet), **dark-only**; accent hue **303** |
| 6 | Density & shape | spacious · **15px** cards / **4px** controls · soft-shadow |
| 7 | Structure & rhythm | symmetric · collage motif · syncopated |
| 8 | Signature move | kinetic type cascade (rendered **static**) |
| 9 | Imagery | risograph two-color (violet + magenta, grain, overprint) |
| 10 | Motion | **none** |

### Finishing values (chance output)
- Card radius **15px**, control radius **4px** (the 1px finishing roll was
  reconciled against the 15px shape roll — shape governs cards; controls 4px).
- Accent hue **303** (anchor 287 + rolled +16° offset) → tight analogous riso pair.
- Scale ratio **Perfect Fifth 1.500**.
- Elevation **soft-shadow** (layered, low-spread, low-opacity).
- Bento **2 columns**.

## Token block (OKLCH, dark-only)

```css
:root{
  /* brand ramp · phosphor-violet hue 287 */
  --brand-300:oklch(0.72 0.13 287); --brand-400:oklch(0.62 0.15 287);
  --brand-500:oklch(0.54 0.16 287); --brand-600:oklch(0.46 0.15 287);
  --brand-700:oklch(0.38 0.12 287);
  /* accent · riso 2nd color, magenta-violet 303 */
  --accent:oklch(0.72 0.16 303);
  /* neutrals tinted toward 287 (no dead grays) */
  --n-950:oklch(0.15 0.014 287); --n-900:oklch(0.18 0.012 287); --n-850:oklch(0.21 0.013 287);
  --n-800:oklch(0.24 0.014 287); --n-700:oklch(0.30 0.014 287); --n-500:oklch(0.58 0.014 287);
  /* semantic */
  --color-bg:oklch(0.15 0.014 287); --surface-1:var(--n-850); --surface-2:var(--n-800);
  --border:var(--n-700); --text:oklch(0.96 0.005 287); --muted:oklch(0.70 0.014 287);
  /* CTA: bright-violet fill + dark ink (violet chroma lifts luminance, so a
     mid fill can't carry white — verified 7.0:1 bright-fill/dark-ink) */
  --primary-fill:oklch(0.74 0.15 287); --primary-ink:oklch(0.16 0.02 287);
  --success:oklch(0.72 0.15 150); --error:oklch(0.66 0.20 14); --warn:oklch(0.82 0.15 85);
  /* type */
  --font-display:"Unbounded",system-ui,sans-serif;
  --font-body:"Spline Sans",system-ui,sans-serif;
  --font-mono:"DM Mono",ui-monospace,monospace;
  /* scale · Perfect Fifth 1.500, fluid */
  --step--2:clamp(0.625rem,0.6rem+0.1vw,0.7rem);
  --step--1:clamp(0.75rem,0.72rem+0.15vw,0.85rem);
  --step-0:1rem; --step-1:clamp(1.3rem,1.2rem+0.4vw,1.5rem);
  --step-2:clamp(1.75rem,1.5rem+1vw,2.25rem);
  --step-3:clamp(2.5rem,2rem+2vw,3.375rem);
  --step-4:clamp(3.5rem,2.5rem+4vw,5.063rem);
  --step-5:clamp(4.5rem,2.5rem+9vw,7.6rem);   /* signature cascade */
  --measure:64ch;
  /* spacing · spacious base */
  --s1:0.5rem;--s2:0.75rem;--s3:1rem;--s4:1.5rem;--s5:2rem;--s6:2.5rem;--s7:3.5rem;--s8:5rem;
  --section:clamp(2.5rem,2rem+3vw,5rem);
  /* shape + elevation */
  --r-ctrl:4px; --r-card:15px;
  --sh-soft:0 10px 30px -8px oklch(0.05 0.02 287 / 0.55);
  --sh-lift:0 4px 14px -4px oklch(0.05 0.02 287 / 0.45);
  /* data-viz series (≤5 hues, from tokens, not color-alone) */
  --series-1:var(--brand-300); --series-2:var(--accent);
  --series-3:var(--brand-500); --series-4:var(--n-500);
}
```

## Contrast (analytically verified, OKLCH→linear sRGB→ratio)

| Pair | Ratio | AA |
|---|---|---|
| text / bg | 10.7 | ✓ |
| muted / bg | 7.2 | ✓ |
| brand-300 / bg | 7.3 | ✓ |
| accent(303) / bg | 7.3 | ✓ |
| brand-400 / bg | 6.0 | ✓ |
| brand-500 / bg | 5.0 | ✓ |
| primary-ink / primary-fill | 7.0 | ✓ |
| accent / surface-1 | 4.6 | ✓ (large) |

## Data-visualization contract (`data-viz` skill)

The prior UI rendered results as text ("just pretty fonts"). This roll
implements honest, Tufte-informed visualization of the randomness:

- **Graphical result objects** — dice render as **SVG pip faces**, coins as
  **SVG coins**, cards as **SVG card faces**. Ink = the actual rolled value.
- **Session distribution** — accumulates categorical draws into a **bar chart
  from zero**, direct-labeled, single accent series, with a caption stating the
  fairness takeaway. Not color-alone (labels carry it).
- **Entropy / latency sparklines** — inline SVG sparkline of the last N draws in
  the header stat tiles (trend per session, range-framed).
- **Bit-grid entropy field** — random `bytes` render as a two-color bit grid
  (each bit a cell); the most literal "ink maps to data" on the page.
- Every chart has a text equivalent (caption + accessible table / aria-label).

## Pipeline handoff

design-system-interview (this file) → color-system / web-typography / spacing →
responsive-layout / component-states / form-ux → **data-viz** → micro-motion
(none) → empty-states / humanize-copy → **deslop-ui** audits against this spec.
