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

/// Search the documents in the directory, the keywords can be a RE or  a plain text, the keywords text in result can be highlighted.
///  - `dir_path` the directory path
/// 
/// # Arguments
/// - `pattern`: RE pattern
/// - `wrapper`: the wrapper or the keywords text in result, like "<span>\t\t\t</span>", the "\t\t\t" is the placehodler, and it will replaced with the keywords text
/// - `context_size`: text size of the context of the keywords text
pub fn search_document(
    dir_path: &Path,
    use_re: bool,
    search: &str,
    wrapper: &str,
    context_size: usize,
) -> Result<Vec<String>, io::Error> {
    let mut results = Vec::new();

    fn process_file(
        file_path: &Path,
        re: &Regex,  // RE
        plain: &str, // if not empty, query like a plaintext
        wrapper: &str,
        context_size: usize,
    ) -> Result<Vec<String>, io::Error> {
        let file = File::open(file_path)?;

        let reader = BufReader::new(file);
        let mut results222 = Vec::new();

        for line_res in reader.lines() {
            let line = line_res?;

            let mut left_context = "";
            let mut matched_text = "";
            let mut right_context = "";

            // RE mode
            if plain == "" {
                let caps = match re.captures(&line) {
                    Some(ccc) => ccc,
                    None => continue,
                };

                let capture_start = caps.get(0).unwrap().start();
                let capture_end = caps.get(0).unwrap().end();
                let left_context_start =
                    std::cmp::max(0, capture_start.saturating_sub(context_size));
                let right_context_end = std::cmp::min(line.len(), capture_end + context_size);
                left_context = &line[left_context_start..capture_start];
                matched_text = caps.get(0).unwrap().as_str();
                right_context = &line[capture_end..right_context_end];
            } else {
                // plaintext mode
                if line.contains(plain) {
                    matched_text = plain;

                    let index_p_s = line.find(plain).unwrap_or(0);
                    let mut start_index = 0;
                    if index_p_s > 0 && index_p_s >= context_size {
                        start_index = index_p_s - context_size;
                    }
                    left_context = &line[start_index..index_p_s];

                    let index_p_e = line.find(plain).unwrap_or(0) + plain.len();
                    let mut end_index = context_size;
                    let end_index__ = index_p_e + context_size;
                    if end_index__ > line.len() {
                        end_index = line.len()
                    }
                    right_context = &line[index_p_e..end_index];
                }
            }

            if matched_text == "" {
                continue;
            }

            let highlighted_line = format!(
                "{}{}{}",
                left_context,
                wrapper.replace("\t\t\t", matched_text),
                right_context
            );

            if line != highlighted_line {
                results222.push(highlighted_line);
            }
        }

        Ok(results222)
    }

    let mut plain = "";
    let mut re: Regex = Regex::new("").unwrap();
    if use_re {
        re = match Regex::new(search) {
            Ok(r) => r,
            Err(e) => return Err(io::Error::new(ErrorKind::InvalidInput, e)),
        };
    } else {
        plain = search;
    }

    for entry_result in fs::read_dir(dir_path)? {
        let entry = entry_result?;
        let path = entry.path();

        if path.is_file() {
            results.extend(process_file(&path, &re, plain, wrapper, context_size)?);
        } else if path.is_dir() {
            results.extend(search_document(
                &path,
                use_re,
                search,
                wrapper,
                context_size,
            )?);
        }
    }

    Ok(results)
}

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    use std::{io::Error, path::Path};

    use crate::fs::search_document;

    #[test]
    fn test_search_and_highlight_document() -> Result<(), Error> {
        let dir_path = Path::new("/xxx");

        let pattern1 = "IPX";
        let wrapper = "<span>\t\t\t</span>";
        let context_size = 10;
        let results = search_document(&dir_path, true, pattern1, wrapper, context_size)?;

        println!(">>> results of RE mode :::");

        let pattern2 = "2535523";
        for result in results {
            println!("{}", result);
        }

        let results = search_document(&dir_path, false, pattern2, wrapper, context_size)?;

        println!("\n\n");

        println!(">>> results of none-RE mode :::");

        for result in results {
            println!("{}", result);
        }

        Ok(())
    }
}