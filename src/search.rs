use html_escape::decode_html_entities;
use log::debug;
use log::error;
use regex::Regex;
use serde::Deserialize;
use serde::Serialize;
use std::cmp::max;
use std::cmp::min;
use std::fs;
use std::fs::File;
use std::io;
use std::io::BufReader;
use std::io::ErrorKind;
use std::io::Read;
use std::path::PathBuf;

use crate::fs::path_buf_to_string;

#[derive(Serialize, Deserialize, Debug)]
pub struct SearchFileRes {
    path: String,
    matches: Vec<String>,
}

pub type SearchCallback = dyn Fn(PathBuf, String) -> String;
const EXT_HTML: &str = "html";
const EXT_XRTM: &str = "xrtm";

fn prepare_html_like_file(file_path: PathBuf, ext: String) -> String {
    let mut string = String::new();

    let file = match File::open(&file_path) {
        Ok(sss) => sss,
        Err(e) => {
            error!("File::open error {}", e);
            return "".to_owned();
        }
    };
    let mut reader = BufReader::new(file);
    match reader.read_to_string(&mut string) {
        Ok(_) => "",
        Err(_) => "",
    };

    let mut pure_text = "".to_string();
    if &ext == EXT_HTML {
        let delete_tages = ["head".to_string(), "script".to_string()].to_vec();
        pure_text = process_html(&string, &delete_tages);
    }
    if &ext == EXT_XRTM {
        let delete_tages = [
            "head".to_string(),
            "script".to_string(),
            "scalable_block".to_string(),
        ]
        .to_vec();
        pure_text = process_html(&string, &delete_tages);
    }

    pure_text
}

fn extract_multibyte_safe(s: &str, start: usize, end: usize) -> Result<String, &'static str> {
    fn is_char_boundary(s: &str, byte_pos: usize) -> bool {
        byte_pos == 0 || s.is_char_boundary(byte_pos)
    }

    let mut start_byte = start;
    let mut end_byte = end;

    if start_byte > 0 && !is_char_boundary(s, start_byte) {
        while start_byte > 0 && !is_char_boundary(s, start_byte) {
            start_byte += 1;
        }
    }

    if end_byte < s.len() && !is_char_boundary(s, end_byte) {
        while end_byte > 0 && !is_char_boundary(s, end_byte) {
            end_byte -= 1;
        }
    }

    match s.get(start_byte..end_byte) {
        Some(valid_range) => Ok(valid_range.to_string()),
        None => Err("Invalid byte range after adjustment."),
    }
}

#[test]
fn test_safe_byte_slice() {
    let string = "He东风送llo, 世界！狂担负了";

    match extract_multibyte_safe(string, 2, 15) {
        Ok(substring) => println!("Extracted substring: {}", substring),

        Err(e) => println!("Error: {}", e),
    }
}

pub fn find_matches(
    string: &str,
    search_plain: &str,
    search_re: &Option<Regex>,
    is_re_mode: bool,
    context_size: usize,
    prefix: &str,
    postfix: &str,
) -> Result<Vec<String>, io::Error> {
    let mut matches: Vec<String> = Vec::new();

    pub fn wrap_text(
        string: &str,
        mat_string: &str,
        start: usize,
        end: usize,
        context_size: usize,
        wrapper_start: &str,
        wrapper_end: &str,
    ) -> (String, usize) {
        let left_start = max(0, start.saturating_sub(context_size));
        let left_end = start;
        let left_context = match extract_multibyte_safe(string, left_start, left_end) {
            Ok(s) => s,
            Err(e) => {
                println!("right_context error: {}", e.to_string());
                "".to_owned()
            }
        };

        let right_start = end;
        let right_end = min(string.len(), end + context_size);
        let right_context = match extract_multibyte_safe(string, right_start, right_end) {
            Ok(s) => s,
            Err(e) => {
                println!("right_context error: {}", e.to_string());
                "".to_owned()
            }
        };

        (
            format!(
                "{}{}{}{}{}",
                left_context, wrapper_start, mat_string, wrapper_end, right_context
            ),
            right_context.len(),
        )
    }

    if is_re_mode {
        let re = match search_re {
            Some(r) => r,
            None => {
                return Err(io::Error::new(
                    ErrorKind::InvalidInput,
                    format!("Regex is None."),
                ));
            }
        };

        for mat in re.find_iter(string) {
            let start = mat.start();
            let end = mat.end();
            let mat_string: String = string[start..end].to_string();

            let w = wrap_text(
                string,
                &mat_string,
                start,
                end,
                context_size,
                &prefix,
                &postfix,
            );

            matches.push(w.0);
        }
    } else {
        let mut start_index = 0;
        let mut sss = string;

        while let Some(mat_index) = sss.find(search_plain) {
            let start = mat_index;
            let end = start + search_plain.len();

            let context = &sss[start..end];
            let mat_string = context.to_string();

            let w = wrap_text(
                string,
                &mat_string,
                start + start_index,
                end + start_index,
                context_size,
                &prefix,
                &postfix,
            );

            sss = &sss[end..];
            start_index += end;

            matches.push(w.0);
        }
    }

    Ok(matches)
}

#[test]
fn test_find_matches_re() {
    let string = String::from(
        "Unable to obtain core configuration information, please check the running environment",
    );

    let search = String::from(r#"co\S"#);
    let is_re_mode = true;
    let wrapper_prefix = "<b>";
    let wrapper_postfix = "</b>";
    let context_size = 50;
    let results = find_matches(
        &string,
        &search,
        &Some(Regex::new(&search).unwrap()),
        is_re_mode,
        context_size,
        wrapper_prefix,
        wrapper_postfix,
    );

    println!(">>> results of RE mode :::");

    if let Ok(result) = results {
        println!("{:?}", result);
    }

    println!("\n\n");
}

#[test]
fn test_find_matches_plain() {
    let string =
        String::from("点语种: 中文,西班牙语,英语,阿拉伯语,葡萄牙语,俄语,法语,德语,日语,韩语");

    let string_search = String::from("语");

    let wrapper1 = "<b>";
    let wrapper2 = "</b>";
    let context_size = 50;
    let results = find_matches(
        &string,
        &string_search,
        &Some(Regex::new("").unwrap()),
        false,
        context_size,
        wrapper1,
        wrapper2,
    );

    println!(">>> results of RE mode :::");

    if let Ok(result) = results {
        println!("{:?}", result);
    }

    println!("\n\n");
}

fn search_file_content(
    file_path: &PathBuf,
    plain: &str,
    re: &Option<Regex>,
    is_re_mode: bool,
    context_size: usize,
    wrapper_prefix: &str,
    wrapper_postfix: &str,
    html_like_exts: &Vec<String>,
) -> Result<Vec<String>, io::Error> {
    let mut string = String::new();

    let ext = match file_path.extension().and_then(|ext| ext.to_str()) {
        Some(v) => v.to_string(),
        None => "".to_string(),
    };

    if ext.len() > 0 && html_like_exts.contains(&ext) {
        string = prepare_html_like_file(file_path.to_path_buf(), ext)
    } else {
        let file = match File::open(file_path) {
            Ok(sss) => sss,
            Err(e) => {
                error!("File::open error {}", e);
                return Ok([].to_vec());
            }
        };
        let mut reader = BufReader::new(file);
        reader.read_to_string(&mut string)?;
    }

    match find_matches(
        &string,
        &plain,
        re,
        is_re_mode,
        context_size,
        wrapper_prefix,
        wrapper_postfix,
    ) {
        Ok(results) => Ok(results),
        Err(e) => Err(io::Error::new(
            ErrorKind::InvalidInput,
            format!("find_matches error:{}", e),
        )),
    }
}

pub fn search_in_dir(
    dir_path: &PathBuf,
    search: &str,
    is_re_mode: bool,
    context_size: usize,
    wrapper_prefix: &str,
    wrapper_postfix: &str,
    html_like_exts: &Vec<String>,
) -> Result<Vec<SearchFileRes>, io::Error> {
    let mut results: Vec<SearchFileRes> = Vec::new();
    let plain = if is_re_mode { "" } else { search };
    let re: Option<Regex> = if is_re_mode {
        match Regex::new(search) {
            Ok(r) => Some(r),

            Err(e) => {
                return Err(io::Error::new(
                    ErrorKind::InvalidInput,
                    format!("Regex::new error: {:?}", e),
                ));
            }
        }
    } else {
        None
    };

    for entry_result in fs::read_dir(dir_path)? {
        let entry = entry_result?;
        let path = entry.path();

        if path.is_file() {
            let sss = match search_file_content(
                &path,
                plain,
                &re,
                is_re_mode,
                context_size,
                &wrapper_prefix,
                &wrapper_postfix,
                &html_like_exts,
            ) {
                Ok(sss) => sss,
                Err(e) => {
                    debug!("process_file error: {}", e);
                    continue;
                }
            };

            if sss.len() > 0 {
                let search_file_res: SearchFileRes = SearchFileRes {
                    path: path_buf_to_string(path),
                    matches: sss,
                };
                results.push(search_file_res);
            }
        } else if path.is_dir() {
            let sss = match search_in_dir(
                &path,
                search,
                is_re_mode,
                context_size,
                wrapper_prefix,
                wrapper_postfix,
                &html_like_exts,
            ) {
                Ok(sss) => sss,
                Err(e) => {
                    error!("search_document error {}", e);
                    continue;
                }
            };
            if sss.len() > 0 {
                results.extend(sss);
            }
        }
    }

    Ok(results)
}

#[test]
fn test_search_document_dir_re() {
    let dir_path = PathBuf::from("/home/xxx/Documents/fivim/user_files");

    let search = r#"(126)"#;
    let is_re_mode = true;
    let context_size = 50;
    let wrapper_prefix = "<b>";
    let wrapper_postfix = "</b>";
    let html_like_exts = [].to_vec();

    let results = match search_in_dir(
        &dir_path,
        search,
        is_re_mode,
        context_size,
        wrapper_prefix,
        wrapper_postfix,
        &html_like_exts,
    ) {
        Ok(sss) => sss,
        Err(e) => return println!("test_search_document_re error: {}", e),
    };

    println!(">>> results of RE mode :::");

    for result in results {
        println!("path: {}", result.path);

        for text in result.matches {
            println!("\t {}", text);
        }
    }

    println!("\n\n");
}

pub fn search_in_file(
    file_path: &PathBuf,
    search: &str,
    is_re_mode: bool,
    context_size: usize,
    wrapper_prefix: &str,
    wrapper_postfix: &str,
    html_like_exts: &Vec<String>,
) -> Result<Vec<SearchFileRes>, io::Error> {
    let mut results: Vec<SearchFileRes> = Vec::new();
    let plain = if is_re_mode { "" } else { search };
    let re: Option<Regex> = if is_re_mode {
        match Regex::new(search) {
            Ok(r) => Some(r),

            Err(e) => {
                return Err(io::Error::new(
                    ErrorKind::InvalidInput,
                    format!("Regex::new error: {:?}", e),
                ));
            }
        }
    } else {
        None
    };

    let sss = match search_file_content(
        &file_path,
        plain,
        &re,
        is_re_mode,
        context_size,
        &wrapper_prefix,
        &wrapper_postfix,
        &html_like_exts,
    ) {
        Ok(sss) => sss,
        Err(e) => {
            return Err(io::Error::new(
                ErrorKind::InvalidInput,
                format!("process_file error: {:?}", e),
            ));
        }
    };

    if sss.len() > 0 {
        let search_file_res: SearchFileRes = SearchFileRes {
            path: path_buf_to_string(file_path.to_path_buf()),
            matches: sss,
        };
        results.push(search_file_res);
    }

    Ok(results)
}

pub fn process_html(html: &str, delete_tages: &Vec<String>) -> String {
    let br_re = Regex::new(r"<br[^>]*>").unwrap();
    let mut html_str = br_re.replace_all(html, "\n").to_string();

    for tag in delete_tages {
        let regex_str = format!(r#"<{}[^>]*>(.*?)<\/script>"#, tag);
        let re = Regex::new(&regex_str).unwrap();
        html_str = re.replace_all(&html_str, "").to_string();
    }

    let tag_re = Regex::new(r"<[^>]+>").unwrap();
    html_str = tag_re.replace_all(&html_str, "").to_string();

    let whitespace_re = Regex::new(r"\s+").unwrap();
    let result = whitespace_re.replace_all(&html_str, " ");

    let res = result.to_string();

    decode_html_entities(&res).to_string()
}

#[test]
fn test_process_html() {
    let delete_tages = ["head".to_string(), "script".to_string()].to_vec();
    let res = process_html(
        r#"
        <head>
            <meta charset="utf-8">
            <meta http-equiv="Content-Language" content="zh-CN">
            <meta http-equiv="X-UA-Compatible" content="IE=edge,chrome=1">
            <meta name="referrer" content="always">
            <meta name="viewport" content="width=device-width, initial-scale=1.0, maximum-scale=1.0">
            <title>OSCHINA - 中文开源技术交流社区</title>
           
            <script type="text/javascript" src="https://static.oschina.net/new-osc/js/utils/plugins/radarChart/radarChart.js"></script><script type="text/javascript" src="https://static.oschina.net/new-osc/js/utils/vue.min.js"></script><script type="text/javascript" src="https://static.oschina.net/new-osc/js/utils/vue.min.js"></script><script type="text/javascript" src="https://static.oschina.net/new-osc/js/utils/plugins/caret/jquery.caret.min.js"></script><script type="text/javascript" src="https://static.oschina.net/new-osc/js/utils/plugins/caret/jquery.caret.min.js"></script><script type="text/javascript" src="https://static.oschina.net/new-osc/js/utils/plugins/atwho/jquery.atwho.min.js"></script><script type="text/javascript" src="https://static.oschina.net/new-osc/js/utils/plugins/atwho/jquery.atwho.min.js"></script>
        </head>
        <body>
            <div class="headline">
                <div class="head-news">
                    <div 
                        data-id="195" 
                        data-href="http://osc.cool/zkcETcNH" 
                        title="“树莓派”成为上市公司" 
                        class="head-news-title advert-count">
                        “树莓派”成为上市公司
                    </div>
                </div>
            </div>
            <div class="item">
                <a 
                    href="https://www.oschina.net/news/297745/hikyuu-2-1-0-released" 
                    title="Hikyuu 2.1.0 发布，开源高性能量化交易框架" 
                    target="_blank" 
                    class="item-title primary  visited  ">
                    Hikyuu 2.1.0 发布，开源高性能量化交易框架
                </a>
            </div>

            <h6 data-uuid="fe5125a0-7c50-4927-82a1-afc3c80e73e5">标题6</h6>
        </body>    
    "#,
        &delete_tages,
    );

    print!("res: {}", res)
}
