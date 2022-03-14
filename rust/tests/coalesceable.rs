#![feature(test)]

use coalescence::{coalesceable::*, expression::Expr, parseable::Parseable};

// TODO: Assert against sequents generated through coalescence
use pretty_assertions::assert_eq;

fn log_init() {
    let _ = env_logger::builder().is_test(true).try_init();
}

#[test]
fn coalesce_axiom() -> Result<(), String> {
    log_init();

    let expr = Expr::parse("a > a")?.normal();
    let _ = expr.coalesce().ok_or("Not coalesceable")?;
    Ok(())
}

#[test]
fn coalesce_duplicate_axiom() -> Result<(), String> {
    log_init();

    let expr = Expr::parse("(a > a) & (a > a)")?.normal();
    let _ = expr.coalesce().ok_or("Not coalesceable")?;
    Ok(())
}

#[test]
fn coalesce_two_axioms() -> Result<(), String> {
    log_init();

    let expr = Expr::parse("(a > a) & (b > b)")?.normal();
    let _ = expr.coalesce().ok_or("Not coalesceable")?;
    Ok(())
}

#[test]
fn coalesce_second_axiom() -> Result<(), String> {
    log_init();

    let expr = Expr::parse("(a & b) | (~a & b) | (a & ~b) | (~a & ~b)")?.normal();
    let _ = expr.coalesce().ok_or("Not coalesceable")?;
    Ok(())
}

#[test]
fn coalesce_second_axiom_invalid() -> Result<(), String> {
    log_init();

    let expr = Expr::parse("(a & b) | (~a & b) | (a & ~b)")?.normal();
    let _ = expr
        .coalesce()
        .ok_or("Not coalesceable")
        .expect_err("False statement coalesceable");
    Ok(())
}

#[test]
fn coalesce_third_axiom() -> Result<(), String> {
    log_init();

    let expr = Expr::parse("(a & b & c) | (a & ~b & c) | (~a & b & c) | (~a & ~b & c) | (a & b & ~c) | (a & ~b & ~c) | (~a & b & ~c) | (~a & ~b & ~c)")?.normal();
    let _ = expr.coalesce().ok_or("Not coalesceable")?;
    Ok(())
}

#[test]
pub fn coalesce_fourth_axiom() -> Result<(), String> {
    log_init();

    let expr = Expr::parse("(a & b & c & d) | (a & ~b & c & d) | (~a & b & c & d) | (~a & ~b & c & d) | (a & b & ~c & d) | (a & ~b & ~c & d) | (~a & b & ~c & d) | (~a & ~b & ~c & d) | (a & b & c & ~d) | (a & ~b & c & ~d) | (~a & b & c & ~d) | (~a & ~b & c & ~d) | (a & b & ~c & ~d) | (a & ~b & ~c & ~d) | (~a & b & ~c & ~d) | (~a & ~b & ~c & ~d)")?.normal();
    let _ = expr.coalesce().ok_or("Not coalesceable")?;
    Ok(())
}
