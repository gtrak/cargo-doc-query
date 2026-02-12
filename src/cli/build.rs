use crate::cli::Command;
use anyhow::Result;

pub struct BuildCommand;

impl Command for BuildCommand {
    fn execute(&self) -> Result<()> {
        println!("Build command executed");
        Ok(())
    }
}
