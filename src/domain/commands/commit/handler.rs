use crate::{
    domain::{
        adapters::{CommitMsgStatus, Git},
        errors::Errors,
        models::Branch,
    },
    utils::string::OptionStr,
};

use super::Commit;

pub fn handler<G: Git>(git: &G, branch: Option<Branch>, commit: Commit) -> Result<String, Errors> {
    let contents = commit
        .commit_message(commit.template.content.clone(), branch)
        .map_err(|e| Errors::Configuration {
            message: "Failed attempting to build commit message".into(),
            source: e,
        })?;

    let template_file = git.template_file_path().map_err(Errors::Git)?;

    std::fs::write(&template_file, &contents).map_err(|e| Errors::ValidationError {
        message: "Failed attempting to write commit template file".into(),
        source: Some(anyhow::anyhow!(e)),
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
