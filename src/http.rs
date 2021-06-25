pub use http::*;

mod http {
	use std::collections::HashMap;

	#[derive(Debug, PartialEq)]
	pub enum Method {
		GET,
		HEAD,
		POST,
		PUT,
		DELETE,
		TRACE,
		OPTIONS,
		CONNECT,
		PATCH,
		UNKNOWN(String),
	}
	impl Method {
		pub fn new(method: &str) -> Method {
			use Method::*;
			match method {
				"GET" => GET,
				"HEAD" => HEAD,
				"POST" => POST,
				"PUT" => PUT,
				"DELETE" => DELETE,
				"TRACE" => TRACE,
				"OPTIONS" => OPTIONS,
				"CONNECT" => CONNECT,
				"PATCH" => PATCH,
				_ => UNKNOWN(method.to_string()),
			}
		}
	}

	pub type URI = (String, String);

	///### Struct that represents HTTP Request
	#[derive(Debug, PartialEq)]
	pub struct Request {
		pub method: Method,
		pub uri: URI,
		pub headers: HashMap<String, String>,
		pub body: Option<Vec<u8>>,
	}
	impl Request {
		pub fn new(gotten_request: &[u8]) -> Option<Request> {
			if gotten_request.len() == 0 {
				return None;
			}

			let mut previous = gotten_request.first().unwrap();
			// Getting first repeat of \n\r\n\r pattern
			let mut start_of_body: usize = 0;
			let mut is_new_line = false;
			for (index, byte) in gotten_request.iter().enumerate().skip(1) {
				if byte == &b'\n' {
					if previous == &b'\r' && is_new_line {
						start_of_body = index + 1;
						break;
					}
					is_new_line = true;
				}
				if byte != &b'\n' && byte != &b'\r' {
					is_new_line = false;
				}
				previous = byte;
			}
			if start_of_body == 0 {
				return None;
			}
			// Splitting request to request line + headers and body
			let (headers, body) = gotten_request.split_at(start_of_body);

			let formatted_body: Vec<u8> = body.into_iter().map(|val| *val).collect();
			// Checking if all bytes are zero adn setting body to None
			let body = if formatted_body.iter().all(|value| *value == 0) {
				None
			} else {
				Some(formatted_body)
			};

			let lines: Vec<String> = String::from_utf8_lossy(headers)
				.split("\r\n")
				.map(String::from)
				.collect();

			// Parsing request line
			let mut line = lines[0].split(" ").into_iter();
			let mut arg = line.next();
			if arg.is_none() {
				return None;
			}
			let method = Method::new(&arg.unwrap());

			arg = line.next();
			if arg.is_none() {
				return None;
			}
			let uri = if let Some(parsed_uri) = parse_from_unsafe(arg.unwrap().to_string()) {
				parsed_uri
			} else {
				return None;
			};

			// Parsing headers
			let mut headers = HashMap::new();
			for line in lines.into_iter().skip(1) {
				let mut colon_index = None;
				// Looking for firstly appeared ':' in line
				for (index, character) in line.chars().enumerate() {
					if character == ':' {
						colon_index = Some(index);
						break;
					}
				}

				if let Some(index) = colon_index {
					headers.insert(
						line.chars().take(index).collect::<String>().to_lowercase(),
						line.chars().skip(index + 1).collect(),
					);
				}
			}

			Some(Request {
				method,
				uri,
				headers,
				body,
			})
		}
	}

	#[derive(Debug, PartialEq)]
	pub struct Response {
		code: usize,
		headers: HashMap<String, String>,
		body: Vec<u8>,
	}
	impl Response {
		pub fn new() -> Response {
			Response {
				code: 0,
				headers: HashMap::new(),
				body: Vec::new(),
			}
		}
		pub fn set_code(&mut self, code: usize) {
			self.code = code;
		}
		pub fn add_header(&mut self, tag: String, value: String) {
			self.headers.insert(tag.to_lowercase(), value);
		}
		pub fn remove_header(&mut self, tag: &String) {
			self.headers.remove(&tag.to_lowercase());
		}
		pub fn set_body(&mut self, new_body: Vec<u8>) {
			self.body = new_body;
		}
		pub fn append_body(&mut self, mut body_to_append: Vec<u8>) {
			self.body.append(&mut body_to_append);
		}
		pub fn to_bytes(self) -> Vec<u8> {
			let mut string = format!(
				"HTTP/1.1 {} {}\r\n",
				self.code,
				resolve_reason_phrase(&self.code)
			)
			.to_string();
			for (index, value) in self.headers {
				string += &format!("{}: {}\r\n", index, value);
			}
			if self.body.len() != 0 {
				string += "\r\n";
			}

			let mut bytes = Vec::from(string.as_bytes());
			bytes.append(&mut self.body.clone());
			bytes
		}
	}

	fn resolve_reason_phrase(code: &usize) -> String {
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
	/// ## Function to decode unsafe URI
	/// Returns `None` if path contains any encoded character
	pub fn parse_from_unsafe(uri: String) -> Option<URI> {
		use std::u8;
		// This function should parse unsafe characters ONLY in request not in path
		let mut parsed_path: String = String::new();
		// Looking for end of request, if we find any unsafe or percent encoded characters in path return None
		let mut request_start = uri.len();
		for (index, letter) in uri.chars().enumerate() {
			if letter == '%' {
				return None;
			} else if letter == '?' {
				request_start = index;
				break;
			}
			parsed_path.push(letter);
		}

		let mut parsed_query = String::new();
		if uri.len() != request_start {
			parsed_query.reserve(uri.len() - request_start);
			// Parsing all unsafe percent-encoded characters
			let mut hex = String::new();
			hex.reserve(2);
			let mut is_reading = false;
			for letter in uri.chars().skip(request_start) {
				if letter == '%' {
					// Parsing next 2 characters to form hexadecimal and transforming them to characters
					is_reading = true;
				} else if is_reading {
					hex.push(letter);
					if hex.len() == 2 {
						parsed_query.push(u8::from_str_radix(&hex, 16).unwrap_or(0) as char);
						is_reading = false;
						hex.truncate(0);
					}
				} else {
					parsed_query.push(letter);
				};
			}
		}

		Some((parsed_path, parsed_query))
	}

	#[test]
	fn test_request_parsing() {
		let parsed_request = Request::new(
			String::from("GET /index.html HTTP/1.1\r\nHost: 0.0.0.0:7878\r\n").as_bytes(),
		);
		let mut map = HashMap::new();
		map.insert("ost".to_string(), " 0.0.0.0:7878".to_string());
		assert_eq!(
			parsed_request.unwrap(),
			Request {
				method: Method::GET,
				uri: (String::from("/index.html"), String::new()),
				headers: map,
				body: None
			}
		);
	}

	#[test]
	fn test_to_bytes_response() {
		let mut headers = HashMap::new();
		headers.insert("Host".to_string(), "0.0.0.0:7878".to_string());
		let response = Response {
			code: 200,
			headers,
			body: Vec::new(),
		};
		let vector = Vec::from("HTTP/1.1 200 OK\r\nHost: 0.0.0.0:7878\r\n".as_bytes());
		assert_eq!(response.to_bytes(), vector);
	}

	#[test]
	fn test_from_unsafe_parsing() {
		let string_to_parse =
			"sample.com/?hello+world+this+is+a+test+string+%25+.+%2B+-+%3D+-+*".to_string();
		assert_eq!(
			parse_from_unsafe(string_to_parse).unwrap(),
			(
				"sample.com/".to_string(),
				"?hello+world+this+is+a+test+string+%+.+++-+=+-+*".to_string()
			)
		);
	}
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
