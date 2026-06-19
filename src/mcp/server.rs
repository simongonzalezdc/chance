use super::protocol::*;
use super::tools;
use serde_json::{json, Value};
use std::io::{self, BufRead, Write};

const PROTOCOL_VERSION: &str = "2024-11-05";

/// Hard cap on a single JSON-RPC request line read from stdio. A line longer
/// than this (no newline within 1 MiB) cannot be a legitimate request and is
/// discarded with a parse error rather than accumulated until the process OOMs.
const MAX_LINE_BYTES: usize = 1024 * 1024; // 1 MiB

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let stdin = io::stdin();
    // Wrap stdin in a BufReader so `fill_buf` yields small (~8 KiB) chunks; the
    // bounded reader below never accumulates more than ~`MAX_LINE_BYTES` plus
    // one chunk, even when a client streams a multi-gigabyte line with no
    // newline. This replaces the previous `reader.lines()` call, which would
    // grow an unbounded `String` for a single huge line and OOM the process.
    let mut reader = io::BufReader::new(stdin.lock());
    let mut initialized = false;

    loop {
        match read_bounded_line(&mut reader, MAX_LINE_BYTES)? {
            LineRead::Eof => break,
            LineRead::TooLong => {
                // The line was unreadably large; respond with a JSON-RPC parse
                // error (-32700) and skip it instead of aborting the server.
                let response = JsonRpcResponse::parse_error(format!(
                    "request line exceeds {} byte limit and was discarded",
                    MAX_LINE_BYTES
                ));
                println!("{}", serde_json::to_string(&response)?);
                io::stdout().flush()?;
            }
            LineRead::Line(raw) => {
                let line = raw.trim();
                if line.is_empty() {
                    continue;
                }
                if let Some(response) = handle_message(line, &mut initialized) {
                    println!("{}", serde_json::to_string(&response)?);
                    io::stdout().flush()?;
                }
            }
        }
    }

    Ok(())
}

/// Outcome of a single bounded line read.
#[derive(Debug)]
enum LineRead {
    /// A complete line (without the trailing newline), decoded as UTF-8.
    Line(String),
    /// The line exceeded the byte cap and was discarded up to its newline.
    TooLong,
    /// End of stream reached.
    Eof,
}

/// Read one line from `reader`, accumulating at most `cap` bytes of content.
///
/// If the line (excluding the trailing newline) is longer than `cap`, the
/// remainder of the line is consumed and discarded without further
/// accumulation, and `TooLong` is returned. Implemented with `fill_buf` /
/// `consume` so peak memory is bounded by `cap` plus one internal buffer
/// regardless of how large a single line grows.
fn read_bounded_line<R: BufRead>(reader: &mut R, cap: usize) -> io::Result<LineRead> {
    let mut buf: Vec<u8> = Vec::new();
    loop {
        let available = reader.fill_buf()?;
        if available.is_empty() {
            // End of stream.
            if buf.is_empty() {
                return Ok(LineRead::Eof);
            }
            // Trailing line without a final newline.
            return if buf.len() > cap {
                Ok(LineRead::TooLong)
            } else {
                Ok(LineRead::Line(decode_line(buf)?))
            };
        }
        match available.iter().position(|&b| b == b'\n') {
            Some(pos) => {
                buf.extend_from_slice(&available[..pos]);
                reader.consume(pos + 1);
                return if buf.len() > cap {
                    Ok(LineRead::TooLong)
                } else {
                    Ok(LineRead::Line(decode_line(buf)?))
                };
            }
            None => {
                // No newline yet: extend and keep reading. Once we exceed the
                // cap, stop accumulating and drain the rest of the line.
                let n = available.len();
                buf.extend_from_slice(available);
                reader.consume(n);
                if buf.len() > cap {
                    drain_until_newline(reader)?;
                    return Ok(LineRead::TooLong);
                }
            }
        }
    }
}

/// Consume and discard bytes until (and including) the next newline, or EOF.
/// Used to skip the remainder of an oversized line without accumulating it.
fn drain_until_newline<R: BufRead>(reader: &mut R) -> io::Result<()> {
    loop {
        // Extract everything we need from the borrowed buffer as plain `usize`
        // values *before* calling `consume`, so the `fill_buf` borrow ends and
        // the borrow checker is satisfied (NLL: no overlapping mutable borrow).
        let (newline_pos, len) = {
            let available = reader.fill_buf()?;
            if available.is_empty() {
                return Ok(());
            }
            (available.iter().position(|&b| b == b'\n'), available.len())
        };
        match newline_pos {
            Some(pos) => {
                reader.consume(pos + 1);
                return Ok(());
            }
            None => reader.consume(len),
        }
    }
}

/// Decode accumulated line bytes to a UTF-8 string, mirroring `BufRead::lines`,
/// which surfaces invalid UTF-8 as an `InvalidData` I/O error.
fn decode_line(buf: Vec<u8>) -> io::Result<String> {
    String::from_utf8(buf).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}

fn handle_message(line: &str, initialized: &mut bool) -> Option<JsonRpcResponse> {
    let request: JsonRpcRequest = match serde_json::from_str(line) {
        Ok(req) => req,
        Err(e) => {
            return Some(JsonRpcResponse::parse_error(format!(
                "failed to parse JSON-RPC request: {}",
                e
            )));
        }
    };

    // Validate jsonrpc version.
    if request.jsonrpc != "2.0" {
        return Some(JsonRpcResponse::invalid_params(
            request.id.clone(),
            "jsonrpc field must be '2.0'",
        ));
    }

    let id = request.id.clone();

    match request.method.as_str() {
        "initialize" => {
            let result = initialize_result();
            *initialized = true;
            Some(JsonRpcResponse::ok(id, result))
        }
        "notifications/initialized" => {
            // No response for notifications.
            None
        }
        method if !*initialized && method != "initialize" => Some(JsonRpcResponse::invalid_params(
            id,
            "server not initialized; send initialize first",
        )),
        "tools/list" => {
            let tools = tools::all_tools();
            let result = json!({ "tools": tools });
            Some(JsonRpcResponse::ok(id, result))
        }
        "tools/call" => {
            let params: CallToolParams = match request.params {
                Some(value) => serde_json::from_value(value).unwrap_or_else(|_| CallToolParams {
                    name: String::new(),
                    arguments: None,
                }),
                None => CallToolParams {
                    name: String::new(),
                    arguments: None,
                },
            };

            if params.name.is_empty() {
                return Some(JsonRpcResponse::invalid_params(
                    id,
                    "missing tool name",
                ));
            }

            let tool_result = tools::call_tool(&params);
            let result = match serde_json::to_value(tool_result) {
                Ok(v) => v,
                Err(e) => {
                    return Some(JsonRpcResponse::internal_error(
                        id,
                        format!("failed to serialize tool result: {}", e),
                    ));
                }
            };
            Some(JsonRpcResponse::ok(id, result))
        }
        _ => Some(JsonRpcResponse::method_not_found(id, &request.method)),
    }
}

fn initialize_result() -> Value {
    json!({
        "protocolVersion": PROTOCOL_VERSION,
        "capabilities": {
            "tools": {
                "listChanged": false
            }
        },
        "serverInfo": {
            "name": "chance",
            "version": env!("CARGO_PKG_VERSION")
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn normal_short_line_read_intact() {
        let mut reader = Cursor::new(b"hello world\n".to_vec());
        match read_bounded_line(&mut reader, MAX_LINE_BYTES).unwrap() {
            LineRead::Line(s) => assert_eq!(s, "hello world"),
            other => panic!("expected Line, got {:?}", other),
        }
        // Next read hits EOF.
        assert!(matches!(
            read_bounded_line(&mut reader, MAX_LINE_BYTES).unwrap(),
            LineRead::Eof
        ));
    }

    /// W9: a >1 MiB line must not allocate unboundedly. It is reported as
    /// `TooLong` (yielding a -32700 parse error in `run`) and fully discarded,
    /// so the subsequent valid line reads cleanly.
    #[test]
    fn oversized_line_is_discarded_and_next_line_survives() {
        let mut input = vec![b'a'; 2 * 1024 * 1024]; // 2 MiB, no newline yet
        input.push(b'\n');
        input.extend_from_slice(b"{\"jsonrpc\":\"2.0\"}\n");
        let mut reader = Cursor::new(input);

        match read_bounded_line(&mut reader, MAX_LINE_BYTES).unwrap() {
            LineRead::TooLong => {}
            other => panic!("expected TooLong, got {:?}", other),
        }
        match read_bounded_line(&mut reader, MAX_LINE_BYTES).unwrap() {
            LineRead::Line(s) => assert!(s.contains("2.0"), "next line corrupted: {}", s),
            other => panic!("expected Line after TooLong, got {:?}", other),
        }
    }

    #[test]
    fn line_exactly_at_cap_is_accepted() {
        let mut input = vec![b'a'; MAX_LINE_BYTES];
        input.push(b'\n');
        let mut reader = Cursor::new(input);
        match read_bounded_line(&mut reader, MAX_LINE_BYTES).unwrap() {
            LineRead::Line(s) => assert_eq!(s.len(), MAX_LINE_BYTES),
            other => panic!("expected Line at cap, got {:?}", other),
        }
    }

    #[test]
    fn one_byte_over_cap_is_too_long() {
        let mut input = vec![b'a'; MAX_LINE_BYTES + 1];
        input.push(b'\n');
        let mut reader = Cursor::new(input);
        assert!(matches!(
            read_bounded_line(&mut reader, MAX_LINE_BYTES).unwrap(),
            LineRead::TooLong
        ));
    }

    #[test]
    fn oversized_final_line_without_newline_is_too_long() {
        let input = vec![b'a'; MAX_LINE_BYTES + 10]; // no trailing newline
        let mut reader = Cursor::new(input);
        assert!(matches!(
            read_bounded_line(&mut reader, MAX_LINE_BYTES).unwrap(),
            LineRead::TooLong
        ));
    }

    #[test]
    fn empty_input_is_eof() {
        let mut reader = Cursor::new(Vec::<u8>::new());
        assert!(matches!(
            read_bounded_line(&mut reader, MAX_LINE_BYTES).unwrap(),
            LineRead::Eof
        ));
    }

    #[test]
    fn multiple_normal_lines_round_trip() {
        let mut reader = Cursor::new(b"a\nbb\nccc\n".to_vec());
        let collected: Vec<String> = std::iter::from_fn(|| {
            match read_bounded_line(&mut reader, MAX_LINE_BYTES).unwrap() {
                LineRead::Line(s) => Some(s),
                _ => None,
            }
        })
        .collect();
        assert_eq!(
            collected,
            vec!["a".to_string(), "bb".to_string(), "ccc".to_string()]
        );
    }
}
