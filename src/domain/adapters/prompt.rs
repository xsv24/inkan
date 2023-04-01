use crate::domain::errors::UserInputError;

pub struct SelectItem<T> {
    pub name: String,
    pub value: T,
    pub description: Option<String>,
}

pub trait Prompter {
    fn text(
        &self,
        question: &str,
        default: Option<String>,
    ) -> Result<Option<String>, UserInputError>;

    fn select<T>(
        &self,
        question: &str,
        options: Vec<SelectItem<T>>,
    ) -> Result<SelectItem<T>, UserInputError>;
}
