
pub fn print_usage() {
    print!(r#"
buildj :::                                             - print this message
buildj :::help                                         - print this message
buildj :::create --java<version> --maven<version>      - create java-version, maven-version project
buildj :::create --java<version> --gradle<version>     - create java-version, gradle-version project
buildj :::java<version> [-version]                     - run java with assigned version, e.g. buildj :::java1.8 -version
buildj :::maven<version> [--java<version>]             - run maven with assigned version and java version, e.g. buildj :::maven3.5.2 --java1.8 ARGS
buildj :::gradle<version> [--java<version>]            - run gradle with assigned version and java version, e.g. buildj :::gradle3.5.1 --java1.8 ARGS
buildj                                                 - run build, run assigned version builder tool
"#);
}

