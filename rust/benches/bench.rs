#![feature(test)]

extern crate test;

#[cfg(test)]
mod bench {
    use test::{black_box, Bencher};

    use coalescence::{coalesceable::Coalesceable, expression::Expr, parseable::Parseable};

    #[bench]
    pub(crate) fn coalesce_third_axiom(bencher: &mut Bencher) {
        let expr = Expr::parse("(a & b & c) | (a & ~b & c) | (~a & b & c) | (~a & ~b & c) | (a & b & ~c) | (a & ~b & ~c) | (~a & b & ~c) | (~a & ~b & ~c)").unwrap().normal();

        bencher.iter(|| {
            black_box(expr.coalesce().ok_or("Not coalesceable").unwrap());
        })
    }

    #[bench]
    #[ignore]
    pub(crate) fn coalesce_fourth_axiom(bencher: &mut Bencher) {
        let expr = Expr::parse("(a & b & c & d) | (a & ~b & c & d) | (~a & b & c & d) | (~a & ~b & c & d) | (a & b & ~c & d) | (a & ~b & ~c & d) | (~a & b & ~c & d) | (~a & ~b & ~c & d) | (a & b & c & ~d) | (a & ~b & c & ~d) | (~a & b & c & ~d) | (~a & ~b & c & ~d) | (a & b & ~c & ~d) | (a & ~b & ~c & ~d) | (~a & b & ~c & ~d) | (~a & ~b & ~c & ~d)").unwrap().normal();

        bencher.iter(|| {
            black_box(expr.coalesce().ok_or("Not coalesceable").unwrap());
        })
    }
}