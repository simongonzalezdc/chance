#!/usr/bin/env python3
"""End-to-end journey validation for the `chance` tool.

Validates all four user/operator surfaces through real execution:
  1. CLI  — every subcommand, output content validated
  2. API  — live server, every route, response schema validated
  3. MCP  — real Claude LLM orchestrating the chance MCP tools (multi-turn)
  4. TUI  — interactive tmux session

Real LLM calls go to the Anthropic Messages API with tool_use.
"""
import os, sys, json, subprocess, time, urllib.request, urllib.error, re, socket

CHANCE = os.path.abspath("target/release/chance")
LLM_MODEL = "claude-haiku-4-5-20251001"
LLM_TOKEN = os.environ["CLAUDE_CODE_OAUTH_TOKEN"]
EV = "artifacts/e2e"
os.makedirs(EV, exist_ok=True)

results = []
transcript = []


def log(msg):
    print(msg)
    transcript.append(msg)


def record(surface, journey, ok, detail=""):
    results.append((surface, journey, "PASS" if ok else "FAIL", detail))
    mark = "PASS" if ok else "FAIL"
    line = f"  [{mark}] {surface}/{journey}"
    if not ok and detail:
        line += f" -- {detail}"
    log(line)


# ─── LLM client (real Anthropic Messages API) ────────────────────────────
def llm(messages, tools=None, max_tokens=2048):
    body = {"model": LLM_MODEL, "max_tokens": max_tokens, "messages": messages}
    if tools:
        body["tools"] = tools
    payload = json.dumps(body).encode()
    for attempt in range(6):
        req = urllib.request.Request(
            "https://api.anthropic.com/v1/messages", data=payload)
        req.add_header("content-type", "application/json")
        req.add_header("anthropic-version", "2023-06-01")
        req.add_header("authorization", "Bearer " + LLM_TOKEN)
        try:
            resp = urllib.request.urlopen(req, timeout=90)
            return json.loads(resp.read())
        except urllib.error.HTTPError as e:
            raw = e.read().decode()
            if e.code == 429 and attempt < 5:
                wait = 5 * (2 ** attempt)  # 5, 10, 20, 40, 80
                time.sleep(wait)
                continue
            return {"_error": f"HTTP {e.code}: {raw[:300]}"}
        except Exception as e:
            if attempt < 5:
                time.sleep(3)
                continue
            return {"_error": f"{type(e).__name__}: {e}"}


# ─── MCP stdio client ────────────────────────────────────────────────────
class McpClient:
    def __init__(self):
        self.proc = subprocess.Popen(
            [CHANCE, "mcp"],
            stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
            text=True, bufsize=1)
        self._id = 0

    def _send(self, method, params=None):
        self._id += 1
        req = {"jsonrpc": "2.0", "id": self._id, "method": method}
        if params is not None:
            req["params"] = params
        self.proc.stdin.write(json.dumps(req) + "\n")
        self.proc.stdin.flush()
        line = self.proc.stdout.readline()
        return json.loads(line) if line.strip() else None

    def initialize(self):
        return self._send("initialize")

    def list_tools(self):
        return self._send("tools/list")["result"]["tools"]

    def call_tool(self, name, arguments):
        return self._send("tools/call", {"name": name, "arguments": arguments})

    def close(self):
        try:
            self.proc.stdin.close()
            self.proc.wait(timeout=5)
        except Exception:
            self.proc.kill()


# ─── CLI runner ──────────────────────────────────────────────────────────
def cli(*args):
    r = subprocess.run([CHANCE] + list(args), capture_output=True,
                       text=True, timeout=15)
    return r.returncode, r.stdout.strip(), r.stderr.strip()


# ─── helpers ─────────────────────────────────────────────────────────────
def is_int(s):
    try:
        int(s)
        return True
    except ValueError:
        return False


def free_port():
    s = socket.socket()
    s.bind(("127.0.0.1", 0))
    p = s.getsockname()[1]
    s.close()
    return p


# ═══════════════════════════════════════════════════════════════════════════
# PHASE 1: CLI JOURNEYS
# ═══════════════════════════════════════════════════════════════════════════
def cli_journeys():
    log("\n══ PHASE 1: CLI USER JOURNEYS ══")

    def chk(name, args, validator):
        rc, out, err = cli(*args)
        ok = rc == 0 and validator(out, err)
        record("CLI", name, ok, f"rc={rc} out={out[:80]!r} err={err[:80]!r}")
        return out

    chk("roll d20", ["roll", "d20"],
        lambda o, e: is_int(o) and 1 <= int(o) <= 20)
    chk("roll 4d6kh3", ["roll", "4d6kh3"],
        lambda o, e: is_int(o) and 3 <= int(o) <= 18)
    chk("roll json", ["--json", "roll", "d6"],
        lambda o, e: json.loads(o)["result"] in range(1, 7))
    chk("flip", ["flip"],
        lambda o, e: o in ("heads", "tails"))
    chk("flip x3", ["flip", "--times", "3"],
        lambda o, e: len(o.split(",")) == 3)
    chk("draw 5", ["draw", "-c", "5"],
        lambda o, e: len(o.splitlines()) == 5)
    chk("pick", ["pick", "alice", "bob", "carol"],
        lambda o, e: o in ("alice", "bob", "carol"))
    chk("shuffle", ["shuffle", "a", "b", "c", "d"],
        lambda o, e: sorted(o.splitlines()) == ["a", "b", "c", "d"])
    chk("int", ["int"],
        lambda o, e: is_int(o) and 1 <= int(o) <= 100)
    chk("int range", ["int", "--min", "1", "--max", "6"],
        lambda o, e: is_int(o) and 1 <= int(o) <= 6)
    chk("bytes hex", ["bytes", "-c", "4", "--hex"],
        lambda o, e: len(o) == 8 and re.fullmatch(r"[0-9a-f]+", o))
    chk("bytes base64", ["bytes", "-c", "8", "--base64"],
        lambda o, e: len(o) > 0)
    chk("uuid", ["uuid"],
        lambda o, e: re.fullmatch(
            r"[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}", o))
    chk("password", ["password", "-l", "16"],
        lambda o, e: len(o) == 16)
    chk("password no-symbols", ["password", "-l", "20", "--no-symbols"],
        lambda o, e: len(o) == 20 and o.isalnum())
    chk("runes", ["runes", "-c", "3"],
        lambda o, e: len(o.splitlines()) == 3)
    chk("iching", ["iching"],
        lambda o, e: "hexagram" in o.lower())
    chk("tarot", ["tarot", "-c", "3"],
        lambda o, e: len(o.splitlines()) == 3 and any(
            k in o for k in ("Cups", "Swords", "Wands", "Pentacles", "Major")))
    chk("dominoes", ["dominoes", "-c", "2"],
        lambda o, e: all(re.match(r"\[\d\|\d\]", l) for l in o.splitlines()))
    chk("roulette", ["roulette"],
        lambda o, e: bool(re.match(r"\d+ (red|black|green)", o)))
    chk("lottery", ["lottery"],
        lambda o, e: len(json.loads(o)) == 6)
    chk("knucklebones", ["knucklebones", "-c", "4"],
        lambda o, e: len(json.loads(o)) == 4)
    chk("teetotum", ["teetotum"],
        lambda o, e: o in ("N", "S", "E", "W") or len(o) <= 4)
    chk("teetotum dreidel", ["teetotum", "--dreidel"],
        lambda o, e: any(n in o.lower() for n in ("nun", "gimel", "gimmel", "hay", "shin")))
    chk("cowrie", ["cowrie", "-s", "4"],
        lambda o, e: len(o) > 0)
    chk("lots", ["lots", "a", "b", "c"],
        lambda o, e: o in ("a", "b", "c"))
    chk("sources", ["sources"],
        lambda o, e: "os-csprng" in o)
    chk("seeded determinism", ["--source", "chacha20", "--seed", "42", "roll", "d100"],
        lambda o, e: is_int(o) and 1 <= int(o) <= 100)

    # determinism check: two seeded runs must match
    rc1, o1, _ = cli("--source", "chacha20", "--seed", "7", "roll", "d20")
    rc2, o2, _ = cli("--source", "chacha20", "--seed", "7", "roll", "d20")
    record("CLI", "seeded reproducibility", o1 == o2, f"{o1} vs {o2}")

    # negative: d0 must fail cleanly
    rc, _, _ = cli("roll", "d0")
    record("CLI", "d0 rejected", rc != 0, f"rc={rc}")


# ═══════════════════════════════════════════════════════════════════════════
# PHASE 2: API JOURNEYS
# ═══════════════════════════════════════════════════════════════════════════
def api_call(port, path, body=None):
    url = f"http://127.0.0.1:{port}{path}"
    data = json.dumps(body).encode() if body is not None else None
    req = urllib.request.Request(url, data=data, method="POST" if body is not None else "GET")
    if data:
        req.add_header("content-type", "application/json")
    try:
        resp = urllib.request.urlopen(req, timeout=10)
        return resp.status, json.loads(resp.read())
    except urllib.error.HTTPError as e:
        try:
            return e.code, json.loads(e.read())
        except Exception:
            return e.code, {}


def api_journeys():
    log("\n══ PHASE 2: API OPERATOR JOURNEYS ══")
    port = free_port()
    srv = subprocess.Popen([CHANCE, "serve", "--port", str(port)],
                           stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    # wait for readiness
    ready = False
    for _ in range(40):
        try:
            urllib.request.urlopen(f"http://127.0.0.1:{port}/v1/health", timeout=1)
            ready = True
            break
        except Exception:
            time.sleep(0.25)
    record("API", "server startup", ready, f"port={port}")
    if not ready:
        srv.kill()
        return

    def chk(route, body, validator):
        st, resp = api_call(port, route, body)
        ok = st == 200 and validator(resp)
        record("API", route, ok,
               f"status={st} keys={list(resp.keys()) if isinstance(resp, dict) else '?'}")

    chk("/v1/health", None, lambda r: r.get("status") == "ok")
    chk("/v1/sources", None, lambda r: isinstance(r, list) and len(r) > 0)
    chk("/v1/roll", {"notation": "d20"},
        lambda r: "result" in r and "provenance" in r)
    chk("/v1/roll", {"notation": "2d6+3", "source": "chacha20", "seed": "1"},
        lambda r: 5 <= r["result"]["total"] <= 15)
    chk("/v1/flip", {"times": 5},
        lambda r: len(r["result"]) == 5)
    chk("/v1/draw", {"count": 7},
        lambda r: len(r["result"]) == 7)
    chk("/v1/pick", {"items": ["x", "y", "z"], "count": 2},
        lambda r: len(r["result"]) == 2)
    chk("/v1/shuffle", {"items": ["a", "b", "c"]},
        lambda r: sorted(r["result"]) == ["a", "b", "c"])
    chk("/v1/int", {"min": 1, "max": 10},
        lambda r: 1 <= r["result"] <= 10)
    chk("/v1/bytes", {"count": 16, "encoding": "hex"},
        lambda r: len(r["result"]) == 32)
    chk("/v1/uuid", {},
        lambda r: "-" in r["result"])
    chk("/v1/password", {"length": 20, "no_symbols": True},
        lambda r: len(r["result"]) == 20)
    chk("/v1/runes", {"count": 3},
        lambda r: len(r["result"]) == 3)
    chk("/v1/iching", {},
        lambda r: "primary_name" in r["result"] and "lines" in r["result"])
    chk("/v1/tarot", {"count": 2},
        lambda r: len(r["result"]) == 2)
    chk("/v1/dominoes", {"count": 2},
        lambda r: len(r["result"]) == 2)
    chk("/v1/roulette", {},
        lambda r: "result" in r)
    chk("/v1/lottery", {},
        lambda r: len(r["result"]["numbers"]) >= 6)
    chk("/v1/knucklebones", {"count": 4},
        lambda r: len(r["result"]) == 4)
    chk("/v1/teetotum", {},
        lambda r: "result" in r)
    chk("/v1/cowrie", {"shells": 4},
        lambda r: "result" in r)
    chk("/v1/lots", {"items": ["a", "b"], "count": 1},
        lambda r: len(r["result"]) == 1)

    # adversarial: bad notation → non-200
    st, _ = api_call(port, "/v1/roll", {"notation": "d0"})
    record("API", "/v1/roll d0 rejected", st != 200, f"status={st}")

    srv.terminate()
    srv.wait(timeout=5)


# ═══════════════════════════════════════════════════════════════════════════
# PHASE 3: MCP + REAL LLM JOURNEYS
# ═══════════════════════════════════════════════════════════════════════════
def mcp_llm_journeys():
    log("\n══ PHASE 3: MCP + REAL LLM ORCHESTRATION ══")
    mc = McpClient()

    # 3a. Protocol handshake
    init = mc.initialize()
    record("MCP", "initialize handshake",
           init and init.get("result", {}).get("protocolVersion"),
           str(init)[:120])
    tools = mc.list_tools()
    record("MCP", "tools/list returns tools", len(tools) >= 20,
           f"count={len(tools)}")

    # Convert MCP tools → Anthropic tool format
    anthropic_tools = [{
        "name": t["name"],
        "description": t.get("description", ""),
        "input_schema": t.get("input_schema", {"type": "object", "properties": {}}),
    } for t in tools]

    def run_agent(label, prompt, expect_tools=None, validator=None):
        """Real multi-turn LLM agent loop driving the MCP server."""
        time.sleep(3)  # pace to respect LLM rate limits
        log(f"\n  >> LLM journey: {label}")
        log(f"     prompt: {prompt[:90]}...")
        messages = [{"role": "user", "content": prompt}]
        tools_called = []
        tool_outputs = []
        rounds = 0
        final_text = ""
        err = None
        while rounds < 8:
            rounds += 1
            resp = llm(messages, anthropic_tools)
            if "_error" in resp:
                err = resp["_error"]
                break
            if resp.get("stop_reason") == "end_turn":
                final_text = "".join(
                    b["text"] for b in resp["content"] if b["type"] == "text")
                break
            # process tool_use blocks
            assistant_content = resp["content"]
            messages.append({"role": "assistant", "content": assistant_content})
            tool_results = []
            for block in resp["content"]:
                if block["type"] != "tool_use":
                    continue
                name = block["name"]
                args = block.get("input", {}) or {}
                mresp = mc.call_tool(name, args)
                tres = mresp.get("result", {})
                is_err = tres.get("is_error", False)
                text = tres.get("content", [{}])[0].get("text", "")
                tools_called.append(name)
                tool_outputs.append((name, is_err, text))
                log(f"     [round {rounds}] LLM→{name}({json.dumps(args)[:60]}) "
                    f"{'ERR' if is_err else 'ok'}: {text[:70]}")
                tool_results.append({
                    "type": "tool_result",
                    "tool_use_id": block["id"],
                    "content": text,
                    "is_error": is_err,
                })
            messages.append({"role": "user", "content": tool_results})

        # validation
        all_ok = all(not e for _, e, _ in tool_outputs)
        hit_expected = True
        if expect_tools:
            hit_expected = all(any(t == e for t in tools_called) for e in expect_tools)
        good_final = len(final_text) > 5
        ok = err is None and all_ok and hit_expected and good_final and rounds <= 8
        detail = (f"rounds={rounds} tools={tools_called} "
                  f"all_ok={all_ok} expected={hit_expected} err={err}")
        record("MCP-LLM", label, ok, detail)
        # save full transcript
        with open(f"{EV}/llm_{label.replace(' ', '_')}.json", "w") as f:
            json.dump({"prompt": prompt, "tools_called": tools_called,
                       "tool_outputs": tool_outputs,
                       "final_text": final_text, "rounds": rounds,
                       "error": err}, f, indent=2, ensure_ascii=False)
        if validator:
            record("MCP-LLM", label + " (output validity)",
                   validator(tool_outputs), "see artifact")
        return tools_called, tool_outputs

    # Journey A: multi-tool orchestration (board game setup)
    run_agent(
        "board-game-orchestration",
        "I'm hosting game night. Do all of the following using the chance tools, "
        "then summarize: (1) roll 2d6 for the first player's score, "
        "(2) flip a coin to decide if we play the red or blue team, "
        "(3) draw 5 cards from a deck. Report every result.",
        expect_tools=["chance_roll", "chance_flip", "chance_draw"],
        validator=lambda outs: sum(1 for n, _, _ in outs if n == "chance_draw") >= 1
        and sum(1 for n, _, _ in outs if n == "chance_roll") >= 1)

    # Journey B: decision-making (LLM reasons over randomness)
    run_agent(
        "dinner-decision",
        "I can't decide between pizza, sushi, and tacos for dinner. "
        "Use the chance pick tool to choose fairly among these three options, "
        "then tell me enthusiastically what we're eating and why.",
        expect_tools=["chance_pick"],
        validator=lambda outs: any(
            all(x in t for x in ("pizza", "sushi", "tacos"))
            for n, _, t in outs if n == "chance_pick"))

    # Journey C: data generation (operator provisioning)
    run_agent(
        "project-provisioning",
        "Provision credentials for a new project using chance tools: generate a "
        "20-character password with no symbols, a UUID, and 16 random bytes as hex. "
        "List all three values clearly.",
        expect_tools=["chance_password", "chance_uuid", "chance_bytes"])

    # Journey D: I Ching divination (operator uses a niche method)
    run_agent(
        "iching-cast",
        "Cast an I Ching hexagram for me using the coin method via the chance tools, "
        "and tell me the hexagram number and name.",
        expect_tools=["chance_iching"])

    mc.close()


# ═══════════════════════════════════════════════════════════════════════════
# PHASE 4: TUI JOURNEYS
# ═══════════════════════════════════════════════════════════════════════════
def tui_journeys():
    log("\n══ PHASE 4: TUI INTERACTIVE JOURNEYS ══")
    if not shutil.which("tmux"):
        record("TUI", "tmux available", False, "tmux not installed")
        return
    session = "chance_e2e"
    subprocess.run(["tmux", "kill-session", "-t", session],
                   capture_output=True)
    subprocess.run(
        ["tmux", "new-session", "-d", "-s", session, "-x", "100", "-y", "30",
         CHANCE, "tui"], capture_output=True)
    time.sleep(1.2)

    def capture():
        r = subprocess.run(["tmux", "capture-pane", "-t", session, "-p"],
                           capture_output=True, text=True)
        return r.stdout

    def send(keys):
        subprocess.run(["tmux", "send-keys", "-t", session] + keys,
                       capture_output=True)
        time.sleep(0.4)

    pane = capture()
    record("TUI", "renders method list", "roll" in pane.lower(),
           pane[:80].replace("\n", "|"))

    # navigate down and run a roll
    send(["Down"])
    send(["Enter"])
    time.sleep(0.3)
    pane = capture()
    record("TUI", "run method shows result",
           any(c.isdigit() for c in pane.split()),
           [l for l in pane.splitlines() if l.strip()][-1:])

    # open source popup, then close with Esc
    send(["s"])
    time.sleep(0.2)
    pane_open = capture()
    src_names = ["os-csprng", "chacha20", "xoshiro256", "pcg64", "splitmix64", "xoroshiro"]
    def src_count(p):
        return sum(p.lower().count(s) for s in src_names)
    record("TUI", "source popup opens", src_count(pane_open) >= 2,
           f"sources_visible={src_count(pane_open)}")
    send(["Escape"])
    time.sleep(0.2)
    pane_closed = capture()
    record("TUI", "esc closes popup",
           src_count(pane_closed) < src_count(pane_open),
           f"open={src_count(pane_open)} closed={src_count(pane_closed)}")

    # open seed popup, type a seed, enter
    send(["S"])
    send(["4", "2"])
    send(["Enter"])
    time.sleep(0.2)
    pane = capture()
    record("TUI", "seed entry accepted", "seed" in pane.lower() or "42" in pane,
           pane[:80].replace("\n", "|"))

    # quit
    send(["q"])
    time.sleep(0.3)
    alive = subprocess.run(
        ["tmux", "has-session", "-t", session], capture_output=True).returncode == 0
    record("TUI", "quit exits session", not alive)
    subprocess.run(["tmux", "kill-session", "-t", session], capture_output=True)


# ═══════════════════════════════════════════════════════════════════════════
# MAIN
# ═══════════════════════════════════════════════════════════════════════════
if __name__ == "__main__":
    import shutil
    log("═══════════════════════════════════════════════════════")
    log(f" CHANCE END-TO-END JOURNEY VALIDATION")
    log(f" binary: {CHANCE}  | LLM model: {LLM_MODEL}")
    log("═══════════════════════════════════════════════════════")
    try:
        cli_journeys()
    except Exception as e:
        record("CLI", "phase", False, f"exception: {e}")
    try:
        api_journeys()
    except Exception as e:
        record("API", "phase", False, f"exception: {e}")
    if not os.environ.get("SKIP_LLM"):
        try:
            mcp_llm_journeys()
        except Exception as e:
            record("MCP-LLM", "phase", False, f"exception: {e}")
    else:
        log("\n══ PHASE 3: MCP + LLM (skipped — run via llm_live.py) ══")
    try:
        tui_journeys()
    except Exception as e:
        record("TUI", "phase", False, f"exception: {e}")

    # Summary
    log("\n═══════════════════════════════════════════════════════")
    log(" SUMMARY")
    log("═══════════════════════════════════════════════════════")
    passes = sum(1 for _, _, s, _ in results if s == "PASS")
    fails = sum(1 for _, _, s, _ in results if s == "FAIL")
    by_surf = {}
    for surf, _, s, _ in results:
        by_surf.setdefault(surf, [0, 0])
        if s == "PASS":
            by_surf[surf][0] += 1
        else:
            by_surf[surf][1] += 1
    for surf in ("CLI", "API", "MCP", "MCP-LLM", "TUI"):
        if surf in by_surf:
            p, f = by_surf[surf]
            log(f"  {surf:10s}  {p} passed, {f} failed")
    log(f"\n  TOTAL: {passes} passed, {fails} failed")

    with open(f"{EV}/journey-report.md", "w") as f:
        f.write("# Chance End-to-End Journey Report\n\n")
        f.write(f"**{passes} passed / {fails} failed** | LLM: {LLM_MODEL}\n\n")
        cur = ""
        for surf, journey, s, detail in results:
            if surf != cur:
                cur = surf
                f.write(f"\n## {surf}\n\n")
            f.write(f"- [{'x' if s == 'PASS' else ' '}] {journey}"
                    + (f" — {detail}" if detail and s == "FAIL" else "") + "\n")
    with open(f"{EV}/journey-transcript.log", "w") as f:
        f.write("\n".join(transcript))
    sys.exit(1 if fails else 0)
