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
    // European/French red numbers.
    let reds = [1, 3, 5, 7, 9, 12, 14, 16, 18, 19, 21, 23, 25, 27, 30, 32, 34, 36];
    if reds.contains(&number) {
        "red"
    } else {
        "black"
    }
}

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
