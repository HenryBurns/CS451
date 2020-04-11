use std::io;
extern crate bing_thread_pool;
use bing_thread_pool::ThreadPool;

// Read from stdin
//
// Print out line number and line

fn main() {
    let num_thread = 5;
    let thread_pool = ThreadPool::new(num_thread);
    let reader = io::stdin();
    let mut line_num = 0;
    let mut line = String::new();

    while reader.read_line(&mut line).unwrap() > 0 {
        line_num += 1;

        let cloned_line = line.clone();

        thread_pool.execute(move || {
            process_line(line_num, &cloned_line);
        });

        line.clear();
    }
}

fn process_line(line_num: u32, line: &str) {
    print!("Line number {}: {}", line_num, line);
}
