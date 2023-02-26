use std::fmt::Display;

use crate::{
    domain::adapters::prompt::{Prompter, SelectItem},
    utils::string::OptionStr,
};
use colored::Colorize;
use inquire::{
    ui::{Attributes, Color, RenderConfig, StyleSheet, Styled},
    Select, Text,
};

impl<T> Display for SelectItem<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {}",
            self.name.green(),
            self.description.clone().unwrap_or_default().italic()
        )
    }
}

pub struct Prompt;

impl Prompt {
    fn filter<T>(input: &str, option: &SelectItem<T>) -> bool {
        option.name.to_lowercase().contains(&input.to_lowercase())
    }

    fn get_render_config() -> RenderConfig {
        // inquire::set_global_render_config(get_render_config());
        RenderConfig {
            highlighted_option_prefix: Styled::new("➜").with_fg(Color::LightBlue),
            selected_checkbox: Styled::new("✅").with_fg(Color::LightGreen),
            unselected_checkbox: Styled::new("🔳"),
            default_value: StyleSheet::new()
                .with_attr(Attributes::ITALIC)
                .with_fg(Color::DarkGrey),
            ..RenderConfig::default()
        }
    }
}

impl Prompter for Prompt {
    fn select<T>(
        &self,
        question: &str,
        options: Vec<SelectItem<T>>,
    ) -> anyhow::Result<SelectItem<T>> {
        let len = options.len();
        let select: Select<SelectItem<T>> = Select {
            message: question,
            options,
            help_message: Select::<SelectItem<T>>::DEFAULT_HELP_MESSAGE,
            page_size: len,
            vim_mode: Select::<SelectItem<T>>::DEFAULT_VIM_MODE,
            starting_cursor: Select::<SelectItem<T>>::DEFAULT_STARTING_CURSOR,
            filter: &|input, option, _, _| Self::filter(input, option),
            formatter: Select::DEFAULT_FORMATTER,
            render_config: Self::get_render_config(),
        };

        Ok(select.prompt()?)
    }

    fn text(&self, question: &str, default: Option<String>) -> anyhow::Result<Option<String>> {
        let result = Text::new(question)
            .with_default(&default.unwrap_or("".into()))
            .with_render_config(Self::get_render_config())
            .prompt_skippable()?;

        Ok(result.none_if_empty())
    }
}

#[cfg(test)]
mod tests {
    use super::Prompt;
    use crate::domain::adapters::prompt::SelectItem;
    use fake::{Fake, Faker};

    #[test]
    fn non_matching_are_filtered_out() {
        let item = SelectItem {
            name: Faker.fake(),
            description: Faker.fake(),
            value: "value",
        };

        assert_eq!(false, Prompt::filter("invalid", &item));
        assert_eq!(false, Prompt::filter("valui", &item));
    }

    #[test]
    fn filter_matches_on_contains_value() {
        let item = SelectItem {
            name: Faker.fake(),
            description: Faker.fake(),
            value: "VALUE",
        };

        assert_eq!(false, Prompt::filter("VALUE", &item));
        assert_eq!(false, Prompt::filter("value", &item));
        assert_eq!(false, Prompt::filter("Contains value in the string", &item));
    }
}
