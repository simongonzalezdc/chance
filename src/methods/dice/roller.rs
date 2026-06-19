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
    let mut total: i128 = 0;
    let mut all_rolls = Vec::new();
    let mut all_dropped = Vec::new();
    let mut modifier_total: i128 = 0;
    let mut success_count: Option<u64> = None;

    for (sign, term) in terms {
        let sign_mult: i64 = if *sign == Sign::Plus { 1 } else { -1 };
        match term {
            Term::Number(n) => {
                modifier_total = modifier_total
                    .checked_add((sign_mult as i128) * (*n as i128))
                    .ok_or_else(|| {
                        crate::core::SourceError::GenerationFailed(
                            "dice total overflow".into(),
                        )
                    })?;
            }
            Term::Dice(dice) => {
                let result = roll_die_term(source, dice)?;
                total = total
                    .checked_add((sign_mult as i128) * (result.total as i128))
                    .ok_or_else(|| {
                        crate::core::SourceError::GenerationFailed(
                            "dice total overflow".into(),
                        )
                    })?;
                all_rolls.extend(result.rolls);
                all_dropped.extend(result.dropped);
                if let Some(sc) = result.success_count {
                    success_count = Some(success_count.unwrap_or(0) + sc);
                }
            }
        }
    }

    total = total
        .checked_add(modifier_total)
        .ok_or_else(|| crate::core::SourceError::GenerationFailed("dice total overflow".into()))?;

    let total_i64 = i64::try_from(total)
        .map_err(|_| crate::core::SourceError::GenerationFailed("dice total out of i64 range".into()))?;
    let modifier_total_i64 = i64::try_from(modifier_total).map_err(|_| {
        crate::core::SourceError::GenerationFailed("dice total out of i64 range".into())
    })?;

    Ok(RollResult {
        total: total_i64,
        rolls: all_rolls,
        dropped: all_dropped,
        modifier_total: modifier_total_i64,
        success_count,
    })
}

fn roll_die_term(source: &mut dyn Source, dice: &DiceTerm) -> Result<RollResult, crate::core::SourceError> {
    // Guard against degenerate inputs that would overflow the accumulator or
    // spin for an unbounded number of rolls. This cap is enforced at the method
    // level so direct callers (not just the API, which caps per-field) are safe.
    if dice.count > 10_000 {
        return Err(crate::core::SourceError::GenerationFailed(
            "dice count per term exceeds 10000".to_string(),
        ));
    }
    let (size, fudge) = match dice.size {
        DieSize::Sides(n) => (
            i64::try_from(n).map_err(|_| {
                crate::core::SourceError::GenerationFailed("die size exceeds i64".into())
            })?,
            false,
        ),
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
    /// Regression: previously totals were accumulated in `i64` with plain `+=`,
    /// which overflowed (debug panic / release wrap) for large counts/sides, and
    /// `n as i64` silently truncated sides larger than `i64::MAX`.
    #[test]
    fn w6_count_cap_rejects_excess_dice() {
        let mut src = FixedSource { value: u64::MAX };
        // 100000 dice is rejected by the per-term count cap (10000) before any
        // rolling happens, instead of overflowing the accumulator.
        let err = roll_dice(&mut src, "100000d20").unwrap_err();
        let msg = format!("{err}");
        assert!(
            msg.contains("dice count per term exceeds 10000"),
            "unexpected error: {msg}"
        );
    }

    #[test]
    fn w6_count_at_cap_succeeds() {
        let mut src = FixedSource { value: u64::MAX };
        // Exactly the cap (10000) is allowed. With an always-max source every d6
        // rolls 6, so the total is exactly 60000, which fits i64 and i128.
        let result = roll_dice(&mut src, "10000d6").expect("10000d6 should succeed");
        assert_eq!(result.total, 60_000);
    }

    #[test]
    fn w6_die_size_exceeding_i64_is_rejected() {
        let mut src = FixedSource { value: u64::MAX };
        // A die with more sides than fit in i64 must error instead of silently
        // truncating via `n as i64`. Count is within the cap so the size check is
        // the failure point.
        let dice = DiceTerm {
            count: 1,
            size: DieSize::Sides(u64::MAX),
            modifiers: vec![],
        };
        let err = roll_die_term(&mut src, &dice).unwrap_err();
        let msg = format!("{err}");
        assert!(
            msg.contains("die size exceeds i64"),
            "unexpected error: {msg}"
        );
    }
}
