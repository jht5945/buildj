use std::env;
use rust_util::util_env::*;

lazy_static! {
    pub static ref VERBOSE: bool = is_env_on("BUILDJ_VERBOSE");
    pub static ref NOAUTH: bool = is_env_on("BUILDJ_NOAUTH");
    pub static ref NOBUILDIN: bool = is_env_on("BUILDJ_NOBUILDIN");
    pub static ref AUTH_TOKEN: Option<String> = env::var("BUILDJ_AUTH_TOKEN").ok();
    pub static ref JAVA_VERSION: Option<String> = env::var("BUILDJ_JAVA").ok();
    pub static ref BUILDER_VERSION: Option<String> = env::var("BUILDJ_BUILDER").ok();
}

pub fn print_usage() {
    println!("\n{}", include_str!("usage.txt"));
}

pub fn print_version() {
  println!(r#"buildj {} - {}
Copyright (C) 2019-2020 Hatter Jiang.
License MIT <https://opensource.org/licenses/MIT>

Written by Hatter Jiang"#, super::BUDERJ_VER, &super::GIT_HASH[0..7]);
}

