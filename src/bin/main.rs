extern crate ziliwebka;

use ziliwebka::files::*;
use ziliwebka::http::*;
use ziliwebka::server::Server;

fn main() {
	let server = Server::new("0.0.0.0:7878".to_string(), 5);
	server.listen(&handle_connection);
	loop {}
}

fn handle_connection(request: Result<SafeRequest, Request>) {
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
