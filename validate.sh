#!/usr/bin/env bash
set -u

BIN="./target/release/chance"
OUTDIR="$(mktemp -d)"
echo "Validation output dir: $OUTDIR"

PASS=0
FAIL=0

record_pass() { echo "  ✓ $1"; PASS=$((PASS+1)); }
record_fail() { echo "  ✗ $1"; FAIL=$((FAIL+1)); }

# ---------------------------------------------------------------------------
# CLI
# ---------------------------------------------------------------------------
echo ""
echo "=== CLI ==="

run_cli() {
    local name="$1"; shift
    if "$BIN" "$@" > "$OUTDIR/cli_${name}.txt" 2>&1; then
        record_pass "cli $name"
    else
        record_fail "cli $name"
    fi
}

run_cli roll roll d20
run_cli flip flip --times 3
run_cli draw draw --count 3
run_cli pick pick --count 1 alice bob carol
run_cli shuffle shuffle alice bob carol diana
run_cli int int --min 1 --max 100
run_cli bytes bytes --hex --count 8
run_cli uuid uuid --version 7
run_cli password password --length 16
run_cli runes runes --count 1
run_cli iching iching --method coin
run_cli tarot tarot --count 1
run_cli dominoes dominoes --set 6 --count 1
run_cli roulette roulette --variant european
run_cli lottery lottery --pool 49 --pick 6
run_cli knucklebones knucklebones --count 4
run_cli teetotum teetotum
run_cli cowrie cowrie --shells 4
run_cli lots lots --count 1 alpha beta gamma
run_cli sources sources
run_cli seeded_roll roll d20 --source chacha20 --seed hello
run_cli mixed roll d20 --source 'mix:os-csprng,chacha20' --seed mixseed

# ---------------------------------------------------------------------------
# API
# ---------------------------------------------------------------------------
echo ""
echo "=== API ==="
PORT=9876
"$BIN" serve --port "$PORT" > "$OUTDIR/api_server.log" 2>&1 &
SERVER_PID=$!
sleep 1

call_api() {
    local name="$1"; shift
    if curl -s -X POST "http://localhost:$PORT/v1/$1" -H 'Content-Type: application/json' -d "$2" > "$OUTDIR/api_${name}.json" 2>&1; then
        record_pass "api $name"
    else
        record_fail "api $name"
    fi
}

call_api_get() {
    local name="$1"; shift
    if curl -s "http://localhost:$PORT/v1/$1" > "$OUTDIR/api_${name}.json" 2>&1; then
        record_pass "api $name"
    else
        record_fail "api $name"
    fi
}

call_api roll roll '{"notation":"d20"}'
call_api flip flip '{"times":3}'
call_api draw draw '{"count":3}'
call_api pick pick '{"items":["alice","bob","carol"],"count":1}'
call_api shuffle shuffle '{"items":["alice","bob","carol","diana"]}'
call_api int int '{"min":1,"max":100}'
call_api bytes bytes '{"count":8,"encoding":"hex"}'
call_api uuid uuid '{"version":7}'
call_api password password '{"length":16}'
call_api runes runes '{"count":1}'
call_api iching iching '{"method":"coin"}'
call_api tarot tarot '{"count":1}'
call_api dominoes dominoes '{"set":6,"count":1}'
call_api roulette roulette '{"variant":"european"}'
call_api lottery lottery '{"pool":49,"pick":6}'
call_api knucklebones knucklebones '{"count":4}'
call_api teetotum teetotum '{}'
call_api cowrie cowrie '{"shells":4}'
call_api lots lots '{"items":["alpha","beta","gamma"],"count":1}'
call_api_get sources sources
call_api_get health health
call_api seeded_roll roll '{"notation":"d20","source":"chacha20","seed":"hello"}'
call_api mixed roll '{"notation":"d20","source":"mix:os-csprng,chacha20","seed":"mixseed"}'

kill "$SERVER_PID" 2>/dev/null || true
wait "$SERVER_PID" 2>/dev/null || true

# ---------------------------------------------------------------------------
# MCP
# ---------------------------------------------------------------------------
echo ""
echo "=== MCP ==="

MCP_IN="$OUTDIR/mcp_input.txt"
MCP_OUT="$OUTDIR/mcp_output.txt"

cat > "$MCP_IN" <<'JSON'
{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"validator","version":"1.0"}}}
{"jsonrpc":"2.0","id":2,"method":"tools/list"}
JSON

for tool in chance_roll chance_flip chance_draw chance_pick chance_shuffle chance_integer chance_bytes chance_uuid chance_password chance_runes chance_iching chance_tarot chance_dominoes chance_roulette chance_lottery chance_knucklebones chance_teetotum chance_cowrie chance_lots chance_sources chance_health; do
    case "$tool" in
        chance_roll) args='{"notation":"d20"}' ;;
        chance_flip) args='{"times":3}' ;;
        chance_draw) args='{"count":3}' ;;
        chance_pick) args='{"items":["alice","bob","carol"],"count":1}' ;;
        chance_shuffle) args='{"items":["alice","bob","carol","diana"]}' ;;
        chance_integer) args='{"min":1,"max":100}' ;;
        chance_bytes) args='{"count":8,"encoding":"hex"}' ;;
        chance_uuid) args='{"version":7}' ;;
        chance_password) args='{"length":16}' ;;
        chance_runes) args='{"count":1}' ;;
        chance_iching) args='{"method":"coin"}' ;;
        chance_tarot) args='{"count":1}' ;;
        chance_dominoes) args='{"set":6,"count":1}' ;;
        chance_roulette) args='{"variant":"european"}' ;;
        chance_lottery) args='{"pool":49,"pick":6}' ;;
        chance_knucklebones) args='{"count":4}' ;;
        chance_teetotum) args='{}' ;;
        chance_cowrie) args='{"shells":4}' ;;
        chance_lots) args='{"items":["alpha","beta","gamma"],"count":1}' ;;
        chance_sources|chance_health) args='{}' ;;
        *) args='{}' ;;
    esac
    echo "{\"jsonrpc\":\"2.0\",\"id\":\"${tool}\",\"method\":\"tools/call\",\"params\":{\"name\":\"${tool}\",\"arguments\":${args}}}" >> "$MCP_IN"
done

"$BIN" mcp < "$MCP_IN" > "$MCP_OUT" 2>&1

MCP_ERRORS=$(grep -c '"is_error":true' "$MCP_OUT" || true)
MCP_OK=$(grep -c '"is_error":false' "$MCP_OUT" || true)
if [ "$MCP_ERRORS" -eq 0 ] && [ "$MCP_OK" -ge 19 ]; then
    record_pass "mcp all tools"
else
    record_fail "mcp all tools (errors=$MCP_ERRORS, ok=$MCP_OK)"
fi

# ---------------------------------------------------------------------------
# TUI visual smoke test via tmux
# ---------------------------------------------------------------------------
echo ""
echo "=== TUI (tmux snapshot) ==="

SESSION="chance-tui-$$"
tmux new-session -d -s "$SESSION" -x 80 -y 30 "$BIN" tui
sleep 0.8
# Capture initial frame
tmux capture-pane -t "$SESSION" -p > "$OUTDIR/tui_initial.txt" 2>&1
# Send Enter to roll, wait, capture again
tmux send-keys -t "$SESSION" Enter
sleep 0.3
tmux capture-pane -t "$SESSION" -p > "$OUTDIR/tui_after_roll.txt" 2>&1
# Navigate down a few methods and generate
tmux send-keys -t "$SESSION" Down Down Down Enter
sleep 0.3
tmux capture-pane -t "$SESSION" -p > "$OUTDIR/tui_after_pick.txt" 2>&1
# Quit
tmux send-keys -t "$SESSION" q
sleep 0.3
tmux kill-session -t "$SESSION" 2>/dev/null || true

if grep -q "Methods" "$OUTDIR/tui_initial.txt" && grep -q "Result" "$OUTDIR/tui_initial.txt"; then
    record_pass "tui renders"
else
    record_fail "tui renders"
fi

if grep -q '"total"' "$OUTDIR/tui_after_roll.txt" || grep -q 'total' "$OUTDIR/tui_after_roll.txt"; then
    record_pass "tui generates roll result"
else
    record_fail "tui generates roll result"
fi

# ---------------------------------------------------------------------------
# Summary
# ---------------------------------------------------------------------------
echo ""
echo "=== Summary ==="
echo "Passed: $PASS"
echo "Failed: $FAIL"
echo "Output directory: $OUTDIR"

if [ "$FAIL" -gt 0 ]; then
    exit 1
fi
