use coalescence::{expression::Expr, parseable::*};

use pretty_assertions::assert_eq;

fn log_init() {
    let _ = env_logger::builder().is_test(true).try_init();
}

#[test]
fn parse_atom() -> Result<(), String> {
    log_init();

    assert_eq!(Expr::parse("a")?,
        Expr::Atom("a".to_string()));

    assert_eq!(Expr::parse("alpha")?,
        Expr::Atom("alpha".to_string()));

    assert_eq!(Expr::parse("~a")?,
        Expr::not(Expr::Atom("a".to_string())));

    Ok(())
}

#[test]
fn parse_operator() -> Result<(), String> {
    log_init();

    assert_eq!(Expr::parse("a & b")?,
        Expr::and(&[
            Expr::Atom("a".to_string()),
            Expr::Atom("b".to_string())]));

    assert_eq!(Expr::parse("a | b")?,
        Expr::or(&[
            Expr::Atom("a".to_string()),
            Expr::Atom("b".to_string())]));

    Ok(())
}

#[test]
fn parse_syntactic_sugar() -> Result<(), String> {
    log_init();

    assert_eq!(Expr::parse("a > b")?,
        Expr::or(&[
            Expr::not(Expr::Atom("a".to_string())),
            Expr::Atom("b".to_string())]));

    assert_eq!(Expr::parse("a = b")?,
        Expr::and(&[
            Expr::or(&[
                Expr::not(Expr::Atom("a".to_string())),
                Expr::Atom("b".to_string())]),
            Expr::or(&[
                Expr::not(Expr::Atom("b".to_string())),
                Expr::Atom("a".to_string())])]));

    Ok(())
}

#[test]
fn parse_braces() -> Result<(), String> {
    log_init();

    assert_eq!(Expr::parse("~(a | b)")?,
        Expr::not(Expr::or(&[
            Expr::Atom("a".to_string()),
            Expr::Atom("b".to_string())])));

    assert_eq!(Expr::parse("a = b")?,
        Expr::and(&[
            Expr::or(&[
                Expr::not(Expr::Atom("a".to_string())),
                Expr::Atom("b".to_string())]),
            Expr::or(&[
                Expr::not(Expr::Atom("b".to_string())),
                Expr::Atom("a".to_string())])]));

    Ok(())
}
