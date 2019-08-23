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

use super::misc::VERBOSE;

pub fn download_url(url: &str, dest: &mut File) -> XResult<()> {
    if *VERBOSE {
        print_message(MessageType::DEBUG, &format!("Start download URL: {}", url));
    }
    let mut response = reqwest::get(url)?;
    let header_content_length: i64 = match response.headers().get("content-length") {
        None => -1,
        Some(len_value) => len_value.to_str().unwrap().parse::<i64>().unwrap(),
    };
    if *VERBOSE {
        print_message(MessageType::DEBUG, &format!("Content-Length: {}", header_content_length));
    }
    copy_io(&mut response, dest, header_content_length)?;
    Ok(())
}

pub fn get_url(url: &str) -> XResult<String> {
    if *VERBOSE {
        print_message(MessageType::DEBUG, &format!("Get URL: {}", url));
    }
    Ok(reqwest::get(url)?.text()?)
}
