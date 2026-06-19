use crate::methods::dice::ast::*;
use thiserror::Error;

#[derive(Debug, Error, PartialEq)]
pub enum ParseError {
    #[error("empty expression")]
    Empty,
    #[error("unexpected character '{0}' at position {1}")]
    UnexpectedChar(char, usize),
    #[error("expected {0} at position {1}")]
    Expected(&'static str, usize),
    #[error("invalid number at position {0}")]
    InvalidNumber(usize),
    #[error("invalid dice expression: {0}")]
    Invalid(&'static str),
}

pub fn parse(input: &str) -> Result<Expr, ParseError> {
    let normalized = normalize(input);
    let mut p = Parser::new(&normalized);
    let expr = p.parse_expr()?;
    if !p.is_at_end() {
        return Err(ParseError::UnexpectedChar(p.peek(), p.pos));
    }
    Ok(expr)
}

/// Normalize input: lowercase, strip spaces, expand shorthand.
fn normalize(input: &str) -> String {
    let s = input.to_lowercase().replace(' ', "");
    // Expand advantage/disadvantage shorthand.
    s.replace("disadvantage", "kl1")
        .replace("advantage", "kh1")
        .replace("adv", "kh1")
        .replace("dis", "kl1")
}

struct Parser<'a> {
    input: &'a str,
    pos: usize,
}

impl<'a> Parser<'a> {
    fn new(input: &'a str) -> Self {
        Self { input, pos: 0 }
    }

    fn is_at_end(&self) -> bool {
        self.pos >= self.input.len()
    }

    fn peek(&self) -> char {
        self.input[self.pos..].chars().next().unwrap_or('\0')
    }

    fn advance(&mut self) -> char {
        let c = self.peek();
        self.pos += c.len_utf8();
        c
    }

    fn parse_expr(&mut self) -> Result<Expr, ParseError> {
        let mut terms = Vec::new();
        let (sign, term) = self.parse_signed_term()?;
        terms.push((sign, term));

        while self.peek() == '+' || self.peek() == '-' {
            let sign = if self.advance() == '+' { Sign::Plus } else { Sign::Minus };
            let term = self.parse_term()?;
            terms.push((sign, term));
        }

        Ok(Expr::Sum(terms))
    }

    fn parse_signed_term(&mut self) -> Result<(Sign, Term), ParseError> {
        let sign = if self.peek() == '-' {
            self.advance();
            Sign::Minus
        } else {
            if self.peek() == '+' {
                self.advance();
            }
            Sign::Plus
        };
        let term = self.parse_term()?;
        Ok((sign, term))
    }

    fn parse_term(&mut self) -> Result<Term, ParseError> {
        // Check for die term starting with 'd' (count defaults to 1).
        if self.peek() == 'd' {
            return self.parse_dice_term_with_count(1);
        }

        let start = self.pos;
        let number = self.parse_number()?;

        if self.peek() == 'd' {
            self.advance();
            return self.parse_die_rest(number);
        }

        // Plain number. Reject values that exceed i64 instead of silently
        // flipping sign via `number as i64`.
        Ok(Term::Number(
            i64::try_from(number).map_err(|_| ParseError::InvalidNumber(start))?,
        ))
    }

    fn parse_dice_term_with_count(&mut self, count: u64) -> Result<Term, ParseError> {
        self.advance(); // consume 'd'
        self.parse_die_rest(count)
    }

    fn parse_die_rest(&mut self, count: u64) -> Result<Term, ParseError> {
        let size = self.parse_die_size()?;
        let modifiers = self.parse_modifiers()?;
        Ok(Term::Dice(DiceTerm {
            count,
            size,
            modifiers,
        }))
    }

    fn parse_die_size(&mut self) -> Result<DieSize, ParseError> {
        if self.peek() == '%' {
            self.advance();
            return Ok(DieSize::Percentile);
        }
        if self.peek() == 'f' {
            self.advance();
            return Ok(DieSize::Fudge);
        }
        let n = self.parse_number()?;
        if n == 0 {
            return Err(ParseError::Invalid("die must have at least one side"));
        }
        Ok(DieSize::Sides(n))
    }

    fn parse_modifiers(&mut self) -> Result<Vec<Modifier>, ParseError> {
        let mut modifiers = Vec::new();
        while !self.is_at_end() && self.peek() != '+' && self.peek() != '-' {
            let start = self.pos;
            match self.advance() {
                '!' => {
                    if self.peek() == '!' {
                        self.advance();
                        modifiers.push(Modifier::ExplodeCompound);
                    } else {
                        modifiers.push(Modifier::Explode);
                    }
                }
                'k' => {
                    let (lowest, n) = self.parse_keep_drop()?;
                    if lowest {
                        modifiers.push(Modifier::KeepLowest(n));
                    } else {
                        modifiers.push(Modifier::KeepHighest(n));
                    }
                }
                'd' => {
                    let (lowest, n) = self.parse_keep_drop()?;
                    if lowest {
                        modifiers.push(Modifier::DropLowest(n));
                    } else {
                        modifiers.push(Modifier::DropHighest(n));
                    }
                }
                'r' => {
                    let once = if self.peek() == 'o' {
                        self.advance();
                        true
                    } else {
                        false
                    };
                    let (cmp, value) = self.parse_comparator_value()?;
                    modifiers.push(Modifier::Reroll {
                        comparator: cmp,
                        value,
                        once,
                    });
                }
                '>' | '<' | '=' => {
                    self.pos = start; // backtrack to parse comparator fully
                    let (cmp, value) = self.parse_comparator_value()?;
                    modifiers.push(Modifier::Success { comparator: cmp, value });
                }
                c => return Err(ParseError::UnexpectedChar(c, start)),
            }
        }
        Ok(modifiers)
    }

    fn parse_keep_drop(&mut self) -> Result<(bool, u64), ParseError> {
        let lowest = if self.peek() == 'l' {
            self.advance();
            true
        } else if self.peek() == 'h' {
            self.advance();
            false
        } else {
            return Err(ParseError::Expected("'h' or 'l'", self.pos));
        };
        let n = self.parse_number()?;
        Ok((lowest, n))
    }

    fn parse_comparator_value(&mut self) -> Result<(Comparator, i64), ParseError> {
        let cmp = match self.advance() {
            '>' => {
                if self.peek() == '=' {
                    self.advance();
                    Comparator::Ge
                } else {
                    Comparator::Gt
                }
            }
            '<' => {
                if self.peek() == '=' {
                    self.advance();
                    Comparator::Le
                } else {
                    Comparator::Lt
                }
            }
            '!' => {
                if self.peek() == '=' {
                    self.advance();
                    Comparator::Ne
                } else {
                    return Err(ParseError::UnexpectedChar('!', self.pos - 1));
                }
            }
            '=' => Comparator::Eq,
            c => return Err(ParseError::UnexpectedChar(c, self.pos - 1)),
        };
        let num_start = self.pos;
        let value =
            i64::try_from(self.parse_number()?).map_err(|_| ParseError::InvalidNumber(num_start))?;
        Ok((cmp, value))
    }

    fn parse_number(&mut self) -> Result<u64, ParseError> {
        let start = self.pos;
        let mut value: u64 = 0;
        let mut has_digit = false;
        while self.peek().is_ascii_digit() {
            has_digit = true;
            let d = self.advance() as u64 - '0' as u64;
            value = value
                .checked_mul(10)
                .and_then(|v| v.checked_add(d))
                .ok_or(ParseError::InvalidNumber(start))?;
        }
        if !has_digit {
            return Err(ParseError::InvalidNumber(start));
        }
        Ok(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
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
    fn test_with_modifiers() {
        let expr = parse("4d6kh3").unwrap();
        assert_eq!(
            expr,
            Expr::Sum(vec![(
                Sign::Plus,
                Term::Dice(DiceTerm {
                    count: 4,
                    size: DieSize::Sides(6),
                    modifiers: vec![Modifier::KeepHighest(3)],
                })
            )])
        );
    }

    #[test]
    fn test_advantage() {
        let expr = parse("2d20adv").unwrap();
        match expr {
            Expr::Sum(v) => match &v[0].1 {
                Term::Dice(d) => assert_eq!(d.modifiers, vec![Modifier::KeepHighest(1)]),
                _ => panic!("expected dice"),
            },
        }
    }

    /// Regression: `normalize()` previously expanded the short tokens `adv`/
    /// `dis` *before* the long forms `advantage`/`disadvantage`, so `2d20advantage`
    /// was mangled into `2d20kh1antage` and failed to parse.
    #[test]
    fn n3_advantage_long_form_parses() {
        let expr = parse("2d20advantage").expect("2d20advantage should parse");
        match expr {
            Expr::Sum(v) => match &v[0].1 {
                Term::Dice(d) => assert_eq!(d.modifiers, vec![Modifier::KeepHighest(1)]),
                _ => panic!("expected dice"),
            },
        }
    }

    #[test]
    fn n3_disadvantage_long_form_parses() {
        let expr = parse("2d20disadvantage").expect("2d20disadvantage should parse");
        match expr {
            Expr::Sum(v) => match &v[0].1 {
                Term::Dice(d) => assert_eq!(d.modifiers, vec![Modifier::KeepLowest(1)]),
                _ => panic!("expected dice"),
            },
        }
    }

    /// Short forms still work after the reorder.
    #[test]
    fn n3_advantage_short_form_still_parses() {
        let expr = parse("2d20dis").unwrap();
        match expr {
            Expr::Sum(v) => match &v[0].1 {
                Term::Dice(d) => assert_eq!(d.modifiers, vec![Modifier::KeepLowest(1)]),
                _ => panic!("expected dice"),
            },
        }
    }

    /// Regression: bare numbers were coerced via `number as i64`, which silently
    /// wrapped values above i64::MAX (e.g. i64::MAX + 1 became a negative). They
    /// must now be rejected.
    #[test]
    fn w6_bare_number_above_i64_max_is_rejected() {
        let res = parse("+9223372036854775808"); // i64::MAX + 1
        assert!(
            matches!(res, Err(ParseError::InvalidNumber(_))),
            "expected InvalidNumber, got {res:?}"
        );
    }

    /// Sanity: the largest legitimate bare number still parses.
    #[test]
    fn w6_bare_number_at_i64_max_parses() {
        let expr = parse("9223372036854775807").unwrap(); // i64::MAX
        assert_eq!(expr, Expr::Sum(vec![(Sign::Plus, Term::Number(i64::MAX))]));
    }

    /// Regression: a comparator value above i64::MAX must be rejected too.
    #[test]
    fn w6_comparator_value_above_i64_max_is_rejected() {
        // `1d20>9223372036854775808` -> success target exceeds i64::MAX.
        let res = parse("1d20>9223372036854775808");
        assert!(
            matches!(res, Err(ParseError::InvalidNumber(_))),
            "expected InvalidNumber, got {res:?}"
        );
    }
}
