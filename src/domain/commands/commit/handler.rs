use std::path::PathBuf;

use crate::{
    domain::{
        adapters::{CommitMsgStatus, Git, Store},
        errors::Errors,
    },
    utils::string::OptionStr,
};

use super::Commit;

pub fn handler<G: Git, S: Store>(git: &G, store: &S, commit: Commit) -> Result<String, Errors> {
    let branch_name = git.branch_name().map_err(Errors::Git)?;
    let repo_name = git.repository_name().map_err(Errors::Git)?;

    let branch = store.get_branch(&branch_name, &repo_name).ok();

    let contents = commit
        .commit_message(commit.template.content.clone(), branch)
        .map_err(|e| Errors::Configuration {
            message: "Failed attempting to build commit message".into(),
            source: e,
        })?;

    let template_file = git.template_file_path().map_err(Errors::Git)?;

    std::fs::write(&template_file, &contents).map_err(|_| Errors::ValidationError {
        message: "Failed attempting to write commit template file".into(),
    })?;

    // Pre-cautionary measure encase 'message' is provided but still matches template exactly.
    // Otherwise git will just abort the commit if theres no difference / change from the template.
    let commit_msg_complete = match commit.message.none_if_empty() {
        Some(_) => CommitMsgStatus::Completed,
        None => CommitMsgStatus::InComplete,
    };

    git.commit_with_template(&template_file, commit_msg_complete)
        .map_err(Errors::Git)?;

    Ok(contents)
}
