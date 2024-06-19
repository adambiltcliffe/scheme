use std::{
    ops::Deref,
    rc::{Rc, Weak},
};

use slab::Slab;

#[derive(Debug)]
enum SError {
    ImproperList,
}

type SResult<T> = Result<T, SError>;

type ConsCell = (Expr, Expr);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ConsCellKey(usize);

#[derive(Debug, Clone, PartialEq, Eq)]
enum Expr {
    Nil,
    Integer(u64),
    Symbol(Rc<str>),
    Pair(ConsCellKey),
}

impl Expr {
    fn is_nil(&self) -> bool {
        matches!(self, Self::Nil)
    }

    fn is_pair(&self) -> bool {
        matches!(self, Self::Pair(_))
    }
}

struct Heap {
    symbols: Expr,
    cells: Slab<ConsCell>,
}

impl Heap {
    fn new() -> Self {
        Self {
            symbols: Expr::Nil,
            cells: Slab::new(),
        }
    }

    fn get_first_rest_by_key(&self, n: ConsCellKey) -> SResult<(Expr, Expr)> {
        return Ok(self.cells.get(n.0).unwrap().clone());
    }

    fn get_first_rest(&self, expr: &Expr) -> SResult<(Expr, Expr)> {
        if let Expr::Pair(k) = expr {
            return self.get_first_rest_by_key(*k);
        }
        return Err(SError::ImproperList);
    }

    fn make_cons(&mut self, first: Expr, rest: Expr) -> SResult<Expr> {
        let key = ConsCellKey(self.cells.insert((first, rest)));
        Ok(Expr::Pair(key))
    }

    fn make_symbol(&mut self, name: &str) -> SResult<Expr> {
        let mut s = self.symbols.clone();
        while !s.is_nil() {
            if s.is_pair() {
                let (first, rest) = self.get_first_rest(&s)?;
                if let Expr::Symbol(r) = first {
                    if Rc::deref(&r) == name {
                        return Ok(Expr::Symbol(Rc::clone(&r)));
                    }
                }
                s = rest;
            } else {
                return Err(SError::ImproperList);
            }
        }
        drop(s);
        let new_symbol: Rc<str> = Rc::from(name);
        self.symbols =
            self.make_cons(Expr::Symbol(Rc::clone(&new_symbol)), self.symbols.clone())?;
        Ok(Expr::Symbol(new_symbol))
    }

    fn make_env(&mut self, parent: Expr) -> SResult<Expr> {
        self.make_cons(parent.clone(), Expr::Nil)
    }

    fn format_expr_inner(&self, expr: &Expr, acc: &mut String) -> SResult<()> {
        Ok(match expr {
            Expr::Nil => acc.push_str("nil"),
            Expr::Integer(n) => acc.push_str(&n.to_string()),
            Expr::Symbol(s) => acc.push_str(s),
            Expr::Pair(k) => {
                acc.push_str("(");
                let (mut first, mut rest) = self.get_first_rest_by_key(*k)?;
                loop {
                    self.format_expr_inner(&first, acc)?;
                    match rest {
                        Expr::Nil => break,
                        Expr::Pair(k) => {
                            acc.push_str(" ");
                            (first, rest) = self.get_first_rest_by_key(k)?;
                        }
                        _ => {
                            acc.push_str(" . ");
                            self.format_expr_inner(&rest, acc)?;
                            break;
                        }
                    }
                }
                acc.push_str(")");
            }
        })
    }

    fn format_expr(&self, expr: &Expr) -> SResult<String> {
        let mut acc = String::new();
        self.format_expr_inner(expr, &mut acc)?;
        Ok(acc)
    }
}

fn main() {
    let mut heap = Heap::new();
    let sym1 = heap.make_symbol("banana").unwrap();
    let sym2 = heap.make_symbol("apple").unwrap();
    let sym5 = heap.make_symbol("orange").unwrap();
    let sym3 = heap.make_symbol("apple").unwrap();
    println!("{:?} {:?} {:?}", sym1, sym2, sym3);
    let sym4 = Expr::Symbol(Rc::from("apple"));
    println!("{} {} -- {}", sym2 == sym3, sym2 == sym4, heap.cells.len());
    let data3 = heap.make_cons(Expr::Integer(3), Expr::Nil).unwrap();
    let data1 = heap.make_cons(Expr::Integer(4), data3).unwrap();
    let data2 = heap.make_cons(Expr::Integer(5), data1).unwrap();
    println!("{}", heap.format_expr(&data2).unwrap());
    println!("{}", heap.format_expr(&heap.symbols).unwrap());
}
