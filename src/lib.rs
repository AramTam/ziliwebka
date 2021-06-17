use std::sync::{mpsc, Arc, Mutex};
use std::thread;

type Job = Box<dyn FnOnce() + Send + 'static>;

enum Message {
	NewJob(Job),
	Terminate,
}

pub struct ThreadPool {
	workers: Vec<Worker>,
	sender: mpsc::Sender<Message>,
}

impl ThreadPool {
	/// Creates new ThreadPool
	///
	/// Size should be greater than 0
	///
	/// # Panics
	///
	/// `new` panics if size is 0
	pub fn new(size: usize) -> ThreadPool {
		assert!(size > 0);

		let (sender, receiver) = mpsc::channel();
		let receiver = Arc::new(Mutex::new(receiver));

		let mut workers = Vec::with_capacity(size);
		for i in 0..size {
			workers.push(Worker::new(i, Arc::clone(&receiver)));
		}
		ThreadPool { workers, sender }
	}
	pub fn execute<F>(&self, closure: F)
	where
		F: FnOnce() + Send + 'static,
	{
		let job = Box::new(closure);
		self.sender.send(Message::NewJob(job)).unwrap();
	}
}
// Implementing Drop trait to join all threads and shut them down
impl Drop for ThreadPool {
	fn drop(&mut self) {
		// Sending terminate message in separated loop to ensure that all threads get the message
		for _ in &self.workers {
			self.sender.send(Message::Terminate).unwrap();
		}
		// Calling join for all threads to unsure they are shut down
		for worker in &mut self.workers {
			if let Some(thread) = worker.thread.take() {
				thread.join().unwrap();
			}
		}
	}
}

struct Worker {
	id: usize,
	thread: Option<thread::JoinHandle<()>>,
}
impl Worker {
	fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Message>>>) -> Worker {
		let thread = thread::spawn(move || loop {
			if let Message::NewJob(job) = receiver.lock().unwrap().recv().unwrap() {
				job();
			} else {
				return;
			}
		});
		Worker {
			id,
			thread: Some(thread),
		}
	}
}
