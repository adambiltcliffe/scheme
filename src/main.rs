use std::{
    ops::Deref,
    rc::{Rc, Weak},
};

#[derive(Debug)]
enum SError {
    ImproperList,
}

type SResult<T> = Result<T, SError>;

type Cell = (Expr, Expr);

#[derive(Debug, Clone)]
struct CellHandle(Weak<Cell>);

impl CellHandle {
    fn get(&self) -> Rc<Cell> {
        self.0.upgrade().expect("internal GC error")
    }
}

#[derive(Debug, Clone)]
enum Expr {
    Nil,
    Integer(u64),
    Symbol(Rc<str>),
    Pair(CellHandle),
}

impl Expr {
    fn is_nil(&self) -> bool {
        matches!(self, Self::Nil)
    }

    fn is_pair(&self) -> bool {
        matches!(self, Self::Pair(_))
    }

    fn first(&self) -> SResult<Expr> {
        match self {
            // mostly this will just be a copy; only have to
            // clone here in case it's another Pair
            Expr::Pair(h) => Ok((*h.get()).0.clone()),
            _ => Err(SError::ImproperList),
        }
    }

    fn rest(&self) -> SResult<Expr> {
        match self {
            Expr::Pair(h) => Ok((*h.get()).1.clone()),
            _ => Err(SError::ImproperList),
        }
    }

    fn first_rest(&self) -> SResult<(Expr, Expr)> {
        match self {
            Expr::Pair(h) => {
                let pair = (*h.get()).clone();
                Ok((pair.0, pair.1))
            }
            _ => Err(SError::ImproperList),
        }
    }
}

impl PartialEq for Expr {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Nil, Self::Nil) => true,
            (Self::Integer(a), Self::Integer(b)) => a == b,
            (Self::Symbol(a), Self::Symbol(b)) => Rc::ptr_eq(&a, &b),
            (Self::Pair(a), Self::Pair(b)) => a.0.ptr_eq(&b.0),
            _ => false,
        }
    }
}

struct Heap {
    symbols: Expr,
    cells: Vec<Rc<(Expr, Expr)>>,
}

impl Heap {
    fn new() -> Self {
        Self {
            symbols: Expr::Nil,
            cells: Vec::new(),
        }
    }

    fn make_cons(&mut self, first: Expr, rest: Expr) -> SResult<Expr> {
        let cell = Rc::new((first, rest));
        let handle = CellHandle(Rc::downgrade(&cell));
        self.cells.push(cell);
        Ok(Expr::Pair(handle))
    }

    fn make_symbol(&mut self, name: &str) -> SResult<Expr> {
        let mut s = self.symbols.clone();
        while !s.is_nil() {
            if s.is_pair() {
                let (first, rest) = s.first_rest()?;
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

    fn format_expr_inner(&self, expr: &Expr, acc: &mut String) {
        match expr {
            Expr::Nil => acc.push_str("nil"),
            Expr::Integer(n) => acc.push_str(&n.to_string()),
            Expr::Symbol(s) => acc.push_str(s),
            Expr::Pair(h) => {
                acc.push_str("(");
                let (mut first, mut rest) = (*h.get()).clone();
                loop {
                    self.format_expr_inner(&first, acc);
                    match rest {
                        Expr::Nil => break,
                        Expr::Pair(h) => {
                            acc.push_str(" ");
                            (first, rest) = (*h.get()).clone()
                        }
                        _ => {
                            acc.push_str(" . ");
                            self.format_expr_inner(&rest, acc);
                            break;
                        }
                    }
                }
                acc.push_str(")");
            }
        }
    }

    fn format_expr(&self, expr: &Expr) -> String {
        let mut acc = String::new();
        self.format_expr_inner(expr, &mut acc);
        acc
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
    println!("{}", heap.format_expr(&data2));
    println!("{}", heap.format_expr(&heap.symbols));
}
