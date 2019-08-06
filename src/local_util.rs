use std::{
    env,
    fs::{self, File},
    io::{Read, ErrorKind},
    path::Path,
    process::Command,
};

use super::rust_util::{
    new_box_error,
    util_msg::{
        print_message,
        MessageType,
    },
    XResult,
    util_io::DEFAULT_BUF_SIZE,
};

use crypto::{
    sha2::Sha256,
    digest::Digest,
};

pub fn get_args_as_vec() -> Vec<String> {
    let mut args_vec:Vec<String> = vec![];
    for arg in env::args() {
        args_vec.push(arg);
    }
    args_vec
}

pub fn is_buildin_args(args: &Vec<String>) -> bool {
    match args.len() <= 1 {
        true => false,
        false => args.get(1).unwrap().starts_with(":::"),
    }
}

pub fn verify_file_integrity(integrity: &str, file_name: &str) -> XResult<bool> {
    if ! integrity.starts_with("sha256:hex-") {
        return Err(new_box_error(&format!("Not supported integrigty: {}", integrity)));
    }
    let sha256_hex = &integrity[11..];
    let calc_sha256_hex = calc_file_sha256(file_name)?;
    let integrity_verify_result = sha256_hex == calc_sha256_hex;
    if ! integrity_verify_result {
        print_message(MessageType::ERROR, &format!("Verify integrity failed, expected: {}, actual: {}", sha256_hex, calc_sha256_hex));
    }
    Ok(integrity_verify_result)
}

pub fn calc_sha256(d: &[u8]) -> String {
    let mut sha256 = Sha256::new();
    sha256.input(d);
    sha256.result_str()
}

pub fn calc_file_sha256(file_name: &str) -> XResult<String> {
    let mut sha256 = Sha256::new();
    let mut buf: [u8; DEFAULT_BUF_SIZE] = [0u8; DEFAULT_BUF_SIZE];
    let mut f = File::open(file_name)?;
    loop {
        let len = match f.read(&mut buf) {
            Ok(0) => return Ok(sha256.result_str()),
            Ok(len) => len,
            Err(ref e) if e.kind() == ErrorKind::Interrupted => continue,
            Err(e) => return Err(Box::new(e)),
        };
        sha256.input(&buf[..len]);
    }
}

pub fn get_user_home() -> XResult<String> {
    let home_dir_o = match dirs::home_dir() {
        None => return Err(new_box_error("Home dir not found!")),
        Some(home_dir_o) => home_dir_o,
    };
    match home_dir_o.to_str() {
        None => return Err(new_box_error("Home dir not found!")),
        Some(home_dir_str) => Ok(home_dir_str.to_string()),
    }
}

pub fn get_user_home_dir(dir: &str) -> XResult<String> {
    Ok(format!("{}/{}", get_user_home()?, dir))
}

pub fn is_path_exists(dir: &str, sub_dir: &str) -> bool {
    let full_path = &format!("{}/{}", dir, sub_dir);
    Path::new(full_path).exists()
}

pub fn run_command_and_wait(cmd: &mut Command) -> XResult<()> {
    cmd.spawn()?.wait()?;
    Ok(())
}

pub fn extract_package_and_wait(dir: &str, file_name: &str) -> XResult<()> {
    let mut cmd: Command;
    if file_name.ends_with(".zip") {
        cmd = Command::new("unzip");
    } else if file_name.ends_with(".tar.gz") {
        cmd = Command::new("tar");
        cmd.arg("-xzvf");
    } else {
        return Err(new_box_error(&format!("Unknown file type: {}", file_name)));
    }
    cmd.arg(file_name).current_dir(dir).spawn()?.wait()?;
    Ok(())
}

pub fn init_home_dir(home_sub_dir: &str) {
    match get_user_home_dir(home_sub_dir) {
        Err(_) => (),
        Ok(user_home_dir) => init_dir(&user_home_dir),
    };
}

pub fn init_dir(dir: &str) {
    if ! Path::new(dir).exists() {
        fs::create_dir_all(dir).unwrap_or_else(|err| {
            print_message(MessageType::ERROR, &format!("Init dir {} failed: {}", dir, err));
        });
    }
}
