pub struct SelectItem<T> {
    pub name: String,
    pub value: T,
    pub description: Option<String>,
}

pub trait Prompter {
    fn text(&self, question: &str) -> anyhow::Result<Option<String>>;

    fn select<T>(
        &self,
        question: &str,
        options: Vec<SelectItem<T>>,
    ) -> anyhow::Result<SelectItem<T>>;
}
