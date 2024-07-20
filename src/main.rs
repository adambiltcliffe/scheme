use std::{io::BufRead, ops::Deref, rc::Rc};

use lexer::tokenize;
use parser::parse_expr;
use primitive::add_primitives;
use slab::Slab;

mod lexer;
mod parser;
mod primitive;

#[derive(Debug)]
enum SError {
    ImproperLambda,
    ImproperList,
    ImproperSymbol,
    ImproperEnvironment,
    NotCallable,
    TypeError,
    UnboundSymbol,
    WrongNumberOfArgs,
}

type SResult<T> = Result<T, SError>;

type ConsCell = (Expr, Expr, bool);

type Native = fn(&Expr, &mut Heap) -> SResult<Expr>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ConsCellKey(usize);

#[derive(Debug, Clone, PartialEq, Eq)]
struct PrimitiveDef {
    name: String,
    func: Native,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Expr {
    Nil,
    Boolean(bool),
    Integer(i64),
    Symbol(Rc<str>),
    Pair(ConsCellKey),
    Closure(ConsCellKey),
    Primitive(Rc<PrimitiveDef>),
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

    fn is_truthy(&self) -> bool {
        // #f is false
        // everything else including 0 and () are true
        !matches!(self, Self::Boolean(false))
    }

    fn is_specific_symbol(&self, s: &str) -> bool {
        if let Expr::Symbol(sym) = self {
            sym.as_ref() == s
        } else {
            false
        }
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
        add_primitives(&mut me).unwrap();
        me
    }

    fn get_first_rest(&self, expr: &Expr) -> SResult<(Expr, Expr)> {
        if let Expr::Pair(k) = expr {
            let cell = self.cells.get((k).0).unwrap().clone();
            Ok((cell.0, cell.1))
        } else {
            Err(SError::ImproperList)
        }
    }

    fn get_first(&self, expr: &Expr) -> SResult<Expr> {
        if let Expr::Pair(k) = expr {
            Ok(self.cells.get((k).0).unwrap().0.clone())
        } else {
            Err(SError::ImproperList)
        }
    }

    fn set_first(&mut self, expr: &Expr, v: Expr) -> SResult<()> {
        if let Expr::Pair(k) = expr {
            self.cells.get_mut((k).0).unwrap().0 = v;
            Ok(())
        } else {
            Err(SError::ImproperList)
        }
    }

    fn get_rest(&self, expr: &Expr) -> SResult<Expr> {
        if let Expr::Pair(k) = expr {
            Ok(self.cells.get((k).0).unwrap().1.clone())
        } else {
            Err(SError::ImproperList)
        }
    }

    fn set_rest(&mut self, expr: &Expr, v: Expr) -> SResult<()> {
        if let Expr::Pair(k) = expr {
            self.cells.get_mut((k).0).unwrap().1 = v;
            Ok(())
        } else {
            Err(SError::ImproperList)
        }
    }

    fn get_lambda_env(&self, expr: &Expr) -> SResult<Expr> {
        if let Expr::Closure(k) = expr {
            Ok(self.cells.get((k).0).unwrap().0.clone())
        } else {
            Err(SError::ImproperLambda)
        }
    }

    fn get_lambda_args(&self, expr: &Expr) -> SResult<Expr> {
        if let Expr::Closure(k) = expr {
            let rest = self.cells.get((k).0).unwrap().1.clone();
            self.get_first(&rest)
        } else {
            Err(SError::ImproperLambda)
        }
    }

    fn get_lambda_body(&self, expr: &Expr) -> SResult<Expr> {
        if let Expr::Closure(k) = expr {
            let rest = self.cells.get((k).0).unwrap().1.clone();
            self.get_rest(&rest)
        } else {
            Err(SError::ImproperLambda)
        }
    }

    fn make_cons(&mut self, first: Expr, rest: Expr) -> SResult<Expr> {
        let key = ConsCellKey(self.cells.insert((first, rest, false)));
        Ok(Expr::Pair(key))
    }

    fn map_list(
        &mut self,
        list: &Expr,
        func: impl Fn(&mut Heap, &Expr) -> SResult<Expr>,
    ) -> SResult<Expr> {
        if list.is_nil() {
            return Ok(Expr::Nil);
        }
        let (mut first, mut rest) = self.get_first_rest(list)?;
        let val = func(self, &first)?;
        let result = self.make_cons(val, Expr::Nil).unwrap();
        let mut result_tail = result.clone();
        while !rest.is_nil() {
            if rest.is_pair() {
                (first, rest) = self.get_first_rest(&rest)?;
                let val = func(self, &first)?;
                let new_tail = self.make_cons(val, Expr::Nil).unwrap();
                self.set_rest(&result_tail, new_tail.clone())?;
                result_tail = new_tail;
            } else {
                return Err(SError::ImproperList);
            }
        }
        Ok(result)
    }

    fn is_proper_list(&self, expr: &Expr) -> SResult<bool> {
        if expr.is_nil() {
            return Ok(true);
        }
        if !expr.is_pair() {
            return Ok(false);
        }
        let rest = self.get_rest(expr)?;
        self.is_proper_list(&rest)
    }

    fn test_length(&self, expr: &Expr, n: usize) -> SResult<bool> {
        if expr.is_nil() {
            return Ok(n == 0);
        } else if n == 0 {
            return Ok(false);
        }
        let rest = self.get_rest(expr)?;
        self.test_length(&rest, n - 1)
    }

    fn make_symbol(&mut self, name: &str) -> SResult<Expr> {
        let name = name.to_ascii_uppercase();
        let mut s = self.symbols.clone();
        while !s.is_nil() {
            let (first, rest) = self.get_first_rest(&s)?;
            if let Expr::Symbol(r) = first {
                if Rc::deref(&r).eq(&name) {
                    return Ok(Expr::Symbol(Rc::clone(&r)));
                }
            }
            s = rest;
        }
        drop(s);
        let new_symbol: Rc<str> = Rc::from(name);
        self.symbols =
            self.make_cons(Expr::Symbol(Rc::clone(&new_symbol)), self.symbols.clone())?;
        Ok(Expr::Symbol(new_symbol))
    }

    fn make_closure(&mut self, env: Expr, arg_list: Expr, body: Expr) -> SResult<Expr> {
        let mut v = arg_list.clone();
        while !v.is_nil() {
            if !self.get_first(&v)?.is_symbol() {
                return Err(SError::ImproperSymbol);
            }
            v = self.get_rest(&v)?.clone();
        }
        let tail = self.make_cons(arg_list, body)?;
        if let Expr::Pair(key) = self.make_cons(env, tail)? {
            Ok(Expr::Closure(key))
        } else {
            unreachable!()
        }
    }

    fn make_env(&mut self, parent: &Expr) -> SResult<Expr> {
        self.make_cons(parent.clone(), Expr::Nil)
    }

    fn env_get(&self, env: &Expr, name: &Expr) -> SResult<Expr> {
        if !env.is_pair() {
            return Err(SError::ImproperEnvironment);
        }
        let (parent, bindings) = self.get_first_rest(env)?;
        if let Expr::Symbol(_) = name {
            let mut e = bindings.clone();
            while !e.is_nil() {
                let (first, rest) = self.get_first_rest(&e)?;
                if first.is_pair() {
                    let (key, _) = self.get_first_rest(&first)?;
                    if key == *name {
                        return self.get_rest(&first);
                    }
                }
                e = rest;
            }
            if parent.is_nil() {
                Err(SError::UnboundSymbol)
            } else if parent.is_pair() {
                self.env_get(&parent, name)
            } else {
                Err(SError::ImproperEnvironment)
            }
        } else {
            Err(SError::ImproperSymbol)
        }
    }

    fn env_set(&mut self, env: &Expr, name: &Expr, val: Expr) -> SResult<()> {
        if !env.is_pair() {
            return Err(SError::ImproperEnvironment);
        }
        let (_parent, bindings) = self.get_first_rest(env)?;
        if let Expr::Symbol(_) = name {
            let mut e = bindings.clone();
            while !e.is_nil() {
                let (first, rest) = self.get_first_rest(&e)?;
                if first.is_pair() {
                    let (key, _) = self.get_first_rest(&first)?;
                    if key == *name {
                        self.set_rest(&first, val)?;
                        return Ok(());
                    }
                }
                e = rest;
            }
            let new_pair = self.make_cons(name.clone(), val)?;
            let new_bindings = self.make_cons(new_pair, bindings)?;
            self.set_rest(env, new_bindings)?;
            Ok(())
        } else {
            Err(SError::ImproperSymbol)
        }
    }

    fn apply(&mut self, op: &Expr, args: &Expr) -> SResult<Expr> {
        if let Expr::Primitive(p) = op {
            (p.func)(args, self)
        } else if let Expr::Closure(_) = op {
            let env = self.make_env(&self.get_lambda_env(op)?)?;
            let mut param_list = self.get_lambda_args(op)?;
            let mut arg_list = args.clone();
            while !param_list.is_nil() {
                if arg_list.is_nil() {
                    return Err(SError::WrongNumberOfArgs);
                }
                let param = self.get_first(&param_list)?;
                let arg = self.get_first(&arg_list)?;
                self.env_set(&env, &param, arg)?;
                param_list = self.get_rest(&param_list)?;
                arg_list = self.get_rest(&arg_list)?;
            }
            if !arg_list.is_nil() {
                return Err(SError::WrongNumberOfArgs);
            }
            let mut body = self.get_lambda_body(op)?;
            let mut result = Expr::Nil;
            while !body.is_nil() {
                let form = self.get_first(&body)?;
                result = self.eval_in(&env, &form)?;
                body = self.get_rest(&body)?;
            }
            Ok(result)
        } else {
            Err(SError::NotCallable)
        }
    }

    fn eval(&mut self, expr: &Expr) -> SResult<Expr> {
        let env = self.root_env.clone();
        self.eval_in(&env, expr)
    }

    fn eval_in(&mut self, env: &Expr, expr: &Expr) -> SResult<Expr> {
        match expr {
            Expr::Nil
            | Expr::Boolean(_)
            | Expr::Integer(_)
            | Expr::Closure(_)
            | Expr::Primitive(_) => Ok(expr.clone()),
            Expr::Symbol(_) => self.env_get(env, expr),
            Expr::Pair(_) => {
                let (first, rest) = self.get_first_rest(expr)?;
                if first.is_specific_symbol("QUOTE") {
                    let args = rest;
                    if !self.test_length(&args, 1)? {
                        return Err(SError::WrongNumberOfArgs);
                    }
                    self.get_first(&args)
                } else if first.is_specific_symbol("DEFINE") {
                    let args = rest;
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
                    Ok(sym)
                } else if first.is_specific_symbol("IF") {
                    let args = rest;
                    if !self.test_length(&args, 3)? {
                        return Err(SError::WrongNumberOfArgs);
                    }
                    let test_expr = self.get_first(&args)?;
                    let true_expr = self.get_first(&self.get_rest(&args)?)?;
                    let false_expr = self.get_first(&self.get_rest(&self.get_rest(&args)?)?)?;
                    let t = self.eval_in(env, &test_expr)?;
                    if t.is_truthy() {
                        self.eval_in(env, &true_expr)
                    } else {
                        self.eval_in(env, &false_expr)
                    }
                } else if first.is_specific_symbol("LAMBDA") {
                    let args = rest;
                    if !self.test_length(&args, 2)? {
                        return Err(SError::WrongNumberOfArgs);
                    }
                    let arg_list = self.get_first(&args)?;
                    let body = self.get_rest(&args)?;
                    Ok(self.make_closure(env.clone(), arg_list, body)?)
                } else {
                    let op = self.eval_in(env, &first)?;
                    let args = self.map_list(&rest, |h, e| h.eval_in(env, e))?;
                    self.apply(&op, &args)
                }
            }
        }
    }

    fn format_expr_inner(&self, expr: &Expr, acc: &mut String) -> SResult<()> {
        match expr {
            Expr::Nil => acc.push_str("()"),
            Expr::Boolean(false) => acc.push_str("#f"),
            Expr::Boolean(true) => acc.push_str("#t"),
            Expr::Integer(n) => acc.push_str(&n.to_string()),
            Expr::Symbol(s) => acc.push_str(s),
            Expr::Closure(_) => acc.push_str("#<lambda>"),
            Expr::Primitive(d) => acc.push_str(&format!("#<primitive {}>", d.name)),
            Expr::Pair(_) => {
                acc.push('(');
                let (mut first, mut rest) = self.get_first_rest(expr)?;
                loop {
                    self.format_expr_inner(&first, acc)?;
                    match rest {
                        Expr::Nil => break,
                        Expr::Pair(_) => {
                            acc.push(' ');
                            (first, rest) = self.get_first_rest(&rest)?;
                        }
                        _ => {
                            acc.push_str(" . ");
                            self.format_expr_inner(&rest, acc)?;
                            break;
                        }
                    }
                }
                acc.push(')');
            }
        }
        Ok(())
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
            if let Expr::Pair(n) | Expr::Closure(n) = ex {
                let cell = self.cells.get_mut(n.0).unwrap();
                if !cell.2 {
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
    while let Some(res) = std::io::stdin().lock().lines().next() {
        let line = res.unwrap();
        let mut token_stream = tokenize(&line).into_iter().peekable();
        while token_stream.peek().is_some() {
            let expr = parse_expr(&mut token_stream, &mut heap).unwrap();
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
