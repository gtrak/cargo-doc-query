pub mod build;
pub mod query;

use anyhow::Result;

pub trait Command {
    fn execute(&self) -> Result<()>;
}
