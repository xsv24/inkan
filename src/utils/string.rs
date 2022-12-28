pub fn into_option<S: Into<String>>(value: S) -> Option<String> {
    let value: String = value.into();

    if !value.trim().is_empty() {
        Some(value)
    } else {
        None
    }
}

pub trait OptionStr<T> {
    fn map_empty_to_none(self) -> Option<T>;

    fn map_non_empty(self, map: fn(T) -> T) -> Option<T>;
}

impl OptionStr<String> for Option<String> {
    fn map_empty_to_none(self) -> Option<String> {
        match self {
            Some(value) if value.trim().is_empty() => None,
            Some(value) => Some(value.trim().to_string()),
            None => None,
        }
    }

    fn map_non_empty(self, map: fn(String) -> String) -> Option<String> {
        match self {
            Some(value) if value.trim().is_empty() => None,
            Some(value) => Some(map(value.trim().to_string())),
            None => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_strings_wrapped_in_some_should_be_mapped_to_none() {
        for item in ["", " ", "\t", "\n"] {
            let option = Some(item.into());
            let option = option.map_empty_to_none();

            assert!(option.is_none());
        }
    }

    #[test]
    fn non_empty_strings_wrapped_in_some_should_not_be_remapped_to_none() {
        for item in [" h ", "hello"] {
            let option = Some(item.into());
            let option = option.map_empty_to_none();

            assert!(option.is_some());
            assert_eq!(item.trim(), option.unwrap());
        }
    }
}
