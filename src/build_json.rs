use std::{ fs, path::Path, };

use rust_util::{
    iff,
    XResult,
    new_box_ioerror,
    util_msg::{
        print_ok,
        print_debug,
        print_warn,
        print_error,
    }
};

use super::http::get_url_content;
use super::misc::VERBOSE;

pub const BUILD_JSON: &str = "build.json";

const GET_ARCHIVER_VERSION_URL: &str= "https://hatter.ink/repo/archive_info_version.json";

pub fn get_archive_version(gid: &str, aid: &str) -> XResult<String> {
    if *VERBOSE {
        print_debug(&format!("Start get archive info: {}:{}", gid, aid));
    }
    let mut url = String::with_capacity(1024);
    url.push_str(GET_ARCHIVER_VERSION_URL);
    url.push_str("?gid=");
    url.push_str(&urlencoding::encode(gid));
    url.push_str("&aid=");
    url.push_str(&urlencoding::encode(aid));
    let version_result = get_url_content(url.as_str())?;
    if *VERBOSE {
        print_debug(&format!("Get archive result: {}", version_result));
    }
    let version_result_object = json::parse(&version_result)?;
    if version_result_object["status"] != 200 {
        Err(new_box_ioerror(&format!("Get archive info version failed: {}", version_result)))
    } else {
        Ok(version_result_object["data"].to_string())
    }
}

pub fn create_build_json(args: &[String]) {
    if find_build_json_in_current().is_some() {
        print_error(&format!("File exits: {}", BUILD_JSON));
        return;
    }

    let mut java_version = "";
    let mut builder = "";
    let mut builder_version = "";
    for arg in args {
        if arg.starts_with("--java") && arg.len() > 6 {
            java_version = &arg.as_str()[6..];
        } else if arg.starts_with("--maven") && arg.len() > 7 {
            builder = "maven";
            builder_version = &arg.as_str()[7..];
        } else if arg.starts_with("--gradle") && arg.len() > 8 {
            builder = "gradle";
            builder_version = &arg.as_str()[8..];
        }
    }
    if java_version.is_empty() || builder.is_empty() || builder_version.is_empty() {
        print_error("Args java version, builder or builder version is not assigned or format error.");
        return;
    }
    let mut build_json_object = object!{
        "java" => java_version,
        "builder" => object! {
            "name" => builder,
            "version" => builder_version,
        },
    };
    match get_archive_version("me.hatter", "commons") {
        Err(err) => print_error(&format!("Get me.hatter:commons version failed: {}", err)),
        Ok(ver) => build_json_object["repo"] = object! {
            "dependencies" => array! [
                format!("me.hatter:commons:{}", ver).as_str()
            ]
        },
    }
    match fs::write(BUILD_JSON, json::stringify_pretty(build_json_object, 4)) {
        Ok(_) => print_ok(&format!("Write file success: {}", BUILD_JSON)),
        Err(err) => print_error(&format!("Write file failed: {}, error message: {}", BUILD_JSON, err)),
    }
}

pub fn find_build_json_in_current() -> Option<String> {
    let path = fs::canonicalize(".").ok()?;
    let p_build_json = format!("{}/{}", path.to_str()?, BUILD_JSON);
    let path_build_json = Path::new(&p_build_json);
    iff!(path_build_json.exists(), Some(p_build_json), None)
}

pub fn find_build_json_in_parents() -> Option<String> {
    let mut path = fs::canonicalize(".").ok()?;
    let mut loop_count = 0_usize;
    loop {
        loop_count += 1_usize;
        if loop_count > 100_usize {
            print_error("Find build.json loop more than 100 loop!");
            return None;
        }

        let p = path.to_str()?;
        if p == "/" {
            return None;
        }
        let p_build_json = format!("{}/{}", p, BUILD_JSON);
        let path_build_json = Path::new(&p_build_json);
        if path_build_json.exists() {
            return Some(p_build_json);
        }
        path = path.parent()?.to_path_buf();
    }
}

pub fn find_build_json() -> Option<String> {
    match find_build_json_in_current() {
        Some(p) => Some(p),
        None => match find_build_json_in_parents() {
            Some(p) => {
                print_warn(&format!("Cannot find {} in current dir, find: {}", BUILD_JSON, p));
                Some(p)
            },
            None => {
                print_error(&format!("Cannot find {}", BUILD_JSON));
                None
            },
        },
    }
}
