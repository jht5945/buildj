use std::{
    fs::File,
};

use rust_util::{
    util_io::copy_io,
    util_msg::{
        print_message,
        MessageType,
    },
    XResult,
};

use super::misc::*;

pub fn download_url(url: &str, dest: &mut File) -> XResult<()> {
    let verbose = is_verbose();
    if verbose {
        print_message(MessageType::INFO, &format!("Download URL: {}", url));
    }
    let mut response = reqwest::get(url)?;
    let header_content_length: i64 = match response.headers().get("content-length") {
        None => -1,
        Some(len_value) => len_value.to_str().unwrap().parse::<i64>().unwrap(),
    };
    if verbose {
        print_message(MessageType::INFO, &format!("Content-Length: {}", header_content_length));
    }
    copy_io(&mut response, dest, header_content_length)?;
    Ok(())
}

pub fn get_url(url: &str) -> XResult<String> {
    if is_verbose() {
        print_message(MessageType::INFO, &format!("Get URL: {}", url));
    }
    Ok(reqwest::get(url)?.text()?)
}
