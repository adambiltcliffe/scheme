use std::rc::Rc;

use crate::{Expr, Heap, Native, PrimitiveDef, SError, SResult};

fn first(args: &Expr, heap: &mut Heap) -> SResult<Expr> {
    if !heap.test_length(args, 1)? {
        return Err(SError::WrongNumberOfArgs);
    }
    let arg = heap.get_first(args)?;
    heap.get_first(&arg)
}

fn rest(args: &Expr, heap: &mut Heap) -> SResult<Expr> {
    if !heap.test_length(args, 1)? {
        return Err(SError::WrongNumberOfArgs);
    }
    let arg = heap.get_first(args)?;
    heap.get_rest(&arg)
}

fn list_p(args: &Expr, heap: &mut Heap) -> SResult<Expr> {
    if !heap.test_length(args, 1)? {
        return Err(SError::WrongNumberOfArgs);
    }
    let arg = heap.get_first(args)?;
    Ok(Expr::Boolean(heap.is_proper_list(&arg)?))
}

fn cons(args: &Expr, heap: &mut Heap) -> SResult<Expr> {
    if !heap.test_length(args, 2)? {
        return Err(SError::WrongNumberOfArgs);
    }
    let arg1 = heap.get_first(args)?;
    let arg2 = heap.get_first(&heap.get_rest(args)?)?;
    heap.make_cons(arg1, arg2)
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
    Ok(())
}
