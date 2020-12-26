use std::fs::File;
use rust_util::{XResult, util_io};

use crate::misc::VERBOSE;

pub fn download_url(url: &str, dest: &mut File) -> XResult<()> {
    if *VERBOSE {
        debugging!("Start download URL: {}", url);
    }
    let mut response = reqwest::get(url)?;
    let header_content_length: i64 = match response.headers().get("content-length") {
        None => -1_i64, Some(len_value) => {
            let len_str = len_value.to_str().unwrap_or_else(|err| {
                warning!("Get content length for {:?}, error: {}", len_value, err);
                "-1"
            });
            len_str.parse::<i64>().unwrap_or_else(|err| {
                warning!("Get content length for {:?}, error: {}", len_value, err);
                -1
            })
        },
    };
    if *VERBOSE {
        debugging!("Content-Length: {}", header_content_length);
    }
    util_io::copy_io_default(&mut response, dest, header_content_length)?;
    Ok(())
}

pub fn get_url_content(url: &str) -> XResult<String> {
    if *VERBOSE {
        debugging!("Get URL: {}", url);
    }
    Ok(reqwest::get(url)?.text()?)
}
