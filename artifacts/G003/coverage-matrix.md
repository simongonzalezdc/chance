# Chance — Public Surface Coverage Matrix

Generated: 2026-06-15
Plan: G003 (ultragoal)
Evidence root: artifacts/G003/, artifacts/G001/, artifacts/G002/

## Summary

| Surface | Total | Covered | Uncovered |
|---------|-------|---------|-----------|
| CLI     | 23    | 23      | 0         |
| API     | 21    | 21      | 0         |
| MCP     | 21    | 21      | 0         |
| TUI     | 9     | 9       | 0         |
| **Total** | **74** | **74** | **0** |

## CLI Subcommands (23)

| # | Command | Covered | Evidence | Gap |
|---|---------|---------|----------|-----|
| 1 | roll | yes | validate.sh, tests/cli_smoke.rs (parser tests), tests/api_smoke.rs | none |
| 2 | flip | yes | validate.sh, tests/api_smoke.rs | none |
| 3 | draw | yes | validate.sh, tests/api_smoke.rs | none |
| 4 | pick | yes | validate.sh, tests/cli_smoke.rs (empty list, count>items), tests/api_smoke.rs | none |
| 5 | shuffle | yes | validate.sh, tests/api_smoke.rs | none |
| 6 | int | yes | validate.sh, tests/cli_smoke.rs (zero/inverted range), tests/api_smoke.rs | none |
| 7 | bytes | yes | validate.sh, tests/api_smoke.rs | none |
| 8 | uuid | yes | validate.sh, tests/api_smoke.rs | none |
| 9 | password | yes | validate.sh, tests/api_smoke.rs | none |
| 10 | runes | yes | validate.sh | none |
| 11 | iching | yes | validate.sh | none |
| 12 | tarot | yes | validate.sh | none |
| 13 | dominoes | yes | validate.sh | none |
| 14 | roulette | yes | validate.sh | none |
| 15 | lottery | yes | validate.sh | none |
| 16 | knucklebones | yes | validate.sh | none |
| 17 | teetotum | yes | validate.sh | none |
| 18 | cowrie | yes | validate.sh | none |
| 19 | lots | yes | validate.sh | none |
| 20 | sources | yes | validate.sh | none |
| 21 | serve | yes | validate.sh (API server launched via serve --port) | none |
| 22 | mcp | yes | validate.sh (MCP JSON-RPC via stdin/stdout) | none |
| 23 | tui | yes | validate.sh (TUI smoke + deterministic snapshot via tmux) | none |

## API Routes (21)

| # | Route | Method | Covered | Evidence | Gap |
|---|-------|--------|---------|----------|-----|
| 1 | /v1/roll | POST | yes | validate.sh, tests/api_smoke.rs | none |
| 2 | /v1/flip | POST | yes | validate.sh, tests/api_smoke.rs | none |
| 3 | /v1/draw | POST | yes | validate.sh, tests/api_smoke.rs | none |
| 4 | /v1/pick | POST | yes | validate.sh, tests/api_smoke.rs (empty, oversized) | none |
| 5 | /v1/shuffle | POST | yes | validate.sh, tests/api_smoke.rs | none |
| 6 | /v1/int | POST | yes | validate.sh, tests/api_smoke.rs (inverted range) | none |
| 7 | /v1/bytes | POST | yes | validate.sh, tests/api_smoke.rs | none |
| 8 | /v1/uuid | POST | yes | validate.sh | none |
| 9 | /v1/password | POST | yes | validate.sh | none |
| 10 | /v1/runes | POST | yes | validate.sh | none |
| 11 | /v1/iching | POST | yes | validate.sh | none |
| 12 | /v1/tarot | POST | yes | validate.sh | none |
| 13 | /v1/dominoes | POST | yes | validate.sh | none |
| 14 | /v1/roulette | POST | yes | validate.sh | none |
| 15 | /v1/lottery | POST | yes | validate.sh | none |
| 16 | /v1/knucklebones | POST | yes | validate.sh | none |
| 17 | /v1/teetotum | POST | yes | validate.sh | none |
| 18 | /v1/cowrie | POST | yes | validate.sh | none |
| 19 | /v1/lots | POST | yes | validate.sh | none |
| 20 | /v1/sources | GET | yes | validate.sh | none |
| 21 | /v1/health | GET | yes | validate.sh | none |

## MCP Tools (21)

| # | Tool | Covered | Evidence | Gap |
|---|------|---------|----------|-----|
| 1 | chance_roll | yes | validate.sh, tests/mcp_smoke.rs | none |
| 2 | chance_flip | yes | validate.sh, tests/mcp_smoke.rs | none |
| 3 | chance_draw | yes | validate.sh | none |
| 4 | chance_pick | yes | validate.sh, tests/mcp_smoke.rs (empty, oversized) | none |
| 5 | chance_shuffle | yes | validate.sh | none |
| 6 | chance_integer | yes | validate.sh, tests/mcp_smoke.rs (inverted range) | none |
| 7 | chance_bytes | yes | validate.sh | none |
| 8 | chance_uuid | yes | validate.sh | none |
| 9 | chance_password | yes | validate.sh | none |
| 10 | chance_runes | yes | validate.sh | none |
| 11 | chance_iching | yes | validate.sh | none |
| 12 | chance_tarot | yes | validate.sh | none |
| 13 | chance_dominoes | yes | validate.sh | none |
| 14 | chance_roulette | yes | validate.sh | none |
| 15 | chance_lottery | yes | validate.sh | none |
| 16 | chance_knucklebones | yes | validate.sh | none |
| 17 | chance_teetotum | yes | validate.sh | none |
| 18 | chance_cowrie | yes | validate.sh | none |
| 19 | chance_lots | yes | validate.sh | none |
| 20 | chance_sources | yes | validate.sh, tests/mcp_smoke.rs | none |
| 21 | chance_health | yes | validate.sh, tests/mcp_smoke.rs | none |

## TUI Actions (9)

| # | Action | Key | Covered | Evidence | Gap |
|---|--------|-----|---------|----------|-----|
| 1 | Quit | q/Q | yes | validate.sh TUI smoke | none |
| 2 | Navigate up | Up/k | yes | src/tui/mod.rs unit tests (up_navigates_methods_upward, up_clamps_at_top_boundary, up_navigates_source_popup_selection) | none |
| 3 | Navigate down | Down/j | yes | validate.sh TUI smoke + deterministic snapshot | none |
| 4 | Run selected method | Enter | yes | validate.sh TUI smoke + deterministic snapshot | none |
| 5 | Open source popup | s | yes | validate.sh deterministic snapshot | none |
| 6 | Open seed popup | S | yes | validate.sh deterministic snapshot | none |
| 7 | Close popup | Esc | yes | src/tui/mod.rs unit tests (esc_closes_source_popup, esc_closes_seed_popup) | none |
| 8 | Seed entry (type/backspace) | chars/Backspace | yes | src/tui/mod.rs unit tests (char_appends_to_seed, backspace_pops_seed_character) | none |
| 9 | Source popup select | Enter | yes | validate.sh deterministic snapshot | none |

## Gaps and Follow-up Items

1. **TUI Esc key** — RESOLVED. Unit tests `esc_closes_source_popup` and `esc_closes_seed_popup` in `src/tui/mod.rs` drive the Escape key through `handle_key` and assert the popup closes.
2. **TUI Up navigation** — RESOLVED. Unit tests `up_navigates_methods_upward`, `up_clamps_at_top_boundary`, and `up_navigates_source_popup_selection` in `src/tui/mod.rs` cover upward movement and the top-boundary clamp.
3. **Statistical distribution** — RESOLVED. `tests/adversarial_stats.rs` runs std-only Pearson chi-square goodness-of-fit tests (d6, d20, coin, byte-nibbles) over 40k–100k samples from a seeded ChaCha20 source, asserting the statistic stays well below the p=0.001 critical value.
4. **Concurrent API requests** — RESOLVED. `tests/concurrent_api.rs` spawns 32–64 threads issuing mixed service-dispatch calls (roll, flip, bytes, shuffle) concurrently against the OS CSPRNG, plus a determinism check proving seeded concurrent rolls stay identical — no data races or panics.
5. **CLI argument edge cases** — RESOLVED. `tests/cli_edge_cases.rs` drives the compiled binary with unicode item lists, 500-item lists, large valid dice counts (1000d6), i64::MAX boundary integers, unicode seeds, and clean d0 rejection.
6. **MCP protocol edge cases** — RESOLVED. `tests/mcp_protocol.rs` exercises the live `chance mcp` subprocess over stdio with out-of-order initialization, wrong JSON-RPC version, malformed JSON, unknown methods, and a 100 KB oversized payload — all yield correct JSON-RPC error codes (-32700, -32601) with no crash.
7. **Error message quality** — RESOLVED. `tests/error_messages.rs` asserts descriptive content in service-layer errors (d0 parse, bogus source name, empty pick, inverted range) and MCP tool errors (unknown tool name, bad notation, oversized pick).
