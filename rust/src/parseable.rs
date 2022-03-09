use crate::expression::Expr;

pub trait Parseable: Sized {
    fn inner_parse(input: &str, parent: &str) -> Result<Self, String>;

    fn parse(input: &str) -> Result<Self, String> {
        Self::inner_parse(input, "[root]")
    }
}

fn nongreedy_parse(input: &str, parent: &str) -> Result<(Expr, usize), String> {
    log::trace!("[nongreedy-parse] {input:?} in {parent:?}");
    match input.chars().nth(0) {
        None => Err(format!("Expected expression in {parent:?}, got empty")),
        Some('(') => {
            let index = 1 + find_closing(&input.chars().skip(1).collect::<String>())?;
            assert_eq!(input.chars().nth(index), Some(')'));
            let expr = Expr::inner_parse(
                &input.chars().take(index).skip(1).collect::<String>(),
                input,
            )?;
            Ok((expr, index))
        }
        Some('~') => {
            let (expr, index) = nongreedy_parse(&input.chars().skip(1).collect::<String>(), input)?;
            Ok((Expr::Not(Box::new(expr)), index + 1))
        }
        _ => {
            let (atom, index) = input
                .find(' ')
                .map(|index| (input.chars().take(index).collect::<String>(), index))
                .unwrap_or((input.to_string(), input.len()));
            Ok((Expr::Atom(atom), index))
        }
    }
}

fn operator_parse(
    input: &str,
    left_expr: Expr,
    parent: &str,
) -> Result<(Option<Box<dyn Fn(Expr) -> Expr>>, usize), String> {
    log::trace!("[operator-parse] ({left_expr:?}) {input:?} in {parent:?}");
    match input.chars().nth(0) {
        None => Ok((None, 0)),
        Some('&') => Ok((
            Some(Box::new(move |right_expr| {
                Expr::and(&[left_expr.clone(), right_expr])
            })),
            0,
        )),
        Some('|') => Ok((
            Some(Box::new(move |right_expr| {
                Expr::or(&[left_expr.clone(), right_expr])
            })),
            0,
        )),
        Some('>') => Ok((
            Some(Box::new(move |right_expr| {
                Expr::or(&[Expr::not(left_expr.clone()), right_expr])
            })),
            0,
        )),
        Some('=') => Ok((
            Some(Box::new(move |right_expr| {
                Expr::and(&[
                    Expr::or(&[Expr::not(left_expr.clone()), right_expr.clone()]),
                    Expr::or(&[left_expr.clone(), Expr::not(right_expr)]),
                ])
            })),
            0,
        )),
        _ => Err(format!(
            "Expected [empty, |, &, >, =] in {parent:?} but got {input:?}"
        )),
    }
}

fn find_closing(input: &str) -> Result<usize, String> {
    log::trace!("[find-closing] {input:?}");
    match (input.find('('), input.find(')')) {
        (Some(open), Some(close)) if open < close => {
            let find_match_close = find_closing(&input.chars().skip(open + 1).collect::<String>())?;
            let match_close = open + 1 + find_match_close;
            assert_eq!(input.chars().nth(match_close), Some(')'));
            let find_next_close =
                find_closing(&input.chars().skip(match_close + 1).collect::<String>())?;
            let next_close = match_close + 1 + find_next_close;
            assert_eq!(input.chars().nth(next_close), Some(')'));
            Ok(next_close)
        }
        (_, Some(close)) => {
            assert_eq!(input.chars().nth(close), Some(')'));
            Ok(close)
        }
        _ => Err(format!("Unmatched open-close in {input:?}")),
    }
}

impl Parseable for Expr {
    fn inner_parse(input: &str, parent: &str) -> Result<Self, String> {
        log::trace!("[inner-parse] {input:?} in {parent:?}");

        let left_inp = input.trim_start();
        let (left_expr, left_idx) = nongreedy_parse(left_inp, &parent)?;

        let mid_inp_string = left_inp.chars().skip(left_idx + 1).collect::<String>();
        let mid_inp = &mid_inp_string.trim_start();

        let (maybe_expr_fn, mid_idx) = operator_parse(mid_inp, left_expr.clone(), left_inp)?;

        Ok(match maybe_expr_fn {
            Some(expr_fn) => {
                let right_inp_string = mid_inp.chars().skip(mid_idx + 1).collect::<String>();
                let right_inp = &right_inp_string.trim_start();
                let right_expr = Self::inner_parse(right_inp, left_inp)?;
                expr_fn(right_expr)
            }
            None => left_expr,
        })
    }
}
