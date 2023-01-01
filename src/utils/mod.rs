pub mod file;
mod merge;
pub mod string;
mod try_convert;

pub use file::get_file_contents;
pub use merge::merge;
pub use try_convert::TryConvert;
