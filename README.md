There seems to be a rule that once you reach a certain level of interest in programming languages, you have to write a Lisp of some sort. This is an interpreter for a Scheme-like (although not standards-compliant) language.

Features:

- Atoms are symbols, 64-bit signed integers and booleans (written #t and #f)
- The empty list is (), regular lists are (A B C) and improper lists are (A B . C)
- Make cons cells with CONS and access their contents with FIRST and REST (not CAR/CDR)
- Numeric primitives: binary =, <, <=, >, >= and n-ary +, -, \*, /
- Quote with (QUOTE body) or just 'body
- Special forms: (DEFINE X value), (DEFINE (F args) body), (LAMBDA (args) body)
- Short-circuiting (IF test-expr true-expr false-expr)
- Garbage collection (only following each iteration of the REPL, though)

Currently missing:

- Proper tail calls
- Quasiquotation
- Variadic functions
- Macros
- More primitives
- Input that isn't a single-line REPL

Example:

```
(define (fact x) (if (= x 0) 1 (* x (fact (- x 1)))))
in:  (DEFINE (FACT X) (IF (= X 0) 1 (* X (FACT (- X 1)))))
out: FACT
(fact 10)
in:  (FACT 10)
out: 3628800
```
