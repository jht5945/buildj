use std::{
    fs::{self, File},
    path::Path,
};

use super::{
    http,
    rust_util::{
        new_box_error,
        util_os::is_macos_or_linux,
        util_msg::{
            print_message,
            MessageType,
        },
        XResult,
    },
    local_util::{self, *},
    misc::*,
};

const M2_HOME: &str = "M2_HOME";
const MAVEN_HOME: &str = "MAVEN_HOME";
const GRADLE_HOME: &str = "GRADLE_HOME";

pub const LOCAL_BUILDER_HOME_BASE_DIR: &str = ".jssp/builder";
const STANDARD_CONFIG_JSON: &str = ".standard_config.json";
const TOOL_PACKAGE_DETAIL_URL: &str = "https://hatter.ink/tool/query_tool_by_name_version.json";
const TOOL_PACKAGE_DETAIL_URL_WITHOUT_AUTH: &str = "https://hatter.ink/tool/query_tool_by_name_version_without_auth.json";

#[derive(Clone, Copy)]
pub enum BuilderName {
    Maven,
    Gradle,
}

pub struct BuilderDesc {
    pub name: BuilderName,
    pub home: String,
    pub bin: Option<String>,
}

impl BuilderDesc {
    pub fn get_builder_home_name(&self) -> Vec<String> {
        match self.name {
            BuilderName::Maven => vec![M2_HOME.to_string(), MAVEN_HOME.to_string()],
            BuilderName::Gradle => vec![GRADLE_HOME.to_string()],
        }
    }

    pub fn get_builder_bin(&self) -> String {
        match &self.bin {
            Some(b) => b.clone(),
            None => {
                match self.name {
                    BuilderName::Maven => format!("{}/bin/mvn", self.home.clone()),
                    BuilderName::Gradle => format!("{}/bin/gradle", self.home.clone()),
                }
            }
        }
    }
}

pub fn get_builder_home(builder: &str, version: &str) -> Option<BuilderDesc> {
    let local_builder_home_base_dir = match get_user_home_dir(LOCAL_BUILDER_HOME_BASE_DIR) {
        Err(_) => return None,
        Ok(o) => o,
    };
    let builder_name = match builder {
        "maven" => BuilderName::Maven,
        "gradle" => BuilderName::Gradle,
        _ => {
            print_message(MessageType::ERROR, &format!("Unknown builder: {}", builder));
            return None;
        },
    };
    let local_builder_home_dir = &format!("{}/{}-{}", local_builder_home_base_dir, builder, version);
    if Path::new(local_builder_home_dir).exists() {
        return get_local_builder_home_sub(builder_name, local_builder_home_dir);
    }

    if get_cloud_builder(builder, version) {
        return get_local_builder_home_sub(builder_name, local_builder_home_dir);
    }

    None
}

pub fn get_cloud_builder(builder: &str, version: &str) -> bool {
    if ! is_macos_or_linux() {
        return false;
    }
    let local_builder_home_base_dir = match local_util::get_user_home_dir(LOCAL_BUILDER_HOME_BASE_DIR) {
        Err(_) => return false,
        Ok(o) => o,
    };
    match get_and_extract_tool_package(&local_builder_home_base_dir, true, builder, version, true) {
        Err(err) => {
            print_message(MessageType::ERROR, &format!("Get builder: {} failed, version: {}, error: {}", builder, version, err));
            return false;
        },
        Ok(_) => true,
    }
}

pub fn get_local_builder_home_sub(builder_name: BuilderName, local_builder_home_dir: &str) -> Option<BuilderDesc> {
    match get_local_builder_home_sub_first_sub_dir(local_builder_home_dir) {
        None => {
            print_message(MessageType::ERROR, &format!("Cannot find builder home in: {}", local_builder_home_dir));
            return None;
        },
        Some(p) => {
            return Some(BuilderDesc{name: builder_name, home: p, bin: None});
        },
    }
}

pub fn get_local_builder_home_sub_first_sub_dir(local_builder_home_dir: &str) -> Option<String> {
    let paths = fs::read_dir(Path::new(&local_builder_home_dir)).ok()?;
    for path in paths {
        match path {
            Err(_) => (),
            Ok(p) => {
                if p.path().is_dir() {
                    return Some(p.path().to_str()?.to_string());
                }
            },
        };
    }
    None
}

pub fn get_extract_dir_name_by_file_name(file_name: &str) -> Option<String> {
    if file_name != "" {
        return None;
    }
    let mut dir_name = file_name;
    if file_name.ends_with(".zip") {
        dir_name = &file_name[..file_name.len()-4];
    } else if file_name.ends_with(".tgz") {
        dir_name = &file_name[..file_name.len()-4];
    } else if file_name.ends_with(".tar.gz") {
        dir_name = &file_name[..file_name.len()-7];
    }
    if dir_name.ends_with("-bin") {
        dir_name = &dir_name[..dir_name.len()-4];
    }
    Some(dir_name.to_string())
}

pub fn get_tool_package_secret() -> XResult<String> {
    let standard_config_file = get_user_home_dir(STANDARD_CONFIG_JSON)?;
    let standard_config_json = fs::read_to_string(&standard_config_file)?;
    let standard_config_object = json::parse(&standard_config_json)?;

    let build_js_auth_token = &standard_config_object["build.js"]["auth_token"];
    if build_js_auth_token.is_null() {
        return Err(new_box_error("Standard json#build.js#auth_token is null."));
    }
    Ok(build_js_auth_token.to_string())
}

pub fn set_tool_package_secret(secret: &str) -> XResult<()> {
    let standard_config_file = get_user_home_dir(STANDARD_CONFIG_JSON)?;

    match fs::metadata(&standard_config_file) {
        Err(_) => {
            match fs::write(&standard_config_file, json::stringify_pretty(
                object!{ "build.js" => object!{
                    "auth_token" => secret, }
                }, 4)) {
                Ok(_) => Ok(()),
                Err(err) => Err(new_box_error(&format!("Write config failed: {}, error message: {}", standard_config_file, err))),
            }
        },
        Ok(f) => {
            if ! f.is_file() {
                return Err(new_box_error(&format!("Config is not a file: {}", standard_config_file)));
            }
            let standard_config_json = fs::read_to_string(&standard_config_file)?;
            let mut standard_config_object = json::parse(&standard_config_json)?;
            if standard_config_object["build.js"].is_null() {
                standard_config_object["build.js"] = object! {
                    "auth_token" => secret,
                };
            } else {
                standard_config_object["build.js"]["auth_token"] = secret.into();
            }
            match fs::write(&standard_config_file, json::stringify_pretty(standard_config_object, 4)) {
                Ok(_) => Ok(()),
                Err(err) => Err(new_box_error(&format!("Write config failed: {}, error message: {}", &standard_config_file, err))),
            }
        }
    }
}

pub fn get_tool_package_detail(name: &str, version: &str) -> XResult<String> {
    let secret = match *NOAUTH {
        true => {
            print_message(MessageType::WARN, "Running in no auth mode!");
            None
        },
        false => match get_tool_package_secret() {
            Err(err) => {
                print_message(MessageType::WARN, &format!("Get package detail secret failed: {}, from file: ~/{}", err, STANDARD_CONFIG_JSON));
                None
            },
            Ok(r) => Some(r),
        },
    };
    
    let mut url = String::new();
    match secret {
        None => url.push_str(TOOL_PACKAGE_DETAIL_URL_WITHOUT_AUTH),
        Some(_) => url.push_str(TOOL_PACKAGE_DETAIL_URL),
    };
    url.push_str("?");
    match secret {
        None => (),
        Some(secret) => {
            url.push_str("__auth_token=");
            url.push_str(&urlencoding::encode(&secret));
        },
    };
    url.push_str("&name=");
    url.push_str(&urlencoding::encode(name));
    url.push_str("&ver=");
    url.push_str(&urlencoding::encode(version));
    Ok(http::get_url(url.as_str())?)
}

pub fn get_and_extract_tool_package(base_dir: &str, dir_with_name: bool, name: &str, version: &str, extract_match: bool) -> XResult<bool> {
    let tool_package_detail = get_tool_package_detail(name, version)?;
    let build_json_object = json::parse(&tool_package_detail)?;
    if *VERBOSE {
        print_message(MessageType::DEBUG, &format!("Get tool {}:{}, result JSON: {}", name, version, json::stringify_pretty(build_json_object.clone(), 4)));
    }
    if build_json_object["status"] != 200 {
        return Err(new_box_error(&format!("Error in get tool package detail: {}", build_json_object["message"])));
    }
    let data = &build_json_object["data"];
    let integrity = &data["integrity"];
    let url = &data["url"];
    let name = &data["name"];
    if integrity.is_null() || url.is_null() || name.is_null() {
        return Err(new_box_error(&format!("Parse tool package detail failed: {}", tool_package_detail)));
    }
    let n = data["n"].to_string();
    let v = data["v"].to_string();

    if extract_match &&  version != &v {
        return Err(new_box_error(&format!("Required version not match, {}: {} vs {}", name, version, &v)));
    }

    let mut target_base_dir = String::new(); 
    target_base_dir.push_str(base_dir);
    if dir_with_name {
        target_base_dir.push_str("/");
        target_base_dir.push_str(&format!("{}-{}", n, v));
    }
    init_dir(&target_base_dir);
    let target_file_name = format!("{}/{}", &target_base_dir, name.to_string());

    print_message(MessageType::INFO, &format!("Start download: {} -> {}", &url.to_string(), &target_file_name));
    http::download_url(&url.to_string(), &mut File::create(&target_file_name)?)?;

    print_message(MessageType::INFO, &format!("Start verify integrity: {} ...", &target_file_name));
    if local_util::verify_file_integrity(&integrity.to_string(), &target_file_name)? {
        print_message(MessageType::OK, "Verify integrity success.");
    }

    print_message(MessageType::INFO, &format!("Start extract file: {}", &target_file_name));
    local_util::extract_package_and_wait(&target_base_dir, &name.to_string())?;

    Ok(true)
}
