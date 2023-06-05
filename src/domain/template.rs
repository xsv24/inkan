use std::collections::HashMap;

use crate::utils::string::OptionStr;
use anyhow::Ok;

pub trait Templator {
    fn replace_or_remove_pairs<S1: Into<String>, S2: Into<String>>(
        &self,
        pairs: HashMap<S1, Option<S2>>,
    ) -> anyhow::Result<String>;

    fn replace_or_remove(&self, target: &str, replace: Option<String>) -> anyhow::Result<String>;
}

fn replace_none(this: &str, target: &str) -> String {
    log::info!("removing '{}' from template", target);
    let template = &format!("{{{target}}}");

    // TODO: Make this more efficient with one round of matching
    // Strip any prefix or postfix matches to simple template
    // so rules order are not as important making matches easier.
    let removed_prefixes_n_postfixes = this
        .replace(&format!("[{template}]"), template)
        .replace(&format!("({template})"), template)
        .replace(&format!("{{{template}}}"), template)
        // Exceptions to the rule and ordering is important
        // to prevent squashing "hi-{target}-bye" => "hibye"
        .replace(&format!("-{template}-"), "-")
        .replace(&format!("_{template}_"), "_")
        .replace(&format!("/{template}/"), "/");

    let remove_separate_joiners = removed_prefixes_n_postfixes
        .replace(&format!("-{template}"), template)
        .replace(&format!("{template}-"), template)
        .replace(&format!("_{template}"), template)
        .replace(&format!("{template}_"), template)
        .replace(&format!("/{template}"), template)
        .replace(&format!("{template}/"), template)
        .replace(&format!("{template} "), template);

    // Finally replace all basic converted templates with empty string
    remove_separate_joiners.replace(template, "").trim().into()
}

impl Templator for String {
    fn replace_or_remove_pairs<S1: Into<String>, S2: Into<String>>(
        &self,
        pairs: HashMap<S1, Option<S2>>,
    ) -> anyhow::Result<String> {
        let mut contents = String::from(self);
        for (target, replacement) in pairs {
            let option: Option<String> = replacement.map(|v| v.into());
            contents = contents.replace_or_remove(&target.into(), option)?;
        }

        Ok(contents)
    }

    fn replace_or_remove(&self, target: &str, replace: Option<String>) -> anyhow::Result<String> {
        let template = format!("{{{target}}}");

        let message = match &replace.none_if_empty() {
            Some(value) => {
                log::info!("replace '{}' from template with '{}'", target, value);
                self.replace(&template, value)
            }
            None => replace_none(self, target),
        };

        Ok(message.trim().into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn brackets_match() {
        let templates = [
            "[{target}]",
            "[{target}] ",
            "[{target}]\t",
            "({target})",
            "({target}) ",
            "({target})\t",
            "{{target}}",
            "{{target}} ",
            "{{target}}\t",
            "[{target}]({target}){{target}}",
        ];

        for template in templates {
            assert!(replace_none(template, "target").is_empty())
        }
    }

    #[test]
    fn hanging_connectors_are_removed() {
        let templates = [
            ("-{target}-", "-"),
            ("hi-{target}-bye", "hi-bye"),
            ("_{target}_", "_"),
            ("hi_{target}_bye", "hi_bye"),
            ("-{target}", ""),
            ("{target}-", ""),
            ("_{target}", ""),
            ("{target}_", ""),
        ];

        for (template, expected) in templates {
            assert_eq!(replace_none(template, "target"), expected);
        }
    }
}
