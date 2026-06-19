use crate::core::source::Source;
use crate::core::SourceError;

/// Generate a uniform `u64` in `[0, n)` using Lemire's nearly-divisionless method.
///
/// See: Lemire, "Fast Random Integer Generation in an Interval", ACM TOMS 2019.
pub fn uniform_u64_lemire(source: &mut dyn Source, n: u64) -> Result<u64, SourceError> {
    if n == 0 {
        return Err(SourceError::GenerationFailed(
            "range must be greater than zero".to_string(),
        ));
    }

    let mut x = source.generate_u64()?;
    let mut m = u128::from(x) * u128::from(n);
    let mut l = m as u64;

    if l < n {
        let t = (!n).wrapping_add(1) % n;
        while l < t {
            x = source.generate_u64()?;
            m = u128::from(x) * u128::from(n);
            l = m as u64;
        }
    }

    Ok((m >> 64) as u64)
}

/// Generate a uniform `u64` in `[min, max]` (inclusive) using Lemire's method.
pub fn uniform_u64_inclusive(
    source: &mut dyn Source,
    min: u64,
    max: u64,
) -> Result<u64, SourceError> {
    if min > max {
        return Err(SourceError::GenerationFailed(
            "min must be <= max".to_string(),
        ));
    }
    // Full u64 range [0, u64::MAX]: range would be u64::MAX and `range + 1`
    // overflows. The full 2^64 range is bijective with a raw u64, so emit one.
    if min == 0 && max == u64::MAX {
        return source.generate_u64();
    }
    // Otherwise range in [0, u64::MAX - 1], so `range + 1` fits in u64.
    let range = max - min;
    let v = uniform_u64_lemire(source, range + 1)?;
    Ok(min + v)
}

/// Generate a uniform `i64` in `[min, max]` (inclusive).
pub fn uniform_i64_inclusive(
    source: &mut dyn Source,
    min: i64,
    max: i64,
) -> Result<i64, SourceError> {
    if min > max {
        return Err(SourceError::GenerationFailed(
            "min must be <= max".to_string(),
        ));
    }
    let span = (max as i128) - (min as i128); // in [0, 2^64 - 1]
    // The full i64 range is exactly 2^64 values, bijective with u64, so a raw
    // u64 is correct and unbiased. For smaller spans, span+1 in [1, 2^64 - 1]
    // fits u64 and the Lemire path is unbiased.
    let offset = if span >= u64::MAX as i128 {
        source.generate_u64()? as i128
    } else {
        uniform_u64_lemire(source, (span + 1) as u64)? as i128
    };
    Ok(((min as i128) + offset) as i64)
}

/// Generate a uniform `f64` in `[0.0, 1.0)` with 53 bits of precision.
pub fn uniform_f64(source: &mut dyn Source) -> Result<f64, SourceError> {
    let x = source.generate_u64()? >> 11;
    Ok(f64::from_bits(0x3FF0000000000000 | x) - 1.0)
}

/// Generate a uniform `f64` in `[min, max)`.
pub fn uniform_f64_range(
    source: &mut dyn Source,
    min: f64,
    max: f64,
) -> Result<f64, SourceError> {
    if !(min < max) {
        return Err(SourceError::GenerationFailed(
            "min must be < max for f64 range".to_string(),
        ));
    }
    let u = uniform_f64(source)?;
    Ok(min + u * (max - min))
}

/// Compute Shannon entropy in bits for a uniform choice among `n` outcomes.
pub fn uniform_entropy_bits(n: u64) -> f64 {
    if n <= 1 {
        0.0
    } else {
        (n as f64).log2()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sources::OsCsprng;

    #[test]
    fn test_uniform_u64_inclusive() {
        let mut src = OsCsprng::new();
        for _ in 0..1000 {
            let v = uniform_u64_inclusive(&mut src, 1, 6).unwrap();
            assert!((1..=6).contains(&v));
        }
    }

    #[test]
    fn test_uniform_f64() {
        let mut src = OsCsprng::new();
        for _ in 0..1000 {
            let v = uniform_f64(&mut src).unwrap();
            assert!((0.0..1.0).contains(&v));
        }
    }

    /// B5(1): full u64 range must not overflow on `range + 1`.
    #[test]
    fn test_uniform_u64_inclusive_full_range() {
        let mut src = OsCsprng::new();
        for _ in 0..1000 {
            let v = uniform_u64_inclusive(&mut src, 0, u64::MAX).unwrap();
            assert!(v <= u64::MAX, "value out of full u64 range");
        }
    }

    /// B5(2b): full i64 range must not panic and must stay in range.
    #[test]
    fn test_uniform_i64_inclusive_full_range() {
        let mut src = OsCsprng::new();
        for _ in 0..1000 {
            let v = uniform_i64_inclusive(&mut src, i64::MIN, i64::MAX).unwrap();
            assert!(
                (i64::MIN..=i64::MAX).contains(&v),
                "value {v} out of full i64 range"
            );
        }
    }

    /// B5(2c): range [i64::MIN, i64::MAX - 1] previously hit the biased
    /// rem_euclid branch; now it uses the unbiased Lemire path.
    #[test]
    fn test_uniform_i64_inclusive_near_full_range() {
        let mut src = OsCsprng::new();
        for _ in 0..1000 {
            let v = uniform_i64_inclusive(&mut src, i64::MIN, i64::MAX - 1).unwrap();
            assert!(
                (i64::MIN..=(i64::MAX - 1)).contains(&v),
                "value {v} out of range [i64::MIN, i64::MAX - 1]"
            );
        }
    }

    /// B5(2d): distribution sanity over a small span (d6), each face within
    /// 15% of the expected count across 60000 rolls.
    #[test]
    fn test_uniform_i64_inclusive_distribution_d6() {
        let mut src = OsCsprng::new();
        let mut counts = [0u64; 6];
        for _ in 0..60_000 {
            let v = uniform_i64_inclusive(&mut src, 1, 6).unwrap();
            counts[(v - 1) as usize] += 1;
        }
        for (face, &c) in counts.iter().enumerate() {
            let deviation = ((c as i64) - 10_000).unsigned_abs() as f64 / 10_000.0;
            assert!(
                deviation <= 0.15,
                "face {} count {} deviates {:.2}% from 10000 (>15%)",
                face + 1,
                c,
                deviation * 100.0
            );
        }
    }
}
