use std::{
    fs,
    path::{Path, PathBuf},
};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PathType {
    Directory,
    File,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum PathError {
    #[error("Expected an absolute path")]
    NotAbsolute,

    #[error("Expected an {expected:?} path but found {actual:?}")]
    InvalidType {
        expected: PathType,
        actual: PathType,
    },

    #[error("Invalid path does not exist")]
    NotExist,

    #[error("Failed to convert into a valid path")]
    Conversion,

    #[error("Found an invalid path")]
    Invalid,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AbsolutePath {
    path: PathBuf,
    path_type: PathType,
}

impl AbsolutePath {
    pub fn try_from(path: String, path_type: PathType) -> Result<Self, PathError> {
        let abs_path: Self = path.trim().to_string().try_into()?;

        if abs_path.path_type != path_type {
            return Err(PathError::InvalidType {
                expected: path_type,
                actual: abs_path.path_type,
            });
        }

        Ok(abs_path)
    }

    pub fn join(&self, path: &str, path_type: PathType) -> Result<Self, PathError> {
        let abs_path: Self = self.path.join(path).try_into()?;

        if abs_path.path_type != path_type {
            return Err(PathError::InvalidType {
                expected: path_type,
                actual: abs_path.path_type,
            });
        }

        Ok(abs_path)
    }
}

impl From<AbsolutePath> for PathBuf {
    fn from(value: AbsolutePath) -> Self {
        value.path
    }
}

impl ToString for AbsolutePath {
    fn to_string(&self) -> String {
        self.path.to_str().unwrap_or_default().into()
    }
}

impl TryInto<String> for &AbsolutePath {
    type Error = PathError;

    fn try_into(self) -> Result<String, Self::Error> {
        let path = self
            .path
            // Returns Err if the slice is not UTF-8 with a description as to why the provided slice is not UTF-8.
            .to_str()
            .ok_or(PathError::Invalid)?;

        Ok(path.into())
    }
}

impl TryInto<String> for AbsolutePath {
    type Error = PathError;

    fn try_into(self) -> Result<String, Self::Error> {
        (&self).try_into()
    }
}

impl TryInto<AbsolutePath> for PathBuf {
    type Error = PathError;

    fn try_into(self) -> Result<AbsolutePath, Self::Error> {
        match (
            self.exists(),
            self.is_dir(),
            self.is_file(),
            self.is_absolute(),
        ) {
            (true, _, true, true) => Ok(AbsolutePath {
                path: self,
                path_type: PathType::File,
            }),
            (true, true, _, true) => Ok(AbsolutePath {
                path: self,
                path_type: PathType::Directory,
            }),
            (false, _, _, _) => Err(PathError::NotExist),
            (true, _, _, false) => Err(PathError::NotAbsolute),
            _ => Err(PathError::Invalid),
        }
    }
}

impl TryInto<AbsolutePath> for String {
    type Error = PathError;

    fn try_into(self) -> Result<AbsolutePath, Self::Error> {
        let path = Path::new(&self);
        let absolute_path = fs::canonicalize(path).map_err(|e| {
            log::error!("Failed to convert string into valid absolute path: {}", e);
            PathError::Conversion
        })?;
        absolute_path.try_into()
    }
}

#[cfg(test)]
mod test {
    use std::path::PathBuf;

    use super::*;

    #[test]
    fn try_into_path_that_does_not_exists_fails_with_not_exist() {
        let not_existent_path = PathBuf::new();
        let error = TryInto::<AbsolutePath>::try_into(not_existent_path).unwrap_err();

        assert_eq!(error, PathError::NotExist);
    }

    #[test]
    fn try_into_non_absolute_path_errors() {
        let relative_path = PathBuf::from(".");
        let error = TryInto::<AbsolutePath>::try_into(relative_path).unwrap_err();

        assert_eq!(error, PathError::NotAbsolute);
    }

    #[test]
    fn try_into_with_absolute_directory_path_maps_to_absolute_path_type() {
        let dir_path = valid_dir_path();
        let abs_path = TryInto::<AbsolutePath>::try_into(dir_path.clone()).unwrap();

        assert_eq!(
            abs_path,
            AbsolutePath {
                path: dir_path,
                path_type: PathType::Directory
            }
        )
    }

    #[test]
    fn try_into_with_absolute_file_path_maps_to_absolute_path_type() {
        let file_path = valid_file_path();
        let abs_path = TryInto::<AbsolutePath>::try_into(file_path.clone()).unwrap();

        assert_eq!(
            abs_path,
            AbsolutePath {
                path: file_path,
                path_type: PathType::File
            }
        )
    }

    #[test]
    fn try_from_allow_directory_paths_when_path_type_is_directory() {
        let dir_path_buf = valid_dir_path();
        let dir_path = dir_path_buf.as_os_str().to_str().unwrap();
        let abs_path =
            AbsolutePath::try_from(dir_path.to_owned().clone(), PathType::Directory).unwrap();

        assert_eq!(
            abs_path,
            AbsolutePath {
                path: dir_path_buf,
                path_type: PathType::Directory
            }
        )
    }

    #[test]
    fn try_from_errors_when_given_file_paths_and_path_type_is_directory() {
        let file_path_buf = valid_file_path();
        let file_path = file_path_buf.as_os_str().to_str().unwrap();
        let error =
            AbsolutePath::try_from(file_path.to_owned().clone(), PathType::Directory).unwrap_err();

        assert_eq!(
            error,
            PathError::InvalidType {
                expected: PathType::Directory,
                actual: PathType::File
            }
        )
    }

    #[test]
    fn try_from_allow_file_paths_when_path_type_is_file() {
        let file_path_buf = valid_file_path();
        let file_path = file_path_buf.as_os_str().to_str().unwrap();
        let abs_path =
            AbsolutePath::try_from(file_path.to_owned().clone(), PathType::File).unwrap();

        assert_eq!(
            abs_path,
            AbsolutePath {
                path: file_path_buf,
                path_type: PathType::File
            }
        )
    }

    #[test]
    fn try_from_errors_when_given_directory_paths_and_path_type_is_file() {
        let dir_path_buf = valid_dir_path();
        let dir_path = dir_path_buf.as_os_str().to_str().unwrap();
        let error =
            AbsolutePath::try_from(dir_path.to_owned().clone(), PathType::File).unwrap_err();

        assert_eq!(
            error,
            PathError::InvalidType {
                expected: PathType::File,
                actual: PathType::Directory
            }
        )
    }

    fn valid_dir_path() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    }

    fn valid_file_path() -> PathBuf {
        let path = valid_dir_path();
        path.join("templates/default.yml")
    }
}
