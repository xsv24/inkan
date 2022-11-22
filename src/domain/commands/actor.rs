use crate::{
    cli::{checkout, commit, context},
    domain::models::Branch,
};

pub trait Actor {
    /// Actions on a context update on the current branch.
    fn current(&self, args: context::Arguments) -> anyhow::Result<Branch>;

    /// Actions on a checkout of a new or existing branch.
    fn checkout(&self, args: checkout::Arguments) -> anyhow::Result<Branch>;

    /// Actions on a commit.
    fn commit(&self, args: commit::Arguments) -> anyhow::Result<String>;
}
