use crate::{domain::{
    adapters::{CheckoutStatus, Git, Store},
    errors::Errors,
    models::Branch, template::Templator,
}, template_config::TemplateConfig};

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

fn build_branch_name(args: &Checkout, template: &TemplateConfig) -> anyhow::Result<String> {
    let args = args.clone();

    let mut contents = template.branch.content
        .replace_or_remove("name", Some(args.name))?
        .replace_or_remove("ticket_num", args.ticket)?
        .replace_or_remove("scope", args.scope)?;

    if let Some(params) = &template.params {
        for (key, value)in params {
            contents = contents.replace_or_remove(key, Some(value.clone()))?;
        }
    } 

    Ok(contents)
}

pub fn handler<G: Git, S: Store>(
    git: &G,
    store: &S,
    template: TemplateConfig,
    args: Checkout
) -> Result<Branch, Errors> {
    // Build name
    let name = build_branch_name(&args, &template)
        .map_err(|e| Errors::ValidationError {
                message: "Failed to build branch name from the specified config".into(),
                source: Some(e)
            }
    )?;

    // Attempt to create branch
    let create = git.checkout(&name, CheckoutStatus::New);

    // If the branch already exists check it out
    if let Err(err) = create {
        log::error!("failed to create new branch: {}", err);

        git.checkout(&args.name, CheckoutStatus::Existing)
            .map_err(Errors::Git)?;
    }

    // We want to store the branch name against and ticket number
    // So whenever we commit we get the ticket number from the branch
    let repo_name = git.repository_name().map_err(Errors::Git)?;

    let branch = Branch::new(&args.name, &repo_name, args.ticket, args.link, args.scope);
    store
        .persist_branch(&branch)
        .map_err(Errors::PersistError)?;

    Ok(branch)
}
