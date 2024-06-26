use crate::{Expr, Heap, SError, SResult};

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

fn cons(args: &Expr, heap: &mut Heap) -> SResult<Expr> {
    if !heap.test_length(args, 2)? {
        return Err(SError::WrongNumberOfArgs);
    }
    let arg1 = heap.get_first(args)?;
    let arg2 = heap.get_first(&heap.get_rest(args)?)?;
    heap.make_cons(arg1, arg2)
}
