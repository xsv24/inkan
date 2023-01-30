use crate::{
    domain::adapters::{CommitMsgStatus, Git, Store},
    utils::string::OptionStr,
};

use super::Commit;

pub fn handler<G: Git, S: Store>(git: &G, store: &S, commit: Commit) -> anyhow::Result<String> {
    let branch = store
        .get_branch(&git.branch_name()?, &git.repository_name()?)
        .ok();

    let contents = commit.commit_message(commit.template.content.clone(), branch)?;

    let template_file = git.template_file_path()?;
    std::fs::write(&template_file, &contents)?;

    // Pre-cautionary measure encase 'message' is provided but still matches template exactly.
    // Otherwise git will just abort the commit if theres no difference / change from the template.
    let commit_msg_complete = match commit.message.none_if_empty() {
        Some(_) => CommitMsgStatus::Completed,
        None => CommitMsgStatus::InComplete,
    };

    git.commit_with_template(&template_file, commit_msg_complete)?;

    Ok(contents)
}
