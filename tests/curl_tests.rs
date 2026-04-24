//! Comparison tests with the system `curl` command.

use https_client::{InitialRequest, Method, Response, Body, Status};
use https_client::interpreter::runner;
use std::process::Command;
use std::io::{Cursor, BufRead};

fn run_curl(method: &str, url: &str) -> Response {
    let output = Command::new("curl")
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
        .expect("Failed to execute curl");
    
    // Note: We use a simplified parser for curl output as it might not have all security headers
    // required for our financial-grade internal client validation.
    let mut br = std::io::BufReader::new(Cursor::new(output.stdout));
    let mut line = String::new();
    br.read_line(&mut line).unwrap();
    let parts: Vec<&str> = line.trim().splitn(3, ' ').collect();
    if parts.len() < 2 {
        panic!("Invalid curl output status line: {:?}", line);
    }
    let status = Status::from(parts[1].parse::<u16>().unwrap());
    
    // Skip headers
    loop {
        line.clear();
        br.read_line(&mut line).unwrap();
        if line.trim().is_empty() { break; }
    }
    
    let mut body = Vec::new();
    std::io::Read::read_to_end(&mut br, &mut body).unwrap();
    
    Response::new(status, vec![], Body::from(body))
}

fn run_my_client(method: Method, url: &str) -> Response {
    let req = InitialRequest::try_new(method, url).unwrap()
        .with_body(Body::default());
    runner::execute(&req).expect("My client failed")
}

#[test]
fn test_compare_with_curl_github() {
    let url = "https://github.com/";
    
    let curl_res = run_curl("GET", url);
    let my_res = run_my_client(Method::Get, url);
    
    assert_eq!(my_res.status().code(), curl_res.status().code());
}
