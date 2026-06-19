use crate::core::source::Source;
use crate::core::range::{uniform_i64_inclusive, uniform_u64_inclusive};
use crate::methods::dice::ast::*;
use crate::methods::dice::parser::parse;

/// Maximum number of times a single die may explode (`Explode` / `ExplodeCompound`).
///
/// Mirrors the reroll guard (`guard < 100`) so that degenerate inputs such as
/// `1d1!` — where every roll equals the explode maximum — terminate instead of
/// looping forever.
const MAX_EXPLOSIONS_PER_DIE: u32 = 100;

#[derive(Debug, Clone)]
pub struct RollResult {
    pub total: i64,
    pub rolls: Vec<DieRoll>,
    pub dropped: Vec<DieRoll>,
    pub modifier_total: i64,
    pub success_count: Option<u64>,
}

#[derive(Debug, Clone)]
pub struct DieRoll {
    pub value: i64,
    pub size: i64,
    pub exploded: bool,
    pub rerolled: bool,
}

pub fn roll_dice(source: &mut dyn Source, notation: &str) -> Result<RollResult, crate::core::SourceError> {
    let expr = parse(notation).map_err(|e| {
        crate::core::SourceError::GenerationFailed(format!("dice parse error: {e}"))
    })?;
    evaluate(source, &expr)
}

fn evaluate(source: &mut dyn Source, expr: &Expr) -> Result<RollResult, crate::core::SourceError> {
    let Expr::Sum(terms) = expr;
    let mut total = 0i64;
    let mut all_rolls = Vec::new();
    let mut all_dropped = Vec::new();
    let mut modifier_total = 0i64;
    let mut success_count: Option<u64> = None;

    for (sign, term) in terms {
        let sign_mult = if *sign == Sign::Plus { 1 } else { -1 };
        match term {
            Term::Number(n) => {
                modifier_total += sign_mult * n;
            }
            Term::Dice(dice) => {
                let result = roll_die_term(source, dice)?;
                total += sign_mult * result.total;
                all_rolls.extend(result.rolls);
                all_dropped.extend(result.dropped);
                if let Some(sc) = result.success_count {
                    success_count = Some(success_count.unwrap_or(0) + sc);
                }
            }
        }
    }

    total += modifier_total;

    Ok(RollResult {
        total,
        rolls: all_rolls,
        dropped: all_dropped,
        modifier_total,
        success_count,
    })
}

fn roll_die_term(source: &mut dyn Source, dice: &DiceTerm) -> Result<RollResult, crate::core::SourceError> {
    let (size, fudge) = match dice.size {
        DieSize::Sides(n) => (n as i64, false),
        DieSize::Percentile => (100, false),
        DieSize::Fudge => (3, true),
    };

    // A 1-sided die cannot meaningfully explode: every roll equals the explode
    // maximum, so the explosion loops below would never terminate naturally.
    // Fail fast rather than rely solely on the per-die explosion cap.
    if size == 1
        && dice
            .modifiers
            .iter()
            .any(|m| matches!(m, Modifier::Explode | Modifier::ExplodeCompound))
    {
        return Err(crate::core::SourceError::GenerationFailed(
            "a d1 cannot meaningfully explode (every roll equals the explode maximum)".to_string(),
        ));
    }

    let mut rolls: Vec<DieRoll> = Vec::new();

    for _ in 0..dice.count {
        let value = if fudge {
            // Fudge dice: 1 -> -1, 2 -> 0, 3 -> +1
            uniform_u64_inclusive(source, 1, 3)? as i64 - 2
        } else {
            uniform_i64_inclusive(source, 1, size)?
        };

        let mut roll = DieRoll {
            value,
            size,
            exploded: false,
            rerolled: false,
        };

        // Apply reroll modifiers before anything else.
        for modifier in &dice.modifiers {
            if let Modifier::Reroll { comparator, value, once } = modifier {
                let mut guard = 0u32;
                while comparator.compare(roll.value, *value) && guard < 100 {
                    roll.value = if fudge {
                        uniform_u64_inclusive(source, 1, 3)? as i64 - 2
                    } else {
                        uniform_i64_inclusive(source, 1, size)?
                    };
                    roll.rerolled = true;
                    guard += 1;
                    if *once {
                        break;
                    }
                }
            }
        }

        rolls.push(roll);

        // Apply exploding.
        let explode_max = if fudge { 1 } else { size };
        for modifier in &dice.modifiers {
            match modifier {
                Modifier::Explode => {
                    let mut explosions = 0u32;
                    loop {
                        let last = rolls.last().unwrap();
                        if last.value != explode_max || last.rerolled {
                            break;
                        }
                        if explosions >= MAX_EXPLOSIONS_PER_DIE {
                            break;
                        }
                        let v = if fudge {
                            uniform_u64_inclusive(source, 1, 3)? as i64 - 2
                        } else {
                            uniform_i64_inclusive(source, 1, size)?
                        };
                        rolls.push(DieRoll {
                            value: v,
                            size,
                            exploded: true,
                            rerolled: false,
                        });
                        explosions += 1;
                    }
                }
                Modifier::ExplodeCompound => {
                    let mut explosions = 0u32;
                    loop {
                        let last = rolls.last().unwrap();
                        if last.value != explode_max || last.rerolled {
                            break;
                        }
                        if explosions >= MAX_EXPLOSIONS_PER_DIE {
                            break;
                        }
                        let v = if fudge {
                            uniform_u64_inclusive(source, 1, 3)? as i64 - 2
                        } else {
                            uniform_i64_inclusive(source, 1, size)?
                        };
                        let prev = rolls.last_mut().unwrap();
                        prev.value += v;
                        prev.exploded = true;
                        explosions += 1;
                    }
                }
                _ => {}
            }
        }
    }

    // Apply keep/drop modifiers.
    let mut kept = rolls.clone();
    let mut dropped = Vec::new();

    for modifier in &dice.modifiers {
        match *modifier {
            Modifier::KeepHighest(n) => {
                kept.sort_by_key(|r| -r.value);
                let split_point = n.min(kept.len() as u64) as usize;
                dropped.extend(kept.split_off(split_point));
            }
            Modifier::KeepLowest(n) => {
                kept.sort_by_key(|r| r.value);
                let split_point = n.min(kept.len() as u64) as usize;
                dropped.extend(kept.split_off(split_point));
            }
            Modifier::DropHighest(n) => {
                kept.sort_by_key(|r| -r.value);
                let split_point = n.min(kept.len() as u64) as usize;
                dropped.extend(kept.drain(..split_point));
            }
            Modifier::DropLowest(n) => {
                kept.sort_by_key(|r| r.value);
                let split_point = n.min(kept.len() as u64) as usize;
                dropped.extend(kept.drain(..split_point));
            }
            _ => {}
        }
    }

    let total: i64 = kept.iter().map(|r| r.value).sum();

    // Success counting.
    let mut success_count: Option<u64> = None;
    for modifier in &dice.modifiers {
        if let Modifier::Success { comparator, value } = modifier {
            let count = kept.iter().filter(|r| comparator.compare(r.value, *value)).count() as u64;
            success_count = Some(success_count.unwrap_or(0) + count);
        }
    }

    Ok(RollResult {
        total,
        rolls: kept,
        dropped,
        modifier_total: 0,
        success_count,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{SourceError, SourceHealth, SourceKind};

    /// Deterministic source that always yields the same `u64`.
    ///
    /// Returning `u64::MAX` forces the worst case for the range samplers:
    /// `uniform_i64_inclusive(1, n)` returns `n` (the explode maximum), so every
    /// die explodes. This is exactly the degenerate condition that made `1d1!`
    /// loop forever before the fix — for a d1 it holds regardless of the source.
    struct FixedSource {
        value: u64,
    }

    impl Source for FixedSource {
        fn name(&self) -> String {
            "fixed".to_string()
        }
        fn kind(&self) -> SourceKind {
            SourceKind::Csprng
        }
        fn generate_u64(&mut self) -> Result<u64, SourceError> {
            Ok(self.value)
        }
        fn fill_bytes(&mut self, buf: &mut [u8]) -> Result<(), SourceError> {
            for byte in buf.iter_mut() {
                *byte = (self.value & 0xFF) as u8;
            }
            Ok(())
        }
        fn health(&self) -> SourceHealth {
            SourceHealth::Healthy
        }
    }

    /// Regression for BUG B4: `1d1!` / `1d1!!` previously looped forever because
    /// every roll of a d1 equals the explode maximum. With an always-maximum
    /// source this is the worst case for *any* die size, so these calls
    /// terminating at all is the proof of the fix.
    #[test]
    fn b4_d1_explode_terminates_quickly() {
        let mut src = FixedSource { value: u64::MAX };

        // `1d1!` and `1d1!!` must terminate quickly (capped result or error),
        // never hang. A d1 cannot meaningfully explode, so they error out.
        assert!(roll_dice(&mut src, "1d1!").is_err());
        assert!(roll_dice(&mut src, "1d1!!").is_err());

        // Normal exploding dice still succeed, exploding up to the cap.
        let result = roll_dice(&mut src, "4d6!");
        assert!(result.is_ok(), "4d6! should succeed");
        let result = result.unwrap();
        assert!(result.total > 0, "4d6! should produce a positive total");
    }
}
