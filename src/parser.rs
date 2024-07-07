use crate::lexer::Token;
use crate::{Expr, Heap, SError, SResult};
use std::iter::Peekable;

pub(crate) fn parse_expr(
    input: &mut Peekable<impl Iterator<Item = Token>>,
    heap: &mut Heap,
) -> Option<SResult<Expr>> {
    match input.next() {
        None => None,
        Some(Token::Value(v)) => Some(parse_value(&v, heap)),
        Some(Token::Dot) => Some(Err(SError::UnexpectedDot)),
        Some(Token::LBracket) => {
            if let Some(Token::RBracket) = input.peek() {
                input.next().unwrap();
                return Some(Ok(Expr::Nil));
            }
            let first = match parse_expr(input, heap) {
                Some(Ok(expr)) => expr,
                Some(e @ Err(_)) => return Some(e),
                None => return Some(Err(SError::UnexpectedEndOfInput)),
            };
            let result = heap.make_cons(first, Expr::Nil).unwrap();
            let mut result_tail = result.clone();
            loop {
                let mut has_dot = false;
                if let Some(Token::RBracket) = input.peek() {
                    input.next().unwrap();
                    return Some(Ok(result));
                }
                if let Some(Token::Dot) = input.peek() {
                    input.next().unwrap();
                    has_dot = true;
                }
                let next = match parse_expr(input, heap) {
                    Some(Ok(expr)) => expr,
                    Some(e @ Err(_)) => return Some(e),
                    None => return Some(Err(SError::UnexpectedEndOfInput)),
                };
                if has_dot {
                    heap.set_rest(&result_tail, next).unwrap();
                    if let Some(Token::RBracket) = input.peek() {
                        input.next().unwrap();
                        return Some(Ok(result));
                    } else {
                        return Some(Err(SError::UnexpectedDot));
                    }
                }
                let new_tail = heap.make_cons(next, Expr::Nil).unwrap();
                heap.set_rest(&result_tail, new_tail.clone()).unwrap();
                result_tail = new_tail;
            }
        }
        Some(Token::RBracket) => Some(Err(SError::UnmatchedBracket)),
    }
}

fn parse_value(v: &str, heap: &mut Heap) -> SResult<Expr> {
    if v.starts_with(|c: char| c.is_ascii_digit()) {
        match v.parse::<u64>() {
            Ok(n) => return Ok(Expr::Integer(n)),
            Err(_) => return Err(SError::AmbiguousValue),
        }
    }
    heap.make_symbol(v)
}
