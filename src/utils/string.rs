pub fn into_option<S: Into<String>>(value: S) -> Option<String> {
    let value: String = value.into();

    if !value.is_empty() {
        Some(value)
    } else {
        None
    }
}
