// TODO Learn a bit about TCP and HTTP.
// TODO Listen for TCP connections on a socket.
// TODO Parse a small number of HTTP requests.
// TODO Create a proper HTTP response.
// TODO Improve the throughput of our server with a thread pool.

use std::fs;
use std::io::prelude::*;

use std::iter::Iterator;
use std::net::TcpListener;
use std::net::TcpStream;
use std::string::String;

fn main() {
	let listener = TcpListener::bind("127.0.0.1:7878").unwrap();

	for stream in listener.incoming() {
		let stream = stream.unwrap();
		handle_connection(stream);
	}
}

fn handle_connection(mut stream: TcpStream) {
	let mut buffer = [0; 1024];
	stream.read(&mut buffer).unwrap();
	// Lossy means that all UTF-8 chars that can't be recognized will be replaced with "�"
	// println!("{}", String::from_utf8_lossy(&buffer));

	//Splitting request and converting pieces into strings for easier work
	let request: Vec<String> = String::from_utf8_lossy(&buffer)
		.split(" ")
		.map(String::from)
		.collect();

	if request.len() > 3 {
		let code;
		let mut headers = String::new();
		let mut body = String::new();
		match &request[0] as &str {
			"GET" => {
				//Firstly checking if it's not API request
				if request[1].ends_with("?") {
					code = 501;
				} else {
					// If client asking for files, than trying to get files or throw error
					let file_t = get_file(&request[1]);
					code = file_t.0;
					headers += &file_t.2;
					body += &String::from_utf8_lossy(&file_t.1);
				}
			}
			_ => {
				code = 501;
			}
		}
		stream
			.write(generate_response(code, headers, body).as_bytes())
			.unwrap();
		// Waiting till all bytes are sent
		stream.flush().unwrap();
	}
}
fn get_file(path_to_file: &str) -> (u32, Vec<u8>, String) {
	use std::io::ErrorKind::*;

	const NUMBER_OF_RETRIES: u32 = 5;
	let mut real_path = format!("root{}", path_to_file);
	if real_path.ends_with("/") {
		real_path += "index.html";
	}
	let mut code = 404;
	// Trying to get file contents
	let content = match fs::read(&real_path) {
		Ok(text) => {
			code = 200;
			text
		}
		Err(err) => {
			//Trying to resolve errors
			match err.kind() {
				TimedOut | Interrupted => {
					// trying NUMBER_OF_RETRIES to get file again
					let mut retries = NUMBER_OF_RETRIES;
					loop {
						if retries == 0 {
							code = 404;
							break get_404().1;
						} else {
							retries -= 1;
						}
						let val = fs::read(&real_path);
						if val.is_ok() {
							break val.unwrap();
						}
					}
				}
				// Arm for not found error and any other
				_ => get_404().1,
			}
		}
	};
	let headers = format!("");
	(code, content, headers)
}

fn get_404() -> (u32, Vec<u8>) {
	//Trying to reach root/404.html file if not just giving default output
	(
		404,
		fs::read("root/404.html").unwrap_or(String::from("Error 404 file not found!").into_bytes()),
	)
}

fn resolve_reason_phrase(code: &u32) -> String {
	String::from(match code {
		100 => "Continue",
		101 => "Switching Protocols",
		200 => "OK",
		201 => "Created",
		202 => "Accepted",
		203 => "Non-Authoritative Information",
		204 => "No Content",
		205 => "Reset Content",
		206 => "Partial Content",
		300 => "Multiple Choices",
		301 => "Moved Permanently",
		302 => "Found",
		303 => "See Other",
		304 => "Not Modified",
		305 => "Use Proxy",
		307 => "Temporary Redirect",
		400 => "Bad Request",
		401 => "Unauthorized",
		402 => "Payment Required",
		403 => "Forbidden",
		404 => "Not Found",
		405 => "Method Not Allowed",
		406 => "Not Acceptable",
		407 => "Proxy Authentication Required",
		408 => "Request Time-out",
		409 => "Conflict",
		410 => "Gone",
		411 => "Length Required",
		412 => "Precondition Failed",
		413 => "Request Entity Too Large",
		414 => "Request-URI Too Large",
		415 => "Unsupported Media Type",
		416 => "Requested range not satisfiable",
		417 => "Expectation Failed",
		500 => "Internal Server Error",
		501 => "Not Implemented",
		502 => "Bad Gateway",
		503 => "Service Unavailable",
		504 => "Gateway Time-out",
		505 => "HTTP Version not supported extension-code",
		_ => "",
	})
}
fn generate_response(code: u32, headers: String, body: String) -> String {
	format!(
		"HTTP/1.1 {} {}\n\r{}\n\r{}",
		&code,
		resolve_reason_phrase(&code),
		headers,
		body
	)
}

// In HTTP request looks like this:
/*
Method Request-URI HTTP-Version CRLF
headers CRLF
message-body
*/
// And response have such appearance
/*
HTTP-Version Status-Code Reason-Phrase CRLF
headers CRLF
message body
*/
