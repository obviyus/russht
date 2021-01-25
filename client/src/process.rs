use std::process::{Command, Stdio};
use std::io::{BufRead, BufReader, Error, ErrorKind};
use serde_json::{Value};

pub fn main() -> Result<(), Error> {
    // Run journalctl -f to continously display SSH logs as they appear
    let stdout = Command::new("journalctl")
        .arg("_COMM=sshd")
        .arg("-f")
        .arg("-o json")
        .stdout(Stdio::piped())
        .spawn()?
        .stdout
        .ok_or_else(|| Error::new(ErrorKind::Other,"Could not capture STDOUT."))?;

    let reader = BufReader::new(stdout);

    reader
        .lines()
        .filter_map(|line| line.ok())
        .for_each(|line| process_line(line));

     Ok(())
}

fn process_line(json_string: String) {
    let v: Value = serde_json::from_str(&json_string).unwrap();

    let timestamp = &v["SYSLOG_TIMESTAMP"];
    let message = &v["MESSAGE"];
    let hostname = &v["HOSTNAME"];
    let pid = &v["PID"];
}