use std::{collections::HashMap, env, fs, str, path::Path, process::Command};
use rust_util::util_os;
use rust_util::util_env;
use crate::{local_util, tool, misc::VERBOSE};
use plist::Value;

const PATH: &str = "PATH";
const JAVA_HOME: &str = "JAVA_HOME";

const OPENJDK_MACOS: &str = "openjdk-osx";
const JDK_LINUX: &str = "jdk-linux";
const OPENJDK_LINUX: &str = "openjdk-linux";

const MACOS_LIBEXEC_JAVAHOME: &str = "/usr/libexec/java_home";

pub const LOCAL_JAVA_HOME_BASE_DIR: &str = ".jssp/jdks";

lazy_static! {
    pub static ref BUILDJ_JAVA_NAME: Option<String> = env::var("BUILDJ_JAVA_NAME").ok();
}

pub fn get_java_home(version: &str) -> Option<String> {
    match get_macos_java_home(version) {
        Some(j) => Some(j),
        None => match get_local_java_home(version) {
            Some(j) => Some(j),
            None => iff!(get_cloud_java(version), get_local_java_home(version), None),
        },
    }
}

pub fn get_cloud_java(version: &str) -> bool {
    if !util_os::is_macos_or_linux() {
        return false;
    }
    let cloud_java_names = match &*BUILDJ_JAVA_NAME {
        None => if util_os::is_macos() {
            vec![OPENJDK_MACOS]
        } else if util_os::is_linux() {
            vec![JDK_LINUX, OPENJDK_LINUX]
        } else {
            vec![]
        },
        Some(buildj_java_name) => vec![buildj_java_name.as_str()],
    };
    let local_java_home_base_dir = match local_util::get_user_home_dir(LOCAL_JAVA_HOME_BASE_DIR) {
        Ok(o) => o,
        Err(_) => return false,
    };
    for cloud_java_name in cloud_java_names {
        if tool::get_and_extract_tool_package(&local_java_home_base_dir, false, cloud_java_name, version, false).is_ok() {
            return true;
        }
    }
    failure!("Get java failed, version: {}", version);
    false
}

pub fn get_macos_java_home(version: &str) -> Option<String> {
    if !util_os::is_macos() || util_env::is_env_on("SKIP_CHECK_JAVA_HOME") {
        return None;
    }
    let java_home_x = Command::new(MACOS_LIBEXEC_JAVAHOME).arg("-x").output().ok()?;
    let java_home_plist_value = match Value::from_reader_xml(&*java_home_x.stdout) {
        Err(e) => {
            debugging!("Parse java_home outputs failed: {}", e);
            return None;
        }
        Ok(val) => val,
    };
    let java_home_plist_value_array = match java_home_plist_value.as_array() {
        None => {
            debugging!("Covert java_home plist output to array failed: {:?}", java_home_plist_value);
            return None;
        }
        Some(val) => val,
    };
    for java_home_plist_item in java_home_plist_value_array {
        debugging!("Checking: {:?}", java_home_plist_item);
        if let Some(jvm_item) = java_home_plist_item.as_dictionary() {
            let jvm_version_value = jvm_item.get("JVMVersion");
            let jvm_home_path_value = jvm_item.get("JVMHomePath");
            if let (Some(Value::String(jvm_version)), Some(Value::String(jvm_path))) = (jvm_version_value, jvm_home_path_value) {
                debugging!("Check version: {} vs {}", jvm_version, version);
                if jvm_version.starts_with(version) {
                    debugging!("Check version success: {} -> {}", jvm_version, jvm_path);
                    return Some(jvm_path.into());
                }
            }
        }
    }
    None
}

pub fn get_local_java_home(version: &str) -> Option<String> {
    let local_java_home_base_dir = local_util::get_user_home_dir(LOCAL_JAVA_HOME_BASE_DIR).ok()?;
    let paths = fs::read_dir(Path::new(&local_java_home_base_dir)).ok()?;
    for path in paths {
        if let Ok(dir_entry) = path {
            if let Some(p) = dir_entry.path().to_str() {
                if *VERBOSE {
                    debugging!("Try match path: {}", p);
                }
                let mut path_name = p;
                if p.ends_with('/') {
                    path_name = &path_name[..path_name.len() - 1]
                }
                if let Some(i) = path_name.rfind('/') {
                    path_name = &path_name[i + 1..];
                }
                let matched_path_opt = if (path_name.starts_with("jdk-") && (&path_name[4..]).starts_with(version))
                    || (path_name.starts_with("jdk") && (&path_name[3..]).starts_with(version)) {
                    Some(p)
                } else {
                    None
                };
                if let Some(matched_path) = matched_path_opt {
                    if *VERBOSE {
                        debugging!("Matched JDK path found: {}", matched_path);
                    }
                    return if local_util::is_path_exists(matched_path, "Contents/Home") {
                        Some(format!("{}/{}", matched_path, "Contents/Home"))
                    } else {
                        Some(matched_path.to_string())
                    };
                }
            }
        }
    }
    None
}

pub fn extract_jdk_and_wait(file_name: &str) {
    if let Ok(local_java_home_base_dir) = local_util::get_user_home_dir(LOCAL_JAVA_HOME_BASE_DIR) {
        local_util::extract_package_and_wait(&local_java_home_base_dir, file_name).unwrap_or_else(|err| {
            failure!("Extract file: {}, failed: {}", file_name, err);
        });
    }
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
        if JAVA_HOME == key_str {
            // IGNORE JAVA_HOME
        } else if PATH == key_str {
            let path = value.to_string();
            let new_path = format!("{}/bin:{}", java_home, path);
            new_env.insert(PATH.to_string(), new_path);
        } else {
            new_env.insert(key, value);
        }
    }
    new_env.insert(JAVA_HOME.to_string(), java_home.to_string());
    new_env
}
