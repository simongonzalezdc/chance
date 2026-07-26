// Statistical distribution tests: verify uniformity of random outputs
// over many samples using a std-only Pearson chi-square goodness-of-fit test.
// No third-party dependencies (no proptest/quickcheck); pure std math.
// Uses a single persistent seeded source so results are reproducible.

use chance::core::range::uniform_u64_inclusive;
use chance::methods::dice::roll_dice;
use chance::sources::create_source;

/// Pearson chi-square statistic for observed counts assuming a uniform
/// expectation across all categories.
fn chi_square(observed: &[u64]) -> f64 {
    let total: u64 = observed.iter().sum();
    assert!(total > 0, "no samples collected");
    let k = observed.len() as f64;
    assert!(k > 0.0);
    let expected = total as f64 / k;
    observed
        .iter()
        .map(|&o| {
            let diff = o as f64 - expected;
            diff * diff / expected
        })
        .sum()
}

#[test]
fn d6_distribution_is_uniform() {
    let mut src = create_source("chacha20", Some("0xCAFE")).unwrap();
    let samples = 60_000u64;
    let mut counts = [0u64; 6];
    for _ in 0..samples {
        let result = roll_dice(src.as_mut(), "d6").unwrap();
        let value = result.rolls[0].value as usize;
        assert!(
            (1..=6).contains(&value),
            "d6 rolled out of range: {}",
            value
        );
        counts[value - 1] += 1;
    }
    // df = 5, p = 0.001 critical value ≈ 20.515; allow generous margin.
    let chi = chi_square(&counts);
    assert!(
        chi < 50.0,
        "d6 chi-square statistic {} exceeds threshold: {:?}",
        chi,
        counts
    );
}

#[test]
fn d20_distribution_is_uniform() {
    let mut src = create_source("chacha20", Some("0xD20")).unwrap();
    let samples = 100_000u64;
    let mut counts = [0u64; 20];
    for _ in 0..samples {
        let result = roll_dice(src.as_mut(), "d20").unwrap();
        let value = result.rolls[0].value as usize;
        assert!(
            (1..=20).contains(&value),
            "d20 rolled out of range: {}",
            value
        );
        counts[value - 1] += 1;
    }
    // df = 19, p = 0.001 critical value ≈ 43.82
    let chi = chi_square(&counts);
    assert!(
        chi < 80.0,
        "d20 chi-square statistic {} exceeds threshold: {:?}",
        chi,
        counts
    );
}

#[test]
fn coin_flip_distribution_is_uniform() {
    let mut src = create_source("chacha20", Some("0xBEEF")).unwrap();
    let samples = 40_000u64;
    let mut heads = 0u64;
    for _ in 0..samples {
        let bit = uniform_u64_inclusive(src.as_mut(), 0, 1).unwrap();
        if bit == 0 {
            heads += 1;
        }
    }
    let tails = samples - heads;
    let chi = chi_square(&[heads, tails]);
    // df = 1, p = 0.001 critical value ≈ 10.828
    assert!(
        chi < 25.0,
        "coin chi-square {} too high: heads={} tails={}",
        chi,
        heads,
        tails
    );
}

#[test]
fn byte_nibbles_are_uniform() {
    let mut src = create_source("chacha20", Some("0x5EED")).unwrap();
    let samples = 64_000u64;
    let mut counts = [0u64; 16];
    for _ in 0..samples {
        let byte = uniform_u64_inclusive(src.as_mut(), 0, 255).unwrap();
        counts[(byte >> 4) as usize] += 1;
    }
    let chi = chi_square(&counts);
    // df = 15, p = 0.001 critical value ≈ 37.697
    assert!(
        chi < 70.0,
        "byte chi-square {} exceeds threshold: {:?}",
        chi,
        counts
    );
}
