use std::path::PathBuf;

use agentscript_compiler::{parse_script_file, ParseFileError};
use clap::Args;
use miette::Report;

#[derive(Args)]
pub struct VerifyAgentscriptArgs {
    /// Path to `.pine` or `.qas` AgentScript source
    #[arg(value_name = "PATH")]
    pub path: PathBuf,
}

pub fn run(args: VerifyAgentscriptArgs) -> Result<(), Box<dyn std::error::Error>> {
    match parse_script_file(&args.path) {
        Ok(script) => {
            println!(
                "ok: parsed AgentScript ({} top-level item(s))",
                script.items.len()
            );
            Ok(())
        }
        Err(ParseFileError::Io(e)) => Err(e.into()),
        Err(ParseFileError::Compile(e)) => {
            eprintln!("{:?}", Report::new(e));
            Err("AgentScript parse failed".into())
        }
    }
}
