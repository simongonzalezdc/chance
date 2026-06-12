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
    let range = max - min;
    uniform_u64_lemire(source, range + 1).map(|v| min + v)
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
    let range = (max as i128) - (min as i128);
    if range >= i64::MAX as i128 {
        // For huge ranges, convert via u64.
        let raw = source.generate_u64()?;
        let offset = (raw as i128).rem_euclid(range + 1);
        return Ok(min + offset as i64);
    }
    let n = (range + 1) as u64;
    let v = uniform_u64_lemire(source, n)?;
    Ok(min + v as i64)
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
}
