use crate::utils::CustomError;
use log::{debug, warn};
use reqwest::header::{HeaderMap, HeaderValue};
use std::env;
use std::error::Error;
use std::path::PathBuf;
use tokio::fs;
use url::Url;

lazy_static::lazy_static! {
    static ref CACHE_DIR: String = {
        env::var("CACHE_DIR").unwrap_or_else(|_| String::from("/cache"))
    };
}

fn get_cache_dir() -> PathBuf {
    PathBuf::from(CACHE_DIR.to_string())
}

fn get_tempfile_path(url: &str) -> Result<PathBuf, Box<dyn Error>> {
    let u = Url::parse(url)?;
    let host = u
        .host_str()
        .ok_or_else(|| Box::new(CustomError(String::from("Invalid Url"))))?;

    let mut dst = get_cache_dir();
    let _ = dst.push("tmp");
    let current_time = chrono::Utc::now().timestamp_micros();

    let path = format!("{}{}_{}", host.to_string(), u.path(), current_time);
    let encode = base32::encode(
        base32::Alphabet::RFC4648 { padding: false },
        path.as_bytes(),
    );

    let _ = dst.push(encode);

    debug!("{}", dst.display());
    return Ok(dst);
}

pub fn get_image_cache_path(url: &str) -> Result<PathBuf, Box<dyn Error>> {
    let u = Url::parse(url)?;

    let host = u
        .host_str()
        .ok_or_else(|| Box::new(CustomError(String::from("Invalid Url"))))?;

    let path = host.to_string() + u.path();
    let mut dst = get_cache_dir();
    let _ = dst.push(path);
    return Ok(dst);
}

pub fn is_image_cache_hit(p: &PathBuf) -> bool {
    p.is_file()
}

pub async fn prepare_cache(url: &str) -> Result<(), Box<dyn Error>> {
    // http header for image
    let mut headers = HeaderMap::new();
    headers.insert(
        "Accept",
        HeaderValue::from_str("image/avif,image/webp,*/*")?,
    );
    headers.insert("Accept-Encoding", HeaderValue::from_str("gzip, deflate")?);

    // http client send request
    let client = reqwest::Client::new();
    let response = client.get(url).headers(headers).send().await?;
    if response.status() != 200 {
        return Err("invalid response".into());
    }
    let content_type = response
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .ok_or_else(|| Box::new(CustomError(String::from("invalid content type"))))?;
    if !content_type.to_str()?.starts_with("image/") {
        // 不是图片，可能是错误信息
        let text = response.text().await?;
        warn!("{}", text);
        // 处理错误信息...
        return Err("invalid content type".into());
    }
    // 处理图片...
    let image = response.bytes().await?;
    // 先写到临时文件内, 再移动到最终文件,确保最终文件的写入是原子的
    let temp_path = get_tempfile_path(url)?;
    fs::create_dir_all(temp_path.parent().unwrap()).await?;

    fs::write(temp_path.clone(), image).await?;

    let final_path = get_image_cache_path(url)?;
    fs::create_dir_all(final_path.parent().unwrap()).await?;

    fs::rename(&temp_path, &final_path).await?;

    Ok(())
}

/// unit test
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_image_cache_path() {
        let want = PathBuf::from("/cache/www.baidu.com/img/bd_logo1.png");
        let input = "https://www.baidu.com/img/bd_logo1.png";

        let got = get_image_cache_path(input).unwrap();
        assert_eq!(got, want);
    }
}
