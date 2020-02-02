use std::{
    env,
    fs::{self, File},
    io::{Read, ErrorKind},
    path::Path,
    process::Command,
    time::SystemTime,
};

use rust_util::{
    XResult,
    new_box_ioerror,
    util_msg::{
        print_message,
        MessageType,
    },
    util_io::*,
};

use crypto::{
    digest::Digest,
    md5::Md5,
    sha1::Sha1,
    sha2::{Sha256, Sha512},
};

pub fn get_args_as_vec() -> Vec<String> {
    let mut args_vec:Vec<String> = vec![];
    for arg in env::args() {
        args_vec.push(arg);
    }
    args_vec
}

pub fn is_buildin_args(args: &[String]) -> bool {
    if args.len() <= 1 {
        false
    } else {
        args.get(1).unwrap().starts_with(":::")
        || args.get(1).unwrap().starts_with("...")
    }
}

pub fn verify_file_integrity(integrity: &str, file_name: &str) -> XResult<bool> {
    match integrity.find('-') {
        None => Err(new_box_ioerror(&format!("Not supported integrigty: {}", integrity))),
        Some(index) => {
            let digest_hex = &integrity[index+1..];
            let calc_digest_hex = match &integrity[0..index] {
                "sha256:hex" => calc_file_digest(&mut Sha256::new(), "SHA256", file_name)?,
                "sha512:hex" => calc_file_digest(&mut Sha512::new(), "SHA512", file_name)?,
                "sha1:hex" => calc_file_digest(&mut Sha1::new(), "SHA1", file_name)?,
                "md5:hex" => calc_file_digest(&mut Md5::new(), "MD5", file_name)?,
                _ => return Err(new_box_ioerror(&format!("Not supported integrigty: {}", integrity))),
            };
            let integrity_verify_result = digest_hex == calc_digest_hex.as_str();
            if ! integrity_verify_result {
                print_message(MessageType::ERROR, &format!("Verify integrity failed, expected: {}, actual: {}", digest_hex, calc_digest_hex));
            }
            Ok(integrity_verify_result)
        },
    }
}

pub fn calc_sha256(d: &[u8]) -> String {
    let mut sha256 = Sha256::new();
    sha256.input(d);
    sha256.result_str()
}

pub fn calc_file_digest(digest: &mut dyn Digest, digest_alg: &str, file_name: &str) -> XResult<String> {
    let mut buf: [u8; DEFAULT_BUF_SIZE] = [0u8; DEFAULT_BUF_SIZE];
    let mut f = File::open(file_name)?;
    let file_len = match f.metadata() {
        Err(_) => -1_i64,
        Ok(meta_data) => meta_data.len() as i64,
    };
    let start = SystemTime::now();
    let mut written = 0_i64;
    loop {
        let len = match f.read(&mut buf) {
            Ok(0) => { println!(); return Ok(digest.result_str()); },
            Ok(len) => len,
            Err(ref e) if e.kind() == ErrorKind::Interrupted => continue,
            Err(e) => return Err(Box::new(e)),
        };
        digest.input(&buf[..len]);
        written += len as i64;
        let cost = SystemTime::now().duration_since(start.clone()).unwrap();
        print_status_last_line(&format!("Calc {}", digest_alg), file_len, written, cost);
    }
}

pub fn get_user_home() -> XResult<String> {
   match dirs::home_dir() {
        None => Err(new_box_ioerror("Home dir not found!")),
        Some(home_dir_o) => match home_dir_o.to_str() {
            None => Err(new_box_ioerror("Home dir not found!")),
            Some(home_dir_str) => Ok(home_dir_str.to_string()),
        },
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
        return Err(new_box_ioerror(&format!("Unknown file type: {}", file_name)));
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
