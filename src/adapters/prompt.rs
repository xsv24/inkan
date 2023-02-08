use std::fmt::Display;

use colored::Colorize;
use inquire::{
    ui::{Color, RenderConfig, Styled},
    Select, Text,
};

use crate::{
    domain::adapters::prompt::{Prompter, SelectItem},
    utils::string::OptionStr,
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
    fn get_render_config() -> RenderConfig {
        // inquire::set_global_render_config(get_render_config());
        RenderConfig {
            highlighted_option_prefix: Styled::new("➜").with_fg(Color::LightBlue),
            selected_checkbox: Styled::new("✅").with_fg(Color::LightGreen),
            unselected_checkbox: Styled::new("🔳"),
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
            filter: Select::DEFAULT_FILTER,
            formatter: Select::DEFAULT_FORMATTER,
            render_config: Self::get_render_config(),
        };

        Ok(select.prompt()?)
    }

    fn text(&self, question: &str) -> anyhow::Result<Option<String>> {
        let result = Text::new(question).prompt_skippable()?;
        Ok(result.none_if_empty())
    }
}
