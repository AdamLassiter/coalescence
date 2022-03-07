use coalescence::{expression::Expr, parseable::Parseable, proveable::*};

use pretty_assertions::assert_eq;

fn log_init() {
    let _ = env_logger::builder().is_test(true).try_init();
}

#[test]
fn proof_axiom() -> Result<(), String> {
    log_init();

    let expr = Expr::parse("a > a")?.normal();
    let proof = expr.proof()?;

    assert_eq!(proof.verify(), Ok(()));

    Ok(())
}

#[test]
fn proof_duplicate_axiom() -> Result<(), String> {
    log_init();

    let expr = Expr::parse("(a > a) & (a > a)")?.normal();
    let proof = expr.proof()?;

    assert_eq!(proof.verify(), Ok(()));

    Ok(())
}

#[test]
fn proof_two_axioms() -> Result<(), String> {
    log_init();

    let expr = Expr::parse("(a > a) & (b > b)")?.normal();
    let proof = expr.proof()?;

    assert_eq!(proof.verify(), Ok(()));

    Ok(())
}

#[test]
fn proof_second_axiom() -> Result<(), String> {
    log_init();

    let expr = Expr::parse("(a & b) | (~a & b) | (a & ~b) | (~a & ~b)")?.normal();
    let proof = expr.proof()?;

    assert_eq!(proof.verify(), Ok(()));

    Ok(())
}

#[test]
#[ignore]
fn proof_second_axiom_invalid() -> Result<(), String> {
    log_init();

    let expr = Expr::parse("(a & b) | (~a & b) | (a & ~b)")?.normal();
    let proof = expr.proof();

    let _ = proof.unwrap_err();

    Ok(())
}

#[test]
#[ignore]
fn proof_fourth_axiom() -> Result<(), String> {
    log_init();

    let expr = Expr::parse("(a & b & c & d) | (a & ~b & c & d) | (~a & b & c & d) | (~a & ~b & c & d) | (a & b & ~c & d) | (a & ~b & ~c & d) | (~a & b & ~c & d) | (~a & ~b & ~c & d) | (a & b & c & ~d) | (a & ~b & c & ~d) | (~a & b & c & ~d) | (~a & ~b & c & ~d) | (a & b & ~c & ~d) | (a & ~b & ~c & ~d) | (~a & b & ~c & ~d) | (~a & ~b & ~c & ~d)")?.normal();
    let proof = expr.proof()?;

    assert_eq!(proof.verify(), Ok(()));

    Ok(())
}
