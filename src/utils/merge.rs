pub fn merge<T>(preferred: Option<T>, default: Option<T>) -> Option<T> {
    match (preferred, default) {
        (None, None) => None,
        (None, Some(def)) => Some(def),
        (Some(pref), None) => Some(pref),
        (Some(pref), Some(_)) => Some(pref),
    }
}
