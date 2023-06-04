use std::collections::HashMap;

use crate::{
    domain::{
        adapters::{CheckoutStatus, Git, Store},
        errors::Errors,
        models::Branch,
        template::Templator,
    },
    template_config::TemplateConfig,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Checkout {
    /// Name of the branch to checkout or create.
    pub name: String,
    /// Issue ticket number related to the branch.
    pub ticket: Option<String>,
    /// Short describing a section of the codebase the changes relate to.
    pub scope: Option<String>,
    /// Issue ticket number link.
    pub link: Option<String>,
}

impl From<Checkout> for HashMap<&str, Option<String>> {
    fn from(value: Checkout) -> Self {
        HashMap::from([
            ("branch_name", Some(value.name)),
            ("ticket_num", value.ticket),
            ("scope", value.scope),
        ])
    }
}

fn build_branch_name(args: &Checkout, template: &TemplateConfig) -> anyhow::Result<String> {
    let args = args.clone();

    if template.branch.is_none() {
        return Ok(args.name);
    }

    let branch_template = template
        .branch
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("Expected valid template branch"))?;

    let contents = branch_template
        .content
        .replace_or_remove_pairs(args.into())?;

    Ok(contents)
}

pub fn handler<G: Git, S: Store>(
    git: &G,
    store: &S,
    template: TemplateConfig,
    args: Checkout,
) -> Result<Branch, Errors> {
    // Build name
    let name = build_branch_name(&args, &template).map_err(|e| Errors::ValidationError {
        message: "Failed to build branch name from the specified config".into(),
        source: Some(e),
    })?;

    // Attempt to create branch
    let create = git.checkout(&name, CheckoutStatus::New);

    // If the branch already exists check it out
    if let Err(err) = create {
        log::error!("failed to create new branch: {}", err);

        git.checkout(&name, CheckoutStatus::Existing)
            .map_err(Errors::Git)?;
    }

    // We want to store the branch name against and ticket number
    // So whenever we commit we get the ticket number from the branch
    let repo_name = git.repository_name().map_err(Errors::Git)?;

    let branch = Branch::new(&name, &repo_name, args.ticket, args.link, args.scope);

    store
        .persist_branch(&branch)
        .map_err(Errors::PersistError)?;

    Ok(branch)
}
