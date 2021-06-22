extern crate ziliwebka;

use std::fs;
use std::io::prelude::*;

use std::net::TcpListener;
use std::net::TcpStream;

use ziliwebka::http::Request;
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
	let mut buffer = [0; 1024];
	stream.read(&mut buffer).unwrap();
	// Lossy means that all UTF-8 chars that can't be recognized will be replaced with "�"
	// println!("{}", String::from_utf8_lossy(&buffer));

	if let Some(request) = Request::new(&buffer) {
	} else {
	}
}
