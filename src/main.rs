#[macro_use] extern crate json;
#[macro_use] extern crate lazy_static;
#[macro_use] extern crate rust_util;

use std::fs;
use std::collections::HashMap;
use std::process::{self, Command};

pub mod jdk;
pub mod local_util;
pub mod http;
pub mod tool;
pub mod build_json;
pub mod misc;

use rust_util::util_cmd;
use tool::*;
use jdk::*;
use build_json::*;
use misc::*;


fn do_with_buildin_arg_java(first_arg: &str, args: &[String]) {
    let ver = &first_arg[7..];
    if ver.is_empty() {
        failure!("Java version is not assigned!");
        return;
    }
    match get_java_home(ver) {
        None => failure!("Assigned java version not found: {}", ver),
        Some(java_home) => {
            success!("Find java home: {}", java_home);
            let java_bin = &format!("{}/bin/java", java_home);
            let mut cmd = Command::new(java_bin);
            cmd.envs(&get_env_with_java_home(&java_home));
            if args.len() > 2 {
                cmd.args(&args[2..]);
            }
            if let Err(err) = util_cmd::run_command_and_wait(&mut cmd) {
                failure!("Exec java failed: {}", err);
            }
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
        failure!("No arguments, get or set.");
        return;
    }
    match args[2].as_str() {
        "get" => match get_tool_package_secret() {
            Err(_) => warning!("No config found."),
            Ok(secret) => success!("Config secret: {}", secret),
        },
        "set" => {
            if args.len() < 4 {
                failure!("Need secret for set, :::config set <secret>");
            } else {
                match set_tool_package_secret(&args[3]) {
                    Err(err) => failure!("Config secret failed: {}", err),
                    Ok(_) => success!("Config secret success."),
                }
            }
        },
        arg => failure!("Unknown argument: {}", arg)
    }
}

fn do_with_buildin_arg_builder(first_arg: &str, args: &[String], builder_name: &str) {
    let builder_version = &first_arg[(builder_name.len() + 3)..];
    if builder_version.is_empty() {
        failure!("Builder version is not assigned!");
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
                    failure!("Assigned java version not found: {}", java_version);
                    return;
                },
            };
        }
    }
    let builder_desc = match tool::get_builder_home(builder_name, builder_version) {
        Some(h) => h, None => {
            failure!("Assigned builder: {}, version: {} not found.", builder_name, builder_version);
            return;
        },
    };
    if has_java {
        success!("JAVA_HOME    = {}", java_home);
    }
    success!("BUILDER_HOME = {}", &builder_desc.home);

    let mut new_env = iff!(has_java, get_env_with_java_home(&java_home), get_env());
    for builder_home_name in builder_desc.get_builder_home_name() {
        new_env.insert(builder_home_name, builder_desc.home.clone());
    }

    let mut cmd = Command::new(builder_desc.get_builder_bin());
    cmd.envs(&new_env);
    let from_index = iff!(has_java, 3, 2);
    for arg in args.iter().skip(from_index) {
        cmd.arg(&arg);
    }
    if let Err(err) = util_cmd::run_command_and_wait(&mut cmd) {
        failure!("Run build command failed: {}", err);
    }
}

fn do_with_buildin_arg_ddd(first_arg: &str, args: &[String]) {
    let build_json_object = match read_build_json_object() {
        Some(object) => object, None => return,
    };
    let build_json_object_xrun = &build_json_object["xRuns"][&first_arg[3..]];
    if build_json_object_xrun.is_null() {
        failure!("Cannot find build.json#xRuns#{}", &first_arg[3..]);
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
        debugging!("Running cmd: {}, args: {:?}", &cmd_name, cmd_args);
    }
    if let Err(err) = util_cmd::run_command_and_wait(&mut cmd) {
        failure!("Run xRun command failed: {}", err);
    }
}

fn do_with_buildin_args(args: &[String]) {
    let first_arg = args.get(1).unwrap();
    match first_arg.as_str() {
        ":::" | ":::help" => print_usage(),
        ":::version"      => print_version(),
        ":::create"       => create_build_json(args),
        ":::config"       => do_with_buildin_arg_config(first_arg, args),
        a if a.starts_with(":::java")   => do_with_buildin_arg_java  (a, args),
        a if a.starts_with(":::maven")  => do_with_buildin_arg_maven (a, args),
        a if a.starts_with(":::gradle") => do_with_buildin_arg_gradle(a, args),
        a if a.starts_with("...")       => do_with_buildin_arg_ddd   (a, args),
        _ => failure!("Unknown args: {:?}", &args),
    }
}

fn get_java_and_builder(build_json_object: &json::JsonValue) -> Option<(String, BuilderDesc)> {
    let java_version_j = &build_json_object["java"];
    let builder_name_j = &build_json_object["builder"]["name"];
    let builder_version_j = &build_json_object["builder"]["version"];

    if java_version_j.is_null() {
        failure!("Java version is not assigned!");
        return None;
    }
    if builder_name_j.is_null() || builder_version_j.is_null() {
        failure!("Builder name or version is not assigned!");
        return None;
    }
    let java_version = java_version_j.as_str().unwrap();
    let builder_name = builder_name_j.as_str().unwrap();
    let builder_version = builder_version_j.as_str().unwrap();
    if *VERBOSE {
        debugging!("Java version: {}", java_version);
        debugging!("Builder name: {}", builder_name);
        debugging!("Builder version: {}", builder_version);
    }

    let java_home = match get_java_home(java_version) {
        Some(h) => h, None => {
            failure!("Assigned java version not found: {}", java_version);
            return None;
        },
    };
    let builder_desc = match tool::get_builder_home(builder_name, builder_version) {
        Some(h) => h, None => {
            failure!("Assigned builder: {}, version: {} not found.", builder_name, builder_version);
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
                warning!("xArgs argument not found: {}", a_cmd);
                if args.len() == 2 {
                    failure!("Only one xArgs argument, exit.");
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
                debugging!("Env: {}", env);
            }
            let (env_k, env_v) = (&env[0], &env[1]);
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
                warning!("Unknown builder: {}", builder_version);
            }
        }
        if *VERBOSE {
            debugging!("Use env configed build.json: {}",  json::stringify(build_json_object.clone()));
        }
        success!("Find build.json @ENV");
        Some(build_json_object)
    } else {
        None
    }
}

fn read_build_json_object() -> Option<json::JsonValue> {
    if let Some(o) = read_build_json_object_from_env() {
        return Some(o);
    }

    let build_json = find_build_json()?;
    success!("Find {} @ {}", BUILD_JSON, build_json);

    let build_json_content = fs::read_to_string(build_json).map_err(|err| {
        failure!("Read {} failed: {}", BUILD_JSON, err);
        err
    }).ok()?;
    json::parse(&build_json_content).map_err(|err| {
        failure!("Parse JSON failed: {}", err);
        err
    }).ok()
}


fn main() {
    match get_short_git_hash() {
        None => information!("{} - version {}", BUILDJ, BUDERJ_VER),
        Some(shot_git_hash) => information!("{} - version {} - {}", BUILDJ, BUDERJ_VER, &shot_git_hash),
    }

    if *VERBOSE {
        if let Some(full_git_hash) = get_full_git_hash() {
            debugging!("Full GIT_HASH: {}", full_git_hash);
        }
        debugging!("Build date: {}", BUILD_DATE);
    }

    let args = local_util::get_args_as_vec();
    information!("Arguments: {:?}", args);

    if (! *NOBUILDIN) && local_util::is_buildin_args(&args) {
        do_with_buildin_args(&args);
        return;
    }
    if *VERBOSE {
        debugging!("Init home dir: {}", tool::LOCAL_BUILDER_HOME_BASE_DIR);
    }
    local_util::init_home_dir(tool::LOCAL_BUILDER_HOME_BASE_DIR);
    if *VERBOSE {
        debugging!("Init home dir: {}", jdk::LOCAL_JAVA_HOME_BASE_DIR);
    }
    local_util::init_home_dir(jdk::LOCAL_JAVA_HOME_BASE_DIR);

    let build_json_object = match read_build_json_object() {
        Some(object) => object, None => return,
    };

    let (java_home, builder_desc) = match get_java_and_builder(&build_json_object) {
        Some((java_home, builder_desc)) => (java_home, builder_desc), None => return,
    };
   
    success!("JAVA_HOME    = {}", java_home);
    success!("BUILDER_HOME = {}", &builder_desc.home);

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
        debugging!("Final arguments: {:?}", &final_args);
    }
    for f_arg in final_args {
        cmd.arg(f_arg);
    }
    if *VERBOSE {
        debugging!("-----BEGIN ENVIRONMENT VARIABLES-----");
        for (k, v) in new_env {
            debugging!("{}={}", k, v);
        }
        debugging!("-----END ENVIRONMENT VARIABLES-----");
    }
    let exit_status = util_cmd::run_command_and_wait(&mut cmd).unwrap_or_else(|err| {
        failure!("Run build command failed: {}", err);
        process::exit(-1);
    });

    if !exit_status.success() {
        if let Some(exit_code) = exit_status.code() {
            process::exit(exit_code);
        }
    }
}
