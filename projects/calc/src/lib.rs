//! A calculator for arithmetic expressions: `(1 + 2) * -3.5`.
//!
//! Two classic stages — tokenize, then parse — using a recursive
//! descent parser. The grammar encodes precedence structurally:
//!
//! ```text
//! expr   := term   (("+" | "-") term)*     lowest precedence
//! term   := factor (("*" | "/") factor)*
//! factor := NUMBER | "-" factor | "(" expr ")"
//! ```
//!
//! Concepts on display: enums for tokens and errors, Result + `?`
//! everywhere, iterators with peeking, exhaustive matching.

use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Number(f64),
    Plus,
    Minus,
    Star,
    Slash,
    LParen,
    RParen,
}

/// One variant per way the input can be wrong. Callers can match on
/// these; the CLI just Displays them.
#[derive(Debug, Clone, PartialEq)]
pub enum CalcError {
    UnexpectedChar(char),
    InvalidNumber(String),
    UnexpectedEnd,
    UnexpectedToken(Token),
    TrailingInput,
    DivisionByZero,
}

impl fmt::Display for CalcError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CalcError::UnexpectedChar(c) => write!(f, "unexpected character '{c}'"),
            CalcError::InvalidNumber(s) => write!(f, "invalid number '{s}'"),
            CalcError::UnexpectedEnd => write!(f, "expression ended unexpectedly"),
            CalcError::UnexpectedToken(t) => write!(f, "unexpected token {t:?}"),
            CalcError::TrailingInput => write!(f, "unexpected input after expression"),
            CalcError::DivisionByZero => write!(f, "division by zero"),
        }
    }
}

impl std::error::Error for CalcError {}

/// Stage 1: characters -> tokens. A peekable char iterator lets us
/// read multi-character numbers without an index variable.
pub fn tokenize(input: &str) -> Result<Vec<Token>, CalcError> {
    let mut tokens = Vec::new();
    let mut chars = input.chars().peekable();

    while let Some(&c) = chars.peek() {
        match c {
            ' ' | '\t' => {
                chars.next(); // skip whitespace
            }
            '+' => {
                chars.next();
                tokens.push(Token::Plus);
            }
            '-' => {
                chars.next();
                tokens.push(Token::Minus);
            }
            '*' => {
                chars.next();
                tokens.push(Token::Star);
            }
            '/' => {
                chars.next();
                tokens.push(Token::Slash);
            }
            '(' => {
                chars.next();
                tokens.push(Token::LParen);
            }
            ')' => {
                chars.next();
                tokens.push(Token::RParen);
            }
            '0'..='9' | '.' => {
                // Greedily consume the whole number, then let the std
                // parser validate it — no point re-implementing float
                // parsing (it would get "1.2.3" wrong anyway).
                let mut text = String::new();
                while let Some(&c) = chars.peek() {
                    if c.is_ascii_digit() || c == '.' {
                        text.push(c);
                        chars.next();
                    } else {
                        break;
                    }
                }
                let value = text
                    .parse()
                    .map_err(|_| CalcError::InvalidNumber(text.clone()))?;
                tokens.push(Token::Number(value));
            }
            other => return Err(CalcError::UnexpectedChar(other)),
        }
    }
    Ok(tokens)
}

/// Stage 2: tokens -> value. Each grammar rule is one method; the
/// call stack mirrors the precedence hierarchy. We evaluate as we
/// parse — a full AST would be the next step (see exercises).
struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    fn next(&mut self) -> Result<Token, CalcError> {
        let token = self
            .tokens
            .get(self.pos)
            .cloned()
            .ok_or(CalcError::UnexpectedEnd)?;
        self.pos += 1;
        Ok(token)
    }

    /// expr := term (("+" | "-") term)*
    fn expr(&mut self) -> Result<f64, CalcError> {
        let mut value = self.term()?;
        // The loop makes +/- left-associative: 1-2-3 = (1-2)-3.
        while let Some(op) = self.peek() {
            match op {
                Token::Plus => {
                    self.pos += 1;
                    value += self.term()?;
                }
                Token::Minus => {
                    self.pos += 1;
                    value -= self.term()?;
                }
                _ => break, // not ours — let the caller look at it
            }
        }
        Ok(value)
    }

    /// term := factor (("*" | "/") factor)*
    fn term(&mut self) -> Result<f64, CalcError> {
        let mut value = self.factor()?;
        while let Some(op) = self.peek() {
            match op {
                Token::Star => {
                    self.pos += 1;
                    value *= self.factor()?;
                }
                Token::Slash => {
                    self.pos += 1;
                    let divisor = self.factor()?;
                    if divisor == 0.0 {
                        return Err(CalcError::DivisionByZero);
                    }
                    value /= divisor;
                }
                _ => break,
            }
        }
        Ok(value)
    }

    /// factor := NUMBER | "-" factor | "(" expr ")"
    fn factor(&mut self) -> Result<f64, CalcError> {
        match self.next()? {
            Token::Number(n) => Ok(n),
            // Unary minus: recursion handles --5 naturally.
            Token::Minus => Ok(-self.factor()?),
            Token::LParen => {
                let value = self.expr()?; // full expression inside parens
                match self.next()? {
                    Token::RParen => Ok(value),
                    other => Err(CalcError::UnexpectedToken(other)),
                }
            }
            other => Err(CalcError::UnexpectedToken(other)),
        }
    }
}

/// The public entry point: the only function callers need.
pub fn evaluate(input: &str) -> Result<f64, CalcError> {
    let tokens = tokenize(input)?;
    let mut parser = Parser { tokens, pos: 0 };
    let value = parser.expr()?;
    // "1 2" parses "1" fine and stops — without this check the junk
    // would be silently ignored.
    if parser.peek().is_some() {
        return Err(CalcError::TrailingInput);
    }
    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_numbers_and_basic_ops() {
        assert_eq!(evaluate("42"), Ok(42.0));
        assert_eq!(evaluate("1 + 2"), Ok(3.0));
        assert_eq!(evaluate("7 - 10"), Ok(-3.0));
        assert_eq!(evaluate("6 * 7"), Ok(42.0));
        assert_eq!(evaluate("9 / 2"), Ok(4.5));
    }

    #[test]
    fn precedence_and_associativity() {
        assert_eq!(evaluate("2 + 3 * 4"), Ok(14.0)); // not 20
        assert_eq!(evaluate("10 - 2 - 3"), Ok(5.0)); // left assoc, not 11
        assert_eq!(evaluate("100 / 10 / 2"), Ok(5.0)); // left assoc, not 20
    }

    #[test]
    fn parentheses_override_precedence() {
        assert_eq!(evaluate("(2 + 3) * 4"), Ok(20.0));
        assert_eq!(evaluate("((1))"), Ok(1.0));
    }

    #[test]
    fn unary_minus() {
        assert_eq!(evaluate("-5"), Ok(-5.0));
        assert_eq!(evaluate("--5"), Ok(5.0));
        assert_eq!(evaluate("2 * -3"), Ok(-6.0));
        assert_eq!(evaluate("-(1 + 2)"), Ok(-3.0));
    }

    #[test]
    fn floats() {
        assert_eq!(evaluate("1.5 + 2.25"), Ok(3.75));
        assert_eq!(evaluate(".5 * 4"), Ok(2.0));
    }

    #[test]
    fn every_error_variant_is_reachable() {
        assert_eq!(evaluate("2 ^ 3"), Err(CalcError::UnexpectedChar('^')));
        assert_eq!(
            evaluate("1.2.3"),
            Err(CalcError::InvalidNumber("1.2.3".into()))
        );
        assert_eq!(evaluate("1 +"), Err(CalcError::UnexpectedEnd));
        assert_eq!(evaluate("1 / 0"), Err(CalcError::DivisionByZero));
        assert_eq!(evaluate("1 2"), Err(CalcError::TrailingInput));
        assert_eq!(
            evaluate("(1 + 2"),
            Err(CalcError::UnexpectedEnd) // closing paren never arrives
        );
        assert!(matches!(
            evaluate("*3"),
            Err(CalcError::UnexpectedToken(Token::Star))
        ));
    }

    #[test]
    fn empty_input_is_an_error() {
        assert_eq!(evaluate(""), Err(CalcError::UnexpectedEnd));
        assert_eq!(evaluate("   "), Err(CalcError::UnexpectedEnd));
    }
}

// Exercises
// ---------
// 1. Add a `%` (remainder) operator at the same precedence as `*`.
// 2. Add a `^` (power) operator — note it should be RIGHT-associative
//    (2^3^2 = 512), which needs a different loop shape than expr/term.
// 3. Split parsing from evaluation: make the parser return an AST enum
//    (Expr::Number / Expr::BinaryOp / Expr::Negate) and write a
//    separate `eval(&Expr) -> Result<f64, CalcError>`.
