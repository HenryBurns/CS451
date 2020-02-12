use std::fs::File;
use std::io::{self, Write};
use std::io::Read;
use std::process;
//use std::fs::OpenOptions;
//use std::io::Write;

fn get_start(buffer: &Vec<u8>) -> usize {
    let mut ansr = 0;
    while buffer[ansr] != ('\n' as u8) {
        ansr += 1;
    }
    ansr += 1;
    //Add the width and the height
    while buffer[ansr] != ('\n' as u8) {
        ansr += 1;
    }
    ansr += 1;
    while buffer[ansr] != ('\n' as u8) {
        ansr += 1;
    }
    ansr += 1;
    return ansr;
}

fn read_file(filename: &str, buffer: &mut Vec<u8>) -> std::result::Result<(), std::io::Error> {
    let mut f = File::open(filename)?;
    f.read_to_end(buffer)?;
    Ok(())
}

/*
fn write_file(message: &Vec<u8>, filename: &String) -> std::result::Result<(), std::io::Error> {
    let mut f = OpenOptions::new().write(true).create(true).truncate(true).open(filename)?;
    f.write(message)?;
    Ok(())
}
*/

fn print_file(data: &Vec<u8>) {
    let stdout = io::stdout();
    let mut handle = stdout.lock();
    handle.write_all(data);
}

fn get_hidden_message(data: &[u8]) -> Vec<u8> {
    let mut temp :u8 = 0;
    let mut ansr : Vec<u8> = Vec::new();
    for i in 0..data.len() {
        if i % 8 == 7 {
            temp |= 0b000_0001 & (data[i]);
            if !temp.is_ascii() {
                //eprintln!("NOT ASCII");
            }
            if temp == 0 {
                //eprintln!("Found null byte at {}", i);
                break;
            }
            ansr.push(temp);
            temp = 0;
        } else {
            temp |= (0b0000_0001 & (data[i])) <<  7 - (i % 8)
        }
    }
    return ansr;
}

fn print_hidden_message(filename: &String){
    let mut data : Vec<u8> = Vec::new();
    read_file(filename, &mut data).expect("We couldn't read the file");
    let start = get_start(&data);
    data = get_hidden_message(&data[start..]);
    for item in  &data {
        print!("{}", *item as char);
    }
    println!("");
}

fn can_fit_message(data : & Vec<u8>, message: &String) -> bool{
    let start = get_start(&data);
    if (data.len() - start) / 8 < message.len() + 1 { // + 1 for \0
        eprintln!("We can't fit this message in our file.");
        return false;
    }
    return true;
}

fn hide_message(filename: &String, message: &String) {
    let mut data : Vec<u8> = Vec::new();
    read_file(filename, &mut data).expect("We couldn't read the file");
    eprintln!("Message: {}, Message len: {}", message, message.len());
    // If we can't fit our message in the file, lets abort.
    if ! can_fit_message(&data, &message) {
        eprintln!("We can't fit the message in this image file.");
        process::abort();
    }
    let start = get_start(&data);
    let mut offset = 0;
    let mut total = 0;

    for c in message.chars() {
        if ! c.is_ascii() {
            eprintln!("NO MORE ASCII! >:(");
        }
        total = start + offset;
        for indx in (total)..(total + 8) {
            if is_one(c as u8, indx-total) {
                data[indx] |= 0b0000_0001;
            } else {
                data[indx] &= 0b1111_1110;
            }
        }
        offset += 8;
    }
    offset += 8;
    total = start + offset;

    for indx in (total)..(total + 8 ) {
        data[indx] &= 0b1111_1110;
    }

    print_file(&data)
    //write_file(&data, "./output.ppm").expect("Error writing to file");
}

fn is_one(c:u8, bit:usize)-> bool {
    if bit > 7 {
        eprintln!("What the fuck");
    }
    if c & (0b1000_0000 >> bit) != 0 {
        return true;
    }
    return false;
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
