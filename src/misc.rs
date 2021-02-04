use std::env;
use rust_util::util_env;
use rust_util::util_term;

lazy_static! {
    pub static ref VERBOSE: bool   = util_env::is_env_on("BUILDJ_VERBOSE");
    pub static ref NOAUTH: bool    = util_env::is_env_on("BUILDJ_NOAUTH");
    pub static ref NOBUILDIN: bool = util_env::is_env_on("BUILDJ_NOBUILDIN");
    pub static ref AUTH_TOKEN: Option<String>      = env::var("BUILDJ_AUTH_TOKEN").ok();
    pub static ref JAVA_VERSION: Option<String>    = env::var("BUILDJ_JAVA").ok();
    pub static ref BUILDER_VERSION: Option<String> = env::var("BUILDJ_BUILDER").ok();
    pub static ref BUILD_YEAR: String              = env::var("BUILD_YEAR").unwrap_or_else(|_| "unknown".to_string());
}

pub fn print_usage() {
    println!("\n{}", include_str!("usage.txt"));
}

pub fn print_version() {
  println!(r#"buildj {} - {}
Full git commit hash: {}{}{}

Copyright (C) 2019-{} Hatter Jiang.
License MIT <{}https://opensource.org/licenses/MIT{}>

Official website: {}https://buildj.ruststack.org/{}
"#, super::BUDERJ_VER,
           &super::GIT_HASH[0..7],
           util_term::BOLD, &super::GIT_HASH, util_term::END,
           *BUILD_YEAR,
           util_term::UNDER, util_term::END,
           util_term::UNDER, util_term::END);
}

