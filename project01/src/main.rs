use std::fs::File;
use std::fs::OpenOptions;
use std::io::Read;
use std::io::Write;

fn read_file(filename: &str, buffer: &mut Vec<u8>) -> std::result::Result<(), std::io::Error> {
    let mut f = File::open(filename)?;
    f.read_to_end(buffer)?;
    Ok(())
}

fn write_file(message: &Vec<u8>, filename: &String) -> std::result::Result<(), std::io::Error> {
    let mut f = OpenOptions::new().write(true).create(true).truncate(true).open(filename)?;
    f.write(message)?;
    Ok(())

}
fn print_hidden_message(filename: &String){

}
fn hide_message(filename: &String, message: &String) {}
fn main() {
    let args: Vec<String> = std::env::args().collect();
    match args.len() {
        1 => {
            eprintln!("cargo run <path/to/PPM/with/hidden/message>\ncargo run <path/to/PPM> 'Message to hide'")
        },
        2 => {
            print_hidden_message(&args[1])
        },
        3 => {
            hide_message(&args[1], &args[2])
        },
        _ => {
            eprintln!("Run this program with 1-2 arguments")
        },
    }

    println!("Hello, world!");
}
