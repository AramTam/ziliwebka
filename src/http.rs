// TODO better error handling for receiving and sending messages

pub use http::*;
mod http {
	use std::cell::Cell;
	use std::collections::HashMap;
	use std::io::prelude::*;
	use std::net::TcpStream;

	#[derive(Debug, PartialEq, Clone)]
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

	#[derive(Debug, PartialEq, Clone)]
	pub enum URIQuery {
		SingleParamsList(Vec<String>),
		PairList(HashMap<String, String>),
	}
	/// ### URI representation
	/// It consists of path and optional query
	pub type URI = (String, Option<URIQuery>);

	///### Struct that represents HTTP Request
	pub struct Request {
		stream: Cell<TcpStream>,
		method: Option<Method>,
		uri: Option<URI>,
		headers: Option<HashMap<String, String>>,
		body: Option<Vec<u8>>,
	}

	pub struct SafeRequest(Request);

	impl Request {
		pub fn new(mut stream: TcpStream) -> Result<SafeRequest, Request> {
			let mut gotten_request = [0; 2048];
			stream.read(&mut gotten_request).unwrap();

			if gotten_request.len() == 0 {
				return Err(Request::from(stream, None, None, None, None));
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
				return Err(Request::from(stream, None, None, None, None));
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
				return Err(Request::from(stream, None, None, None, None));
			}
			let method = Some(Method::new(&arg.unwrap()));

			arg = line.next();
			if arg.is_none() {
				return Err(Request::from(stream, method, None, None, None));
			}

			let uri = if let Some(parsed_uri) = parse_from_unsafe(arg.unwrap().to_string()) {
				Some(parsed_uri)
			} else {
				return Err(Request::from(stream, method, None, None, None));
			};

			// Parsing headers
			let mut headers = Some(HashMap::new());
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
					headers.as_mut().unwrap().insert(
						line.chars().take(index).collect::<String>().to_lowercase(),
						line.chars().skip(index + 1).collect(),
					);
				}
			}

			Ok(SafeRequest(Request {
				stream: Cell::new(stream),
				method: method,
				uri: uri,
				headers: headers,
				body,
			}))
		}
		fn from(
			mut stream: TcpStream,
			method: Option<Method>,
			uri: Option<URI>,
			headers: Option<HashMap<String, String>>,
			body: Option<Vec<u8>>,
		) -> Request {
			Request {
				stream: Cell::new(stream),
				method,
				uri,
				headers,
				body,
			}
		}
		pub fn method(&self) -> Option<Method> {
			self.method.clone()
		}
		pub fn uri(&self) -> Option<URI> {
			self.uri.clone()
		}
		pub fn headers(&self) -> Option<HashMap<String, String>> {
			self.headers.clone()
		}
		pub fn body(&self) -> Option<Vec<u8>> {
			self.body.clone()
		}
		pub fn respond(self, response: Response) {
			self.stream.into_inner().write(&response.to_bytes());
		}
	}

	impl SafeRequest {
		pub fn method(&self) -> Method {
			self.0.method.clone().unwrap()
		}
		pub fn uri(&self) -> URI {
			self.0.uri.clone().unwrap()
		}
		pub fn headers(&self) -> HashMap<String, String> {
			self.0.headers.clone().unwrap()
		}
		pub fn body(&self) -> Vec<u8> {
			self.0.body.clone().unwrap()
		}
		pub fn respond(self, response: Response) {
			self.0.stream.into_inner().write(&response.to_bytes());
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

	/// Function to parse all hex encoded characters
	pub fn parse_hex(string_with_hex: &str) -> String {
		let mut parsed = String::new();
		parsed.reserve(string_with_hex.len());
		let mut hex = String::new();
		hex.reserve(2);
		let mut is_reading = false;
		for letter in string_with_hex.chars() {
			if letter == '%' {
				// Parsing next 2 characters to form hexadecimal and transforming them to characters
				is_reading = true;
			} else if is_reading {
				hex.push(letter);
				if hex.len() == 2 {
					parsed.push(u8::from_str_radix(&hex, 16).unwrap_or(0) as char);
					is_reading = false;
					hex.truncate(0);
				}
			} else {
				parsed.push(letter);
			};
		}
		parsed
	}

	/// ## Function to decode unsafe URI
	/// Returns `None` if path contains any encoded character
	pub fn parse_from_unsafe(uri: String) -> Option<URI> {
		// This function should parse unsafe characters ONLY in request not in path
		// Looking for end of request, if we find any unsafe or percent encoded characters in path return None
		let mut request_start = uri.len();
		for (index, letter) in uri.chars().enumerate() {
			if letter == '%' {
				return None;
			} else if letter == '?' {
				request_start = index;
				break;
			}
		}
		let (parsed_path, query) = uri.split_at(request_start);

		// Identifying if query consists from single values or pairs of key and value
		let mut parsed_query = Option::<URIQuery>::None;
		if query.len() > 0 {
			// If query has at least one "=" character it will be parsed as key-value pairs if not it is list of values
			if query.contains("=") {
				let mut map = HashMap::<String, String>::new();
				for query_value in query.split("&") {
					let pair: Vec<&str> = query_value.split("=").collect();
					if pair.len() == 2 {
						&map.insert(pair[0].to_string(), pair[1].to_string());
					} else {
						continue;
					}
				}
				parsed_query = Some(URIQuery::PairList(map));
			} else {
				let mut list = Vec::<String>::new();
				for query_value in query.split("&") {
					list.push(query_value.to_string());
				}
				parsed_query = Some(URIQuery::SingleParamsList(list));
			}
		}

		Some((parsed_path.to_string(), parsed_query))
	}

	#[cfg(test)]
	mod test {
		use super::*;
		// TODO create integration test for request parsing

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
