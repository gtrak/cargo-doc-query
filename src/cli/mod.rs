pub mod build;

use anyhow::Result;

pub trait Command {
    fn execute(&self) -> Result<()>;
}
