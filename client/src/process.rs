use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::env;
use std::io::{BufRead, BufReader, Error, ErrorKind};
use std::process::{Command, Stdio};

#[derive(Serialize, Deserialize)]
struct SshLog {
    failed: bool,
    hostname: String,
    ip: String,
    pid: String,
    port: String,
    timestamp: String,
    user: String,
}

struct IpAndPort<'a> {
    ip: &'a str,
    port: &'a str,
}

#[derive(Deserialize, Debug)]
struct AlphaLog {
    id: String,
    logged: bool,
}

fn process_message(message: &str) -> Result<IpAndPort, &'static str> {
    let re = Regex::new(
        r"
        (?x)([0-9]{1,3}\.[0-9]{1,3}\.[0-9]{1,3}\.[0-9]{1,3}) 
        \s+ port \s+ 
        ([0-9]+)
        ",
    )
    .unwrap();

    match re.captures(message) {
        Some(caps) => Ok(IpAndPort {
            ip: caps.get(1).map_or("", |m| m.as_str()),
            port: caps.get(2).map_or("", |m| m.as_str()),
        }),
        None => Err("match not found"),
    }
}

fn process_line(json_string: String) -> Result<Value, String> {
    let log_line: Value = serde_json::from_str(&json_string).unwrap();

    let message = &log_line["MESSAGE"].as_str().unwrap();
    match process_message(message) {
        Ok(ip_and_port) => {
            let timestamp_microseconds = &log_line["_SOURCE_REALTIME_TIMESTAMP"].as_str().unwrap();
            let timestamp_seconds: String =
                timestamp_microseconds[..timestamp_microseconds.len() - 6].to_string();

            Ok(json!({
                "failed": "true",
                "hostname": log_line["_HOSTNAME"].as_str(),
                "ip": ip_and_port.ip.to_owned(),
                "port": ip_and_port.port.to_owned(),
                "pid": log_line["_PID"].as_str(),
                "timestamp": timestamp_seconds.to_owned(),
                "user": "[placeholder]".to_string(),
            }))
        }
        Err(e) => {
            eprintln!("cannot process log message: {}", e);
            Err(e.to_string())
        },
    }
}

fn send_log(log_message: Value) -> Result<(), std::env::VarError> {
    let request_url = match env::var("AS_ADDRESS") {
        Ok(url) => {
            let response = ureq::post(&url).send_json(log_message);
            println!("Log reported with id: {:#?}", response.unwrap());
        }
        Err(e) => return Err(e),
    };
    Ok(request_url)
}

pub fn main() -> Result<(), Error> {
    // Run journalctl -f to continously display SSH logs as they appear
    let stdout = Command::new("journalctl")
        // .arg("_COMM=sshd")
        .arg("-f")
        .arg("-o")
        .arg("json")
        .stdout(Stdio::piped())
        .spawn()?
        .stdout
        .ok_or_else(|| Error::new(ErrorKind::Other, "Could not capture STDOUT."))?;

    let reader = BufReader::new(stdout);

    reader
        .lines()
        .filter_map(|line| line.ok())
        .filter_map(|line| process_line(line).ok())
        .for_each(|line| println!("{}", line));
    // .for_each(|line| send_log(line).unwrap());

    Ok(())
}
