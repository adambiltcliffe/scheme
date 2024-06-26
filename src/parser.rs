use crate::lexer::Token;
use crate::{Expr, Heap, SError, SResult};
use std::iter::Peekable;

pub(crate) fn parse_expr(
    input: &mut Peekable<impl Iterator<Item = Token>>,
    heap: &mut Heap,
) -> Option<SResult<Expr>> {
    match input.next() {
        None => return None,
        Some(Token::Value(v)) => return Some(parse_value(&v, heap)),
        Some(Token::Dot) => return Some(Err(SError::UnexpectedDot)),
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
            let mut tail_key = result.key().unwrap();
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
                    heap.set_rest_by_key(tail_key, next).unwrap();
                    if let Some(Token::RBracket) = input.peek() {
                        input.next().unwrap();
                        return Some(Ok(result));
                    } else {
                        return Some(Err(SError::UnexpectedDot));
                    }
                }
                let new_tail = heap.make_cons(next, Expr::Nil).unwrap();
                let new_tail_key = new_tail.key().unwrap();
                heap.set_rest_by_key(tail_key, new_tail).unwrap();
                tail_key = new_tail_key;
            }
        }
        Some(Token::RBracket) => return Some(Err(SError::UnmatchedBracket)),
    }
}

fn parse_value(v: &str, heap: &mut Heap) -> SResult<Expr> {
    if v.starts_with(|c| c >= '0' && c <= '9') {
        match v.parse::<u64>() {
            Ok(n) => return Ok(Expr::Integer(n)),
            Err(_) => return Err(SError::AmbiguousValue),
        }
    }
    return heap.make_symbol(v);
}
