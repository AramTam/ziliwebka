extern crate ziliwebka;

use std::net::TcpListener;
use std::net::TcpStream;

use ziliwebka::files::*;
use ziliwebka::http::*;
use ziliwebka::threads::ThreadPool;

fn main() {
	let listener = TcpListener::bind("0.0.0.0:7878").unwrap();
	let pool = ThreadPool::new(4);

	for stream in listener.incoming() {
		let stream = stream.unwrap();
		pool.execute(|| handle_connection(stream));
	}
}

fn handle_connection(mut stream: TcpStream) {
	// println!("{}", String::from_utf8_lossy(&buffer));
	let request = Request::new(stream);
	match request {
		Ok(request) => {
			let mut response = Response::new();

			let (code, file, size) = get_file(&request.uri().0);
			response.set_code(code as usize);
			response.add_header("Content-Length".to_string(), size.to_string());
			response.set_body(file);

			request.respond(response);
		}
		Err(request) => {
			let mut response = Response::new();
			response.set_code(400);
			response.set_body(Vec::from("You have sent a wrong request"));

			request.respond(response);
		}
	}
}
