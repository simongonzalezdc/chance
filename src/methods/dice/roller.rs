use crate::core::source::Source;
use crate::core::range::{uniform_i64_inclusive, uniform_u64_inclusive};
use crate::methods::dice::ast::*;
use crate::methods::dice::parser::parse;

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
                    loop {
                        let last = rolls.last().unwrap();
                        if last.value != explode_max || last.rerolled {
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
                    }
                }
                Modifier::ExplodeCompound => {
                    loop {
                        let last = rolls.last().unwrap();
                        if last.value != explode_max || last.rerolled {
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
