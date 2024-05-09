use anyhow::Result;
use base64::engine::general_purpose::STANDARD as b64_STANDARD;
use base64::Engine;
use reqwest::{self, header, Client, RequestBuilder};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::{
    collections::HashMap,
    fs::File,
    io::{Read, Write},
};

use crate::fs as x_fs;
use crate::progress as xu_progress;

#[derive(PartialEq, Eq, Debug)]
pub enum HttpMethod {
    None,
    Get,
    Post,
    Put,
    Delete,
    Head,
}

#[derive(Debug, PartialEq, Eq)]
pub enum ReaponseDataType {
    None,
    Text,
    Base64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct HttpResponse {
    pub text: String,
    pub status: u16,
    pub headers: HashMap<String, String>,
    #[serde(rename = "errorMsg")]
    pub error_msg: String,
}

impl HttpResponse {
    pub fn new() -> HttpResponse {
        HttpResponse {
            text: "".to_owned(),
            status: 0,
            headers: HashMap::new(),
            error_msg: "".to_string(),
        }
    }
}

fn build_client(
    url: &str,
    method: HttpMethod,
    headers_map: HashMap<String, String>,
    params_map: HashMap<String, String>,
) -> (Client, RequestBuilder) {
    let client = reqwest::Client::new();
    let mut req: reqwest::RequestBuilder;

    match method {
        HttpMethod::Get => {
            req = client.get(url);
        }
        HttpMethod::Post => {
            req = client.post(url);
        }
        HttpMethod::Put => {
            req = client.put(url);
        }
        HttpMethod::Delete => {
            req = client.delete(url);
        }
        HttpMethod::Head => {
            req = client.head(url);
        }
        HttpMethod::None => todo!(),
    }

    for (k, v) in headers_map {
        req = req.header(k, v);
    }

    let mut params = HashMap::new();
    for (k, v) in params_map {
        params.insert(k, v);
    }

    return (client, req.query(&params));
}

pub async fn request_data(
    method: HttpMethod,
    url: &str,
    headers_map: &HashMap<String, String>,
    params_map: &HashMap<String, String>,
    body: String,
    resp_data_type: ReaponseDataType,
) -> Result<HttpResponse, Box<dyn std::error::Error>> {
    let bs = build_client(url, method, headers_map.clone(), params_map.clone());
    let mut request = bs.1;

    if body.len() > 0 {
        request = request.body(body);
    }

    let ret = request.send().await?;

    let mut res = HttpResponse::new();
    res.status = ret.status().as_u16();
    res.headers = HashMap::new();
    for (k, v) in ret.headers() {
        // HeaderValue's to_str() only support ASCII
        // let vvv = &v.to_str();

        // support Chinese, but there is an extra pair of double quotation marks outside the value, so it needs to be removed
        let vs = format!("{:?}", &v);
        let vss = &vs[1..vs.len() - 1];
        res.headers.insert(k.to_string(), vss.to_owned());
    }

    if resp_data_type == ReaponseDataType::Text {
        res.text = ret.text().await?;
    } else if resp_data_type == ReaponseDataType::Base64 {
        let bytes = ret.bytes().await?;
        res.text = b64_STANDARD.encode(bytes.as_ref());
    } else {
        res.text = "<empty>".to_owned()
    }

    Ok(res)
}

pub async fn downlaod_file(
    method: HttpMethod,
    url: &str,
    file_path: &str,
    headers_map: &HashMap<String, String>,
    params_map: &HashMap<String, String>,
) -> Result<(), Box<dyn std::error::Error>> {
    crate::fs::check_or_create_dir(&x_fs::get_parent_dir_path(&file_path))?;

    let (_client, req) = build_client(url, method, headers_map.clone(), params_map.clone());
    let ret = req.send().await?;

    let body = ret.bytes().await?;
    let mut file = File::create(Path::new(file_path))?;
    let content = body.bytes();
    let data: std::result::Result<Vec<_>, _> = content.collect();
    file.write_all(&data.unwrap())?;

    Ok(())
}

pub async fn downlaod_file_large(
    method: HttpMethod,
    url: &str,
    file_path: &str,
    headers_map: &HashMap<String, String>,
    params_map: &HashMap<String, String>,
    progress_name: &str,
) -> Result<i32, Box<dyn std::error::Error>> {
    let _ = crate::fs::check_or_create_dir(&x_fs::get_parent_dir_path(&file_path))?;

    let path = Path::new(&file_path);
    xu_progress::insert_new(&progress_name);

    let bs = build_client(url, method, headers_map.clone(), params_map.clone());
    let client = bs.0;
    let mut request = bs.1;

    let total_size = {
        let resp = client.head(url).send().await?;
        if resp.status().is_success() {
            resp.headers()
                .get(header::CONTENT_LENGTH)
                .and_then(|ct_len| ct_len.to_str().ok())
                .and_then(|ct_len| ct_len.parse().ok())
                .unwrap_or(0)
        } else {
            // println!(">>> not is_success status : {} ", resp.status());
            0
        }
    };

    if path.exists() {
        let size = path.metadata()?.len().saturating_sub(1);
        request = request.header(header::RANGE, format!("bytes={}-", size));
    }
    let mut source = request.send().await?;
    let mut dest = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)?;
    let mut downloaded_size: i32 = 0;
    while let Some(chunk) = source.chunk().await? {
        dest.write_all(&chunk)?;

        if total_size > 0 {
            let pct = downloaded_size as f32 / total_size as f32;
            xu_progress::set(&progress_name, pct, "");
            downloaded_size += chunk.len() as i32;
        }
    }
    xu_progress::set(&progress_name, 1.0, "");
    Ok(total_size)
}

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_request_text() {
        let mut header: HashMap<String, String> = HashMap::new();
        header.insert(
            "PRIVATE-TOKEN".to_string(),
            "glpat-13-fgsbyU9ti2Euew_DL".to_string(),
        );

        let params: HashMap<String, String> = HashMap::new();
        let url = "https://framagit.org/api/v4/projects/123456/repository/files/test.md?ref=main";
        let ret = match request_data(
            HttpMethod::Get,
            url,
            &header,
            &params,
            "".to_string(),
            ReaponseDataType::Text,
        )
        .await
        {
            Ok(o) => o,
            Err(e) => return println!("request_data error: {:#}", e),
        };

        println!(">>> request_data retuslt:: {:?}", ret);
    }

    #[tokio::test]
    async fn test_request_base64() {
        let header: HashMap<String, String> = HashMap::new();
        let params: HashMap<String, String> = HashMap::new();

        let url = "http://xxx.com/m/images/banner01.jpg";
        let ret = request_data(
            HttpMethod::Get,
            url,
            &header,
            &params,
            "".to_string(),
            ReaponseDataType::Base64,
        )
        .await
        .unwrap();

        println!(">>> request_data retuslt:: {:#?}", ret);
    }

    #[tokio::test]
    pub async fn test_downlaod_file() {
        let mut header: HashMap<String, String> = HashMap::new();
        let params: HashMap<String, String> = HashMap::new();

        header.insert("PRIVATE-TOKEN".to_string(), "glpat-TgvZJhz-".to_string());

        let url = "https://gitlab.com/api/v4/projects/123456/repository/archive.zip";
        let path = "/xxx/archive111.zip";
        let _ = downlaod_file(HttpMethod::Get, url, path, &header, &params).await;
    }

    #[tokio::test]
    async fn test_downlaod_large() {
        let mut header: HashMap<String, String> = HashMap::new();
        let params: HashMap<String, String> = HashMap::new();

        header.insert("PRIVATE-TOKEN".to_string(), "glpat-TgvZJhz-".to_string());

        let url = "https://gitlab.com/api/v4/projects/123456/repository/archive.zip";
        let path = "/xxx/archive111.zip";

        let _ = downlaod_file_large(
            HttpMethod::Get,
            url,
            path,
            &header,
            &params,
            "test_downlaod_large",
        )
        .await;
    }
}
