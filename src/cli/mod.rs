pub mod args;
pub mod build;
pub mod commands;
pub mod expand;

use anyhow::Result;

pub trait Command {
    fn execute(&self) -> Result<()>;
}

