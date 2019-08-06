use std::{
    fs::File,
};

use super::rust_util::{
    util_io::copy_io,
    XResult,
};

pub fn download_url(url: &str, dest: &mut File) -> XResult<()> {
    let mut response = reqwest::get(url)?;
    let header_content_length: i64 = match response.headers().get("content-length") {
        None => -1,
        Some(len_value) => len_value.to_str().unwrap().parse::<i64>().unwrap(),
    };
    copy_io(&mut response, dest, header_content_length)?;
    Ok(())
}

pub fn get_url(url: &str) -> XResult<String> {
    Ok(reqwest::get(url)?.text()?)
}
