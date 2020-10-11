# buildj
buildj - java build tool, website: [https://buildj.ruststack.org/](https://buildj.ruststack.org/)

## Install

```
cargo install --git https://github.com/jht5945/buildj [--force]
```

## Usage

### Help
```
$ buildj :::
[INFO] buildj - version 0.1
[INFO] Arguments: ["./buildj", ":::"]

buildj :::                                             - print this message
buildj :::help                                         - print this message
buildj :::create --java<version> --maven<version>      - create java-version, maven-version project
buildj :::create --java<version> --gradle<version>     - create java-version, gradle-version project
buildj :::java<version> [-version]                     - run java with assigned version, e.g. buildj :::java1.8 -version
buildj :::maven<version> [--java<version>]             - run maven with assigned version and java version, e.g. buildj :::maven3.5.2 --java1.8 ARGS
buildj :::gradle<version> ]--java<version>]            - run gradle with assigned version and java version, e.g. buildj :::gradle3.5.1 --java1.8 ARGS
buildj                                                 - run build, run assigned version builder tool
```

### Run Java
```
$ buildj :::java9 -version
[INFO] buildj - version 0.1
[INFO] Arguments: ["./buildj", ":::java9", "-version"]
[OK   ] Find java home: /Library/Java/JavaVirtualMachines/jdk-9.0.4.jdk/Contents/Home
java version "9.0.4"
Java(TM) SE Runtime Environment (build 9.0.4+11)
Java HotSpot(TM) 64-Bit Server VM (build 9.0.4+11, mixed mode)
```

Run Maven:
```
$ buildj :::maven3.5.2 -version
[INFO] buildj - version 0.1
[INFO] Arguments: ["./buildj", ":::maven3.5.2", "-version"]
[OK   ] BUILDER_HOME = /Users/hatterjiang/.jssp/builder/maven-3.5.2/apache-maven-3.5.2
Apache Maven 3.5.2 (138edd61fd100ec658bfa2d307c43b76940a5d7d; 2017-10-18T15:58:13+08:00)
Maven home: /Users/hatterjiang/.jssp/builder/maven-3.5.2/apache-maven-3.5.2
Java version: 1.8.0, vendor: Oracle Corporation
Java home: /Library/Java/JavaVirtualMachines/jdk1.8.0.jdk/Contents/Home/jre
Default locale: en_US, platform encoding: UTF-8
OS name: "mac os x", version: "10.14.4", arch: "x86_64", family: "mac"
```

### Run Gradle
```
$ buildj :::gradle3.5.1 -version
[INFO] buildj - version 0.1
[INFO] Arguments: ["./buildj", ":::gradle3.5.1", "-version"]
[OK   ] BUILDER_HOME = /Users/hatterjiang/.jssp/builder/gradle-3.5.1/gradle-3.5.1

------------------------------------------------------------
Gradle 3.5.1
------------------------------------------------------------

Build time:   2017-06-16 14:36:27 UTC
Revision:     d4c3bb4eac74bd0a3c70a0d213709e484193e251

Groovy:       2.4.10
Ant:          Apache Ant(TM) version 1.9.6 compiled on June 29 2015
JVM:          1.8.0 (Oracle Corporation 25.0-b70)
OS:           Mac OS X 10.14.4 x86_64
```

Create build.json
```
$ buildj :::create --java1.8 --maven3.5.1
[INFO] buildj - version 0.1
[INFO] Arguments: ["./buildj", ":::create", "--java1.8", "--maven3.5.1"]
[OK   ] Write file success: build.json

$ cat build.json 
{
    "java": "1.8",
    "builder": {
        "name": "gradle",
        "version": "3.5.1"
    }
}
```

Run:
```
$ buildj
```

<br>

Add environment in build:
```
{
    "envs": [
        ["VAR_NAME", "VAR_VALUE"]
    ]
}
```

<br>

Use xArgs in build:
```
{
    xArgs: {
        "build": ["clean", "install"]
    }
}
```
Command: `buildj ::build`

<br>

Use xRuns in build:
```
{
    xRuns: {
        "pub": ["./publish"]
    }
}
```
Command: `buildj ...pub`

