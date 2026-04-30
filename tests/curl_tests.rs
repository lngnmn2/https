//! Comparison tests with the system `curl` command.
//! Refactored for Pure Functional compliance.

use https_client::{get, Response, Status, HttpError};
use https_client::interpreter::runner;
use std::process::Command;
use std::rc::Rc;
use std::ops::Deref;

fn run_curl(method: &str, url: &str) -> Result<Response, HttpError> {
    Command::new("curl")
        .arg("-i")
        .arg("-s")
        .arg("--http1.1")
        .arg("-X")
        .arg(method)
        .arg("--tls-max")
        .arg("1.3")
        .arg("--tlsv1.2")
        .arg(url)
        .output()
        .map_err(|e| HttpError::TransportError(Rc::from(e.to_string())))
        .and_then(|output| parse_curl_output(Rc::from(output.stdout)))
}

fn parse_curl_output(stdout: Rc<[u8]>) -> Result<Response, HttpError> {
    std::str::from_utf8(stdout.deref())
        .map_err(|_| HttpError::ResponseError(Rc::from("UTF8 Fail")))
        .and_then(|s| s.split("\r\n").next().ok_or(HttpError::ResponseError(Rc::from("Empty Curl Output"))))
        .and_then(|status_line| parse_curl_status(status_line))
        .map(|status| Response::new(status, Rc::from([]), https_client::Body::default()))
}

fn parse_curl_status(line: &str) -> Result<Status, HttpError> {
    let parts: Rc<[&str]> = line.splitn(3, ' ').collect();
    parts.get(1)
        .ok_or(HttpError::ResponseError(Rc::from(line)))
        .and_then(|s| s.parse::<u16>().map(Status::from).map_err(|_| HttpError::ResponseError(Rc::from(*s))))
}

fn run_my_client(url: &str) -> Result<Response, HttpError> {
    get!(url).and_then(|req| runner::execute(&req))
}

#[test]
fn test_compare_with_curl_github() -> Result<(), Box<dyn std::error::Error>> {
    let url = "https://github.com/";
    
    let curl_res = run_curl("GET", url)?;
    let my_res = run_my_client(url)?;
    
    if my_res.status().code() == curl_res.status().code() {
        Ok(())
    } else {
        Err(format!("Status mismatch: client={} curl={}", my_res.status().code(), curl_res.status().code()).into())
    }
}
