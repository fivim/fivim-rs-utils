use log::error;
use regex::Regex;
use std::fs::{self, File};
use std::io::{self, BufRead, BufReader, ErrorKind};
use std::path::{Path, PathBuf};

pub use crate::fs_dir::check_or_create_dir;
pub use crate::fs_dir::dir_is_empty;
pub use crate::fs_dir::get_children_list;
pub use crate::fs_dir::get_current_dir;
pub use crate::fs_dir::get_current_dir_path;
pub use crate::fs_dir::get_dir_size;
pub use crate::fs_dir::get_file_list;
pub use crate::fs_dir::get_parent_dir_path;
pub use crate::fs_dir::get_sub_dir;
pub use crate::fs_dir::tree_info;
pub use crate::fs_dir::tree_info_vec;
pub use crate::fs_dir::DirChildren;
pub use crate::fs_dir::FileNode;

pub use crate::fs_file::file_info;
pub use crate::fs_file::get_accessed;
pub use crate::fs_file::get_created;
pub use crate::fs_file::get_file_size;
pub use crate::fs_file::get_modified;
pub use crate::fs_file::get_modified_time_f64;
pub use crate::fs_file::read_to_bytes;
pub use crate::fs_file::read_to_string;
pub use crate::fs_file::set_modified_from_iso8601;
pub use crate::fs_file::write_base64_str;
pub use crate::fs_file::write_bytes;
pub use crate::fs_file::write_str;
pub use crate::fs_file::FileInfo;

pub fn exists(path_str: &str) -> bool {
    return Path::new(&path_str).exists();
}

pub fn path_buf_to_string(path_buf: PathBuf) -> String {
    return path_buf.into_os_string().into_string().unwrap();
}
