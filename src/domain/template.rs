use anyhow::Context;
use regex::Regex;

use crate::utils::string::OptionStr;

pub trait Templator {
    fn replace_or_remove(&self, target: &str, replace: Option<String>) -> anyhow::Result<String>;
}

fn brackets_regex(target: &str) -> anyhow::Result<Regex> {
    // Replace any surrounding brackets without content with an empty string and remove any trailing spaces.
    // ({target}) | [{target}] | {{target}} | {target}
    // example: http://regexr.com/75aee
    let regex = Regex::new(&format!(
        r"(\(\{{{target}\}}\)\s?)|(\[\{{{target}\}}\]\s?)|(\{{\{{{target}\}}\}}\s?)|(\{{{target}\}}\s?)"
    ))?;

    Ok(regex)
}

impl Templator for String {
    fn replace_or_remove(&self, target: &str, replace: Option<String>) -> anyhow::Result<String> {
        let template = format!("{{{target}}}");

        let message = match &replace.none_if_empty() {
            Some(value) => {
                log::info!("replace '{}' from template with '{}'", target, value);
                self.replace(&template, value)
            }
            None => {
                log::info!("removing '{}' from template", target);
                brackets_regex(target)
                    .with_context(|| format!("Invalid template for parameter '{target}'."))?
                    .replace_all(self, "")
                    .into()
            }
        };

        Ok(message.trim().into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn brackets_match() {
        let regex = brackets_regex("target").unwrap();
        assert!(regex.is_match("[{target}]"));
        assert!(regex.is_match("[{target}] "));
        assert!(regex.is_match("({target})"));
        assert!(regex.is_match("({target})\t"));
        assert!(regex.is_match("{{target}} "));
        assert!(regex.is_match("{target}"));
    }
}
