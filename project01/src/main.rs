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

fn get_hidden_message(data: Vec<u8>) -> Vec<u8> {
    let mut temp :u8 = 0;
    let mut ansr : Vec<u8> = Vec::new();
    for i in 0..data.len() {
        if i % 7 == 6 {
            temp |= 0b0100_0000 & (data[i]);
            ansr.push(temp);
            if temp == 0 {
                break;
            }
            temp = 0;
        } else {
            temp |= ((0b0010_0000) >>  i % 7) & (data[i])
        }
    }
    return ansr;
}

fn print_hidden_message(filename: &String){

}

fn hide_message(filename: &String, message: &String) {
    let mut data : Vec<u8> = Vec::new();
    read_file(filename, &mut data);
    if data.len() < message.len() {
        eprintln!("We can't fit this message in our file.");
    }
    // If we can't fit our message in the file, lets abort.
}

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
}
