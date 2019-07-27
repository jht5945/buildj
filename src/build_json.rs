use std::{
    fs,
    path::Path,
};

use super::rust_util::{
    print_message,
    MessageType,
};

pub const BUILD_JSON: &str = "build.json";

pub fn create_build_json(args: &Vec<String>) {
    match find_build_json_in_current() {
        Some(_) => {
            print_message(MessageType::ERROR, &format!("File exits: {}", BUILD_JSON));
            return;
        }, 
        None => (), // OK
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
    if java_version == "" || builder == "" || builder_version == "" {
        print_message(MessageType::ERROR, "Args java version, builder or builder version is not assigned or format error.");
        return;
    }
    let build_json_object = object!{
        "java" => java_version,
        "builder" => object! {
            "name" => builder,
            "version" => builder_version,
        },
    };
    match fs::write(BUILD_JSON, json::stringify_pretty(build_json_object, 4)) {
        Ok(_) => {
            print_message(MessageType::OK, &format!("Write file success: {}", BUILD_JSON));
        },
        Err(err) => {
            print_message(MessageType::ERROR, &format!("Write file failed: {}, error message: {}", BUILD_JSON, err));
        }
    };
 }

pub fn find_build_json_in_current() -> Option<String> {
    let path = fs::canonicalize(".").ok()?;
    let p_build_json = &format!("{}/{}", path.to_str()?, BUILD_JSON);
    let path_build_json = Path::new(p_build_json);
    if path_build_json.exists() {
        return Some(p_build_json.to_string());
    }
    None
}

pub fn find_build_json_in_parents() -> Option<String> {
    let mut path = fs::canonicalize(".").ok()?;
    loop {
        let p = path.to_str()?;
        if p == "/" {
            return None;
        }
        let p_build_json = &format!("{}/{}", p, BUILD_JSON);
        let path_build_json = Path::new(p_build_json);
        if path_build_json.exists() {
            return Some(p_build_json.to_string());
        }
        path = path.parent()?.to_path_buf();
    }
}

pub fn find_build_json() -> Option<String> {
    match find_build_json_in_current() {
        Some(p) => {
            Some(p)
        },
        None => match find_build_json_in_parents() {
            Some(p) => {
                print_message(MessageType::WARN, &format!("Cannot find {} in current dir, find: {}", BUILD_JSON, p));
                Some(p)
            },
            None => {
                print_message(MessageType::ERROR, &format!("Cannot find {}", BUILD_JSON));
                None
            },
        },
    }
}
