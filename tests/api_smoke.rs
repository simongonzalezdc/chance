// Integration tests for the API surface: service dispatch layer.
// Tests the same functions the axum routes call, covering happy paths
// and adversarial inputs (bad notation, empty pools, inverted ranges,
// unsupported sources, oversized counts).

use chance::core::SourceError;
use chance::services;
use chance::services::dto::*;

// ── Roll: Happy Paths ────────────────────────────────────────────────────

#[test]
fn roll_default_d20() {
    let req = RollRequest::default();
    let resp = services::roll(&req).unwrap();
    assert!(resp.result.total >= 1);
    assert!(resp.result.total <= 20);
    assert!(!resp.result.rolls.is_empty());
    assert!(resp.provenance.source.len() > 0);
}

#[test]
fn roll_with_seeded_source_is_deterministic() {
    let req = RollRequest {
        source: SourceRequest {
            source: Some("chacha20".into()),
            seed: Some("reproducible".into()),
        },
        notation: "d20".into(),
    };
    let r1 = services::roll(&req).unwrap();
    let r2 = services::roll(&req).unwrap();
    assert_eq!(
        r1.result.total, r2.result.total,
        "seeded roll must be deterministic"
    );
}

#[test]
fn roll_complex_notation() {
    let req = RollRequest {
        source: SourceRequest::default(),
        notation: "4d6kh3".into(),
    };
    let resp = services::roll(&req).unwrap();
    // 4d6kh3 keeps the highest 3; total should be between 3 and 18
    assert!(resp.result.total >= 3);
    assert!(resp.result.total <= 18);
    // Exactly 3 rolls should be kept
    assert_eq!(resp.result.rolls.len(), 3);
}

// ── Roll: Adversarial ────────────────────────────────────────────────────

#[test]
fn roll_malformed_notation_errors() {
    let req = RollRequest {
        source: SourceRequest::default(),
        notation: "not-dice".into(),
    };
    assert!(services::roll(&req).is_err());
}

#[test]
fn roll_empty_notation_errors() {
    let req = RollRequest {
        source: SourceRequest::default(),
        notation: "".into(),
    };
    assert!(services::roll(&req).is_err());
}

#[test]
fn roll_d0_errors() {
    let req = RollRequest {
        source: SourceRequest::default(),
        notation: "d0".into(),
    };
    assert!(services::roll(&req).is_err());
}

#[test]
fn roll_unsupported_source_errors() {
    let req = RollRequest {
        source: SourceRequest {
            source: Some("fake-source".into()),
            seed: None,
        },
        notation: "d20".into(),
    };
    let err = services::roll(&req).unwrap_err();
    assert!(matches!(err, SourceError::InvalidSource(_)));
}

// ── Pick: Happy Paths ────────────────────────────────────────────────────

#[test]
fn pick_happy_path() {
    let req = ListRequest {
        source: SourceRequest::default(),
        items: vec!["alice".into(), "bob".into(), "carol".into()],
        count: 1,
    };
    let resp = services::pick(&req).unwrap();
    assert_eq!(resp.result.len(), 1);
    assert!(req.items.contains(&resp.result[0]));
}

// ── Pick: Adversarial ────────────────────────────────────────────────────

#[test]
fn pick_empty_items_errors() {
    let req = ListRequest {
        source: SourceRequest::default(),
        items: vec![],
        count: 1,
    };
    assert!(services::pick(&req).is_err());
}

#[test]
fn pick_count_exceeds_items_errors() {
    let req = ListRequest {
        source: SourceRequest::default(),
        items: vec!["a".into()],
        count: 10,
    };
    assert!(services::pick(&req).is_err());
}

// ── Integer: Happy Paths ─────────────────────────────────────────────────

#[test]
fn integer_in_range() {
    let req = IntRequest {
        source: SourceRequest::default(),
        min: 1,
        max: 100,
    };
    let resp = services::integer(&req).unwrap();
    assert!(resp.result >= 1 && resp.result <= 100);
}

// ── Integer: Adversarial ─────────────────────────────────────────────────

#[test]
fn integer_inverted_range_errors() {
    let req = IntRequest {
        source: SourceRequest::default(),
        min: 100,
        max: 1,
    };
    assert!(services::integer(&req).is_err());
}

#[test]
fn integer_negative_range() {
    let req = IntRequest {
        source: SourceRequest::default(),
        min: -50,
        max: -1,
    };
    let resp = services::integer(&req).unwrap();
    assert!(resp.result >= -50 && resp.result <= -1);
}

// ── Flip: Happy Paths ────────────────────────────────────────────────────

#[test]
fn flip_single() {
    let req = FlipRequest::default();
    let resp = services::flip(&req).unwrap();
    assert_eq!(resp.result.len(), 1);
    let val = resp.result[0].to_lowercase();
    assert!(val == "heads" || val == "tails");
}

#[test]
fn flip_multiple() {
    let req = FlipRequest {
        source: SourceRequest::default(),
        times: 100,
    };
    let resp = services::flip(&req).unwrap();
    assert_eq!(resp.result.len(), 100);
}

// ── Shuffle: Happy Paths ─────────────────────────────────────────────────

#[test]
fn shuffle_preserves_elements() {
    let original = vec![
        "a".to_string(),
        "b".into(),
        "c".into(),
        "d".into(),
        "e".into(),
    ];
    let req = ShuffleRequest {
        source: SourceRequest::default(),
        items: original.clone(),
    };
    let resp = services::shuffle(&req).unwrap();
    assert_eq!(resp.result.len(), original.len());
    let mut sorted = resp.result.clone();
    sorted.sort();
    let mut orig_sorted = original.clone();
    orig_sorted.sort();
    assert_eq!(sorted, orig_sorted);
}

// ── Shuffle: Adversarial ─────────────────────────────────────────────────

#[test]
fn shuffle_empty_items() {
    let req = ShuffleRequest {
        source: SourceRequest::default(),
        items: vec![],
    };
    // Empty shuffle should succeed with empty result (no elements to shuffle)
    let resp = services::shuffle(&req);
    assert!(resp.is_ok());
    assert!(resp.unwrap().result.is_empty());
}

// ── Bytes: Happy Paths ───────────────────────────────────────────────────

#[test]
fn bytes_correct_count() {
    let req = BytesRequest {
        source: SourceRequest::default(),
        count: 32,
        encoding: "hex".into(),
    };
    let resp = services::bytes(&req).unwrap();
    // 32 bytes = 64 hex chars
    assert_eq!(resp.result.len(), 64);
}

// ── Source Names & Health ────────────────────────────────────────────────

#[test]
fn source_names_nonempty() {
    let names = services::source_names();
    assert!(names.contains(&"os-csprng".to_string()));
    assert!(names.contains(&"chacha20".to_string()));
}

#[test]
fn health_returns_ok() {
    let h = services::health();
    // health should be a serializable value
    assert!(serde_json::to_string(&h).is_ok());
}
