// CLI argument edge-case tests: exercise the compiled `chance` binary with
// extreme and unusual inputs (very long arguments, unicode item lists,
// boundary integers, large dice counts, unicode seeds). Verifies graceful
// handling with no crashes or uncontrolled panics.

use std::process::Command;

const BIN: &str = env!("CARGO_BIN_EXE_chance");

fn run(args: &[&str]) -> (bool, String, String) {
    let output = Command::new(BIN)
        .args(args)
        .output()
        .expect("failed to spawn chance binary");
    (
        output.status.success(),
        String::from_utf8_lossy(&output.stdout).to_string(),
        String::from_utf8_lossy(&output.stderr).to_string(),
    )
}

#[test]
fn cli_handles_unicode_items_in_pick() {
    let (ok, stdout, _) = run(&["pick", "café", "ñandú", "日本語", "🎲", "αβγ"]);
    assert!(ok, "unicode pick failed: {}", stdout);
    // The chosen winner must be one of the unicode items.
    let trimmed = stdout.trim();
    assert!(
        ["café", "ñandú", "日本語", "🎲", "αβγ"]
            .iter()
            .any(|s| trimmed.contains(s)),
        "pick output did not contain a unicode item: {}",
        trimmed
    );
}

#[test]
fn cli_handles_very_long_item_list() {
    // 500 items, each moderately long.
    let items: Vec<String> = (0..500).map(|i| format!("candidate-{:04}", i)).collect();
    let mut args: Vec<String> = vec!["pick".to_string(), "--count".into(), "1".into()];
    args.extend(items);
    let owned: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
    let (ok, stdout, stderr) = run(&owned);
    assert!(
        ok,
        "long item list failed: stdout={} stderr={}",
        stdout.trim(),
        stderr.trim()
    );
}

#[test]
fn cli_large_but_valid_dice_count_succeeds() {
    // 1000d6 is valid notation and should succeed without crashing.
    let (ok, stdout, stderr) = run(&["roll", "1000d6"]);
    assert!(
        ok,
        "1000d6 failed: stdout={} stderr={}",
        stdout.trim(),
        stderr.trim()
    );
    // Total must be in [1000, 6000].
    let total: i64 = stdout
        .trim()
        .parse()
        .expect("roll output should be an integer");
    assert!(
        (1000..=6000).contains(&total),
        "1000d6 total out of range: {}",
        total
    );
}

#[test]
fn cli_d0_notation_errors_cleanly() {
    let (ok, stdout, stderr) = run(&["roll", "d0"]);
    // d0 is invalid; the binary should exit non-zero with a message, no panic.
    assert!(
        !ok,
        "d0 should have failed but succeeded: {}",
        stdout.trim()
    );
    let combined = format!("{}\n{}", stdout.to_lowercase(), stderr.to_lowercase());
    assert!(
        combined.contains("error") || combined.contains("parse"),
        "d0 error output not descriptive: combined={}",
        combined.trim()
    );
}

#[test]
fn cli_boundary_integer_max() {
    let (ok, stdout, _) = run(&[
        "int",
        "--min",
        "9223372036854775807",
        "--max",
        "9223372036854775807",
    ]);
    assert!(ok, "i64::MAX boundary int failed: {}", stdout.trim());
    assert!(
        stdout.trim().contains("9223372036854775807"),
        "expected i64::MAX in output: {}",
        stdout.trim()
    );
}

#[test]
fn cli_seeded_roll_is_deterministic() {
    // Two runs with the same seed and deterministic source must match exactly.
    let (ok1, out1, _) = run(&["--source", "chacha20", "--seed", "42", "roll", "d100"]);
    let (ok2, out2, _) = run(&["--source", "chacha20", "--seed", "42", "roll", "d100"]);
    assert!(ok1 && ok2, "seeded roll invocations failed");
    assert_eq!(out1, out2, "seeded roll output is not deterministic");
}

#[test]
fn cli_unicode_seed_is_deterministic() {
    // A unicode string seed must be accepted and produce deterministic output.
    let (ok1, out1, _) = run(&["--source", "chacha20", "--seed", "🎲🎲", "roll", "d20"]);
    let (ok2, out2, _) = run(&["--source", "chacha20", "--seed", "🎲🎲", "roll", "d20"]);
    assert!(ok1 && ok2, "unicode-seeded roll invocations failed");
    assert_eq!(
        out1, out2,
        "unicode-seeded roll output is not deterministic"
    );
}

#[test]
fn cli_empty_shuffle_succeeds() {
    let (ok, stdout, _) = run(&["shuffle"]);
    assert!(ok, "empty shuffle failed: {}", stdout.trim());
}
