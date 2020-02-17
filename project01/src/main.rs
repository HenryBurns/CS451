use std::fs::File;
use std::str;
use std::io::{self, Write};
use std::io::Read;
use std::process;

fn get_start(buffer: &Vec<u8>, start: &mut usize) -> bool {

    let mut ansr = 0;
    if buffer[0] != 'P' as u8 {
        eprintln!("first character wasn't P");
        return false;
    }

    else if buffer[1] != ('3' as u8) && buffer[1] != ('6' as u8) {
        eprintln!("second character wasn't 3 or 6, Char: {}", buffer[1] as char);
        return false;
    }

    // Read P6 or P3
    ansr += 3;
    while buffer[ansr] != ('\n' as u8) && buffer[ansr] != (' ' as u8) {
        if buffer[ansr] < '0' as u8 || buffer[ansr] > '9' as u8 {
            eprintln!("Header width contained non numbers. Val: {}", buffer[ansr] as char);
            return false;
        }
        ansr += 1;
    }
    ansr += 1;

    // This gets us past width

    while buffer[ansr] != ('\n' as u8) && buffer[ansr] != (' ' as u8) {
        if buffer[ansr] < '0' as u8 || buffer[ansr] > '9' as u8 {
            eprintln!("Header height contained non numbers. Val: {}", buffer[ansr] as char);
            return false;
        }
        ansr += 1;
    }
    ansr += 1;

    // This gets us past height

    while buffer[ansr] != ('\n' as u8) && buffer[ansr] != (' ' as u8) {
        if buffer[ansr] < '0' as u8 || buffer[ansr] > '9' as u8 {
            eprintln!("Pixel format contained non numbers. Val: {}", buffer[ansr] as char);
            return false;
        }
        ansr += 1;
    }
    ansr += 1;

    // This should get us past pixel format to the actual data

    *start = ansr;
    return true;
}

fn read_file(filename: &str, buffer: &mut Vec<u8>) -> std::result::Result<(), std::io::Error> {
    let mut f = File::open(filename)?;
    f.read_to_end(buffer)?;
    buffer.pop(); // We read in 1 extra byte for some reason
    Ok(())
}

fn print_file(data: &Vec<u8>) {
    let stdout = io::stdout();
    let mut handle = stdout.lock();
    handle.write_all(data).expect("There was a problem with writing output to stdout");
}

fn get_hidden_message(data: &[u8]) -> Vec<u8> {
    let mut temp :u8 = 0;
    let mut ansr : Vec<u8> = Vec::new();
    let mut found_null = false;
    for i in 0..data.len() {
        if i % 8 == 7 {
            temp |= 0b000_0001 & (data[i]);
            if !temp.is_ascii() {
                //eprintln!("NOT ASCII");
            }
            if temp == 0 {
                found_null = true;
                break;
            }
            ansr.push(temp);
            temp = 0;
        } else {
            temp |= (0b0000_0001 & (data[i])) <<  7 - (i % 8)
        }
    }
    if !found_null {
        eprintln!("This message wasn't terminated by a null character");
        process::abort();
    }
    return ansr;
}

fn print_hidden_message(filename: &String){
    let mut data : Vec<u8> = Vec::new();
    read_file(filename, &mut data).expect("We couldn't read the file");
    let mut start = 0;
    let is_valid = get_start(&data, &mut start);
    if !is_valid {
        eprintln!("This image has an invalid header");
        process::abort();
    }

    data = get_hidden_message(&data[start..]);

    for item in  &data {
        print!("{}", *item as char);
    }
}

fn can_fit_message(data : & Vec<u8>, message: &String) -> bool{
    let mut start = 0;
    let is_valid = get_start(&data, &mut start);
    if !is_valid {
        eprintln!("This image has an invalid header");
        process::abort();
    }
    if (data.len() - start) / 8 < message.len() + 1 { // + 1 for \0
        eprintln!("We can't fit this message in our file.");
        return false;
    }
    return true;
}

fn hide_message(filename: &String, message_filename: &String) {
    let mut data : Vec<u8> = Vec::new();
    let mut message_data : Vec<u8> = Vec::new();
    //let message_str = read_string(message_filename, &mut message_data);
    read_file(message_filename, &mut message_data).expect("We couldn't read the input file");

    let message = match str::from_utf8(&message_data[..]) {
        Ok(v) => String::from(v),
        Err(_v) => panic!("Invalid input provided"),
    };
    read_file(filename, &mut data).expect("We couldn't read the file");
    
    // If we can't fit our message in the file, lets abort.
    if ! can_fit_message(&data, &message) {
        eprintln!("We can't fit the message in this image file.");
        process::abort();
    }
    let mut start = 0;
    let is_valid = get_start(&data, &mut start);
    if !is_valid {
        eprintln!("This image has an invalid header");
        process::abort();
    }
    let mut offset = 0;
    let mut total = start;

    for c in &message_data {
        if ! c.is_ascii() {
            eprintln!("We are reading a non ascii charater");
        }
        for indx in (total)..(total + 8) {
            if is_one(*c, indx-total) {
                data[indx] |= 0b0000_0001;
            } else {
                data[indx] &= 0b1111_1110;
            }
        }
        offset += 8;
        total = start + offset;
    }

    for indx in (total)..(total + 8 ) {
        data[indx] &= 0b1111_1110;
    }

    print_file(&data)
}

fn is_one(c:u8, bit:usize)-> bool {
    if bit > 7 {
        eprintln!("Tried to get a bit beyond the end of this u8");
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
            eprintln!("Usage1: cargo run <path/to/PPM/with/hidden/message>\nUsage2: cargo run <path/to/PPM> <path/to/input/file>")
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
