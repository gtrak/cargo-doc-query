pub mod build;
pub mod expand;
pub mod query;

use anyhow::Result;

pub trait Command {
    fn execute(&self) -> Result<()>;
}

pub use expand::ExpandCommand;
