use crate::core::range::uniform_u64_inclusive;
use crate::core::source::Source;

#[derive(Debug, Clone)]
pub struct RouletteResult {
    pub number: u8,
    pub color: &'static str,
    pub variant: &'static str,
    pub house_edge_percent: f64,
}

impl std::fmt::Display for RouletteResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {} ({}, house edge {:.2}%)", self.number, self.color, self.variant, self.house_edge_percent)
    }
}

fn roulette_color(number: u8, variant: &str) -> &'static str {
    if number == 0 {
        return "green";
    }
    if variant == "american" && number == 37 {
        return "green"; // 00 represented as 37
    }
    // European red numbers.
    let reds = [1, 3, 5, 7, 9, 12, 14, 16, 18, 19, 21, 23, 25, 27, 30, 32, 34, 36];
    if reds.contains(&number) {
        "red"
    } else {
        "black"
    }
}

/// Spin a roulette wheel.
///
/// Only `american` (0 and 00, ~5.26% house edge) and `european` (single zero,
/// ~2.70% house edge) are implemented. Any other `variant` string — including
/// the previously-advertised `french` — falls back to the single-zero european
/// wheel. La Partage is NOT implemented, so `french` is intentionally not
/// advertised as a distinct variant by the MCP/API schemas.
pub fn spin_roulette(
    source: &mut dyn Source,
    variant: &str,
) -> Result<RouletteResult, crate::core::SourceError> {
    let (max, edge) = if variant == "american" {
        (37u64, 5.26) // 0 and 00 (37)
    } else {
        (36u64, 2.70) // European
    };

    let raw = uniform_u64_inclusive(source, 0, max)? as u8;
    let number = if variant == "american" && raw == 37 { 37 } else { raw };
    let color = roulette_color(number, variant);
    let variant_label = if variant == "american" { "american" } else { "european" };

    Ok(RouletteResult {
        number,
        color,
        variant: variant_label,
        house_edge_percent: edge,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sources::OsCsprng;

    #[test]
    fn american_and_european_in_range() {
        let mut src = OsCsprng::new();
        for _ in 0..200 {
            let eu = spin_roulette(&mut src, "european").unwrap();
            assert!(eu.number <= 36, "european number out of range: {}", eu.number);
            assert_eq!(eu.variant, "european");
            assert_eq!(eu.house_edge_percent, 2.70);

            let am = spin_roulette(&mut src, "american").unwrap();
            assert!(am.number <= 37, "american number out of range: {}", am.number);
            assert_eq!(am.variant, "american");
            assert_eq!(am.house_edge_percent, 5.26);
        }
    }

    /// Honesty: a previously-advertised `french` (or any unrecognized) variant
    /// is treated as european and never as a distinct La-Partage variant.
    #[test]
    fn unrecognized_variant_falls_back_to_european() {
        let mut src = OsCsprng::new();
        for v in ["french", "european", "FRENCH", "", "xyz"] {
            let r = spin_roulette(&mut src, v).unwrap();
            assert_eq!(r.variant, "european", "variant {:?} must map to european", v);
            assert_eq!(r.house_edge_percent, 2.70);
            assert!(r.number <= 36);
        }
    }

    #[test]
    fn zero_and_double_zero_coloring() {
        // 0 is green on both wheels; 37 (00) is green only on american.
        assert_eq!(roulette_color(0, "european"), "green");
        assert_eq!(roulette_color(0, "american"), "green");
        assert_eq!(roulette_color(37, "american"), "green");
        assert_eq!(roulette_color(37, "european"), "black");
    }
}
