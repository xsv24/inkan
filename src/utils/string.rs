pub fn into_option<S: Into<String>>(value: S) -> Option<String> {
    let value: String = value.into().trim().to_string();

    if !value.is_empty() {
        Some(value)
    } else {
        None
    }
}

pub trait OptionStr<T> {
    fn none_if_empty(self) -> Option<T>;
}

impl OptionStr<String> for Option<String> {
    fn none_if_empty(self) -> Option<String> {
        match self {
            Some(value) => into_option(value),
            None => None,
        }
    }
}

impl OptionStr<String> for String {
    fn none_if_empty(self) -> Option<String> {
        if self.is_empty() {
            None
        } else {
            Some(self)
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
            let option = option.none_if_empty();

            assert!(option.is_none());
        }
    }

    #[test]
    fn non_empty_strings_wrapped_in_some_should_not_be_remapped_to_none() {
        for item in [" h ", "hello"] {
            let option = Some(item.into());
            let option = option.none_if_empty();

            assert!(option.is_some());
            assert_eq!(item.trim(), option.unwrap());
        }
    }
}
