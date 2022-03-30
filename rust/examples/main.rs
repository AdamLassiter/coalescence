use coalescence::{coalesceable::Coalesceable, expression::Expr, parseable::Parseable};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    while let Some(input) = rprompt::prompt_reply_stdout("ψ. ").ok() {
        let expr = Expr::parse(&input)?.normal();
        log::info!("Input: {expr:?}");
        let (_, proof) = expr.coalesce().ok_or("Not coalesceable")?;

        log::info!("Proof: {proof:?}");
    }

    Ok(())
}
