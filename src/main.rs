#[macro_use]
extern crate json;
#[macro_use]
extern crate lazy_static;
extern crate term;
extern crate dirs;
extern crate crypto;
extern crate urlencoding;
extern crate rust_util;

pub mod jdk;
pub mod local_util;
pub mod http;
pub mod tool;
pub mod build_json;
pub mod misc;

use std::{
    collections::HashMap,
    fs,
    process::Command,
};

use rust_util::{
    util_msg::{
        print_message,
        MessageType,
    },
    util_cmd::run_command_and_wait,
};
use tool::*;
use jdk::*;
use build_json::*;
use misc::*;

const BUILDJ: &str = "buildj";
const BUDERJ_VER: &str = env!("CARGO_PKG_VERSION");
const GIT_HASH: &str = env!("GIT_HASH");
const BUILD_DATE: &str = env!("BUILD_DATE");


fn do_with_buildin_arg_java(first_arg: &str, args: &[String]) {
    let ver = &first_arg[7..];
    if ver.is_empty() {
        print_message(MessageType::ERROR, "Java version is not assigned!");
        return;
    }
    match get_java_home(ver) {
        None => print_message(MessageType::ERROR, &format!("Assigned java version not found: {}", ver)),
        Some(java_home) => {
            print_message(MessageType::OK, &format!("Find java home: {}", java_home));
            let java_bin = &format!("{}/bin/java", java_home);
            let mut cmd = Command::new(java_bin);
            cmd.envs(&get_env_with_java_home(&java_home));
            if args.len() > 2 {
                cmd.args(&args[2..]);
            }
            run_command_and_wait(&mut cmd).unwrap_or_else(|err| {
                print_message(MessageType::ERROR, &format!("Exec java failed: {}", err));
            });
        },
    };
}

fn do_with_buildin_arg_maven(first_arg: &str, args: &[String]) {
    do_with_buildin_arg_builder(first_arg, args, "maven")
}

fn do_with_buildin_arg_gradle(first_arg: &str, args: &[String]) {
    do_with_buildin_arg_builder(first_arg, args, "gradle")
}

fn do_with_buildin_arg_config(_first_arg: &str, args: &[String]) {
    if args.len() <= 2 {
        print_message(MessageType::ERROR, "No arguments, get or set.");
        return;
    }
    match args[2].as_str() {
        "get" => match get_tool_package_secret() {
            Err(_) => print_message(MessageType::WARN, "No config found."),
            Ok(secret) => print_message(MessageType::OK, &format!("Config secret: {}", secret)),
        },
        "set" => {
            if args.len() < 4 {
                print_message(MessageType::ERROR, "Need secret for set, :::config set <secret>");
            } else {
                match set_tool_package_secret(&args[3]) {
                    Err(err) => print_message(MessageType::ERROR, &format!("Config secret failed: {}", err)),
                    Ok(_) => print_message(MessageType::OK, "Config secret success."),
                }
            }
        },
        arg => print_message(MessageType::ERROR, &format!("Unknown argument: {}", arg))
    }
}

fn do_with_buildin_arg_builder(first_arg: &str, args: &[String], builder_name: &str) {
    let builder_version = &first_arg[(builder_name.len() + 3)..];
    if builder_version.is_empty() {
        print_message(MessageType::ERROR, "Builder version is not assigned!");
        return;
    }
    let mut has_java = false;
    let mut java_home = String::new();
    if args.len() > 2 && args[2].starts_with("--java") {
        has_java = true;
        let java_version = &args[2][6..];
        if !java_version.is_empty() {
            java_home = match get_java_home(java_version) {
                Some(h) => h, None => {
                    print_message(MessageType::ERROR, &format!("Assigned java version not found: {}", java_version));
                    return;
                },
            };
        }
    }
    let builder_desc = match tool::get_builder_home(builder_name, builder_version) {
        Some(h) => h, None => {
            print_message(MessageType::ERROR, &format!("Assigned builder: {}, version: {} not found.", builder_name, builder_version));
            return;
        },
    };
    if has_java {
        print_message(MessageType::OK, &format!("JAVA_HOME    = {}", java_home));
    }
    print_message(MessageType::OK, &format!("BUILDER_HOME = {}", &builder_desc.home));

    let mut new_env = if has_java {
        get_env_with_java_home(&java_home)
    } else {
        get_env()
    };
    for builder_home_name in builder_desc.get_builder_home_name() {
        new_env.insert(builder_home_name, builder_desc.home.clone());
    }

    let mut cmd = Command::new(builder_desc.get_builder_bin());
    cmd.envs(&new_env);
    let from_index = if has_java { 3 } else { 2 };
    for arg in args.iter().skip(from_index) {
        cmd.arg(&arg);
    }
    run_command_and_wait(&mut cmd).unwrap_or_else(|err| {
        print_message(MessageType::ERROR, &format!("Run build command failed: {}", err));
    });
}

fn do_with_buildin_arg_ddd(first_arg: &str, args: &[String]) {
    let build_json_object = match read_build_json_object() {
        Some(object) => object, None => return,
    };
    let build_json_object_xrun = &build_json_object["xRuns"][&first_arg[3..]];
    if build_json_object_xrun.is_null() {
        print_message(MessageType::ERROR, &format!("Cannot find build.json#xRuns#{}", &first_arg[3..]));
        return;
    }
    let cmd_name = build_json_object_xrun[0].to_string();
    let mut cmd = Command::new(&cmd_name);
    cmd.current_dir(".");
    let mut cmd_args = vec![];
    for i in 1..build_json_object_xrun.len() {
        if *VERBOSE {
            cmd_args.push(build_json_object_xrun[i].to_string());
        }
        cmd.arg(build_json_object_xrun[i].to_string());
    }
    for arg in args.iter().skip(3) {
        if *VERBOSE {
            cmd_args.push(arg.to_string());
        }
        cmd.arg(arg.to_string());
    }
    if *VERBOSE {
        print_message(MessageType::DEBUG, &format!("Running cmd: {}, args: {:?}", &cmd_name, cmd_args));
    }
    run_command_and_wait(&mut cmd).unwrap_or_else(|err| {
        print_message(MessageType::ERROR, &format!("Run xRun command failed: {}", err));
    });
}

fn do_with_buildin_args(args: &[String]) {
     let first_arg = args.get(1).unwrap();
    if first_arg == ":::" || first_arg == ":::help" {
        print_usage();
    } else if first_arg == ":::version" {
        print_version();
    } else if first_arg == ":::create" {
        create_build_json(args);
    } else if first_arg == ":::config" {
        do_with_buildin_arg_config(first_arg, args);
    } else if first_arg.starts_with(":::java") {
        do_with_buildin_arg_java(first_arg, args);
    } else if first_arg.starts_with(":::maven") {
        do_with_buildin_arg_maven(first_arg, args);
    } else if first_arg.starts_with(":::gradle") {
        do_with_buildin_arg_gradle(first_arg, args);
    } else if first_arg.starts_with("...") {
        do_with_buildin_arg_ddd(first_arg, args);
    } else {
        print_message(MessageType::ERROR, &format!("Unknown args: {:?}", &args));
    }
}

fn get_java_and_builder(build_json_object: &json::JsonValue) -> Option<(String, BuilderDesc)> {
    let java_version_j = &build_json_object["java"];
    let builder_name_j = &build_json_object["builder"]["name"];
    let builder_version_j = &build_json_object["builder"]["version"];

    if java_version_j.is_null() {
        print_message(MessageType::ERROR, "Java version is not assigned!");
        return None;
    }
    if builder_name_j.is_null() || builder_version_j.is_null() {
        print_message(MessageType::ERROR, "Builder name or version is not assigned!");
        return None;
    }
    let java_version = java_version_j.as_str().unwrap();
    let builder_name = builder_name_j.as_str().unwrap();
    let builder_version = builder_version_j.as_str().unwrap();
    if *VERBOSE {
        print_message(MessageType::DEBUG, &format!("Java version: {}", java_version));
        print_message(MessageType::DEBUG, &format!("Builder name: {}", builder_name));
        print_message(MessageType::DEBUG, &format!("Builder version: {}", builder_version));
    }

    let java_home = match get_java_home(java_version) {
        Some(h) => h, None => {
            print_message(MessageType::ERROR, &format!("Assigned java version not found: {}", java_version));
            return None;
        },
    };
    let builder_desc = match tool::get_builder_home(builder_name, builder_version) {
        Some(h) => h, None => {
            print_message(MessageType::ERROR, &format!("Assigned builder: {}, version: {} not found.", builder_name, builder_version));
            return None;
        },
    };
    Some((java_home, builder_desc))
}

fn get_final_args(args: &[String], build_json_object: &json::JsonValue) -> Option<Vec<String>> {
    let mut final_args:Vec<String> = vec![];
    if args.len() > 1 {
        let arg1 = &args[1];
        if arg1.starts_with("::") {
            let a_cmd = &arg1[2..];
            let a_cmd_j = &build_json_object["xArgs"][a_cmd];
            if a_cmd_j.is_null() {
                print_message(MessageType::WARN, &format!("xArgs argument not found: {}", a_cmd));
                if args.len() == 2 {
                    print_message(MessageType::ERROR, "Only one xArgs argument, exit.");
                    return None;
                }
                final_args.push(arg1.to_string());
            } else {
                for a_j in a_cmd_j.members() {
                    if ! a_j.is_null() {
                        final_args.push(a_j.as_str().unwrap().to_string());
                    }
                }
            }
        } else {
            final_args.push(arg1.to_string());
        }
    }
    if args.len() > 2 {
        for arg in args.iter().skip(2) {
            final_args.push(arg.to_string());
        }
    }
    Some(final_args)
}

fn process_envs(the_env: &mut HashMap<String, String>, build_json_object: &json::JsonValue) {
    let envs_j = &build_json_object["envs"];
    if ! envs_j.is_null() {
        for env in envs_j.members() {
            if *VERBOSE {
                print_message(MessageType::DEBUG, &format!("Env: {}", env));
            }
            let env_k = &env[0];
            let env_v = &env[1];
            if let (Some(env_k_str), Some(env_v_str)) = (env_k.as_str(), env_v.as_str()) {
                the_env.insert(env_k_str.to_owned(), env_v_str.to_owned());
            }
        }
    }
}

fn read_build_json_object_from_env() -> Option<json::JsonValue> {
    if (*JAVA_VERSION).is_some() || (*BUILDER_VERSION).is_some() {
        let mut build_json_object = object!{};
        if (*JAVA_VERSION).is_some() {
            build_json_object["java"] = (*JAVA_VERSION).as_ref().unwrap().to_string().into();
        }
        if (*BUILDER_VERSION).is_some() {
            let builder_version = (*BUILDER_VERSION).as_ref().unwrap().to_string();
            if builder_version.starts_with("gradle") {
                build_json_object["builder"] = object! {
                    "name" => "gradle",
                    "version" => builder_version[6..],
                };
            } else if builder_version.starts_with("maven") {
                build_json_object["builder"] = object! {
                    "name" => "maven",
                    "version" => builder_version[5..],
                };
            } else {
                print_message(MessageType::WARN, &format!("Unknown builder: {}", builder_version));
            }
        }
        if *VERBOSE {
            print_message(MessageType::DEBUG, &format!("Use env configed build.json: {}",  json::stringify(build_json_object.clone())));
        }
        print_message(MessageType::OK, "Find build.json @ENV");
        Some(build_json_object)
    } else {
        None
    }
}

fn read_build_json_object() -> Option<json::JsonValue> {
    if let Some(o) = read_build_json_object_from_env() {
        return Some(o);
    }

    let build_json = match find_build_json() {
        Some(p) => p, None => return None,
    };

    print_message(MessageType::OK, &format!("Find {} @ {}", BUILD_JSON, build_json));
    let build_json_content = match fs::read_to_string(build_json) {
        Ok(content) => content, Err(err) => {
            print_message(MessageType::ERROR, &format!("Read {} failed: {}", BUILD_JSON, err));
            return None;
        },
    };
    match json::parse(&build_json_content) {
        Ok(object) => Some(object), Err(err) => {
            print_message(MessageType::ERROR, &format!("Parse JSON failed: {}", err));
            None
        },
    }
}


fn main() {
    print_message(MessageType::INFO, &format!("{} - version {} - {}", BUILDJ, BUDERJ_VER, &GIT_HASH[0..7]));

    if *VERBOSE {
        print_message(MessageType::DEBUG, &format!("Full GIT_HASH: {}", GIT_HASH));
        print_message(MessageType::DEBUG, &format!("Build date: {}", BUILD_DATE));
    }

    let args = local_util::get_args_as_vec();
    print_message(MessageType::INFO, &format!("Arguments: {:?}", args));

    if (! *NOBUILDIN) && local_util::is_buildin_args(&args) {
        do_with_buildin_args(&args);
        return;
    }
    if *VERBOSE {
        print_message(MessageType::DEBUG, &format!("Init home dir: {}", tool::LOCAL_BUILDER_HOME_BASE_DIR));
    }
    local_util::init_home_dir(tool::LOCAL_BUILDER_HOME_BASE_DIR);
    if *VERBOSE {
        print_message(MessageType::DEBUG, &format!("Init home dir: {}", jdk::LOCAL_JAVA_HOME_BASE_DIR));
    }
    local_util::init_home_dir(jdk::LOCAL_JAVA_HOME_BASE_DIR);

    let build_json_object = match read_build_json_object() {
        Some(object) => object, None => return,
    };

    let (java_home, builder_desc) = match get_java_and_builder(&build_json_object) {
        None => return,
        Some((java_home, builder_desc)) => (java_home, builder_desc),
    };
   
    print_message(MessageType::OK, &format!("JAVA_HOME    = {}", java_home));
    print_message(MessageType::OK, &format!("BUILDER_HOME = {}", &builder_desc.home));

    let mut new_env = get_env_with_java_home(&java_home);
    for builder_home_name in builder_desc.get_builder_home_name() {
        new_env.insert(builder_home_name, builder_desc.home.clone());
    }
    process_envs(&mut new_env, &build_json_object);

    let mut cmd = Command::new(builder_desc.get_builder_bin());
    cmd.envs(&new_env);

    let final_args = match get_final_args(&args, &build_json_object) {
        Some(fa) => fa, None => return,
    };
    if *VERBOSE {
        print_message(MessageType::DEBUG, &format!("Final arguments: {:?}", &final_args));
    }
    for f_arg in final_args {
        cmd.arg(f_arg);
    }
    if *VERBOSE {
        print_message(MessageType::DEBUG, "-----BEGIN ENVIRONMENT VARIABLES-----");
        for (k, v) in new_env {
            print_message(MessageType::DEBUG, &format!("{}={}", k, v));
        }
        print_message(MessageType::DEBUG, "-----END ENVIRONMENT VARIABLES-----");
    }
    run_command_and_wait(&mut cmd).unwrap_or_else(|err| {
        print_message(MessageType::ERROR, &format!("Run build command failed: {}", err));
    });
}
