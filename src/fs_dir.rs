extern crate fs_extra;
use fs_extra::dir as f_dir;
use std::cmp::Ordering;
use std::path::{Path, PathBuf};
use std::{env, error::Error, fs};
use walkdir::WalkDir;

use serde::{Deserialize, Serialize};

use crate::fs_file::{self as x_file, file_info, FileInfo};

pub fn check_or_create_dir(dir_str: &str) -> Result<(), Box<dyn std::error::Error>> {
    if !Path::new(&dir_str).exists() {
        std::fs::create_dir_all(&dir_str)?;
    }

    Ok(())
}

pub fn dir_is_empty<P: AsRef<Path>>(dir_path: P) -> Result<bool, Box<dyn std::error::Error>> {
    let entries = fs::read_dir(dir_path)?;
    Ok(entries.count() == 0)
}

pub fn get_current_dir() -> String {
    return get_current_dir_path().to_str().unwrap().into();
}

pub fn get_current_dir_path() -> PathBuf {
    return env::current_dir().unwrap();
}

pub fn get_sub_dir(root_path: &str) -> Result<Vec<String>, Box<dyn Error>> {
    let mut path_list = vec![root_path.to_string()];
    let mut start_index = 0;

    loop {
        let list_len = path_list.len();
        for index in start_index..path_list.len() {
            let path = &path_list[index];
            if fs::metadata(path)?.is_dir() {
                for child_dir in fs::read_dir(&path)? {
                    path_list.push(child_dir?.path().as_os_str().to_str().unwrap().to_string());
                }
            }
        }
        if list_len == start_index {
            break;
        }
        start_index = list_len;
    }
    return Ok(path_list);
}

pub fn get_parent_dir_path(path_str: &str) -> String {
    let dir = std::path::Path::new(&path_str);

    match dir.parent() {
        Some(parent_dir) => {
            return parent_dir
                .as_os_str()
                .to_os_string()
                .to_str()
                .unwrap()
                .to_string();
        }
        None => {
            return "".to_string();
        }
    }
}

// Including file type only
pub fn get_file_list(dir: &str) -> Result<Vec<String>, Box<dyn Error>> {
    let mut res: Vec<String> = Vec::new();

    let entries = fs::read_dir(dir)?;

    for entry in entries {
        let entry = entry.unwrap();
        res.push(entry.file_name().into_string().unwrap());
    }
    Ok(res)
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DirChildren {
    name: String,
    is_dir: bool,
    modified_time_stamp: f64,
}
impl Clone for DirChildren {
    fn clone(&self) -> Self {
        self.clone()
    }

    fn clone_from(&mut self, source: &Self) {
        *self = source.clone()
    }
}

// Including file and dir, if **is_dir** is true, it is a directory, or it is a file
pub fn get_children_list(dir: &str) -> Result<Vec<DirChildren>, Box<dyn Error>> {
    let _ = check_or_create_dir(dir);

    let mut res: Vec<DirChildren> = Vec::new();

    let children = fs::read_dir(&dir)?;

    for entry in children {
        let entry = entry.unwrap();
        let item = DirChildren {
            name: entry.file_name().into_string().unwrap(),
            is_dir: entry.path().is_dir(),
            modified_time_stamp: x_file::get_modified_time_f64(&entry),
        };
        res.push(item);
    }

    Ok(res)
}

pub fn get_dir_size(dir: &str) -> Result<u64, Box<dyn Error>> {
    let size = f_dir::get_size(dir)?;
    Ok(size)
}

pub fn tree_info_vec(
    dir_path: &str,
    exclude_dires: &Vec<String>,
) -> Result<Vec<FileInfo>, Box<dyn Error>> {
    let mut res: Vec<FileInfo> = [].to_vec();

    let mut excludes: Vec<String> = [].to_vec();
    for item in exclude_dires {
        excludes.push(
            Path::new(dir_path)
                .join(item)
                .to_str()
                .unwrap_or("")
                .to_owned(),
        )
    }

    for entry in WalkDir::new(dir_path) {
        match entry {
            Ok(entry) => {
                let p = entry.path();

                let mut skip = false;
                let ps = p.to_str().unwrap_or("").to_string();
                for e in &excludes {
                    let ed = format!("{}{}", e, std::path::MAIN_SEPARATOR_STR.to_string());
                    if ps.starts_with(&ed) {
                        // println!("skip file path: >>>{}<<< >>>{}<<<", &ps, &ed);
                        skip = true;
                    }
                }
                if skip {
                    continue;
                }

                if entry.file_type().is_dir() {
                    // println!("Directory: {}", ps);
                } else {
                    // println!("File:      {}", ps);

                    res.push(file_info(p));
                }
            }
            Err(_) => continue,
        };
    }
    return Ok(res);
}

#[test]
fn test_tree_info_flat() {
    let eee = ["van".to_string()].to_vec();
    let ttt = tree_info_vec("/xxx/images", &eee).unwrap();

    for item in ttt {
        println!("{}", item.path);
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileNode {
    pub key: String,
    pub title: String,
    pub children: Vec<FileNode>,
    pub is_dir: bool,
}

impl FileNode {
    pub fn sort_children(&mut self) {
        self.children.sort_by(|a, b| {
            // The order of directories takes precedence over files
            let a_is_dir = a.is_dir;
            let b_is_dir = b.is_dir;
            match (a_is_dir, b_is_dir) {
                (true, false) => Ordering::Less,
                (false, true) => Ordering::Greater,
                _ => a.title.cmp(&b.title),
            }
        });
    }

    pub fn new() -> FileNode {
        FileNode {
            key: "".to_string(),
            title: "".to_string(),
            children: [].to_vec(),
            is_dir: false,
        }
    }
}

impl Clone for FileNode {
    fn clone(&self) -> Self {
        FileNode {
            key: self.key.clone(),
            title: self.title.clone(),
            children: self.children.clone(),
            is_dir: self.is_dir,
        }
    }
}

pub fn tree_info(path: &Path, dir_prefix: &str) -> Result<FileNode, Box<dyn Error>> {
    let is_dir = path.is_dir();
    let key = if is_dir {
        format!("{}{}", dir_prefix, path.to_string_lossy())
    } else {
        path.to_string_lossy().to_string()
    };

    let mut node = FileNode {
        key,
        is_dir,
        title: path.file_name().unwrap().to_string_lossy().to_string(),
        children: Vec::new(),
    };

    if is_dir {
        for entry in fs::read_dir(path).unwrap() {
            let entry = entry.unwrap();
            let child_node = tree_info(&entry.path(), dir_prefix);
            if child_node.is_ok() {
                node.children.push(child_node.unwrap());
            };
        }

        node.sort_children();
    }

    return Ok(node);
}

#[test]
fn test_tree_info() {
    let root_path = Path::new("/xxx/fivim");
    let tree = tree_info(&root_path, "\r\r\r").unwrap();
    // println!("{:?}", tree);

    let j = serde_json::to_string(&tree).unwrap();
    println!("{}", j);
}
