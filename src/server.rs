// TODO add logging

pub use server::*;

mod server {
	use crate::http::{Request, SafeRequest};
	use crate::threads::ThreadPool;
	use std::net::TcpListener;
	use std::sync::mpsc;
	use std::thread::{spawn, JoinHandle};

	enum ServerMessage {
		Stop,
		Pause,
		Start(&'static (dyn Fn(std::result::Result<SafeRequest, Request>) + Send + Sync)),
	}

	pub struct Server {
		sender: mpsc::Sender<ServerMessage>,
		main_thread: JoinHandle<()>,
	}
	impl Server {
		pub fn new(address: String, count_of_threads: usize) -> Server {
			// Checking threads count to later create a main server thread for listening
			if count_of_threads <= 1 {
				panic!("To start a server you should use more than 1 thread");
			}

			let listener = TcpListener::bind(address).unwrap();
			let pool = ThreadPool::new(count_of_threads - 1);
			let (sender, receiver) = mpsc::channel::<ServerMessage>();

			// Starting main thread that will respond to all state changes sent
			let main_thread = spawn(move || {
				let mut currentStatus = ServerMessage::Pause;
				let mut incoming_connections = listener.incoming();

				loop {
					// Trying to get new messages from user and act for them
					// If no messages doing last assigned status
					if let Ok(result) = receiver.try_recv() {
						currentStatus = result;
					}

					match currentStatus {
						ServerMessage::Start(function) => {
							// If we have start status, we listen to all incoming requests
							let stream = incoming_connections.next().unwrap().unwrap();
							pool.execute(move || {
								let request = Request::new(stream);
								function(request);
							});
						}
						ServerMessage::Pause => {}
						ServerMessage::Stop => break,
					}
				}
			});

			Server {
				sender,
				main_thread: main_thread,
			}
		}

		pub fn listen<F>(&self, request_callback: &'static F)
		where
			F: Fn(Result<SafeRequest, Request>) + Send + Sync,
		{
			self.sender.send(ServerMessage::Start(request_callback));
		}
	}
	impl Drop for Server {
		fn drop(&mut self) {
			self.sender.send(ServerMessage::Stop);
		}
	}
}
