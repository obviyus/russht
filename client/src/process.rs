use std::env;
use std::io::{BufRead, BufReader, Error, ErrorKind};
use std::process::{Command, Stdio};

use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::{Value};

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
        (?x)([0-9]{1,3}\.[0-9]{1,3}\.[0-9]{1,3}\.[0-9]{1,3}) # IP Address
        \s+ port \s+                                         
        ([0-9]+)                                             # Port
        ",
    )
    .unwrap();

    println!("{:#?}", message);

    match re.captures(message) {
        Some(caps) => Ok(IpAndPort {
            ip: caps.get(1).map_or("", |m| m.as_str()),
            port: caps.get(2).map_or("", |m| m.as_str()),
        }),
        None => Err("match not found"),
    }
}

fn process_line(json_string: String) -> Result<SshLog, &'static str> {
    let log_line: Value = serde_json::from_str(&json_string).unwrap();

    match &log_line["MESSAGE"].as_str() {
        Some(message) => match process_message(message) {
            Ok(ip_and_port) => {
                let timestamp_microseconds =
                    &log_line["_SOURCE_REALTIME_TIMESTAMP"].as_str().unwrap();
                let timestamp_seconds: String =
                    timestamp_microseconds[..timestamp_microseconds.len() - 6].to_string();

                Ok(SshLog {
                    failed: false,
                    hostname: log_line["_HOSTNAME"].as_str().unwrap().parse().unwrap(),
                    ip: ip_and_port.ip.parse().unwrap(),
                    pid: log_line["_PID"].to_string(),
                    port: ip_and_port.port.parse().unwrap(),
                    timestamp: timestamp_seconds.to_string(),
                    user: "[placeholder]".to_string(),
                })
            }
            Err(e) => {
                Err(e)
            }
        },
        None => Err("no MESSAGE field present in log"),
    }
}

fn send_log(log_message: Value) -> Result<(), std::env::VarError> {
    let request_url = match env::var("ALPHA_SERVER_ADDRESS") {
        Ok(url) => {
            let response = ureq::post(&url).send_json(log_message);
            println!("Log reported with id: {:#?}", response.unwrap());
        }
        Err(e) => return Err(e),
    };

    Ok(request_url)
}

pub fn main() -> Result<(), Error> {
    // Run journalctl -f to continuously display SSH logs as they appear
    let stdout = Command::new("journalctl")
        .arg("_COMM=sshd")
        .arg("-f")
        .arg("-o")
        .arg("json")
        .stdout(Stdio::piped())
        .spawn()?
        .stdout
        .ok_or_else(|| Error::new(ErrorKind::Other, "Could not capture STDOUT"))?;

    let reader = BufReader::new(stdout);

    reader
        .lines()
        .filter_map(|line| line.ok())
        .filter_map(|line| process_line(line).ok())
        .for_each(|line| println!("{}", line.ip));
    // .for_each(|line| send_log(line).unwrap());

    Ok(())
}
