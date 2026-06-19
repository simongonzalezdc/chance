# DESIGN-SYSTEM.md — chance web UI (`web/index.html`)

> The committed direction in one line: **"MERIDIAN — an antiquarian observatory of
> chance; a 1780 copperplate treatise on probability rendered as a warm, flat-color
> ritual studio. Engraved armillary-sphere backdrop, reading-plate result stage,
> treatise-index method rail. Offline-first single HTML; GSAP as progressive
> enhancement only."**

This supersedes the earlier phosphor-violet / dark-mode direction. The shipped
UI is `web/index.html` — a single self-contained file baked into the binary via
`include_str!`. Core cast / fetch / render works fully offline; GSAP motion is
feature-detected and no-ops when the CDN is unreachable.

## Canon (owner taste — hard constraints)

- **NO gradients.** Fills are flat. Period.
- **NO purple, NO reef-teal / SaaS-blue, NO green.** Warm signal colors only.
- **`prefers-reduced-motion`** disables ALL motion (armilla rotation, reveal,
  dice tumble, roulette spin, tarot flip). Verified.
- Offline-first: no required CDN. GSAP is progressive enhancement.

## Identity & signature

- **Wordmark:** "MERIDIAN" set in **Fraunces** opsz-144 / weight 900 (copperplate).
- **Signature move:** an **engraved armillary sphere** backdrop — pure inline SVG
  + CSS (meridian ring, dashed degree scale, equator / ecliptic / tropics /
  horizon, polar gnomon + pole nodes, constellation hairlines), rotating 180s.
  Replaces the earlier Three.js particle cosmos; the ~600KB Three.js dependency
  was dropped for stricter offline + crisper, on-canon rendering.
- **Result stage:** a **reading plate** with a double-keyline engraved frame and
  corner registration ticks (SVG-data-URI crosses) wrapping `#result`.
- **Method rail:** a numbered **treatise index** of all 19 methods.

## Token block (warm flat signal palette)

```css
:root{
  /* ink + paper ground */
  --ink:#17130d; --paper:#f4ecdb; --bone:#e8dcc0;
  /* warm signal fills — flat, never gradient */
  --oxblood:#6e1f14; --vermilion:#bf3417;
  --gold:#9c6f0c; --gold-2:#a87810; --amber:#7d5a00;
  /* muted text darkened for AA */
  --muted:#635740;
  /* CTA */
  --primary-fill:var(--oxblood);
  /* type */
  --font-display:"Fraunces",Georgia,serif;
  --font-body:system-ui,sans-serif;
  --font-mono:ui-monospace,monospace;
}
```

JS object palettes (`K` token map) are tuned to match the chrome: dice / cards /
roulette pockets / tarot faces render in ink / oxblood / vermilion / gold / amber
on bone. Notable: roulette `0` pocket and tarot "upright" label are **gold**
(was green — the "mold" fix), so the page carries **zero green**.

## Contrast (analytically verified)

| Pair | Ratio | AA |
|---|---|---|
| ink / paper | 12.12 | ✓ |
| muted / paper | 5.61 | ✓ |
| gold / paper | ≥ 4.5 | ✓ (large) |

## Motion (GSAP, progressive enhancement)

Stamped-impression reveal, count-up, dice tumble, roulette spin (settles to
identity matrix), tarot flip, rail cursor, sparkline draw. All gated behind a
GSAP feature-detect; with GSAP absent the page sets
`__meridian="motion:no-gsap"` and remains fully functional. `prefers-reduced-
motion` zeroes every animation.

## Surfaces preserved verbatim

All 19 methods, the JS generation / rendering logic, the `/v1` API, and the
provenance block are unchanged from the backend contract.
