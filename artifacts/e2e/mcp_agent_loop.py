#!/usr/bin/env python3
"""MCP agent-loop execution validation.

Validates the complete MCP integration path that `chance` owns:
  - protocol handshake (initialize, capabilities, protocol version)
  - tool advertisement (21 tools with valid JSON schemas)
  - tool execution (tools/call over the real stdio JSON-RPC transport)

The tool-CALL requests mirror exactly what a real LLM tool-use client emits.
The only layer NOT exercised is the LLM's inference of *which* tool to call --
that is Anthropic's infrastructure (unreachable this session: the account's
7-day usage limit is at 100%, retry-after ~tomorrow). Every request below is
the verbatim JSON-RPC a real LLM agent would send after deciding to call a tool.
"""
import os, json, subprocess, re

CHANCE = os.path.abspath("target/release/chance")
EV = "artifacts/e2e"
results = []


def record(j, ok, detail=""):
    ok = bool(ok)
    results.append((j, ok, detail))
    print(f"  [{'PASS' if ok else 'FAIL'}] {j}" + (f" -- {detail}" if not ok else ""))


class Mcp:
    def __init__(self):
        self.p = subprocess.Popen([CHANCE, "mcp"], stdin=subprocess.PIPE,
                                  stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                                  text=True, bufsize=1)
        self.n = 0

    def call(self, method, params=None, want_resp=True):
        self.n += 1
        r = {"jsonrpc": "2.0", "id": self.n, "method": method}
        if params is not None:
            r["params"] = params
        self.p.stdin.write(json.dumps(r) + "\n")
        self.p.stdin.flush()
        if not want_resp:
            return None
        line = self.p.stdout.readline()
        return json.loads(line) if line.strip() else None

    def tool(self, name, args):
        r = self.call("tools/call", {"name": name, "arguments": args})
        res = r.get("result", {})
        return res.get("is_error", True), res.get("content", [{}])[0].get("text", "")

    def close(self):
        try:
            self.p.stdin.close()
            self.p.wait(timeout=5)
        except Exception:
            self.p.kill()


def val_text(text):
    return text.strip()


print("══ PHASE 3 (MCP execution path): real tools/call over stdio JSON-RPC ══")
mc = Mcp()

# 3a. Handshake
init = mc.call("initialize")
pv = init.get("result", {}).get("protocolVersion")
caps = init.get("result", {}).get("capabilities", {})
record("initialize: protocol + capabilities",
       pv == "2024-11-05" and "tools" in caps,
       f"pv={pv} caps={list(caps)}")

# 3b. Tool advertisement
tools = mc.call("tools/list")["result"]["tools"]
schemas_ok = all(t.get("input_schema", {}).get("type") == "object" for t in tools)
record("tools/list: 21 tools with valid schemas",
       len(tools) == 21 and schemas_ok,
       f"count={len(tools)} schemas_ok={schemas_ok}")

# ── Journey A: board-game orchestration (multi-tool) ─────────────────────
print("\n  -- journey: board-game-orchestration (roll + flip + draw) --")
errs = []
# roll 2d6
e, t = mc.tool("chance_roll", {"notation": "2d6"})
roll = json.loads(t) if not e else None
if e or not (2 <= roll["result"]["total"] <= 12):
    errs.append(f"roll: e={e} t={t[:60]}")
record("A1 chance_roll 2d6", not e and 2 <= roll["result"]["total"] <= 12,
       f"total={roll['result']['total'] if roll else '?'}")
# flip
e, t = mc.tool("chance_flip", {"times": 1})
flips = json.loads(t)["result"] if not e else None
record("A2 chance_flip", not e and flips and flips[0].lower() in ("heads", "tails"),
       f"flip={flips}")
# draw 5
e, t = mc.tool("chance_draw", {"count": 5})
draw = json.loads(t)["result"] if not e else None
record("A3 chance_draw 5", not e and len(draw) == 5,
       f"cards={len(draw) if draw else '?'}")
record("A journey multi-tool chain", not errs, "; ".join(errs))

# ── Journey B: decision-making (pick) ────────────────────────────────────
print("\n  -- journey: dinner-decision (pick among 3) --")
e, t = mc.tool("chance_pick", {"items": ["pizza", "sushi", "tacos"], "count": 1})
pick = json.loads(t)["result"] if not e else None
record("B chance_pick decision", not e and pick and pick[0] in ("pizza", "sushi", "tacos"),
       f"chose={pick}")

# ── Journey C: data generation (password + uuid + bytes) ─────────────────
print("\n  -- journey: project-provisioning (password + uuid + bytes) --")
e, t = mc.tool("chance_password", {"length": 20, "symbols": False})
pw = json.loads(t)["result"] if not e else None
record("C1 chance_password 20 no-symbols",
       not e and len(pw) == 20 and pw.isalnum(), f"pw_len={len(pw) if pw else '?'}")
e, t = mc.tool("chance_uuid", {})
uu = json.loads(t)["result"] if not e else None
record("C2 chance_uuid",
       not e and bool(re.fullmatch(r"[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}", str(uu))),
       f"uuid={uu}")
e, t = mc.tool("chance_bytes", {"count": 16, "encoding": "hex"})
by = json.loads(t)["result"] if not e else None
record("C3 chance_bytes 16 hex",
       not e and len(by) == 32 and re.fullmatch(r"[0-9a-f]+", str(by)),
       f"hex_len={len(by) if by else '?'}")

# ── Journey D: I Ching divination ────────────────────────────────────────
print("\n  -- journey: iching-divination --")
e, t = mc.tool("chance_iching", {"method": "coin"})
ic = json.loads(t)["result"] if not e else None
record("D chance_iching coin",
       not e and "primary_name" in ic and 1 <= ic["primary"] <= 64 and len(ic["lines"]) == 6,
       f"hex={ic['primary'] if ic else '?'} {ic.get('primary_name') if ic else '?'}")

# ── Adversarial: LLM-style malformed tool calls ──────────────────────────
print("\n  -- adversarial: malformed tool invocations --")
e, t = mc.tool("chance_roll", {"notation": "d0"})
record("E1 chance_roll d0 → graceful error", e and "parse" in t.lower() or "dice" in t.lower() or e,
       f"is_error={e}")
e, t = mc.tool("chance_pick", {"items": [], "count": 1})
record("E2 chance_pick empty → graceful error", e, f"is_error={e}")
e, t = mc.tool("chance_nonexistent", {})
record("E3 unknown tool → graceful error", e, f"is_error={e}")

mc.close()

passes = sum(1 for _, ok, _ in results if ok)
fails = sum(1 for _, ok, _ in results if not ok)
print(f"\n  MCP execution path: {passes} passed, {fails} failed")

json.dump({"passed": passes, "failed": fails,
           "journeys": [{"name": j, "pass": ok, "detail": d} for j, ok, d in results]},
          open(f"{EV}/mcp_agent_loop_results.json", "w"), indent=2)
