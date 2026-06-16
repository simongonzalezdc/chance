#!/usr/bin/env python3
"""Real-LLM (Ollama) -> MCP (chance stdio) validation harness.

Drives multi-turn tool-use journeys against the live `chance mcp` JSON-RPC
server using a real LLM served by the local Ollama OpenAI-compatible endpoint.
Python 3 standard library ONLY. Does NOT call Anthropic/Claude.

Output:
  artifacts/e2e/llm_ollama.log      (progress log, appended)
  artifacts/e2e/llm_results.json    (canonical structured results)
Exit: 0 if all journeys pass, else 1.
"""
import os, sys, json, subprocess, time, urllib.request, urllib.error

CHANCE = os.path.abspath("target/release/chance")
PRIMARY = "kimi-k2.5:cloud"
FALLBACK = "qwen3.5:cloud"
URL = "http://127.0.0.1:11434/v1/chat/completions"
EV = "artifacts/e2e"
LOG = f"{EV}/llm_ollama.log"
OUT = f"{EV}/llm_results.json"
TIMEOUT = 180

_logf = open(LOG, "a", buffering=1)


def log(m):
    line = f"[{time.strftime('%H:%M:%S')}] {m}"
    print(line, flush=True)
    _logf.write(line + "\n")


class Mcp:
    """Stdio JSON-RPC client for `chance mcp`. One request -> one response line."""

    def __init__(self):
        self.p = subprocess.Popen([CHANCE, "mcp"], stdin=subprocess.PIPE,
                                  stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                                  text=True, bufsize=1)
        self.n = 0

    def call(self, method, params=None):
        self.n += 1
        r = {"jsonrpc": "2.0", "id": self.n, "method": method}
        if params is not None:
            r["params"] = params
        self.p.stdin.write(json.dumps(r) + "\n")
        self.p.stdin.flush()
        line = self.p.stdout.readline()
        return json.loads(line) if line.strip() else None

    def close(self):
        try:
            self.p.stdin.close()
            self.p.wait(timeout=5)
        except Exception:
            self.p.kill()


def ollama(messages, tools, model=PRIMARY):
    payload = json.dumps({"model": model, "stream": False, "messages": messages,
                          "tools": tools, "tool_choice": "auto"}).encode()
    req = urllib.request.Request(URL, data=payload)
    req.add_header("content-type", "application/json")
    try:
        return json.loads(urllib.request.urlopen(req, timeout=TIMEOUT).read())
    except urllib.error.HTTPError as e:
        return {"_error": f"HTTP {e.code}: {e.read().decode()[:200]}"}
    except Exception as e:
        return {"_error": f"{type(e).__name__}: {e}"}


def journey(mc, tools, label, prompt, expect, model):
    log(f"\n=== JOURNEY: {label} (model={model}) ===")
    log(f"prompt: {prompt[:100]}")
    messages = [{"role": "user", "content": prompt}]
    called, outputs = [], []
    final, err, rounds = "", None, 0
    for rnd in range(8):
        rounds = rnd + 1
        log(f"  -- round {rounds}: calling {model}")
        resp = ollama(messages, tools, model)
        if "_error" in resp:
            err = resp["_error"]
            log(f"  round {rounds}: ollama error: {err}")
            break
        choice = resp["choices"][0]
        msg = choice["message"]
        finish = choice.get("finish_reason")
        tool_calls = msg.get("tool_calls") or []
        # echo assistant turn into history
        a = {"role": "assistant", "content": msg.get("content") or ""}
        if tool_calls:
            a["tool_calls"] = tool_calls
        messages.append(a)
        if not tool_calls:
            final = msg.get("content") or ""
            log(f"  round {rounds}: finish={finish} (no tool_calls) -> final text")
            break
        for call in tool_calls:
            fname = call["function"]["name"]
            try:
                args = json.loads(call["function"]["arguments"])
            except Exception as je:
                log(f"  round {rounds}: bad arguments JSON for {fname}: {je}")
                args = {}
            mr = mc.call("tools/call", {"name": fname, "arguments": args})
            tr = mr.get("result", {})
            content = tr.get("content") or [{}]
            txt = content[0].get("text", "") if content else ""
            is_err = tr.get("is_error", False)
            called.append(fname)
            outputs.append((fname, args, is_err, txt))
            log(f"  round {rounds}: LLM -> {fname}({json.dumps(args)[:50]}) "
                f"{'ERR' if is_err else 'ok'}: {txt[:80]}")
            messages.append({"role": "tool", "tool_call_id": call["id"],
                             "content": txt})
        # loop continues; next round lets the model summarize or call more
    ok = (err is None
          and all(not e for _, _, e, _ in outputs)
          and all(any(t == x for t in called) for x in expect)
          and len(final) > 5)
    log(f"  RESULT: {'PASS' if ok else 'FAIL'} | model={model} | tools={called} | err={err}")
    log(f"  final: {final[:200]}")
    return {"label": label, "pass": ok, "model": model, "tools_called": called,
            "rounds": rounds, "error": err, "final_text": final,
            "outputs": [{"tool": n, "input": i, "error": e, "text": t}
                        for n, i, e, t in outputs]}


def run_with_fallback(mc, tools, label, prompt, expect):
    res = journey(mc, tools, label, prompt, expect, PRIMARY)
    if res["pass"]:
        return res
    log(f"  -- primary failed for {label}; retrying once with {FALLBACK}")
    res2 = journey(mc, tools, label, prompt, expect, FALLBACK)
    if res2["pass"]:
        return res2
    return res  # report primary failure; keep primary as recorded model


def main():
    log("================ OLLAMA -> MCP JOURNEY RUNNER STARTING ================")
    mc = Mcp()
    init = mc.call("initialize")
    pv = init.get("result", {}).get("protocolVersion")
    log(f"MCP initialize: protocolVersion={pv}")
    if not pv:
        log("FATAL: no protocolVersion from initialize")
        mc.close()
        sys.exit(2)
    tools_raw = mc.call("tools/list")["result"]["tools"]
    log(f"MCP tools/list: {len(tools_raw)} tools")
    tools = [{"type": "function", "function": {
        "name": t["name"], "description": t.get("description", ""),
        "parameters": t.get("input_schema", {"type": "object"})}}
        for t in tools_raw]

    journeys = [
        ("board-game-orchestration",
         "I'm hosting game night. Using the chance tools, do all of the following "
         "then summarize: (1) roll 2d6 for the first player's score, (2) flip a coin "
         "to pick red or blue team, (3) draw 5 cards. Report every result.",
         ["chance_roll", "chance_flip", "chance_draw"]),
        ("dinner-decision",
         "I can't decide between pizza, sushi, and tacos for dinner. Use the chance "
         "pick tool to choose fairly among these three, then tell me what we're eating.",
         ["chance_pick"]),
        ("project-provisioning",
         "Provision a new project using chance tools: generate a 20-character password "
         "with no symbols, a UUID, and 16 random bytes as hex. List all three values clearly.",
         ["chance_password", "chance_uuid", "chance_bytes"]),
        ("iching-divination",
         "Cast an I Ching hexagram via the coin method using the chance tools, "
         "and tell me the hexagram number and name.",
         ["chance_iching"]),
    ]
    results = []
    for label, prompt, expect in journeys:
        results.append(run_with_fallback(mc, tools, label, prompt, expect))
    mc.close()

    passed = sum(1 for r in results if r["pass"])
    log(f"\n================ DONE: {passed}/{len(results)} journeys passed ================")
    json.dump({"provider": "ollama", "model_primary": PRIMARY,
               "model_fallback": FALLBACK, "passed": passed,
               "total": len(results), "tool_count": len(tools_raw),
               "journeys": results},
              open(OUT, "w"), indent=2, ensure_ascii=False)
    log(f"wrote {OUT}")
    sys.exit(0 if passed == len(results) else 1)


if __name__ == "__main__":
    try:
        main()
    except Exception as e:
        log(f"FATAL: {type(e).__name__}: {e}")
        sys.exit(2)
