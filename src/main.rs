use std::{io::BufRead, ops::Deref, rc::Rc};

use lexer::tokenize;
use parser::parse_expr;
use slab::Slab;

mod lexer;
mod parser;

#[derive(Debug)]
enum SError {
    AmbiguousValue,
    ImproperList,
    ImproperSymbol,
    ImproperEnvironment,
    UnboundSymbol,
    UnexpectedDot,
    UnexpectedEndOfInput,
    UnknownForm,
    UnmatchedBracket,
    WrongNumberOfArgs,
}

type SResult<T> = Result<T, SError>;

type ConsCell = (Expr, Expr, bool);

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

    fn is_symbol(&self) -> bool {
        matches!(self, Self::Symbol(_))
    }

    fn is_specific_symbol(&self, s: &str) -> bool {
        if let Expr::Symbol(sym) = self {
            return sym.as_ref() == s;
        }
        false
    }

    fn key(&self) -> Option<ConsCellKey> {
        if let Expr::Pair(k) = self {
            return Some(*k);
        }
        None
    }
}

struct Heap {
    symbols: Expr,
    root_env: Expr,
    cells: Slab<ConsCell>,
}

impl Heap {
    fn new() -> Self {
        let mut me = Self {
            symbols: Expr::Nil,
            root_env: Expr::Nil,
            cells: Slab::new(),
        };
        let env = me.make_env(&Expr::Nil).unwrap();
        me.root_env = env;
        me
    }

    fn get_first_rest_by_key(&self, n: ConsCellKey) -> SResult<(Expr, Expr)> {
        let cell = self.cells.get(n.0).unwrap().clone();
        return Ok((cell.0, cell.1));
    }

    fn get_first_by_key(&self, n: ConsCellKey) -> SResult<(Expr)> {
        return Ok(self.cells.get(n.0).unwrap().0.clone());
    }

    fn get_rest_by_key(&self, n: ConsCellKey) -> SResult<(Expr)> {
        return Ok(self.cells.get(n.0).unwrap().1.clone());
    }

    fn set_rest_by_key(&mut self, n: ConsCellKey, v: Expr) -> SResult<()> {
        self.cells.get_mut(n.0).unwrap().1 = v;
        return Ok(());
    }

    fn get_first_rest(&self, expr: &Expr) -> SResult<(Expr, Expr)> {
        if let Expr::Pair(k) = expr {
            return self.get_first_rest_by_key(*k);
        }
        return Err(SError::ImproperList);
    }

    fn get_first(&self, expr: &Expr) -> SResult<Expr> {
        if let Expr::Pair(k) = expr {
            return self.get_first_by_key(*k);
        }
        return Err(SError::ImproperList);
    }

    fn get_rest(&self, expr: &Expr) -> SResult<Expr> {
        if let Expr::Pair(k) = expr {
            return self.get_rest_by_key(*k);
        }
        return Err(SError::ImproperList);
    }

    fn set_rest(&mut self, expr: &Expr, v: Expr) -> SResult<()> {
        if let Expr::Pair(k) = expr {
            return self.set_rest_by_key(*k, v);
        }
        return Err(SError::ImproperList);
    }

    fn make_cons(&mut self, first: Expr, rest: Expr) -> SResult<Expr> {
        let key = ConsCellKey(self.cells.insert((first, rest, false)));
        Ok(Expr::Pair(key))
    }

    fn is_proper_list(&self, expr: &Expr) -> SResult<bool> {
        match expr {
            Expr::Nil => Ok(true),
            Expr::Pair(k) => {
                let rest = self.get_rest_by_key(*k)?;
                self.is_proper_list(&rest)
            }
            _ => Ok(false),
        }
    }

    fn test_length(&self, expr: &Expr, n: usize) -> SResult<bool> {
        match expr {
            Expr::Nil => Ok(n == 0),
            Expr::Pair(k) => {
                let rest = self.get_rest_by_key(*k)?;
                self.test_length(&rest, n - 1)
            }
            _ => Ok(false),
        }
    }

    fn make_symbol(&mut self, name: &str) -> SResult<Expr> {
        let name = name.to_ascii_uppercase();
        let mut s = self.symbols.clone();
        while !s.is_nil() {
            if s.is_pair() {
                let (first, rest) = self.get_first_rest(&s)?;
                if let Expr::Symbol(r) = first {
                    if Rc::deref(&r).eq(&name) {
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

    fn make_env(&mut self, parent: &Expr) -> SResult<Expr> {
        self.make_cons(parent.clone(), Expr::Nil)
    }

    fn env_get(&self, env: &Expr, name: &Expr) -> SResult<Expr> {
        if !env.is_pair() {
            return Err(SError::ImproperEnvironment);
        }
        let (parent, bindings) = self.get_first_rest(env)?;
        if let Expr::Symbol(s) = name {
            let mut e = bindings.clone();
            while !e.is_nil() {
                if let Expr::Pair(k) = e {
                    let (first, rest) = self.get_first_rest_by_key(k)?;
                    if let Expr::Pair(b) = first {
                        let (key, _) = self.get_first_rest_by_key(b)?;
                        if key == *name {
                            return self.get_rest_by_key(b);
                        }
                    }
                    e = rest;
                } else {
                    return Err(SError::ImproperList);
                }
            }
            if parent.is_nil() {
                return Err(SError::UnboundSymbol);
            } else if parent.is_pair() {
                return self.env_get(&parent, name);
            } else {
                return Err(SError::ImproperEnvironment);
            }
        } else {
            return Err(SError::ImproperSymbol);
        }
    }

    fn env_set(&mut self, env: &Expr, name: &Expr, val: Expr) -> SResult<()> {
        if !env.is_pair() {
            return Err(SError::ImproperEnvironment);
        }
        let (_parent, bindings) = self.get_first_rest(env)?;
        if let Expr::Symbol(s) = name {
            let mut e = bindings.clone();
            while !e.is_nil() {
                if let Expr::Pair(k) = e {
                    let (first, rest) = self.get_first_rest_by_key(k)?;
                    if let Expr::Pair(b) = first {
                        let (key, _) = self.get_first_rest_by_key(b)?;
                        if key == *name {
                            self.set_rest_by_key(b, val)?;
                            return Ok(());
                        }
                    }
                    e = rest;
                } else {
                    return Err(SError::ImproperList);
                }
            }
            let new_pair = self.make_cons(name.clone(), val)?;
            let new_bindings = self.make_cons(new_pair, bindings)?;
            self.set_rest(env, new_bindings)?;
            return Ok(());
        } else {
            return Err(SError::ImproperSymbol);
        }
    }

    fn eval(&mut self, expr: &Expr) -> SResult<Expr> {
        let env = self.root_env.clone();
        self.eval_in(&env, expr)
    }

    fn eval_in(&mut self, env: &Expr, expr: &Expr) -> SResult<Expr> {
        match expr {
            Expr::Nil | Expr::Integer(_) => Ok(expr.clone()),
            Expr::Symbol(_) => self.env_get(env, expr),
            Expr::Pair(k) => {
                let first = self.get_first_by_key(*k)?;
                if first.is_specific_symbol("QUOTE") {
                    let args = self.get_rest_by_key(*k)?;
                    if !self.test_length(&args, 1)? {
                        return Err(SError::WrongNumberOfArgs);
                    }
                    return self.get_first(&args);
                } else if first.is_specific_symbol("DEFINE") {
                    let args = self.get_rest_by_key(*k)?;
                    if !self.test_length(&args, 2)? {
                        return Err(SError::WrongNumberOfArgs);
                    }
                    let sym = self.get_first(&args)?;
                    let rexpr = self.get_first(&self.get_rest(&args)?)?;
                    if !sym.is_symbol() {
                        return Err(SError::ImproperSymbol);
                    }
                    let val = self.eval_in(env, &rexpr)?;
                    self.env_set(env, &sym, val)?;
                    return Ok(sym);
                } else {
                    return Err(SError::UnknownForm);
                }
            }
        }
    }

    fn format_expr_inner(&self, expr: &Expr, acc: &mut String) -> SResult<()> {
        Ok(match expr {
            Expr::Nil => acc.push_str("()"),
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

    fn collect(&mut self) {
        for (_, c) in self.cells.iter_mut() {
            c.2 = false;
        }
        let mut worklist = vec![self.symbols.clone(), self.root_env.clone()];
        while let Some(ex) = worklist.pop() {
            if let Expr::Pair(n) = ex {
                let cell = self.cells.get_mut(n.0).unwrap();
                if cell.2 == false {
                    cell.2 = true;
                    worklist.push(cell.0.clone());
                    worklist.push(cell.1.clone());
                }
            }
        }
        self.cells.retain(|_, c| c.2);
    }

    fn dump(&self) -> SResult<()> {
        for (k, _) in self.cells.iter() {
            println!(
                "cell {}: {}",
                k,
                self.format_expr(&Expr::Pair(ConsCellKey(k)))?
            )
        }
        Ok(())
    }
}

fn main() {
    let mut heap = Heap::new();
    loop {
        let line = std::io::stdin().lock().lines().next().unwrap().unwrap();
        let mut token_stream = tokenize(&line).into_iter().peekable();
        while token_stream.peek().is_some() {
            let expr = parse_expr(&mut token_stream, &mut heap).unwrap().unwrap();
            println!("in:  {}", heap.format_expr(&expr).unwrap());
            match heap.eval(&expr) {
                Ok(result) => println!("out: {}", heap.format_expr(&result).unwrap()),
                Err(e) => println!("err: {:?}", e),
            }
        }
        heap.collect();
        //let _ = heap.dump();
    }
}
