use std::thread;
use std::sync::mpsc;
use std::sync::Arc;
use std::sync::Mutex;

enum Message {
    NewJob(Job),
    Shutdown,
}

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<Message>,
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        // We want to join all of our threads so that we can
        // gracefully exit
        //
        // 1. Send shutdown messages
        for _ in &mut self.workers {
            self.sender.send(Message::Shutdown).unwrap();
        }

        // 2. join all threads
        for worker in &mut self.workers {
            if let Some(thread) = worker.my_thread.take() {
                thread.join().unwrap();
            }
        }
    }
}

type Job = Box<dyn FnBox + Send + 'static>;
// Define a Worker struct that has an ID and holds a thread handle

impl ThreadPool {
    pub fn new(num_threads: usize) -> ThreadPool {
        assert!(num_threads > 0);

        let (sender, receiver) = mpsc::channel();

        let receiver = Arc::new(Mutex::new(receiver));
        let mut workers = Vec::with_capacity(num_threads);

        for indx in 0..num_threads {
            eprintln!("Indx: {}", indx);
            // Create thread
            let new_worker = Worker::new(indx, Arc::clone(&receiver));
            workers.push(new_worker)
        }
        ThreadPool{
           workers,
           sender,
        }
    }
    pub fn execute<F>(&self, f: F)
    where F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);

        self.sender.send(Message::NewJob(job)).unwrap();
    }
}

struct Worker {
    my_thread: Option<thread::JoinHandle<()>>,
    id: usize
}

trait FnBox {
    fn call_box(self: Box<Self>);
}

impl<F: FnOnce()> FnBox for F {
    fn call_box(self: Box<F>) {
        (*self)()
    }
}

impl Worker {
    fn new(new_id : usize, receiver: Arc<Mutex<mpsc::Receiver<Message>>>) -> Worker {
        // We dont know how to deal with this yet
        let id = new_id;
        let my_thread = thread::spawn(move ||{
            loop {
                // We need a mutex here to prevent race conditions
                let message = receiver.lock().unwrap().recv().unwrap();
                //let job = receiver.lock().unwrap().recv().unwrap();
                //
                match message {
                    Message::NewJob(job) => {
                        eprintln!("Worker {} is now working", id);
                        job.call_box();
                    }
                    Message::Shutdown => {
                        eprintln!("Worker {} is shutting down", id);
                        break;
                    }
                };
            }
        });

        Worker {
            my_thread: Some(my_thread),
            id,
        }
    }
}
