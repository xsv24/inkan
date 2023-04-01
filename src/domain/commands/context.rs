use crate::domain::{
    adapters::{Git, Store},
    errors::Errors,
    models::Branch,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Context {
    /// Issue ticket number related to the current branch.
    pub ticket: Option<String>,
    /// Short describing a section of the codebase the changes relate to.
    pub scope: Option<String>,
    /// Issue ticket number link.
    pub link: Option<String>,
}

pub fn handler<G: Git, S: Store>(git: &G, store: &S, args: Context) -> Result<Branch, Errors> {
    // We want to store the branch name against and ticket number
    // So whenever we commit we get the ticket number from the branch
    let repo_name = git.repository_name().map_err(Errors::Git)?;

    let branch_name = git.branch_name().map_err(Errors::Git)?;

    let branch = Branch::new(&branch_name, &repo_name, args.ticket, args.link, args.scope);
    store
        .persist_branch(&branch)
        .map_err(Errors::PersistError)?;

    Ok(branch)
}
