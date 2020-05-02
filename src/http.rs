use std::fs::File;

use rust_util::{
    XResult,
    util_io::copy_io,
    util_msg::{
        print_debug,
        print_warn,
    },
};

use super::misc::VERBOSE;

pub fn download_url(url: &str, dest: &mut File) -> XResult<()> {
    if *VERBOSE {
        print_debug(&format!("Start download URL: {}", url));
    }
    let mut response = reqwest::get(url)?;
    let header_content_length: i64 = match response.headers().get("content-length") {
        None => -1_i64,
        Some(len_value) => {
            let len_str = match len_value.to_str() {
                Ok(len_str) => len_str, Err(err) => {
                    print_warn(&format!("Get content length for {:?}, error: {}", len_value, err));
                    "-1"
                },
            };
            match len_str.parse::<i64>() {
                Ok(len) => len, Err(err) => {
                    print_warn(&format!("Get content length for {:?}, error: {}", len_value, err));
                    -1
                }
            }
        },
    };
    if *VERBOSE {
        print_debug(&format!("Content-Length: {}", header_content_length));
    }
    copy_io(&mut response, dest, header_content_length)?;
    Ok(())
}

pub fn get_url_content(url: &str) -> XResult<String> {
    if *VERBOSE {
        print_debug(&format!("Get URL: {}", url));
    }
    Ok(reqwest::get(url)?.text()?)
}
