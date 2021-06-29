// TODO learn more about caching
pub use files::*;

pub mod files {
	use std::fs;

	pub fn get_file(path_to_file: &str) -> (u32, Vec<u8>, usize) {
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
		let len = content.len();
		(code, content, len)
	}

	pub fn get_404() -> (u32, Vec<u8>) {
		//Trying to reach root/404.html file if not found just giving default output
		(
			404,
			fs::read("root/404.html")
				.unwrap_or(String::from("Error 404 file not found!").into_bytes()),
		)
	}
}
