mod steg;

fn print_usage() {
    eprintln!("Usage1: cargo run <num threads> <Path/To/ImageDir/");
    eprintln!("Usage2: cargo run <num threads> <path/to/message/to/encode> <path/to/Images/>\n\t\t\t<path/to/output/dir/>")
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    match args.len() {
        1 => {
            print_usage()
        },
        3 => {
            match args[1].parse() {
                Err(_e) => {eprintln!("Enter an integer for the number of threads (shown below");
                           print_usage()
                },
                Ok(n_threads) => steg::decode_directory(n_threads, args[2].clone())
            }
        },
        5 => {
             match args[1].parse() {
                Err(_e) => {eprintln!("Enter an integer for the number of threads (shown below");
                           print_usage()
                },
                Ok(n_threads) => steg::encode_directory(n_threads, args[2].clone(), args[3].clone(), args[4].clone())
            }
        },
        _ => {
            eprintln!("Run this program with 2 or 4 arguments, as so.");
            print_usage()
        },
    }
}
