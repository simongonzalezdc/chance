// Integration tests for the CLI surface: dice parsing, range generation,
// pick/shuffle, source creation, and seed parsing.
// Covers happy paths and adversarial inputs.

use chance::core::range::{
    uniform_u64_inclusive, uniform_i64_inclusive, uniform_u64_lemire, uniform_entropy_bits,
};
use chance::methods::dice::parser::parse;
use chance::methods::dice::ast::{Expr, Sign, Term, DiceTerm, DieSize};
use chance::methods::pick::{pick_one, pick_distinct};
use chance::sources::{create_source, parse_seed, OsCsprng};

// ── Dice Parser: Happy Paths ─────────────────────────────────────────────

#[test]
fn parse_basic_d20() {
    let expr = parse("d20").unwrap();
    assert_eq!(
        expr,
        Expr::Sum(vec![(
            Sign::Plus,
            Term::Dice(DiceTerm {
                count: 1,
                size: DieSize::Sides(20),
                modifiers: vec![],
            })
        )])
    );
}

#[test]
fn parse_multiple_dice_with_modifiers() {
    let expr = parse("4d6kh3").unwrap();
    match expr {
        Expr::Sum(terms) => {
            assert_eq!(terms.len(), 1);
            match &terms[0].1 {
                Term::Dice(d) => {
                    assert_eq!(d.count, 4);
                    assert_eq!(d.size, DieSize::Sides(6));
                    assert!(!d.modifiers.is_empty());
                }
                _ => panic!("expected dice term"),
            }
        }
    }
}

#[test]
fn parse_percentile_and_fudge() {
    assert!(parse("d%").is_ok());
    assert!(parse("dF").is_ok());
}

#[test]
fn parse_complex_expression() {
    assert!(parse("2d20+3").is_ok());
    assert!(parse("1d8+2d6-1").is_ok());
}

// ── Dice Parser: Adversarial ─────────────────────────────────────────────

#[test]
fn parse_empty_expression_errors() {
    // Empty input normalizes to empty → parser error
    assert!(parse("").is_err());
}

#[test]
fn parse_whitespace_only_errors() {
    // Whitespace-only normalizes to empty → parser error
    assert!(parse("   ").is_err());
}

#[test]
fn parse_d0_errors() {
    // d0 → die must have at least one side
    assert!(parse("d0").is_err());
}

#[test]
fn parse_garbage_chars_error() {
    assert!(parse("@#$").is_err());
    assert!(parse("xd20").is_err());
    assert!(parse("d20abc").is_err());
}

#[test]
fn parse_trailing_operator_errors() {
    assert!(parse("d20+").is_err());
    assert!(parse("d20-").is_err());
}

#[test]
fn parse_huge_count_parses_or_overflows_gracefully() {
    // Very large counts should not panic — either parse or error cleanly.
    let result = parse("999999999999999999999999d6");
    assert!(result.is_ok() || result.is_err());
}

// ── Range: Happy Paths ───────────────────────────────────────────────────

#[test]
fn uniform_u64_produces_values_in_range() {
    let mut src = OsCsprng::new();
    for _ in 0..1000 {
        let v = uniform_u64_inclusive(&mut src, 1, 6).unwrap();
        assert!((1..=6).contains(&v));
    }
}

#[test]
fn uniform_i64_negative_range() {
    let mut src = OsCsprng::new();
    for _ in 0..1000 {
        let v = uniform_i64_inclusive(&mut src, -10, 10).unwrap();
        assert!((-10..=10).contains(&v));
    }
}

#[test]
fn uniform_entropy_bits_known_values() {
    assert_eq!(uniform_entropy_bits(2), 1.0);
    assert_eq!(uniform_entropy_bits(6), (6.0f64).log2());
    assert_eq!(uniform_entropy_bits(1), 0.0);
    assert_eq!(uniform_entropy_bits(0), 0.0);
}

// ── Range: Adversarial ───────────────────────────────────────────────────

#[test]
fn uniform_u64_zero_range_errors() {
    let mut src = OsCsprng::new();
    assert!(uniform_u64_lemire(&mut src, 0).is_err());
}

#[test]
fn uniform_u64_inverted_range_errors() {
    let mut src = OsCsprng::new();
    assert!(uniform_u64_inclusive(&mut src, 10, 5).is_err());
}

#[test]
fn uniform_i64_inverted_range_errors() {
    let mut src = OsCsprng::new();
    assert!(uniform_i64_inclusive(&mut src, 5, -5).is_err());
}

#[test]
fn uniform_u64_single_value_range() {
    let mut src = OsCsprng::new();
    let v = uniform_u64_inclusive(&mut src, 42, 42).unwrap();
    assert_eq!(v, 42);
}

// ── Pick: Happy Paths ────────────────────────────────────────────────────

#[test]
fn pick_one_returns_element_from_list() {
    let mut src = OsCsprng::new();
    let items = vec!["a", "b", "c"];
    let chosen = pick_one(&mut src, &items).unwrap();
    assert!(items.contains(&chosen));
}

#[test]
fn pick_distinct_returns_exact_count() {
    let mut src = OsCsprng::new();
    let items: Vec<u32> = (0..100).collect();
    let winners = pick_distinct(&mut src, &items, 10).unwrap();
    assert_eq!(winners.len(), 10);
    // All winners must be distinct.
    let mut sorted = winners.clone();
    sorted.sort();
    sorted.dedup();
    assert_eq!(sorted.len(), 10);
}

#[test]
fn pick_distinct_zero_count_returns_empty() {
    let mut src = OsCsprng::new();
    let items = vec![1, 2, 3];
    let winners = pick_distinct(&mut src, &items, 0).unwrap();
    assert!(winners.is_empty());
}

// ── Pick: Adversarial ────────────────────────────────────────────────────

#[test]
fn pick_one_empty_list_errors() {
    let mut src = OsCsprng::new();
    let items: Vec<i32> = vec![];
    assert!(pick_one(&mut src, &items).is_err());
}

#[test]
fn pick_distinct_more_than_available_errors() {
    let mut src = OsCsprng::new();
    let items = vec![1, 2, 3];
    assert!(pick_distinct(&mut src, &items, 5).is_err());
}

#[test]
fn pick_distinct_empty_list_errors() {
    let mut src = OsCsprng::new();
    let items: Vec<i32> = vec![];
    assert!(pick_distinct(&mut src, &items, 1).is_err());
}

// ── Source Creation: Happy Paths ─────────────────────────────────────────

#[test]
fn create_os_csprng_source() {
    let src = create_source("os-csprng", None).unwrap();
    assert_eq!(src.name(), "os-csprng");
}

#[test]
fn create_chacha20_seeded_source() {
    let src = create_source("chacha20", Some("test-seed")).unwrap();
    assert_eq!(src.name(), "chacha20");
    assert!(src.seed().is_some());
}

#[test]
fn create_all_known_sources() {
    for name in chance::sources::source_names() {
        // drand requires network; "mix" is a prefix (mix:...) not a standalone source
        if *name == "drand" || *name == "mix" {
            continue;
        }
        let src = create_source(name, Some("seed")).unwrap_or_else(|e| {
            panic!("failed to create source '{}': {}", name, e)
        });
        assert_eq!(src.name(), *name);
    }
}

// ── Source Creation: Adversarial ─────────────────────────────────────────

#[test]
fn create_unknown_source_errors() {
    assert!(create_source("not-a-real-source", None).is_err());
}

#[test]
fn create_empty_source_name_errors() {
    assert!(create_source("", None).is_err());
}

#[test]
fn create_mix_single_source_errors() {
    // mix: requires at least two sources
    assert!(create_source("mix:chacha20", Some("seed")).is_err());
}

#[test]
fn create_mix_with_unknown_source_errors() {
    assert!(create_source("mix:chacha20,nonexistent", Some("seed")).is_err());
}

// ── Seed Parsing: Happy Paths ────────────────────────────────────────────

#[test]
fn parse_decimal_seed() {
    assert_eq!(parse_seed("12345").unwrap(), 12345);
}

#[test]
fn parse_hex_seed() {
    assert_eq!(parse_seed("0xCAFE").unwrap(), 0xCAFE);
}

#[test]
fn parse_binary_seed() {
    assert_eq!(parse_seed("0b1010").unwrap(), 0b1010);
}

#[test]
fn parse_string_seed_is_deterministic() {
    let s1 = parse_seed("hello").unwrap();
    let s2 = parse_seed("hello").unwrap();
    assert_eq!(s1, s2, "same string seed must produce same u64");
}

// ── Seed Parsing: Adversarial ────────────────────────────────────────────

#[test]
fn parse_empty_seed_errors() {
    assert!(parse_seed("").is_err());
}

#[test]
fn parse_invalid_hex_seed_errors() {
    assert!(parse_seed("0xGGGG").is_err());
}

#[test]
fn parse_invalid_binary_seed_errors() {
    assert!(parse_seed("0b2").is_err());
}