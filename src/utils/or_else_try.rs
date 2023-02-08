pub trait OrElseTry<T> {
    fn or_else_try<F>(self, or: F) -> anyhow::Result<Option<T>>
    where
        F: FnOnce() -> anyhow::Result<Option<T>>;
}

impl<T> OrElseTry<T> for Option<T> {
    fn or_else_try<F>(self, or: F) -> anyhow::Result<Option<T>>
    where
        F: FnOnce() -> anyhow::Result<Option<T>>,
    {
        let result = match self {
            Some(value) => Some(value),
            None => or()?,
        };

        Ok(result)
    }
}
