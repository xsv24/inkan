pub struct SelectItem<T> {
    pub name: String,
    pub value: T,
    pub description: Option<String>,
}

pub trait Prompter {
    fn select<T>(
        &self,
        question: &str,
        options: Vec<SelectItem<T>>,
    ) -> anyhow::Result<SelectItem<T>>;
}
