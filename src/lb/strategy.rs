use crate::lb::backend::Backend;
use anyhow::Result;

pub trait SelectionStrategy: Send + Sync + 'static {
    fn select<'a>(&self, backends: &'a [Backend]) -> Result<&'a Backend>;
}
