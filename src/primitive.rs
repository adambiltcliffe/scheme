use std::rc::Rc;

use crate::{Expr, Heap, Native, PrimitiveDef, SError, SResult};

fn validate_arg_count(heap: &Heap, args: &Expr, n: usize) -> SResult<()> {
    if !heap.test_length(args, n)? {
        Err(SError::WrongNumberOfArgs)
    } else {
        Ok(())
    }
}

fn first(args: &Expr, heap: &mut Heap) -> SResult<Expr> {
    validate_arg_count(heap, args, 1)?;
    let arg = heap.get_first(args)?;
    heap.get_first(&arg)
}

fn rest(args: &Expr, heap: &mut Heap) -> SResult<Expr> {
    validate_arg_count(heap, args, 1)?;
    let arg = heap.get_first(args)?;
    heap.get_rest(&arg)
}

fn list_p(args: &Expr, heap: &mut Heap) -> SResult<Expr> {
    validate_arg_count(heap, args, 1)?;
    let arg = heap.get_first(args)?;
    Ok(Expr::Boolean(heap.is_proper_list(&arg)?))
}

fn cons(args: &Expr, heap: &mut Heap) -> SResult<Expr> {
    validate_arg_count(heap, args, 2)?;
    let arg1 = heap.get_first(args)?;
    let arg2 = heap.get_first(&heap.get_rest(args)?)?;
    heap.make_cons(arg1, arg2)
}

fn as_integer(expr: &Expr) -> SResult<i64> {
    match expr {
        Expr::Integer(n) => Ok(*n),
        _ => Err(SError::TypeError),
    }
}

fn do_arithmetic(
    args: &Expr,
    heap: &mut Heap,
    identity: i64,
    bin_op: impl Fn(i64, i64) -> i64,
) -> SResult<Expr> {
    if args.is_nil() {
        return Err(SError::WrongNumberOfArgs);
    } else if heap.get_rest(args)?.is_nil() {
        return Ok(Expr::Integer(bin_op(
            identity,
            as_integer(&heap.get_first(args)?)?,
        )));
    }
    let mut result = as_integer(&heap.get_first(args)?)?;
    let mut v = heap.get_rest(args)?.clone();
    while !v.is_nil() {
        result = bin_op(result, as_integer(&heap.get_first(&v)?)?);
        v = heap.get_rest(&v)?;
    }
    Ok(Expr::Integer(result))
}

fn do_plus(args: &Expr, heap: &mut Heap) -> SResult<Expr> {
    do_arithmetic(args, heap, 0, |a, b| a + b)
}

fn do_minus(args: &Expr, heap: &mut Heap) -> SResult<Expr> {
    do_arithmetic(args, heap, 0, |a, b| a - b)
}

fn do_times(args: &Expr, heap: &mut Heap) -> SResult<Expr> {
    do_arithmetic(args, heap, 1, |a, b| a * b)
}

fn do_divide(args: &Expr, heap: &mut Heap) -> SResult<Expr> {
    do_arithmetic(args, heap, 1, |a, b| a / b)
}

fn do_predicate(args: &Expr, heap: &mut Heap, pred: impl Fn(i64, i64) -> bool) -> SResult<Expr> {
    validate_arg_count(heap, args, 2)?;
    let arg1 = as_integer(&heap.get_first(args)?)?;
    let arg2 = as_integer(&heap.get_first(&heap.get_rest(args)?)?)?;
    let result = pred(arg1, arg2);
    Ok(Expr::Boolean(result))
}

fn do_numeq(args: &Expr, heap: &mut Heap) -> SResult<Expr> {
    do_predicate(args, heap, |a, b| a == b)
}

fn do_lt(args: &Expr, heap: &mut Heap) -> SResult<Expr> {
    do_predicate(args, heap, |a, b| a < b)
}

fn do_lte(args: &Expr, heap: &mut Heap) -> SResult<Expr> {
    do_predicate(args, heap, |a, b| a <= b)
}

fn do_gt(args: &Expr, heap: &mut Heap) -> SResult<Expr> {
    do_predicate(args, heap, |a, b| a > b)
}

fn do_gte(args: &Expr, heap: &mut Heap) -> SResult<Expr> {
    do_predicate(args, heap, |a, b| a >= b)
}

fn add_primitive(heap: &mut Heap, name: &str, func: Native) -> SResult<()> {
    let sym = heap.make_symbol(name)?;
    let env = heap.root_env.clone();
    heap.env_set(
        &env,
        &sym,
        Expr::Primitive(Rc::new(PrimitiveDef {
            name: name.to_owned(),
            func,
        })),
    )?;
    Ok(())
}

pub(crate) fn add_primitives(heap: &mut Heap) -> SResult<()> {
    add_primitive(heap, "first", first)?;
    add_primitive(heap, "rest", rest)?;
    add_primitive(heap, "list?", list_p)?;
    add_primitive(heap, "cons", cons)?;
    add_primitive(heap, "+", do_plus)?;
    add_primitive(heap, "-", do_minus)?;
    add_primitive(heap, "*", do_times)?;
    add_primitive(heap, "/", do_divide)?;
    add_primitive(heap, "=", do_numeq)?;
    add_primitive(heap, "<", do_lt)?;
    add_primitive(heap, "<=", do_lte)?;
    add_primitive(heap, ">", do_gt)?;
    add_primitive(heap, ">=", do_gte)?;
    Ok(())
}
