use std::env;
use rust_util::{util_env, util_term};

pub const BUILDJ:     &str = "buildj";
pub const BUDERJ_VER: &str = env!("CARGO_PKG_VERSION");
pub const BUILD_DATE: &str = env!("BUILD_DATE");
const     GIT_HASH:   &str = env!("GIT_HASH");


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
  println!(r#"buildj {}{}{}
Build date: {}

Copyright (C) 2019-{} Hatter Jiang.
License MIT <{}https://opensource.org/licenses/MIT{}>

Official website: {}https://buildj.ruststack.org/{}
"#, BUDERJ_VER,
           get_short_git_hash().map(|h| format!(" - {}", h)).unwrap_or("".into()),
           get_full_git_hash().map(|h| format!("\nFull git commit hash: {}{}{}", util_term::BOLD, h, util_term::END)).unwrap_or("".into()),
           BUILD_DATE,
           *BUILD_YEAR,
           util_term::UNDER, util_term::END,
           util_term::UNDER, util_term::END);
}

pub fn get_full_git_hash() -> Option<&'static str> {
    // build from crates, git hash is empty
    iff!(GIT_HASH.is_empty(), None, Some(GIT_HASH))
}

pub fn get_short_git_hash() -> Option<&'static str> {
    get_full_git_hash().map(|h| &h[0..7])
}
