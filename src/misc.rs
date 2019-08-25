
use rust_util::util_env::*;

lazy_static! {
    pub static ref VERBOSE: bool = is_env_on("BUILDJ_VERBOSE");
    pub static ref NOAUTH: bool = is_env_on("BUILDJ_NOAUTH");
    pub static ref NOBUILDIN: bool = is_env_on("BUILDJ_NOBUILDIN");
}

pub fn print_usage() {
    print!(r#"
buildj :::                                          - print this message
buildj :::help                                      - print this message
buildj :::version                                   - print version
buildj :::create --java<version> --maven<version>   - create java + maven project
  e.g. buildj :::create --java1.8 --maven3.5.2
buildj :::create --java<version> --gradle<version>  - create java + gradle project
  e.g. buildj :::create --java1.8 --gradle3.5.1
buildj :::java<version> [-version]                  - run java with assigned version
  e.g. buildj :::java1.8 -version
buildj :::maven<version> [--java<version>]          - run maven with assigned version and java version
  e.g. buildj :::maven3.5.2 --java1.8 ARGS
buildj :::gradle<version> [--java<version>]         - run gradle with assigned version and java version
  e.g. buildj :::gradle3.5.1 --java1.8 ARGS
buildj                                              - run build, run assigned version builder tool
BUILDJ_VERBOSE=1 buildj                             - run buildj in verbose mode
"#);
}

pub fn print_version() {
  print!(r#"buildj {} - {}
Copyright (C) 2019 Hatter Jiang.
License MIT <https://opensource.org/licenses/MIT>

Written by Hatter Jiang
"#, super::BUDERJ_VER, &super::GIT_HASH[0..7]);
}

