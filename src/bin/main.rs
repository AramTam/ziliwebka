extern crate ziliwebka;

use std::io::prelude::*;

use std::net::TcpListener;
use std::net::TcpStream;

use ziliwebka::http::*;
use ziliwebka::threads::ThreadPool;

use ziliwebka::files::*;

fn main() {
	let listener = TcpListener::bind("0.0.0.0:7878").unwrap();
	let pool = ThreadPool::new(4);

	for stream in listener.incoming() {
		let stream = stream.unwrap();
		pool.execute(|| handle_connection(stream));
	}
}

fn handle_connection(mut stream: TcpStream) {
	let mut buffer = [0; 2048];
	stream.read(&mut buffer).unwrap();
	// println!("{}", String::from_utf8_lossy(&buffer));
	let response = if let Some(request) = Request::new(&buffer) {
		let mut response = Response::new();

		let (code, file, size) = get_file(&request.uri.0);
		response.set_code(code as usize);
		response.add_header("Content-Length".to_string(), size.to_string());
		response.set_body(file);
		response
	} else {
		let mut response = Response::new();
		response.set_code(400);

		response
	};
	stream.write(&response.to_bytes()).unwrap();
}
