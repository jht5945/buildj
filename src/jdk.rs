use std::{
    collections::HashMap,
    env,
    fs,
    str,
    path::Path,
    process::Command,
};

use super::{
    rust_util::{
        is_macos,
        is_linux,
        is_macos_or_linux,
        print_message,
        MessageType,
    },
    local_util,
    tool,
};

const OPENJDK_MACOS: &str = "openjdk-osx";
const JDK_LINUX: &str = "jdk-linux";
const OPENJDK_LINUX: &str = "openjdk-linux";

const MACOS_LIBEXEC_JAVAHOME: &str = "/usr/libexec/java_home";

pub const LOCAL_JAVA_HOME_BASE_DIR: &str = ".jssp/jdks";

pub fn get_java_home(version: &str) -> Option<String> {
    match get_macos_java_home(version) {
        Some(j) => Some(j),
        None => match get_local_java_home(version) {
            Some(j) => Some(j),
            None => match get_cloud_java(version) {
                    true => get_local_java_home(version),
                    false => None,
            },
        },
    }
}

pub fn get_cloud_java(version: &str) -> bool {
    if ! is_macos_or_linux() {
        return false;
    }
    let cloud_java_names = if is_macos() {
        vec![OPENJDK_MACOS]
    } else if is_linux() {
        vec![JDK_LINUX, OPENJDK_LINUX]
    } else {
        vec![] 
    };
    let local_java_home_base_dir = match local_util::get_user_home_dir(LOCAL_JAVA_HOME_BASE_DIR) {
        Err(_) => return false,
        Ok(o) => o,
    };
    for i in 0..cloud_java_names.len() {
        let cloud_java_name = cloud_java_names[i];
        match tool::get_and_extract_tool_package(&local_java_home_base_dir, false, cloud_java_name, version, false) {
            Err(_) => (),
            Ok(_) => return true,
        }
    }
    print_message(MessageType::ERROR, &format!("Get java failed, version: {}", version));
    false
}

pub fn get_macos_java_home(version: &str) -> Option<String> {
    if ! is_macos() {
        return None;
    }
    let output = match Command::new(MACOS_LIBEXEC_JAVAHOME).arg("-version").arg(version).output() {
        Err(_) => return None,
        Ok(o) => o,
    };
    match str::from_utf8(&output.stderr) {
        Err(_) => (),
        Ok(o) => {
            // Unable to find any JVMs matching version "1.6".
            if o.contains("Unable to find any JVMs") {
                return None;
            }
        },
    };
    Some(str::from_utf8(&output.stdout).ok()?.trim().to_string())
}

pub fn get_local_java_home(version: &str) -> Option<String> {
    let local_java_home_base_dir = local_util::get_user_home_dir(LOCAL_JAVA_HOME_BASE_DIR).ok()?;
    let paths = fs::read_dir(Path::new(&local_java_home_base_dir)).ok()?;
    for path in paths {
        match path {
            Err(_) => (),
            Ok(dir_entry) => match dir_entry.path().to_str() {
                None => (),
                Some(p) => {
                    let mut path_name = p;
                    if p.ends_with("/") {
                        path_name = &path_name[..path_name.len() - 1]
                    }
                    let last_index_of_slash = path_name.rfind('/');
                    match last_index_of_slash {
                        None => {},
                        Some(i) => path_name = &path_name[i+1..],
                    };
                    let mut matched_path = "";
                    if path_name.starts_with("jdk-") && (&path_name[4..]).starts_with(version) {
                        matched_path = p;
                    } else if path_name.starts_with("jdk") && (&path_name[3..]).starts_with(version) {
                        matched_path = p;
                    }
                    if matched_path != "" {
                        if local_util::is_path_exists(matched_path, "Contents/Home") {
                            return Some(format!("{}/{}", matched_path, "Contents/Home"));
                        } else {
                            return Some(matched_path.to_string());
                        }
                    }
                },
            },
        };
    }
    None
}

pub fn extract_jdk_and_wait(file_name: &str) {
    let local_java_home_base_dir = match local_util::get_user_home_dir(LOCAL_JAVA_HOME_BASE_DIR) {
        Err(_) => return,
        Ok(o) => o,
    };
    local_util::extract_package_and_wait(&local_java_home_base_dir, file_name).unwrap_or_else(|err| {
        print_message(MessageType::ERROR, &format!("Extract file: {}, failed: {}", file_name, err));
    });
}

pub fn get_env() -> HashMap<String, String> {
    let mut new_env: HashMap<String, String> = HashMap::new();
    for (key, value) in env::vars() {
        new_env.insert(key, value);
    }
    new_env
}

pub fn get_env_with_java_home(java_home: &str) -> HashMap<String, String> {
    let mut new_env: HashMap<String, String> = HashMap::new();
    for (key, value) in env::vars() {
        let key_str = key.as_str();
        if "JAVA_HOME" == key_str {
            // IGNORE JAVA_HOME
        } else if "PATH" == key_str {
            let path = value.to_string();
            let new_path = format!("{}/bin:{}", java_home, path);
            new_env.insert("PATH".to_string(), new_path);
        } else {
            new_env.insert(key, value);
        }
    }
    new_env.insert("JAVA_HOME".to_string(), java_home.to_string());
    new_env
}
