pub trait OrElseTry<T, E> {
    fn or_else_try<F>(self, or: F) -> Result<Option<T>, E>
    where
        F: FnOnce() -> Result<Option<T>, E>;
}

impl<T, E> OrElseTry<T, E> for Option<T> {
    fn or_else_try<F>(self, or: F) -> Result<Option<T>, E>
    where
        F: FnOnce() -> Result<Option<T>, E>,
    {
        let result = match self {
            Some(value) => Some(value),
            None => or()?,
        };

        Ok(result)
    }
}
