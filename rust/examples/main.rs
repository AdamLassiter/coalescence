use coalescence::{expression::Expr, parseable::Parseable, proveable::Proveable};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    while let Some(input) = rprompt::prompt_reply_stdout("ψ. ").ok() {
        let expr = Expr::parse(&input)?.normal();
        log::info!("Input: {expr:?}");
        let proof = expr.proof()?;

        log::info!("Proof: {proof:?}");
    }

    Ok(())
}
