#!/usr/bin/env python3
"""Detached, patient LLM→MCP journey runner.

Launched in the background because the only available LLM credential is the
OAuth token shared with the active Claude Code session, which saturates the
per-minute rate limit while the session is busy. This process retries patiently
(60s spacing) so it catches the rate-limit windows that open while the session
is idle between turns.

Output: artifacts/e2e/llm_live.log (progress) + artifacts/e2e/llm_results.json
"""
import os, sys, json, subprocess, time, urllib.request, urllib.error

CHANCE = os.path.abspath("target/release/chance")
MODEL = "claude-haiku-4-5-20251001"
TOKEN = os.environ["CLAUDE_CODE_OAUTH_TOKEN"]
EV = "artifacts/e2e"
LOG = f"{EV}/llm_live.log"

_logf = open(LOG, "a", buffering=1)


def log(m):
    line = f"[{time.strftime('%H:%M:%S')}] {m}"
    print(line, flush=True)
    _logf.write(line + "\n")


def llm(messages, tools, max_tokens=2048, max_tries=40):
    payload = json.dumps({"model": MODEL, "max_tokens": max_tokens,
                          "messages": messages, "tools": tools}).encode()
    for attempt in range(max_tries):
        req = urllib.request.Request(
            "https://api.anthropic.com/v1/messages", data=payload)
        req.add_header("content-type", "application/json")
        req.add_header("anthropic-version", "2023-06-01")
        req.add_header("authorization", "Bearer " + TOKEN)
        try:
            return json.loads(urllib.request.urlopen(req, timeout=90).read())
        except urllib.error.HTTPError as e:
            if e.code == 429:
                if attempt % 5 == 0:
                    log(f"  ...429, retry {attempt+1}/{max_tries}")
                time.sleep(60)
                continue
            return {"_error": f"HTTP {e.code}: {e.read().decode()[:200]}"}
        except Exception as e:
            time.sleep(30)
    return {"_error": "exhausted retries (rate limit)"}


class Mcp:
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


def journey(mc, tools, label, prompt, expect):
    log(f"\n=== JOURNEY: {label} ===")
    log(f"prompt: {prompt[:100]}")
    messages = [{"role": "user", "content": prompt}]
    called, outputs = [], []
    final, err = "", None
    for rnd in range(8):
        resp = llm(messages, tools)
        if "_error" in resp:
            err = resp["_error"]
            break
        if resp.get("stop_reason") == "end_turn":
            final = "".join(b["text"] for b in resp["content"] if b["type"] == "text")
            break
        messages.append({"role": "assistant", "content": resp["content"]})
        results = []
        for b in resp["content"]:
            if b["type"] != "tool_use":
                continue
            mr = mc.call("tools/call", {"name": b["name"], "arguments": b.get("input", {}) or {}})
            tr = mr.get("result", {})
            txt = tr.get("content", [{}])[0].get("text", "")
            is_err = tr.get("is_error", False)
            called.append(b["name"])
            outputs.append((b["name"], b.get("input", {}), is_err, txt))
            log(f"  round {rnd+1}: LLM -> {b['name']}({json.dumps(b.get('input',{}))[:50]}) "
                f"{'ERR' if is_err else 'ok'}: {txt[:80]}")
            results.append({"type": "tool_result", "tool_use_id": b["id"],
                            "content": txt, "is_error": is_err})
        messages.append({"role": "user", "content": results})
    ok = (err is None and all(not e for _, _, e, _ in outputs)
          and all(any(t == x for t in called) for x in expect)
          and len(final) > 5)
    log(f"  RESULT: {'PASS' if ok else 'FAIL'} | tools={called} | err={err}")
    log(f"  final: {final[:200]}")
    return {"label": label, "pass": ok, "tools_called": called,
            "outputs": [{"tool": n, "input": i, "error": e, "text": t}
                        for n, i, e, t in outputs],
            "final_text": final, "error": err}


def main():
    log("======== DETACHED LLM→MCP JOURNEY RUNNER STARTING ========")
    mc = Mcp()
    init = mc.call("initialize")
    log(f"MCP initialize: protocolVersion={init.get('result',{}).get('protocolVersion')}")
    tools_raw = mc.call("tools/list")["result"]["tools"]
    log(f"MCP tools/list: {len(tools_raw)} tools")
    tools = [{"name": t["name"], "description": t.get("description", ""),
              "input_schema": t.get("input_schema", {"type": "object"})}
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
         "Provision a new project using chance tools: generate a 20-char password with "
         "no symbols, a UUID, and 16 random bytes as hex. List all three values clearly.",
         ["chance_password", "chance_uuid", "chance_bytes"]),
        ("iching-divination",
         "Cast an I Ching hexagram via the coin method using the chance tools, "
         "and tell me the hexagram number and name.",
         ["chance_iching"]),
    ]
    results = []
    for label, prompt, expect in journeys:
        results.append(journey(mc, tools, label, prompt, expect))
    mc.close()

    passed = sum(1 for r in results if r["pass"])
    log(f"\n======== DONE: {passed}/{len(results)} journeys passed ========")
    json.dump({"model": MODEL, "tool_count": len(tools_raw),
               "passed": passed, "total": len(results), "journeys": results},
              open(f"{EV}/llm_results.json", "w"), indent=2, ensure_ascii=False)
    sys.exit(0 if passed == len(results) else 1)


if __name__ == "__main__":
    try:
        main()
    except Exception as e:
        log(f"FATAL: {type(e).__name__}: {e}")
        sys.exit(2)
