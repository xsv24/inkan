pub mod file;
pub mod string;
mod try_convert;

pub use file::get_file_contents;
pub use try_convert::TryConvert;
