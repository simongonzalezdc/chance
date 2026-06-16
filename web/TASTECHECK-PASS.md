# TASTECHECK PASS — chance web UI — 2026-06-16

Spec: `web/DESIGN-SYSTEM.md` — *"Phosphor-violet instrument console; Hasselblad
catalog × brutalist-terminal; hue 287 / accent 303 riso two-color on dark;
Unbounded / Spline Sans / DM Mono; Perfect-Fifth 1.500; static type-cascade
signature; motion=none. Every value rolled by the `chance` binary."*

The browser mechanical gate was driven through a real headless Chromium
(puppeteer/CDP) against `http://localhost:8484` on a fresh, untouched load.
Auditor injected from
`tastecheck/skills/tastecheck-pass/assets/gate-audit.js`; structured result read
from `window.__gateAudit`. Measured numbers are in the Notes column.

| Skill | Self-check | Notes |
|---|---|---|
| design-system-interview | ✓ | spec present, re-rolled by `chance` (os-csprng), built-to |
| color-system | ✓ | OKLCH ramp H287; worst text pair 5.0:1 (brand-500/bg); CTA bright-violet/dark-ink 7.0:1 |
| web-typography | ✓ | Unbounded/Spline Sans/DM Mono (Google Fonts); Perfect Fifth 1.500; measure 64ch |
| spacing-system | ✓ | spacious 8-step `--s1..--s8` + fluid `--section` |
| theming | ✓ | dark-only (rolled); primitive→semantic tokens |
| responsive-layout | ✓ | 320px reflow: scrollW=clientW=320, no h-scroll; all grids → 1 col |
| component-states | ✓ | hover/focus/disabled/loading/empty/error all present |
| form-ux | ✓ | Enter-to-generate; switches are `role=switch`+`aria-checked`+keyboard |
| empty-states | ✓ | `∅` empty, `·· ··` loading, `⚠` error |
| micro-motion | ✓ | rolled `motion=none`; `prefers-reduced-motion` zeroes transitions (verified) |
| data-viz | ✓ | **NEW** — SVG dice pips / coins / card faces; session distribution bars; entropy+latency sparklines; bit-grid entropy field (the "pretty fonts" fix) |
| art-direction | ✓ | riso two-color flat instrument shapes + grain; deviation from rolled duotone (no photo source) documented |
| a11y-pass | ✓ | keyboard tab-through: 23 els, logical order, no trap; focus-visible 2px violet; contrast AA |
| deslop-ui | ✓ | 0 tells (Inter/Roboto 0, indigo 0, gradient-text 0, glass 0, pill 0); 1 auditor WARN = method nav rail — accepted (nav list, not the marketing-card tell) |
| humanize-copy | ✓ | plain-voice labels; zero emoji |
| **tastecheck-pass (gate)** | **✓** | auditor `REVIEW WARNS` **0 fails**; cold-load clean; 320px PASS; 400%-reflow PASS; keyboard PASS; reduced-motion PASS |

## Mechanical evidence (from the real browser run)

**gate-audit.js verdict: `REVIEW WARNS` — 0 fail / 1 warn / 1 note**
- `— note: display face resolves to "Unbounded"` (computed, after load — not the safe-font tell)
- `⚠ warn: uniform card grid: 18× button.mbtn in nav#mlist` — **accepted against
  spec**: this is a vertical method-selection `<nav>`, the correct semantic for
  picking a method. Uniformity in a nav rail is correct UX, not the "three
  identical marketing cards" slop tell the heuristic targets. (Warns are
  "evidence for judgment against the committed spec, not verdicts" — auditor's
  own commentary.)
- **Fixed during gate (1):** `[hidden]` on the `#copy` button was defeated by
  `.copy{display:inline-flex}`, so "copy" rendered on the fresh load with nothing
  to copy. Fixed with `[hidden]{display:none!important}` (attribute beats author
  display). Re-ran → 0 fails, `#copy` confirmed `display:none`.

**Cold load (fresh page, no interaction):** `scrollW=clientW=1280` (no
horizontal overflow); 0 visible error/alert text; 0 stuck `aria-busy`; title +
brand present; Unbounded loaded (no FOUT fallback). Body 725 chars.

**320px reflow (WCAG 1.4.10):** viewport 320×640 → `scrollW=320, clientW=320`,
`hOver=false`; `.bento-head`/`.bento-main`/`.ctrl-row` all collapse to a single
column track; smallest computed text 12px (0.75rem, readable). **PASS.**

**400% zoom:** WCAG 1.4.10 is satisfied at 320 CSS px (browser zoom reflows the
layout viewport to 320px, which passes above). A CSS `zoom:4` probe overflowed —
that is the `zoom` property scaling without reflowing the viewport (a
test-method artifact), not a real defect; it is not how browsers implement zoom.

**Keyboard tab-through (1280px):** 23 interactive elements all reachable in
logical DOM/visual order — `select#source → input#seed → 19 method buttons →
notation input → button#go`. No trap (cycles back to source). Focus-visible
outline: `2px solid oklch(0.62 0.15 287)` (visible violet). **PASS.**

**prefers-reduced-motion (CDP `Emulation.setEmulatedMedia`):**
`matchMedia` matches; computed `transition:none, animation:none` on `.go` and
`.mbtn`. **PASS** (and consistent with the rolled `motion=none`).

**Contrast (analytic, OKLCH→linear sRGB→ratio):** text/bg 10.7, muted/bg 7.2,
brand-300/bg 7.3, accent(303)/bg 7.3, brand-400/bg 6.0, brand-500/bg 5.0,
primary-ink/primary-fill 7.0, accent/surface-1 4.6 — all AA.

## Verdict

**Gate: PASS** — 16 skill checks, 1 defect fixed during gate (the `[hidden]`
copy-button), 1 warn accepted against spec with rationale. The browser-only
mechanical gap that was partial in the prior session is now closed with
injected-auditor + zoom + keyboard + cold-load + reduced-motion evidence.
