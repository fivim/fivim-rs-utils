use chrono::{DateTime, FixedOffset, TimeZone, Utc};
use filetime::{set_file_mtime, FileTime};
use std::error::Error;
use std::path::PathBuf;
use std::{
    fs::{self, DirEntry, File},
    io::{self, Read, Write},
    path::Path,
    time::SystemTime,
};

use crate::{datetime::get_timezone_offset_of_utc, fs as x_fs};

use base64::engine::{general_purpose::STANDARD as b64_STANDARD, Engine};

#[derive(Debug, Clone)]
pub struct FileInfo {
    pub name: String,
    pub path: String,
    pub size: u64,
    pub created: DateTime<FixedOffset>,
    pub accessed: DateTime<FixedOffset>,
    pub modified: DateTime<FixedOffset>,
    pub err_msg: String,
}

impl FileInfo {
    pub fn new() -> FileInfo {
        let empty_time = FixedOffset::west_opt(0)
            .unwrap()
            .with_ymd_and_hms(1970, 1, 1, 0, 0, 0)
            .unwrap();

        FileInfo {
            name: "".to_string(),
            path: "".to_string(),
            size: 0,
            created: empty_time,
            accessed: empty_time,
            modified: empty_time,
            err_msg: "".to_string(),
        }
    }
}

pub fn get_file_size(file_path: &str) -> u64 {
    match fs::metadata(&file_path) {
        Ok(metadata) => return metadata.len(),
        Err(_) => return 0,
    }
}

pub fn read_to_string(file_path: &str) -> Result<String, Box<dyn Error>> {
    if !x_fs::exists(&file_path) {
        return Ok("".to_string());
    };

    let mut file = File::open(&file_path)?;
    let mut s = "".to_string();
    let _ = file.read_to_string(&mut s)?;

    Ok(s)
}

pub fn read_to_bytes(file_path: &str, log_open_err: bool) -> Result<Vec<u8>, Box<dyn Error>> {
    if !x_fs::exists(&file_path) {
        return Ok([].to_vec());
    };

    let mut file = File::open(&file_path)?;

    let mut buf: Vec<u8> = vec![0; get_file_size(&file_path) as usize];
    let _ = file.read(&mut buf)?;
    Ok(buf)
}

pub fn write_str(file_path: &str, file_content: &str) -> Result<(), Box<dyn Error>> {
    let _ = crate::fs::check_or_create_dir(x_fs::get_parent_dir_path(file_path).as_str());

    let mut file = File::create(&file_path)?;

    file.write_all(file_content.as_bytes())?;
    Ok(())
}

pub fn write_bytes(file_path: &str, file_content: &Vec<u8>) -> Result<(), Box<dyn Error>> {
    let _ = crate::fs::check_or_create_dir(x_fs::get_parent_dir_path(&file_path).as_str());

    let mut file = File::create(&file_path)?;

    file.write_all(&file_content)?;
    Ok(())
}

pub fn write_base64_str(file_path: &str, file_content_base64: &str) -> Result<(), Box<dyn Error>> {
    let file_content_string: String;

    // Remove the prefix of base64 string, such as: "data:image/png;base64,"
    let separator = ";base64,";
    if file_content_base64.contains(separator) {
        let arr: Vec<&str> = file_content_base64.split(separator).collect();
        if arr.len() >= 2 {
            file_content_string = arr[1].to_string();
        } else {
            file_content_string = "".to_string();
        }
    } else {
        file_content_string = file_content_base64.to_owned();
    }

    let file_content = b64_STANDARD.decode(file_content_string)?;
    let mut file = File::create(&file_path)?;
    file.write_all(&file_content)?;

    Ok(())
}

pub fn get_modified_time_f64(entry: &DirEntry) -> f64 {
    let mmm = entry.metadata().unwrap();
    let modified = mmm.modified().unwrap();

    if let Ok(time) = modified.duration_since(SystemTime::UNIX_EPOCH) {
        time.as_secs_f64()
    } else {
        0.0
    }
}

pub fn get_modified(file_path: &str) -> SystemTime {
    match fs::metadata(&file_path) {
        Ok(metadata) => return metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH),
        Err(_) => {
            SystemTime::UNIX_EPOCH
        }
    }
}

pub fn set_modified_from_iso8601(file_path: &str, iso8601_str: &str) -> Result<(), std::io::Error> {
    let parsed_date: DateTime<Utc> = match DateTime::parse_from_rfc3339(iso8601_str) {
        Ok(dt) => dt.into(),

        Err(e) => return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, e)),
    };

    let file_time = FileTime::from_unix_time(parsed_date.timestamp(), 0);
    let path = PathBuf::from(file_path);

    set_file_mtime(&path, file_time)?;

    Ok(())
}

pub fn get_accessed(file_path: &str) -> SystemTime {
    match fs::metadata(&file_path) {
        Ok(metadata) => return metadata.accessed().unwrap_or(SystemTime::UNIX_EPOCH),
        Err(_) => {
            SystemTime::UNIX_EPOCH
        }
    }
}

pub fn get_created(file_path: &str) -> SystemTime {
    match fs::metadata(&file_path) {
        Ok(metadata) => return metadata.created().unwrap_or(SystemTime::UNIX_EPOCH),
        Err(_) => {
            SystemTime::UNIX_EPOCH
        }
    }
}

fn file_time(rtm: io::Result<SystemTime>) -> DateTime<FixedOffset> {
    let stm = rtm.unwrap_or(SystemTime::UNIX_EPOCH);
    let utm: DateTime<Utc> = stm.into();

    let offset_sec = get_timezone_offset_of_utc();
    let offset: FixedOffset;
    if offset_sec > 0 {
        offset = FixedOffset::east_opt(offset_sec).unwrap();
    } else {
        offset = FixedOffset::west_opt(offset_sec).unwrap();
    }

    utm.with_timezone(&offset)
}

pub fn file_info(file_path: &Path) -> FileInfo {
    let mut item = FileInfo::new();
    match fs::metadata(&file_path) {
        Ok(metadata) => {
            let default_name = "File_name_with_encoding_exception";
            item.name = match file_path.file_name() {
                Some(ost) => ost.to_str().unwrap_or(default_name).to_owned(),
                None => "Unable_to_obtain_file_name".to_owned(),
            };
            item.path = file_path.to_str().unwrap_or("").to_owned();
            item.accessed = file_time(metadata.accessed());
            item.created = file_time(metadata.created());
            item.modified = file_time(metadata.modified());
            item.size = metadata.len();
            item
        }
        Err(e) => {
            item.err_msg = e.to_string();
            item
        }
    }
}

#[test]
fn test_fi() {
    let fi = file_info(&Path::new(
        "/xxx/files_index.txt",
    ));
    println!("{}", fi.modified.format("%Y-%m-%d %H:%M:%S.%f %z"));
}
