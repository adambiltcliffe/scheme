use std::{
    ops::Deref,
    rc::{Rc, Weak},
};

type Cell = (Expr, Expr);

#[derive(Debug)]
struct CellHandle(Weak<Cell>);

impl CellHandle {
    fn get(&self) -> Rc<Cell> {
        self.0.upgrade().expect("internal GC error")
    }
}

#[derive(Debug)]
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
    symbols: Vec<Rc<str>>,
    cells: Vec<Rc<Expr>>,
}

impl Heap {
    fn new() -> Self {
        Self {
            symbols: Vec::new(),
            cells: Vec::new(),
        }
    }
}

fn make_symbol(name: &str, heap: &mut Heap) -> Expr {
    match heap.symbols.iter().find(|s| Rc::deref(*s) == name) {
        Some(s) => Expr::Symbol(Rc::clone(s)),
        None => {
            let new_symbol: Rc<str> = Rc::from(name);
            heap.symbols.push(Rc::clone(&new_symbol));
            Expr::Symbol(new_symbol)
        }
    }
}

fn main() {
    let mut heap = Heap::new();
    let sym1 = make_symbol("banana", &mut heap);
    let sym2 = make_symbol("apple", &mut heap);
    let sym3 = make_symbol("apple", &mut heap);
    println!("{:?} {:?} {:?}", sym1, sym2, sym3);
    let sym4 = Expr::Symbol(Rc::from("apple"));
    println!("{} {}", &sym2 == &sym3, &sym2 == &sym4);
}
