use coalescence::{expression::*, parseable::Parseable, Set};

use pretty_assertions::assert_eq;

fn log_init() {
    let _ = env_logger::builder().is_test(true).try_init();
}

#[test]
fn normal_idempotent() -> Result<(), String> {
    log_init();

    assert_eq!(Expr::parse("a | a")?.normal(), Expr::parse("a")?);

    assert_eq!(Expr::parse("a & a")?.normal(), Expr::parse("a")?);

    Ok(())
}

#[test]
fn normal_inverse() -> Result<(), String> {
    log_init();

    assert_eq!(Expr::parse("~(~a)")?.normal(), Expr::parse("a")?);

    Ok(())
}

#[test]
fn normal_commutative() -> Result<(), String> {
    log_init();

    assert_eq!(
        Expr::parse("(a | b) | c")?.normal(),
        Expr::parse("a | (b | c)")?.normal()
    );

    assert_eq!(
        Expr::parse("(a & b) & c")?.normal(),
        Expr::parse("a & (b & c)")?.normal()
    );

    Ok(())
}

#[test]
fn normal_idempotent_commutative() -> Result<(), String> {
    log_init();

    assert_eq!(
        Expr::parse("((a & a) & a) & (a & a) | a")?.normal(),
        Expr::parse("a")?
    );
    assert_eq!(
        Expr::parse("((a | a) | a) | (a | a) & a")?.normal(),
        Expr::parse("a")?
    );

    Ok(())
}

#[test]
fn normal_syntactic_sugar() -> Result<(), String> {
    log_init();

    assert_eq!(
        Expr::parse("a > b")?.normal(),
        Expr::parse("~a | b")?.normal()
    );

    assert_eq!(
        Expr::parse("a = b")?.normal(),
        Expr::parse("(a > b) & (b > a)")?.normal()
    );
    assert_eq!(
        Expr::parse("a = b")?.normal(),
        Expr::parse("(~a | b) & (~b | a)")?.normal()
    );

    Ok(())
}

#[test]
fn inverse() -> Result<(), String> {
    log_init();

    assert_eq!(
        Expr::parse("a")?.inverse().normal(),
        Expr::parse("~a")?.normal()
    );

    assert_eq!(Expr::parse("~a")?.inverse().normal(), Expr::parse("a")?);

    assert_eq!(
        Expr::parse("~a | ~b")?.inverse().normal(),
        Expr::parse("a & b")?
    );

    assert_eq!(
        Expr::parse("~a & ~b")?.inverse().normal(),
        Expr::parse("a | b")?
    );

    Ok(())
}

#[test]
fn names() -> Result<(), String> {
    log_init();

    assert_eq!(
        Expr::parse("(a & ~b) | (c & ~a)")?.names(),
        Set::from(["a".to_string(), "b".to_string(), "c".to_string()])
    );

    Ok(())
}

#[test]
fn atoms() -> Result<(), String> {
    log_init();

    assert_eq!(
        Expr::parse("(a & ~b) | (c & ~a)")?.normal().atoms(),
        Set::from([
            &Expr::Atom("a".to_string()),
            &Expr::NotAtom("a".to_string()),
            &Expr::NotAtom("b".to_string()),
            &Expr::Atom("c".to_string())
        ])
    );

    Ok(())
}

#[test]
fn subexprs() -> Result<(), String> {
    log_init();

    Ok(())
}

#[test]
fn lineage() -> Result<(), String> {
    log_init();

    Ok(())
}
