# Chance End-to-End Journey Report

**78 passed / 0 failed** (CLI 30 · API 24 · MCP 14 · TUI 6 · Live LLM 4)

LLM provider: Ollama `kimi-k2.5:cloud` (fallback `qwen3.5:cloud`). Anthropic/Claude
was at its 7-day usage ceiling, so the live orchestration was driven via Ollama
instead — still a real model deciding which tools to call and summarizing results.



## CLI

- [x] roll d20
- [x] roll 4d6kh3
- [x] roll json
- [x] flip
- [x] flip x3
- [x] draw 5
- [x] pick
- [x] shuffle
- [x] int
- [x] int range
- [x] bytes hex
- [x] bytes base64
- [x] uuid
- [x] password
- [x] password no-symbols
- [x] runes
- [x] iching
- [x] tarot
- [x] dominoes
- [x] roulette
- [x] lottery
- [x] knucklebones
- [x] teetotum
- [x] teetotum dreidel
- [x] cowrie
- [x] lots
- [x] sources
- [x] seeded determinism
- [x] seeded reproducibility
- [x] d0 rejected

## API

- [x] server startup
- [x] /v1/health
- [x] /v1/sources
- [x] /v1/roll
- [x] /v1/roll
- [x] /v1/flip
- [x] /v1/draw
- [x] /v1/pick
- [x] /v1/shuffle
- [x] /v1/int
- [x] /v1/bytes
- [x] /v1/uuid
- [x] /v1/password
- [x] /v1/runes
- [x] /v1/iching
- [x] /v1/tarot
- [x] /v1/dominoes
- [x] /v1/roulette
- [x] /v1/lottery
- [x] /v1/knucklebones
- [x] /v1/teetotum
- [x] /v1/cowrie
- [x] /v1/lots
- [x] /v1/roll d0 rejected

## TUI

- [x] renders method list
- [x] run method shows result
- [x] source popup opens
- [x] esc closes popup
- [x] seed entry accepted
- [x] quit exits session
## MCP (execution-path, stdio JSON-RPC)

*Source: `mcp_agent_loop.py` → `mcp_agent_loop_results.json`. Every request is the
verbatim JSON-RPC a real LLM agent emits after deciding to call a tool.*

- [x] initialize: protocol + capabilities (pv=2024-11-05, caps=['tools'])
- [x] tools/list: 21 tools with valid JSON schemas
- [x] A1 chance_roll 2d6
- [x] A2 chance_flip
- [x] A3 chance_draw 5
- [x] A journey multi-tool chain
- [x] B chance_pick decision
- [x] C1 chance_password 20 no-symbols
- [x] C2 chance_uuid
- [x] C3 chance_bytes 16 hex
- [x] D chance_iching coin
- [x] E1 chance_roll d0 → graceful error (is_error)
- [x] E2 chance_pick empty → graceful error (is_error)
- [x] E3 unknown tool → graceful error (is_error)

## Live LLM → MCP orchestration (real model tool-use)

*Source: `llm_ollama.py` → `llm_results.json`, `llm_ollama.log`. A real LLM (Ollama
OpenAI-compatible endpoint) runs a multi-turn agent loop against the live `chance mcp`
stdio server: it decides which tool to call, executes it, then writes a final summary.*

- [x] board-game-orchestration (kimi-k2.5:cloud) — chance_roll, chance_flip, chance_draw
- [x] dinner-decision (kimi-k2.5:cloud) — chance_pick → "tacos"
- [x] project-provisioning (qwen3.5:cloud) — chance_password(no symbols), chance_uuid, chance_bytes
- [x] iching-divination (kimi-k2.5:cloud) — chance_iching → Hexagram #60 Chieh/Limitation

All 8 tool calls returned valid results with `os-csprng` provenance, zero `is_error`.
`project-provisioning` retried once on the `qwen3.5:cloud` fallback only because
`kimi-k2.5:cloud` emitted an empty stop summary *after* calling all three tools correctly
— a model verbosity quirk, not an MCP/server defect.
