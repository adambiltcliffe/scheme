use crate::lexer::Token;
use crate::{Expr, Heap};
use std::iter::Peekable;

#[derive(Debug)]
pub enum ParseError {
    AmbiguousValue,
    UnexpectedDot,
    UnexpectedEndOfInput,
    UnmatchedBracket,
}

pub(crate) fn parse_expr(
    input: &mut Peekable<impl Iterator<Item = Token>>,
    heap: &mut Heap,
) -> Result<Expr, ParseError> {
    match input.next() {
        None => Err(ParseError::UnexpectedEndOfInput),
        Some(Token::Value(v)) => parse_value(&v, heap),
        Some(Token::Dot) => Err(ParseError::UnexpectedDot),
        Some(Token::LBracket) => {
            if let Some(Token::RBracket) = input.peek() {
                input.next().unwrap();
                return Ok(Expr::Nil);
            }
            let first = parse_expr(input, heap)?;
            let result = heap.make_cons(first, Expr::Nil).unwrap();
            let mut result_tail = result.clone();
            loop {
                let mut has_dot = false;
                if let Some(Token::RBracket) = input.peek() {
                    input.next().unwrap();
                    return Ok(result);
                }
                if let Some(Token::Dot) = input.peek() {
                    input.next().unwrap();
                    has_dot = true;
                }
                let next = parse_expr(input, heap)?;
                if has_dot {
                    heap.set_rest(&result_tail, next).unwrap();
                    if let Some(Token::RBracket) = input.peek() {
                        input.next().unwrap();
                        return Ok(result);
                    } else {
                        return Err(ParseError::UnexpectedDot);
                    }
                }
                let new_tail = heap.make_cons(next, Expr::Nil).unwrap();
                heap.set_rest(&result_tail, new_tail.clone()).unwrap();
                result_tail = new_tail;
            }
        }
        Some(Token::RBracket) => Err(ParseError::UnmatchedBracket),
    }
}

fn parse_value(v: &str, heap: &mut Heap) -> Result<Expr, ParseError> {
    if v.starts_with('#') {
        match v {
            "#f" => return Ok(Expr::Boolean(false)),
            "#t" => return Ok(Expr::Boolean(true)),
            _ => return Err(ParseError::AmbiguousValue),
        }
    }
    if v.starts_with(|c: char| c.is_ascii_digit() || c == '-') {
        match v.parse::<i64>() {
            Ok(n) => return Ok(Expr::Integer(n)),
            Err(_) => {
                if v != "-" {
                    // "-" alone is the symbol bound to the subtraction primitive
                    return Err(ParseError::AmbiguousValue);
                }
            }
        }
    }
    Ok(heap.make_symbol(v).unwrap())
}
